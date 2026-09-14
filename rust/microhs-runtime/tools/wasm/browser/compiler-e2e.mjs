import { spawn } from "node:child_process";
import { mkdtemp, readdir, readFile, rm, stat, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { createCompiler } from "./compiler.mjs";
import { instantiateMicroHsRuntime } from "./host.mjs";

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
const lifecycleSource = `module JsLifecycle where
import Mhs.JavaScript(JSVal)
import Foreign.C.String(CString, peekCString, withCString)

foreign import javascript "return $0 + 1" jsAdd :: Int -> IO Int
foreign import javascript "return { value: $0 }" newObject :: Int -> IO JSVal
foreign import javascript "wrapper" mkCallback :: (Int -> IO Int) -> IO JSVal
foreign import javascript "return $0($1) + 1" callCallback :: JSVal -> Int -> IO Int
foreign import javascript "return stringToNewUTF8('PRE' + UTF8ToString($0))" prefix :: CString -> IO CString

foreign export javascript "callAdd" callAdd :: Int -> IO Int
foreign export javascript "makeObject" makeObject :: Int -> IO JSVal
foreign export javascript "callWrapped" callWrapped :: Int -> IO Int
foreign export javascript "stringLength" stringLength :: Int -> IO Int

callAdd :: Int -> IO Int
callAdd = jsAdd

makeObject :: Int -> IO JSVal
makeObject = newObject

callback :: Int -> IO Int
callback x = return (x + 10)

callWrapped :: Int -> IO Int
callWrapped x = do
  f <- mkCallback callback
  callCallback f x

stringLength :: Int -> IO Int
stringLength _ = do
  s <- withCString "-test" $ \\p -> prefix p >>= peekCString
  return (length s)
`;
const schedulerSource = `module JsScheduler where
import Control.Concurrent (forkIO, yield)
import Control.Concurrent.MVar
import Data.IORef
import Mhs.JavaScript (JSVal)
import System.Mem (performGC)
import System.Mem.Weak

foreign import javascript "wrapper" mkCallback :: (Int -> IO Int) -> IO JSVal
foreign import javascript "return $0($1)" callCallback :: JSVal -> Int -> IO Int
foreign import javascript "wrapper" mkUnitCallback :: (Int -> IO ()) -> IO JSVal
foreign import javascript "$0($1); return $1" callUnitCallback :: JSVal -> Int -> IO Int

foreign export javascript "finalizerCount" finalizerCount :: Int -> IO Int
foreign export javascript "suspended" suspended :: Int -> IO Int
foreign export javascript "nested" nested :: Int -> IO Int
foreign export javascript "nestedUnit" nestedUnit :: Int -> IO Int
foreign export javascript "busy" busy :: Int -> Int

makeWeak :: IORef Int -> IO (Weak ())
makeWeak counter = do
  key <- newIORef (0 :: Int)
  mkWeak key () (Just (modifyIORef' counter (+ 1)))

finalizerCount :: Int -> IO Int
finalizerCount n = do
  counter <- newIORef n
  weak <- makeWeak counter
  performGC
  yield
  _ <- deRefWeak weak
  performGC
  yield
  readIORef counter

suspended :: Int -> IO Int
suspended n = do
  value <- newEmptyMVar
  _ <- forkIO (putMVar value n)
  takeMVar value

nested :: Int -> IO Int
nested n = do
  counter <- newIORef n
  weak <- makeWeak counter
  callback <- mkCallback (\\x -> performGC >> yield >> return x)
  result <- callCallback callback n
  _ <- deRefWeak weak
  yield
  count <- readIORef counter
  return (result + count - n)

nestedUnit :: Int -> IO Int
nestedUnit n = do
  counter <- newIORef n
  weak <- makeWeak counter
  callback <- mkUnitCallback (\\_ -> performGC >> yield)
  result <- callUnitCallback callback n
  _ <- deRefWeak weak
  yield
  count <- readIORef counter
  return (result + count - n)

busy :: Int -> Int
busy n = sum [1 .. n]

main :: IO ()
main = putStrLn "scheduler main"
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
    const native = await compileNative(tmp, source, moduleName);
    const browserNativeMatch = bytesEqual(first.dump, native.dump);
    assert(browserNativeMatch, "browser dump differed from native dump");

    const second = compiler.compile(source, {
      module: moduleName,
      flags: ["--no-main"],
    });
    const warmMatch = bytesEqual(first.dump, second.dump);
    assert(second.status === "ok", `warm compile status ${second.status}: ${second.error}`);
    assert(warmMatch, "warm recompile dump differed");

    await testCompilerDiagnostics(files);
    await testStaleExports(native.comb);

    const lifecycle = compiler.compile(lifecycleSource, {
      module: "JsLifecycle",
      flags: ["--no-main"],
    });
    assert(
      lifecycle.status === "ok",
      `JS lifecycle compile status ${lifecycle.status}: ${lifecycle.error}`
    );
    const nativeLifecycle = await compileNative(tmp, lifecycleSource, "JsLifecycle");
    assert(
      bytesEqual(lifecycle.dump, nativeLifecycle.dump),
      "JS lifecycle browser dump differed from native dump"
    );
    await testJsLifecycle(nativeLifecycle.comb);
    const scheduler = await compileNative(tmp, schedulerSource, "JsScheduler");
    const schedulerMain = await compileNative(tmp, schedulerSource, "JsScheduler", []);
    await testJsScheduler(scheduler.comb, schedulerMain.comb);

    console.log("PASS compiler-e2e");
    console.log(`status: ${first.status}`);
    console.log(`dump_length: ${first.dump.length}`);
    console.log(`browser_native_match: ${browserNativeMatch}`);
    console.log(`warm_recompile_match: ${warmMatch}`);
    console.log("diagnostic_paths: ok");
    console.log("program_lifetimes: ok");
    console.log("js_registry_and_finalizers: ok");
    console.log("js_export_scheduler: ok");
  } finally {
    compiler.close();
    compiler.close();
    if (tmp) {
      await rm(tmp, { recursive: true, force: true });
    }
  }
}

async function testCompilerDiagnostics(files) {
  const invalid = await createCompiler({
    wasm: wasmPath,
    comb: await readFile(combPath),
    files,
  });
  try {
    const result = invalid.compile("module Broken where\nbroken =", { module: "Broken" });
    assert(result.status !== "ok", "invalid source unexpectedly compiled");
    assert(result.error.length > 0, "invalid source returned an empty diagnostic");
    assert(!result.error.includes("\u0000"), "invalid source diagnostic contained binary stats");
  } finally {
    invalid.close();
  }

  const cancelled = await createCompiler({
    wasm: wasmPath,
    comb: await readFile(combPath),
    files,
    onPoll() {
      return true;
    },
  });
  try {
    const compileResult = cancelled.compile(source, {
      module: moduleName,
      flags: ["--no-main"],
    });
    assert(compileResult.status === "cancelled", `cancel status was ${compileResult.status}`);
    assert(
      compileResult.error === "cancelled by host poll",
      `cancel diagnostic was ${JSON.stringify(compileResult.error)}`
    );

    const entryResult = cancelled.toCombinators(source, "f", { module: moduleName });
    assert(entryResult.status === "cancelled", `entry cancel status was ${entryResult.status}`);
    assert(
      entryResult.error === "cancelled by host poll",
      `entry cancel diagnostic was ${JSON.stringify(entryResult.error)}`
    );
  } finally {
    cancelled.close();
  }
}

async function testStaleExports(comb) {
  const runtime = await instantiateMicroHsRuntime(wasmPath);
  const firstHandle = runtime.newProgram(comb);
  const stale = runtime.exportObject(firstHandle).f;
  assert(stale(1) === 2, "first export returned the wrong value");
  runtime.freeProgram(firstHandle);

  const secondHandle = runtime.newProgram(comb);
  try {
    assert(secondHandle !== firstHandle, "freed program handle was reused");
    assertThrows(
      () => stale(1),
      /program has been freed/,
      "stale export did not reject the freed program"
    );
    assert(runtime.exportObject(secondHandle).f(1) === 2, "second export returned the wrong value");
  } finally {
    runtime.freeProgram(secondHandle);
  }
}

async function testJsLifecycle(comb) {
  const runtime = await instantiateMicroHsRuntime(wasmPath);
  const anchorHandle = runtime.newProgram(comb);
  const handle = runtime.newProgram(comb);
  const exports = runtime.exportObject(handle);
  try {
    try {
      assert(exports.callAdd(1) === 2, "first JavaScript import call failed");
      const registered = runtime.state.reg.size;
      assert(exports.callAdd(2) === 3, "second JavaScript import call failed");
      assert(exports.callAdd(3) === 4, "third JavaScript import call failed");
      assert(runtime.state.reg.size === registered, "repeated import calls grew the registry");

      assert(exports.makeObject(1).value === 1, "first JavaScript object result failed");
      assert(exports.makeObject(2).value === 2, "second JavaScript object result failed");
      assert(exports.callWrapped(5) === 16, "synchronous wrapped callback failed");
      assert(exports.stringLength(0) === 8, "isolated string helpers failed");
      assert(liveObjectCount(runtime.state) > 0, "lifecycle test did not allocate JS objects");
      assert(runtime.state.programPtrs.has(handle), "lifecycle test did not map a pointer");
    } finally {
      runtime.freeProgram(handle);
    }
    assert(
      !runtime.state.programPtrs.has(handle),
      "freeProgram retained pointers while another program remained live"
    );
    assert(
      !runtime.state.programRegs.has(handle),
      "freeProgram retained functions while another program remained live"
    );
  } finally {
    runtime.freeProgram(anchorHandle);
  }
  assert(runtime.state.reg.size === 0, "freeProgram retained registered JavaScript functions");
  assert(liveObjectCount(runtime.state) === 0, "freeProgram retained JavaScript object handles");
  assert(runtime.state.programPtrs.size === 0, "freeProgram retained synthetic pointers");
  assert(runtime.state.argbuf.length === 0, "freeProgram retained JavaScript call arguments");
  assert(runtime.state.wres === undefined, "freeProgram retained a JavaScript callback result");
  assertThrows(
    () => exports.callAdd(1),
    /program has been freed/,
    "freed JavaScript export remained callable"
  );
}

async function testJsScheduler(exportComb, mainComb) {
  for (const withMain of [false, true]) {
    let cancel = false;
    let polls = 0;
    let stdout = "";
    const runtime = await instantiateMicroHsRuntime(wasmPath, {
      stdout(bytes) {
        stdout += new TextDecoder().decode(bytes);
      },
      onPoll() {
        polls += 1;
        return cancel;
      },
    });
    const handle = runtime.newProgram(withMain ? mainComb : exportComb);
    try {
      const exports = runtime.exportObject(handle);
      assert(exports.busy(2) === 3, "pure export returned the wrong value");
      if (withMain) {
        assert(runtime.reduceMain(handle, 0xffffffff) === 0, "export replaced the main root");
        assert(stdout === "scheduler main\n", "export lost the original main action");
      }
      assert(exports.finalizerCount(10) === 11, "export lost a weak finalizer");
      assert(exports.finalizerCount(20) === 21, "weak finalizer did not run exactly once");
      assert(exports.suspended(42) === 42, "export did not resume after blocking");
      assert(exports.nested(30) === 31, "callback lost the outer scheduler or finalizer");
      assert(exports.nestedUnit(35) === 36, "unit callback lost a deferred scheduler switch");

      cancel = true;
      assertThrows(
        () => exports.busy(10_000_000),
        /reduction cancelled by host/,
        "export did not report cooperative cancellation"
      );
      assert(polls > 0, "export did not poll the host");
      cancel = false;
      assert(exports.finalizerCount(40) === 41, "cancelled export corrupted the next call");
    } finally {
      runtime.freeProgram(handle);
    }
  }
}

function liveObjectCount(state) {
  return Object.keys(state.obj).filter((key) => key !== "0").length;
}

function assertThrows(fn, pattern, message) {
  try {
    fn();
  } catch (error) {
    assert(pattern.test(String(error?.message ?? error)), message);
    return;
  }
  assert(false, message);
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

async function compileNative(tmp, sourceText, module, flags = ["--no-main"]) {
  const sourcePath = path.join(tmp, `${module}.hs`);
  const dumpPath = path.join(tmp, `${module}.dump`);
  const combOutputPath = path.join(tmp, `${module}.comb`);
  await writeFile(sourcePath, sourceText);
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
      ...flags,
      `-ddump-combinator-out=${dumpPath}`,
      `-o${combOutputPath}`,
      module,
    ],
    {
      env: {
        ...process.env,
        MHS_GC_NODE_INTERVAL: "78643200",
      },
    }
  );
  return {
    dump: await readFile(dumpPath),
    comb: await readFile(combOutputPath),
  };
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
