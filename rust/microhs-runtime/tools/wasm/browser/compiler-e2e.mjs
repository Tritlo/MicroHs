import { spawn } from "node:child_process";
import { mkdtemp, readdir, readFile, rm, stat, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { createCompiler } from "./compiler.mjs";

const here = path.dirname(fileURLToPath(import.meta.url));
const repo = path.resolve(here, "../../../../..");
const wasmPath = path.join(repo, "target/wasm32-unknown-unknown/release/microhs_runtime.wasm");
const benchPath = path.join(repo, "target/release/mhs-rust-bench");
const combPath = path.join(repo, "generated/mhs.comb");
const moduleName = "Foo";
const source = `module ${moduleName} where

foreign export javascript "f" f :: Int -> Int

f :: Int -> Int
f x = x + 1
`;

async function main() {
  await ensureWasm();
  await run("cargo", [
    "build",
    "--release",
    "--manifest-path",
    path.join(repo, "rust/microhs-runtime/Cargo.toml"),
    "--bin",
    "mhs-rust-bench",
  ]);

  const files = {};
  await preloadDirectory(files, path.join(repo, "mhs"), "/mhs");
  await preloadDirectory(files, path.join(repo, "src"), "/src");
  await preloadDirectory(files, path.join(repo, "lib"), "/lib");

  const compiler = await createCompiler({
    wasm: wasmPath,
    comb: await readFile(combPath),
    files,
  });

  let tmp = null;
  try {
    const first = compiler.compile(source, {
      module: moduleName,
      flags: ["--no-main"],
    });
    assert(first.status === "ok", `browser compile status ${first.status}: ${first.error}`);
    assert(first.dump && first.dump.length > 0, "browser dump was empty");

    tmp = await mkdtemp(path.join(os.tmpdir(), "mhs-compiler-e2e-"));
    const nativeDump = await compileNative(tmp);
    const browserNativeMatch = bytesEqual(first.dump, nativeDump);
    assert(browserNativeMatch, "browser dump differed from native dump");

    const second = compiler.compile(source, {
      module: moduleName,
      flags: ["--no-main"],
    });
    const warmMatch = bytesEqual(first.dump, second.dump);
    assert(second.status === "ok", `warm compile status ${second.status}: ${second.error}`);
    assert(warmMatch, "warm recompile dump differed");

    console.log("PASS compiler-e2e");
    console.log(`status: ${first.status}`);
    console.log(`dump_length: ${first.dump.length}`);
    console.log(`browser_native_match: ${browserNativeMatch}`);
    console.log(`warm_recompile_match: ${warmMatch}`);
  } finally {
    compiler.close();
    if (tmp) {
      await rm(tmp, { recursive: true, force: true });
    }
  }
}

async function ensureWasm() {
  if (await exists(wasmPath)) {
    return;
  }
  await run("bash", [
    "-lc",
    `. ~/emsdk/emsdk_env.sh >/dev/null && bash ${shellQuote(
      path.join(repo, "rust/microhs-runtime/tools/wasm/browser/build-browser-bench.sh")
    )}`,
  ]);
}

async function compileNative(tmp) {
  const sourcePath = path.join(tmp, `${moduleName}.hs`);
  const dumpPath = path.join(tmp, "foo.dump");
  await writeFile(sourcePath, source);
  await run(
    benchPath,
    [
      "--input",
      combPath,
      "--mode",
      "main",
      "--warmup-iters",
      "0",
      "--iters",
      "1",
      "--",
      "mhs",
      "-i",
      `-i${tmp}`,
      "-imhs",
      "-isrc",
      "-ilib",
      "--no-main",
      `-ddump-combinator-out=${dumpPath}`,
      moduleName,
    ],
    {
      env: {
        ...process.env,
        MHS_GC_NODE_INTERVAL: "78643200",
      },
    }
  );
  return readFile(dumpPath);
}

async function preloadDirectory(files, localRoot, wasmRoot) {
  for (const entry of await readdir(localRoot, { withFileTypes: true })) {
    const local = path.join(localRoot, entry.name);
    const wasm = `${wasmRoot}/${entry.name}`;
    if (entry.isDirectory()) {
      await preloadDirectory(files, local, wasm);
    } else if (entry.isFile()) {
      files[wasm] = await readFile(local);
    } else if (entry.isSymbolicLink()) {
      const info = await stat(local).catch(() => null);
      if (info?.isFile()) {
        files[wasm] = await readFile(local);
      }
    }
  }
}

async function exists(file) {
  return stat(file).then(
    () => true,
    () => false
  );
}

async function run(command, args, options = {}) {
  const result = await runCapture(command, args, options);
  if (result.status !== 0) {
    throw new Error(
      `${command} ${args.join(" ")} failed with status ${result.status}\n${result.stderr}${result.stdout}`
    );
  }
  return result;
}

function runCapture(command, args, options = {}) {
  return new Promise((resolve, reject) => {
    const child = spawn(command, args, {
      cwd: repo,
      env: options.env ?? process.env,
      stdio: ["ignore", "pipe", "pipe"],
    });
    const stdout = [];
    const stderr = [];
    child.stdout.on("data", (chunk) => stdout.push(chunk));
    child.stderr.on("data", (chunk) => stderr.push(chunk));
    child.on("error", reject);
    child.on("close", (status) => {
      resolve({
        status,
        stdout: Buffer.concat(stdout).toString("utf8"),
        stderr: Buffer.concat(stderr).toString("utf8"),
      });
    });
  });
}

function bytesEqual(a, b) {
  if (!a || !b || a.length !== b.length) {
    return false;
  }
  for (let idx = 0; idx < a.length; idx += 1) {
    if (a[idx] !== b[idx]) {
      return false;
    }
  }
  return true;
}

function assert(condition, message) {
  if (!condition) {
    console.error("FAIL compiler-e2e");
    throw new Error(message);
  }
}

function shellQuote(value) {
  return `'${String(value).replaceAll("'", "'\\''")}'`;
}

main().catch((error) => {
  console.error(error?.stack ?? String(error));
  process.exit(1);
});
