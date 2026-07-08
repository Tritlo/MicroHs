import { instantiateMicroHsRuntime } from "./host.mjs";

const encoder = new TextEncoder();
const decoder = new TextDecoder();
const REDUCE_LIMIT = 0xffffffff;

// Stable embedder compile boundary for Combinate's worker.ts:
// createCompiler({ wasm, comb, files, onPoll, packages }) warms one runtime,
// preloads caller-owned include files, and returns compile(source, { module,
// flags }) plus close(). onPoll (optional) is the cooperative-cancel hook: it
// is called periodically with the reduction step count, and returning truthy
// cancels the current compile (surfaced as status "cancelled"). packages
// (optional) is a list of VFS paths to MicroHs package files (.pkg) added as
// -p<path> to every compile, so pre-typechecked library modules (e.g. base.pkg)
// load from the package instead of being recompiled from source -- this is what
// replaces the prewarm .mhscache / -CR path.
// compile() writes /work/<Module>.hs, runs mhs with a deterministic
// -ddump-combinator-out=/work/<Module>.dump artifact, and returns
// { status, dump, stderr, error, stats } without scraping stdout for the dump.
export async function createCompiler({ wasm, comb, files, onPoll, packages = [] }) {
  if (wasm == null) {
    throw new TypeError("createCompiler requires wasm");
  }
  if (comb == null) {
    throw new TypeError("createCompiler requires comb");
  }
  if (files == null || typeof files !== "object") {
    throw new TypeError("createCompiler requires files");
  }
  if (!Array.isArray(packages)) {
    throw new TypeError("createCompiler packages must be an array");
  }

  let captured = [];
  const runtime = await instantiateMicroHsRuntime(wasm, {
    stdout(bytes) {
      captured.push(bytes);
    },
    stderr(bytes) {
      captured.push(bytes);
    },
    onPoll,
  });
  const combBytes = toBytes(comb);
  const workFiles = new Set();
  let closed = false;

  preloadFiles(runtime, files);
  runtime.hostMkdirp("/work");

  return {
    compile(source, opts = {}) {
      if (closed) {
        throw new Error("MicroHs compiler is closed");
      }
      const module = opts.module ?? "Main";
      const flags = opts.flags ?? [];
      validateModuleName(module);
      if (!Array.isArray(flags)) {
        throw new TypeError("compile opts.flags must be an array");
      }

      cleanupWork(runtime, workFiles);
      captured = [];

      const sourcePath = `/work/${module.replaceAll(".", "/")}.hs`;
      const dumpPath = `/work/${module}.dump`;
      mkdirp(runtime, dirname(sourcePath));
      runtime.hostWriteFile(sourcePath, textBytes(source));
      workFiles.add(sourcePath);
      workFiles.add(dumpPath);

      let handle = 0;
      let status = "error";
      let dump = null;
      let error = "";
      let stats = null;
      try {
        handle = runtime.newProgram(combBytes);
        runtime.setArgs(handle, [
          "mhs",
          "-i",
          "-i/work",
          "-imhs",
          "-isrc",
          "-ilib",
          ...packages.map((p) => `-p${p}`),
          ...flags.map(String),
          `-ddump-combinator-out=${dumpPath}`,
          module,
        ]);
        runtime.setExecutablePath(handle, "/mhs");

        const reduceStatus = runtime.reduceMain(handle, REDUCE_LIMIT);
        status = statusName(runtime.reduceMainStatus(), reduceStatus);
        dump = readOptional(runtime, dumpPath);
        stats = runtime.stats(handle);
        if (status !== "ok") {
          error = runtime.lastError() || runtime.resultText();
        }
      } catch (err) {
        dump = readOptional(runtime, dumpPath);
        error = runtime.lastError?.() || String(err?.message ?? err);
      } finally {
        if (handle !== 0) {
          runtime.freeProgram(handle);
        }
      }

      return {
        status,
        dump,
        stderr: decodeCaptured(captured),
        error,
        stats,
      };
    },
    // Compile a main-less value module and return its entry's pruned combinator
    // closure as structured JSON: { status, root, defs, ... }. `entry` is the
    // unqualified value name; `root` comes back as "<Module>.<entry>". Uses the
    // compiler's --entry flag, so there is no fake main, no failure to scrape,
    // and no client-side re-pruning: on success `status` is "ok" and `defs` is
    // the reachable closure, each `{ name, body }` with `body` a structured
    // combinator tree (see docs/javascript-ffi.md for the schema).
    toCombinators(source, entry, opts = {}) {
      if (closed) {
        throw new Error("MicroHs compiler is closed");
      }
      const module = opts.module ?? "Main";
      const flags = opts.flags ?? [];
      validateModuleName(module);
      if (typeof entry !== "string" || entry.length === 0) {
        throw new TypeError("toCombinators requires a non-empty entry name");
      }
      if (!Array.isArray(flags)) {
        throw new TypeError("toCombinators opts.flags must be an array");
      }

      cleanupWork(runtime, workFiles);
      captured = [];

      const sourcePath = `/work/${module.replaceAll(".", "/")}.hs`;
      const artifactPath = `/work/${module}.entry.json`;
      mkdirp(runtime, dirname(sourcePath));
      runtime.hostWriteFile(sourcePath, textBytes(source));
      workFiles.add(sourcePath);
      workFiles.add(artifactPath);

      let handle = 0;
      let status = "error";
      let root = null;
      let defs = null;
      let error = "";
      let stats = null;
      try {
        handle = runtime.newProgram(combBytes);
        runtime.setArgs(handle, [
          "mhs",
          "-i",
          "-i/work",
          "-imhs",
          "-isrc",
          "-ilib",
          ...packages.map((p) => `-p${p}`),
          ...flags.map(String),
          `--entry=${entry}`,
          `-ddump-combinator-out=${artifactPath}`,
          module,
        ]);
        runtime.setExecutablePath(handle, "/mhs");

        const reduceStatus = runtime.reduceMain(handle, REDUCE_LIMIT);
        status = statusName(runtime.reduceMainStatus(), reduceStatus);
        stats = runtime.stats(handle);
        const artifact = readOptional(runtime, artifactPath);
        if (status === "ok" && artifact) {
          const parsed = JSON.parse(decoder.decode(artifact));
          root = parsed.root;
          defs = parsed.defs;
        } else if (status !== "ok") {
          error = runtime.lastError() || runtime.resultText();
        } else {
          status = "error";
          error = "toCombinators: --entry artifact missing";
        }
      } catch (err) {
        status = "error";
        error = runtime.lastError?.() || String(err?.message ?? err);
      } finally {
        if (handle !== 0) {
          runtime.freeProgram(handle);
        }
      }

      return { status, root, defs, stderr: decodeCaptured(captured), error, stats };
    },
    close() {
      cleanupWork(runtime, workFiles);
      closed = true;
    },
  };
}

function preloadFiles(runtime, files) {
  for (const path of Object.keys(files).sort()) {
    if (!path.startsWith("/")) {
      throw new Error(`preload path must be absolute: ${path}`);
    }
    mkdirp(runtime, dirname(path));
    runtime.hostWriteFile(path, toBytes(files[path]));
  }
}

function cleanupWork(runtime, workFiles) {
  for (const path of workFiles) {
    try {
      runtime.hostRemove(path);
    } catch {
      // Missing files are fine; cleanup is only to avoid stale compile artifacts.
    }
  }
  workFiles.clear();
  runtime.hostMkdirp("/work");
}

function mkdirp(runtime, path) {
  if (path && path !== "/") {
    runtime.hostMkdirp(path);
  }
}

function dirname(path) {
  const idx = path.lastIndexOf("/");
  return idx <= 0 ? "/" : path.slice(0, idx);
}

function readOptional(runtime, path) {
  try {
    return runtime.hostReadFile(path);
  } catch {
    return null;
  }
}

function decodeCaptured(chunks) {
  const size = chunks.reduce((total, chunk) => total + chunk.length, 0);
  const joined = new Uint8Array(size);
  let offset = 0;
  for (const chunk of chunks) {
    joined.set(chunk, offset);
    offset += chunk.length;
  }
  return decoder.decode(joined);
}

function statusName(status, fallback) {
  switch (status >= 0 ? status : fallback) {
    case 0:
      return "ok";
    case 1:
      return "error";
    case 2:
      return "step-limit";
    case 3:
      return "raised";
    case 4:
      return "cancelled";
    default:
      return "error";
  }
}

function validateModuleName(module) {
  if (typeof module !== "string" || module.length === 0) {
    throw new TypeError("compile opts.module must be a non-empty string");
  }
  const component = "[A-Z][A-Za-z0-9_']*";
  const re = new RegExp(`^${component}(\\.${component})*$`);
  if (!re.test(module)) {
    throw new Error(`invalid module name: ${module}`);
  }
}

function textBytes(value) {
  return typeof value === "string" ? encoder.encode(value) : toBytes(value);
}

function toBytes(value) {
  if (value instanceof Uint8Array) {
    return value.slice();
  }
  if (ArrayBuffer.isView(value)) {
    return new Uint8Array(value.buffer, value.byteOffset, value.byteLength).slice();
  }
  if (value instanceof ArrayBuffer) {
    return new Uint8Array(value).slice();
  }
  throw new TypeError("expected bytes");
}
