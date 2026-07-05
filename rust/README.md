# MicroHs Rust Runtime

This directory contains the staged Rust replacement for the MicroHs runtime.
The Haskell compiler remains the authoritative `.comb` producer; Rust consumes
and executes compiler-produced `.comb` files.

## Layout

| path | purpose |
|---|---|
| `microhs-runtime/src/lib.rs` | library entry point and public parser/runtime API |
| `microhs-runtime/src/main.rs` | native `mhs-rust` CLI |
| `microhs-runtime/src/bin/mhs-rust-bench.rs` | primary benchmark and self-host harness |
| `microhs-runtime/src/wasm.rs` | browser wasm exports and JS-FFI boundary |
| `microhs-runtime/src/runtime/` | evaluator, GC, host support, codecs, and tests |
| `microhs-runtime/tools/native/` | native self-host comparison and PGO helpers |
| `microhs-runtime/tools/wasm/browser/` | browser-shaped wasm/JS-FFI harnesses and C/Emscripten comparison |
| `microhs-runtime/tools/wasm/wasi/` | WASI self-host harness |

Generated local artifacts are ignored: `target/`,
`microhs-runtime/tools/wasm/node_modules/`, and
`microhs-runtime/tools/wasm/browser/browser-bench-c.mjs`.

## Core Checks

```sh
cargo fmt --check --manifest-path rust/microhs-runtime/Cargo.toml
cargo test --manifest-path rust/microhs-runtime/Cargo.toml --lib
cargo check --manifest-path rust/microhs-runtime/Cargo.toml --features eval-phase-profile
cargo check --manifest-path rust/microhs-runtime/Cargo.toml --features gc-phase-profile
cargo build --release --manifest-path rust/microhs-runtime/Cargo.toml --bin mhs-rust-bench
```

## Native Benchmarks

Run the Rust harness directly:

```sh
target/release/mhs-rust-bench --scenario identity-chain:1000 --iters 1000
target/release/mhs-rust-bench --input /tmp/mhs-selfhost.comb --mode main \
  --warmup-iters 0 --iters 1 -- \
  ./bin/mhs -i -imhs -isrc -ilib MicroHs.Main -o/tmp/mhs-selfhost-out.comb
```

For C comparisons, use the in-process C benchmark, not the old process-spawning
`mhseval` path:

```sh
make bin/mhsbench
target/release/mhs-rust-bench --scenario identity-chain:1000 --iters 100 \
  --warmup-iters 100 --c-mhsbench ./bin/mhsbench
```

The self-host neutrality and PGO helpers live under `tools/native/`:

```sh
rust/microhs-runtime/tools/native/bench-selfhost-neutrality.sh
rust/microhs-runtime/tools/native/build-selfhost-pgo.sh
```

## Browser/WASM

Build the Rust browser wasm plus the C/Emscripten comparison module:

```sh
rust/microhs-runtime/tools/wasm/browser/build-browser-bench.sh
```

Then run the browser-shaped Node harnesses:

```sh
node rust/microhs-runtime/tools/wasm/browser/browser-compare.mjs --iters 1000 --warmup-iters 100
node rust/microhs-runtime/tools/wasm/browser/browser-selfhost.mjs --target rust
node rust/microhs-runtime/tools/wasm/browser/browser-selfhost.mjs --target c
```

## WASI

Install the JS WASI shim once from the wasm tool root:

```sh
cd rust/microhs-runtime/tools/wasm
npm install
```

Build and run the WASI self-host harness:

```sh
cargo build --release --manifest-path rust/microhs-runtime/Cargo.toml \
  --target wasm32-wasip1 --bin mhs-rust-bench
node rust/microhs-runtime/tools/wasm/wasi/wasi-selfhost.mjs /tmp/mhs-selfhost.comb
```
