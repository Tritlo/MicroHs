# MicroHs Rust Runtime

This directory contains the staged Rust replacement for the MicroHs runtime.
The Haskell compiler remains the authoritative `.comb` producer; Rust consumes
and executes compiler-produced `.comb` files.

## Status & performance

The Rust runtime **self-hosts byte-identically**: running the compiler's own
`.comb` (`MicroHs.Main` compiling MicroHs, ~3.66 B reductions) through the Rust
evaluator produces the same output `.comb` - identical SHA - as the C runtime
(`src/runtime/eval.c`).

**Memory is the axis that matters.** A Rust cell is a packed **8 bytes** (a 4-bit
tag plus two 30-bit arena indices for an `App`, or a compact scalar; wide scalars
— int64/float64/bytes/foreign — spill to a side table). A C node is **16 bytes**:
two machine words holding native `NODEPTR` pointers or inline 64-bit values. So at
equal *cell count* Rust uses ~60% of C's memory — meaning the older "128 M cells
each" comparison silently handed C **2.09 GB against Rust's 1.26 GB**. The fair
comparison holds *memory* constant, not cell count. (The flip side of 8-byte
density: the packed cell's 30-bit indices cap the arena at ~1.07 B cells; a
program needing more aborts. `cargo build --features wide-cell` swaps in a 16-byte
cell that lifts the cap to ~4.29 B — a correctness escape hatch, not a perf option:
at equal memory (~787 MB) it holds half the cells and runs **+38%** slower. See
`microhs-runtime/docs/cell-redesign-plan.md`.)

**Equal memory — C's default heap (~790 MB), median of 3 interleaved runs, RSS
matched within ~1% (Rust 796 MB vs C 787 MB), every Rust output byte-identical:**

| runtime (~790 MB RSS) | wall (median) | Rust ÷ C |
|---|---:|---:|
| C `eval.c` (`-O3`) | 48.2 s | — |
| **Rust (shipped, `cargo build --release`)** | 48.0 s | **≈1.0x — ties (±0.5% across runs)** |
| C `eval.c` (`-O3`, PGO) | 46.5 s | — |
| **Rust (PGO)** | 43.0 s | **0.924x — 7.6% faster** |

At C's out-of-the-box memory budget, **Rust ties C without PGO and is 7.6% faster
with PGO.** A tight budget is GC-bound, and Rust's 2× cell density means far fewer
collections — its packed representation, a liability on raw reduction speed,
becomes the advantage. Absolute wall drifts a few percent between sessions, so the
interleaved **ratio** is the stable metric.

**Loose memory (128 M cells each) inverts it**, and exposes the per-reduction gap
underneath:

| runtime (128 M cells) | RSS | wall |
|---|---:|---:|
| C `eval.c` (`-O3`) | 2.09 GB | ~41.8 s |
| Rust (shipped) | 1.26 GB | ~44.8 s |

Given equal *cells* — and thus 66% more RAM — C's direct-pointer node is ~6%
faster per reduction (see below). Reproduce either budget with
`tools/native/matrix.sh` (defaults to the equal-memory point; override `CHEAP` /
`RINT` for others).

### The per-reduction gap (why C leads at loose memory)

Given ample memory the self-host is **instruction-bound, not memory-bound**. Callgrind's cache
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

GC is a minor share at a 128 M-cell heap (a reused mark bitmap plus direct-tag
mark traversal keep the non-moving mark-sweep collector close to C's) — but at a
tight budget GC becomes the dominant cost, and that is exactly where Rust's 8-byte
cell fits ~2× the live nodes per MB and collects far less often. So the per-op
representational gap and the memory efficiency pull in opposite directions; which
one dominates is set by the memory budget.

PGO buys C ~5-6% and Rust more (it recovers the packed reducer's per-op overhead),
so at equal memory **Rust+PGO overtakes C+PGO** (0.924x). The shipped build is
plain `cargo build --release`; PGO is deliberately not shipped (it complicates the
reproducible-build story) and only marks the codegen floor — build it with
`tools/native/build-selfhost-pgo.sh`. The default build also carries in-place
`cold_path` hints that recover part of PGO's hot/cold block placement (−1.6% wall).
Note the reducer is mildly sensitive to the Rust toolchain: LLVM 22 (Rust ≥1.95)
regressed the hot dispatch ~+2.9% vs LLVM 21, not recoverable by any stable build
flag; see `microhs-runtime/docs/perf-gap-analysis.md` for the full analysis.

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
cargo +1.97.1 clippy --manifest-path rust/microhs-runtime/Cargo.toml --all-targets -- -D warnings
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
