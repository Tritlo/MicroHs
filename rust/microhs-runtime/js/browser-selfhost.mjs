import { createHash } from "node:crypto";
import { readdir, readFile, stat } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

import createCModule from "./browser-bench-c.mjs";
import { instantiateMicroHsRuntime } from "./host.mjs";

const here = path.dirname(fileURLToPath(import.meta.url));
const repo = path.resolve(here, "../../..");
const defaultInput = "/tmp/mhs-selfhost.comb";
const rustWasm = path.join(repo, "target/wasm32-unknown-unknown/release/microhs_runtime.wasm");
const compilerArgs = [
  "mhs",
  "-i",
  "-imhs",
  "-isrc",
  "-ilib",
  "MicroHs.Main",
  "-o/tmp/browser-selfhost-out.comb",
];

async function main() {
  const options = parseArgs(process.argv.slice(2));
  const inputPath = options.input ?? defaultInput;
  const input = await readFile(inputPath);
  const referenceSha = sha256(input);

  console.log(`input: ${inputPath}`);
  console.log(`input_bytes: ${input.length}`);
  console.log(`reference_sha256: ${referenceSha}`);

  if (options.target !== "c") {
    await runRustSelfHost(input, referenceSha);
  }
  if (options.target !== "rust") {
    await runCSelfHost(input, referenceSha);
  }
}

async function runRustSelfHost(input, referenceSha) {
  const runtime = await instantiateMicroHsRuntime(rustWasm);
  const handle = runtime.newProgram(input);
  try {
    runtime.setArgs(handle, compilerArgs);
    runtime.setExecutablePath(handle, "/mhs");
    const started = performance.now();
    const status = runtime.reduceMain(handle, 0xffffffff);
    const elapsedMs = performance.now() - started;
    console.log("rust_browser_selfhost:");
    console.log(`  status: ${statusName(status)}`);
    console.log(`  elapsed_ms: ${elapsedMs.toFixed(3)}`);
    if (status !== 0) {
      const message = runtime.resultText().trim();
      console.log(`  error: ${message || "<no message>"}`);
      console.log("  note: wasm32-unknown-unknown still uses browser ENOSYS host-file/env stubs");
    } else {
      console.log(`  reference_sha256: ${referenceSha}`);
      console.log("  note: output capture is not available until browser filesystem writes are implemented");
    }
  } finally {
    runtime.freeProgram(handle);
  }
}

async function runCSelfHost(input, referenceSha) {
  const stdout = [];
  const stderr = [];
  const module = await createCModule({
    noInitialRun: true,
    print(line) {
      stdout.push(line);
    },
    printErr(line) {
      stderr.push(line);
    },
  });

  mkdirp(module.FS, "/tmp");
  module.FS.writeFile("/input.comb", input);
  await preloadDirectory(module.FS, path.join(repo, "mhs"), "/mhs");
  await preloadDirectory(module.FS, path.join(repo, "src"), "/src");
  await preloadDirectory(module.FS, path.join(repo, "lib"), "/lib");

  const started = performance.now();
  try {
    module.callMain([
      "--mode",
      "main",
      "--warmup-iters",
      "0",
      "--iters",
      "1",
      "/input.comb",
      "--",
      ...compilerArgs,
    ]);
  } catch (error) {
    if (!isEmscriptenExitZero(error)) {
      console.log("c_emscripten_selfhost:");
      console.log("  status: error");
      console.log(`  error: ${String(error)}`);
      if (stderr.length) {
        console.log(`  stderr_tail: ${stderr.slice(-10).join("\\n")}`);
      }
      return;
    }
  }
  const elapsedMs = performance.now() - started;
  const output = module.FS.readFile("/tmp/browser-selfhost-out.comb");
  const outputSha = sha256(output);
  console.log("c_emscripten_selfhost:");
  console.log("  status: ok");
  console.log(`  elapsed_ms: ${elapsedMs.toFixed(3)}`);
  console.log(`  output_bytes: ${output.length}`);
  console.log(`  output_sha256: ${outputSha}`);
  console.log(`  byte_match: ${outputSha === referenceSha}`);
  const timing = parseCOutput(stdout.join("\n"));
  if (timing) {
    console.log(`  c_parse_eval_serialize_total_ms: ${timing.totalMs.toFixed(3)}`);
    console.log(`  c_bench_sink: ${timing.sink}`);
  }
}

async function preloadDirectory(fs, localRoot, wasmRoot) {
  mkdirp(fs, wasmRoot);
  for (const entry of await readdir(localRoot, { withFileTypes: true })) {
    const local = path.join(localRoot, entry.name);
    const wasm = `${wasmRoot}/${entry.name}`;
    if (entry.isDirectory()) {
      await preloadDirectory(fs, local, wasm);
    } else if (entry.isFile()) {
      fs.writeFile(wasm, await readFile(local));
    } else if (entry.isSymbolicLink()) {
      const info = await stat(local).catch(() => null);
      if (info?.isFile()) {
        fs.writeFile(wasm, await readFile(local));
      }
    }
  }
}

function mkdirp(fs, dir) {
  const parts = dir.split("/").filter(Boolean);
  let current = "";
  for (const part of parts) {
    current += `/${part}`;
    try {
      fs.mkdir(current);
    } catch (error) {
      if (!pathExists(fs, current)) {
        throw error;
      }
    }
  }
}

function pathExists(fs, target) {
  try {
    fs.lookupPath(target);
    return true;
  } catch {
    return false;
  }
}

function sha256(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

function isEmscriptenExitZero(error) {
  return error?.status === 0 || /exit\(0\)/.test(String(error));
}

function parseCOutput(output) {
  const parsed = {};
  for (const line of output.split(/\r?\n/)) {
    const idx = line.indexOf(": ");
    if (idx < 0) continue;
    parsed[line.slice(0, idx)] = line.slice(idx + 2);
  }
  const totalMs = Number(parsed.c_parse_eval_serialize_total_ms);
  const sink = Number(parsed.c_bench_sink);
  if (!Number.isFinite(totalMs) || !Number.isFinite(sink)) {
    return null;
  }
  return { totalMs, sink };
}

function statusName(status) {
  switch (status) {
    case 0:
      return "ok";
    case 1:
      return "error";
    case 2:
      return "step_limit";
    case 3:
      return "raised";
    default:
      return `unknown(${status})`;
  }
}

function parseArgs(argv) {
  const options = { target: "both", input: null };
  for (let idx = 0; idx < argv.length; idx += 1) {
    const arg = argv[idx];
    switch (arg) {
      case "--target":
        options.target = required(argv, ++idx, arg);
        if (!["both", "rust", "c"].includes(options.target)) {
          throw new Error("--target must be both, rust, or c");
        }
        break;
      case "--input":
        options.input = required(argv, ++idx, arg);
        break;
      case "-h":
      case "--help":
        console.log(
          "usage: node rust/microhs-runtime/js/browser-selfhost.mjs " +
            "[--target both|rust|c] [--input FILE]"
        );
        process.exit(0);
        break;
      default:
        throw new Error(`unknown argument: ${arg}`);
    }
  }
  return options;
}

function required(argv, idx, arg) {
  if (idx >= argv.length) {
    throw new Error(`${arg} requires a value`);
  }
  return argv[idx];
}

main().catch((error) => {
  console.error(error instanceof Error ? error.stack || error.message : String(error));
  process.exit(1);
});
