# MicroHs Rust Matrix

Updated: 2026-07-03

This is the working dashboard for the Rust rewrite of the MicroHs C
runtime/evaluator. The Haskell compiler stays in Haskell; Rust consumes and
executes compiler-produced `.comb`.

`EVALLESSONS.md` is the detailed scratch note. This file should stay short
enough to answer: what is comparable, what is slow, what lesson is active, and
what should not be retried blindly.

## Current State

| item | state |
|---|---|
| branch | `microhs-rust`, ahead of `origin/microhs-rust`, not pushed |
| current runtime checkpoint | `c561db52 Box cold node payloads` |
| compiler scope | unchanged; keep the compiler in Haskell |
| runtime scope | Rust replacement for the C runtime/evaluator and host support |
| node layout | `Node` is 32 bytes, down from 56; cold `ForeignPtr`, `JsCall`, `Weak`, and `MutableBytes` payloads are boxed |
| parity state | benchmark/smoke rows are sink-comparable; full Rust self-host compile is not at parity |
| untracked notes | `EVALLESSONS.md` remains local working material |

## Verification

| gate | current status |
|---|---|
| Rust format/check/test/build | passed for `c561db52`: `cargo fmt --all --check`, `cargo check -p microhs-runtime --bins --quiet`, `cargo test -p microhs-runtime --quiet`, release bin build, wasm lib build |
| `git diff --check` | passed before `c561db52` |
| common benchmark sinks | matching |
| rare/smoke benchmark sinks | matching; no longer tracked as a live performance table here |
| self-host `--help` proxy | runs and prints identical usage |
| full self-host compile | C completes; Rust still times out without an output comb |

## Coverage

| area | state |
|---|---|
| `.comb` parse/render for benchmark programs | comparable |
| core reducer/combinators | comparable; now performance work |
| numeric, bytes, arrays, IO core | comparable |
| MVar, StablePtr, Weak, ForeignPtr | comparable smoke; ForeignPtr finalizers still need GC |
| FFI, filesystem/env/process, BFILE codecs, MD5/compression | comparable smoke/rare paths; not current performance targets |
| JS FFI hooks | partial; wasm shim smokes pass, high-level browser glue still pending |
| `IO.serialize`/sharing/cycles | incomplete; defer until after evaluator/GC shape is clearer |
| GC/F5 | not implemented; blocked on explicit roots/stack protocol |
| self-hosting | `--help` proxy runs; full compile remains the main parity/perf target |

## Performance Snapshot

Current numbers are from the 32-byte `Node` runtime. They are same-machine
measurements, so treat ratios and sinks as more useful than single raw times.

| scenario | Rust ns/iter | C ns/iter | ratio | note |
|---|---:|---:|---:|---|
| `arith-chain:200` | 32,942 | 111,871 | 0.29 | common scalar |
| `int64-chain:200` | 39,461 | 112,530 | 0.35 | common scalar |
| `float64-chain:200` | 46,549 | 121,874 | 0.38 | common scalar |
| `float32-chain:200` | 41,795 | 122,829 | 0.34 | common scalar |
| `bytes-chain:200` | 54,156 | 180,043 | 0.30 | common bytes |
| `data-chain:300` | 57,598 | 140,868 | 0.41 | common graph |
| `zoo-chain:300` | 67,478 | 134,055 | 0.50 | regressed under 32-byte layout |
| `io-control-chain:200` | 52,687 | 145,050 | 0.36 | common IO control |
| `performio-apply-chain:200` | 59,504 | 126,326 | 0.47 | common IO/apply |
| `stdio-chain:200` | 110,693 | 129,220 | 0.86 | near C |
| `bfile-read-chain:200` | 275,811 | 283,242 | 0.97 | near C |
| self-host `--help` proxy | 54,222,897 | 13,337,937 | 4.1 | main current gap |

Compared with the previous 56-byte-node self-host proxy, the 32-byte layout
improves Rust from about 64.1 ms to 54.2 ms per iteration. The `zoo` regression
means the layout is promising but not settled.

## Self-Host Profile

| measure | latest value | implication |
|---|---:|---|
| reductions | 297,417 | work count is stable across F2 changes |
| profiled iteration time | 93.984 ms | profile overhead; compare distributions, not wall time |
| nodes after run | 615,854 | graph size unchanged by loop cleanup |
| app allocations | 347,838 | allocation pressure remains central |
| arg materializations | 8,523 / 37,356 nodes | measurable, but secondary |
| spine rewrites | 62,496 / 251,284 extra args | substantial tail rebuild traffic |
| app rewrites | 226,403 / 1,300,478 extra args | strongest F2 performance signal |
| resolve calls | 5,014,674 | classification/resolve traffic dominates reductions |
| max resolve chain | 4 | deep indirections are not the current bottleneck |

Top self-host heads are still `B`, `C`, `C'`, `P`, `C'B`, `K`, `S`, `Z`, `O`,
and `A`. The hot path is evaluator/spine/update mechanics, not smoke coverage.

## Active Lessons

| lesson | implemented | current reading |
|---|---|---|
| one evaluator loop should own strict forcing | mostly yes | generic strict helper folding is done; do not reopen this without new evidence |
| compact node representation matters | in progress | first useful cut is 56 -> 32 bytes; keep measuring the `zoo` tradeoff |
| redex-root and app reuse are load-bearing | partial | next F2 work should reduce app rebuild/update traffic |
| primitive heads want tags, not strings | partial | narrow static-name interning regressed self-host; revisit only as part of a real representation change |
| GC is a performance feature, not just correctness | no | F5 needs explicit roots and stack ownership before implementation |
| smoke parity is no longer the bottleneck | yes | keep smokes for regressions, but stop expanding the matrix around them |

## Current Theory

The remaining Rust/C gap is evaluator throughput. Rust wins most small common
benchmarks, but self-host still pays for repeated resolve/classification,
transient app allocation, and extra-argument rebuilds where C stays in one
compact stack/goto evaluator and mutates redex/root cells directly.

The 32-byte node cut improves self-host by about 15%, which supports the cache
locality lesson from `eval.c`. The `zoo` regression says layout/codegen effects
are real; the next work should improve representation and update mechanics
together instead of adding another narrow cache or helper-unification pass.

## Rejected Probes Still Worth Remembering

| probe | reason not to retry as a narrow tweak |
|---|---|
| skipping outer-app rethreading / C-style continuation reuse | self-host jumped from milliseconds to seconds; resolve chains grew to 5,556 |
| first-extra-app compression only | fixed the blow-up but did not improve self-host and mixed common rows |
| app-result-only tail reuse | removed writes but lost useful extra-arg compression |
| guarded `App` writes | branch/read cost beat avoided stores |
| static `Prim` names for current runtime prims | scalar microbench win, self-host regression |
| older one-off caches/layout probes | measured no clear win; only revisit as part of a wider representation/evaluator change |

## Next

| priority | work |
|---|---|
| 1 | decide whether the 32-byte node layout becomes the new baseline after one fuller matrix run |
| 2 | finish F2 around app update/rebuild and resolve/classification volume |
| 3 | use the self-host proxy as the primary benchmark; rerun full self-host with bounded time only when the proxy improves |
| 4 | unblock F5 by making roots and evaluator stack ownership explicit |
