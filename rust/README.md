# MicroHs Rust Runtime

This directory contains the staged Rust replacement for the MicroHs runtime.
The Haskell compiler remains the authoritative `.comb` producer; Rust consumes
and executes compiler-produced `.comb` files.

## Status & performance

The Rust runtime **self-hosts byte-identically**: running the compiler's own
`.comb` (`MicroHs.Main` compiling MicroHs) through the Rust evaluator produces the
same output `.comb` - identical SHA - as the C runtime (`src/runtime/eval.c`).

Self-host compile, one machine, 3.66 B reductions, C `eval.c` (`-O3`) at its
normal heap as the baseline. Every Rust output is byte-identical to C.

| runtime | wall (median) | vs C | cell memory | GC pause |
|---|---|---|---|---|
| **C `eval.c` (`-O3`)** | **46.4 s** | 1.00x | 50M cells / 800 MB | 11.4 s / 89 GCs |
| Rust - default (128M GC interval) | **53.6 s** | **1.16x** | ~1.1 GB | 8.66 s / 31 GCs |
| Rust - C-matched memory (80Mi interval) | **55.9 s** | **1.21x** | ~705 MB | 11.1 s / 49 GCs |

Where the time goes (at C-matched memory):

- **GC is at parity with C** - ~11.1 s vs C's ~11.4 s. An 8-byte packed cell (a
  4-bit tag plus two 30-bit node ids; wide scalars demoted to a cold side-table)
  and a sweep fast-path closed what was a ~12 s collector gap.
- The residual gap is the **mutator/locality** (~44.8 s vs C's ~35.0 s), which is
  ~85% memory-latency at the free-list allocator: each allocation reads a cold dead
  cell for the next-free pointer and hands back scattered slots, so the following
  spine descent misses cache. C's bitmap allocator returns the lowest free slot and
  self-densifies.

**Codegen floor - the fair PGO/heap matrix (measurement-only, not shipped).** A
profile-guided (PGO) build of the same Rust source - no algorithmic change - runs
the self-host at 45.5 s (128M heap) / 49.4 s (C-matched memory). But that is a Rust
codegen floor, not a win over C: give C the same compiler and heap treatment and it
stays ahead. Self-host at the matched 128M heap, all runs byte-identical:

| self-host @ 128M heap | `-O3` | PGO |
|---|---|---|
| **C `eval.c`** | 40.3 s | **38.6 s** |
| **Rust** | 53.6 s | **45.5 s** |

At C's own default heap, C `-O3` is 46.4 s (89 GCs); the 128M heap alone is a ~13%
C win (GC drops from 89 to 34 collections). The decisive equalized peer is therefore
**Rust-PGO@128M 45.5 s vs C-PGO@128M 38.6 s = 1.18x C**: PGO buys Rust a large
speedup but does not erase the gap once C gets the same treatment. PGO is
deliberately not shipped (a build flag that complicates the reproducible-build
story); it only marks the floor. Build it with `tools/native/build-selfhost-pgo.sh`.

Size: the Rust runtime is ~22k LOC (including tests, the wasm/JS-FFI boundary, and
the bench harness) against ~8k for the C runtime.

_Self-host compile, measured 2026-07-05. The full experiment ledger and structural
analysis live in `MATRIX.md` and `NOTES.md` at the repository root._

## Layout

| path | purpose |
|---|---|
| `microhs-runtime/src/lib.rs` | library entry point and public parser/runtime API |
| `microhs-runtime/src/main.rs` | native `mhs-rust` CLI |
| `microhs-runtime/src/bin/mhs-rust-bench.rs` | primary benchmark and self-host harness |
| `microhs-runtime/src/wasm.rs` | browser wasm exports and JS-FFI boundary |
| `microhs-runtime/src/runtime/` | evaluator, GC, host support, codecs, and tests |
| `microhs-runtime/ARCHITECTURE.md` | reducer, heap, GC, and host-boundary overview |
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
cargo check --manifest-path rust/microhs-runtime/Cargo.toml --features profile
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
