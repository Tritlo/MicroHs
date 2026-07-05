// Run the Rust wasm32-wasip1 self-host under a pure-JS WASI shim with an
// in-memory filesystem — the browser-equivalent of running the WASI build.
// This makes the Rust self-host comparable to the emscripten eval.c MEMFS run
// (see browser-selfhost.mjs), on a real standard WASI interface rather than the
// bespoke host.mjs JS-FFI bridge, and — unlike node's native `node:wasi` — the
// pure-JS shim tolerates the ~627MB arena growth the full self-host needs.
//
// Prereq:  npm install            (in this directory; pulls @bjorn3/browser_wasi_shim)
//          cargo build --release --target wasm32-wasip1 --bin mhs-rust-bench
// Usage:   node wasi-selfhost.mjs [comb] [gcInterval]
import { WASI, File, Directory, PreopenDirectory, OpenFile, ConsoleStdout } from "@bjorn3/browser_wasi_shim";
import { readFileSync, readdirSync, statSync } from "node:fs";
import { createHash } from "node:crypto";
import path from "node:path";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const repo = path.resolve(here, "../../..");
const wasmPath = path.join(repo, "target/wasm32-wasip1/release/mhs-rust-bench.wasm");
const combPath = process.argv[2] ?? "/tmp/mhs-selfhost.comb";
const gcInterval = process.argv[3] ?? "33554432";

function loadDir(real) {
  const m = new Map();
  for (const name of readdirSync(real)) {
    const full = path.join(real, name);
    const st = statSync(full);
    if (st.isDirectory()) m.set(name, loadDir(full));
    else if (st.isFile()) m.set(name, new File(readFileSync(full)));
  }
  return new Directory(m);
}

console.log(`wasm: ${wasmPath}`);
console.log(`comb: ${combPath}  gc_interval: ${gcInterval}`);
const combBytes = readFileSync(combPath);
console.log(`reference_sha256: ${createHash("sha256").update(combBytes).digest("hex")}`);
console.log("building in-memory FS (mhs/src/lib + comb)...");

// Preopen "." holds the source tree + input comb + a writable tmp dir, so the
// compiler's relative -imhs/-isrc/-ilib and -otmp/out.comb resolve as on native.
const root = new Map();
root.set("mhs", loadDir(path.join(repo, "mhs")));
root.set("src", loadDir(path.join(repo, "src")));
root.set("lib", loadDir(path.join(repo, "lib")));
root.set("bin", new Directory(new Map()));
root.set("selfhost.comb", new File(combBytes));
const tmp = new Directory(new Map());
root.set("tmp", tmp);

const args = ["mhs-rust-bench", "--input", "selfhost.comb", "--mode", "main",
  "--warmup-iters", "0", "--iters", "1", "--",
  "./bin/mhs", "-i", "-imhs", "-isrc", "-ilib", "MicroHs.Main", "-otmp/out.comb"];
const env = [`MHS_GC_NODE_INTERVAL=${gcInterval}`];
const fds = [
  new OpenFile(new File([])),
  ConsoleStdout.lineBuffered((m) => process.stdout.write(`[wasm] ${m}\n`)),
  ConsoleStdout.lineBuffered((m) => process.stderr.write(`[wasm-err] ${m}\n`)),
  new PreopenDirectory(".", root),
];

const wasi = new WASI(args, env, fds, { debug: false });
const mod = await WebAssembly.compile(readFileSync(wasmPath));
const inst = await WebAssembly.instantiate(mod, { wasi_snapshot_preview1: wasi.wasiImport });

const t0 = performance.now();
let code = 0;
try {
  wasi.start(inst);
} catch (e) {
  if (e && e.constructor && e.constructor.name === "WASIProcExit") code = e.code;
  else { console.error("start threw:", e); code = -1; }
}
const elapsedMs = performance.now() - t0;

const refSha = createHash("sha256").update(combBytes).digest("hex");
const out = tmp.contents.get("out.comb");
console.log("wasi_selfhost:");
console.log(`  status: ${code === 0 ? "ok" : "error"}`);
console.log(`  elapsed_ms: ${elapsedMs.toFixed(1)}`);
console.log(`  exit_code: ${code}`);
if (out && out.data) {
  const sha = createHash("sha256").update(out.data).digest("hex");
  console.log(`  output_bytes: ${out.data.length}`);
  console.log(`  output_sha256: ${sha}`);
  console.log(`  byte_match_reference: ${sha === refSha}`);
} else {
  console.log("  NO OUTPUT (tmp/out.comb missing)");
}
