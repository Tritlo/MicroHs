# MicroHs Rust Runtime

This directory contains the staged Rust replacement for the MicroHs runtime.
The Haskell compiler remains the authoritative `.comb` producer; Rust consumes
and executes compiler-produced `.comb` files.

## Status & performance

The Rust runtime **self-hosts byte-identically**: running the compiler's own
`.comb` (`MicroHs.Main` compiling MicroHs, ~3.66 B reductions) through the Rust
evaluator produces the same output `.comb` - identical SHA - as the C runtime
(`src/runtime/eval.c`).

Self-host compile, one quiet machine, both runtimes given a 128 M-cell heap
(`-H134217728` / `MHS_GC_NODE_INTERVAL=134217728`), median of 3 interleaved runs,
every Rust output byte-identical to C:

| runtime (128 M cells) | wall (median) | vs C non-PGO |
|---|---|---|
| **C `eval.c` (`-O3`)** | 44.3 s | **1.00x** |
| C `eval.c` (`-O3`, PGO) | 41.8 s | 0.94x |
| **Rust (shipped, `cargo build --release`)** | 46.8 s | **1.06x** |
| Rust (PGO) | 44.3 s | 1.00x |

Absolute wall drifts a few percent with machine load between sessions, so the
**ratio** (Rust/C, measured interleaved in one session) is the stable metric, not
the raw seconds. A run of representational reducer tuning moved the non-PGO gap
from **1.16x to 1.06x** C. (C's own default heap is 50 M cells; both sides get
128 M here. Rust's 8-byte packed cell uses ~1.1 GB at that budget against C's
~2 GB, so equal cells is if anything generous to C on time and to Rust on memory.)

### Where the gap is

The self-host is **instruction-bound, not memory-bound**. Callgrind's cache
simulation puts the last-level miss rate at ~0.2%, and ~88% of all instructions
execute inside the single reduction-dispatch function (`stack_eval_step`). Cutting
the reducer's instruction count tracked wall closely: the tuning rounds took the
bounded 80 M-step prefix from ~17.3 B to ~13.9 B I-refs (~19%) and the wall ratio
fell in step. This **revises the earlier heap-locality hypothesis** — the
bottleneck is instruction count and branch prediction in the mutator, not cache
misses in spine descent (two direct allocator-locality experiments, a C-style
bitmap allocator and an address-ordered free-list, had already failed to confirm
the locality idea).

The residual ~6% is **representational**. The reducer indexes a `Vec<Cell>` arena
by `NodeId`; even with unchecked, invariant-guarded hot-path access (`push_cell`
asserts the arena can never reach the packed-id limit, so every `NodeId` is a
valid index and the trusted writes are sound) that is index arithmetic plus a cold
side-table hop for wide scalars, where C dereferences raw `NODEPTR` pointers
straight into `FUN`/`ARG` struct fields. Closing it further means matching C's
representation more closely: the instruction-count wins from bounds-check removal
are largely spent, and the newest reducer changes now cut I-refs without moving
wall — per-instruction throughput (i-cache, branch prediction) has become the
limit, not instruction count.

GC is a minor share at this heap: a reused mark bitmap plus direct-tag mark
traversal keep the non-moving mark-sweep collector close to C's.

PGO buys each side ~5-6% (Rust+PGO ~= C non-PGO), so the gap is **structural, not
a codegen artifact PGO closes** — which is why the shipped build is plain
`cargo build --release`. PGO is deliberately not shipped (it complicates the
reproducible-build story); it only marks the codegen floor. Build it with
`tools/native/build-selfhost-pgo.sh`.

Size: the Rust runtime is ~22k LOC (including tests, the wasm/JS-FFI boundary, and
the bench harness) against ~8k for the C runtime.

_Self-host compile, measured 2026-07-06._

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
