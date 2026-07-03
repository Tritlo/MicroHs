# MicroHs Rust Matrix

Updated: 2026-07-03

Working dashboard for the Rust rewrite of the MicroHs C runtime/evaluator. The
compiler stays in Haskell; Rust consumes and executes compiler-produced `.comb`.

`EVALLESSONS.md` is local scratch material. This file tracks only the baseline,
the current bottleneck, and decisions that should steer the next rewrite work.

## Current State

| item | state |
|---|---|
| branch | `microhs-rust`, ahead of `origin/microhs-rust`, not pushed |
| committed checkpoint | `08ef23f4 Add bounded bench step limits` |
| active working tree | clean except local scratch notes |
| compiler scope | unchanged; keep the compiler in Haskell |
| runtime scope | Rust replacement for the C runtime/evaluator and host support |
| node layout | `Node` is 32 bytes; cold `ForeignPtr`, `JsCall`, `Weak`, and `MutableBytes` payloads are boxed |
| parity state | benchmark and smoke sinks match; full Rust self-host compile is not at parity |

## Verification Gates

| gate | status |
|---|---|
| Rust fmt/check/test/build | passed for `08ef23f4`; rerun during the rejected spill-buffer probe |
| release binary and wasm library | passed for `08ef23f4`; rerun during the rejected spill-buffer probe |
| common benchmark sinks | matching |
| rare/smoke benchmark sinks | matching; no longer expanded here |
| self-host `--help` proxy | sink-comparable with C using `--c-mhsbench-mode main` |
| full self-host compile | C completes; Rust `539c52d0` timed out at 600s with no output comb |

## Open Work

| area | state |
|---|---|
| evaluator/F2 | active bottleneck; reduce resolve/classification volume and app rebuild/update traffic |
| GC/F5 | blocked on explicit roots and evaluator stack ownership |
| JS FFI | wasm shim smokes pass; high-level browser glue still pending |
| `IO.serialize`/sharing/cycles | incomplete; defer until evaluator/GC shape is clearer |
| self-hosting | `--help` proxy runs; full compile remains the main parity/perf target |

Smoke-only subsystems now stay out of the live table unless they regress:
MVar, StablePtr, Weak, ForeignPtr, MD5, compression, process/env/filesystem,
BFILE codecs, and broad FFI coverage.

## Common Performance Snapshot

Same-machine measurements after 32-byte nodes, lazy spine arguments, borrowed
parser tokens, and large-input parser preallocation. Ratios and sink agreement
are more useful than raw timings.

| scenario | Rust ns/iter | C ns/iter | ratio | note |
|---|---:|---:|---:|---|
| `identity-chain:1000` | 26,602 | 131,914 | 0.20 | graph |
| `arith-chain:200` | 32,306 | 112,396 | 0.29 | scalar |
| `int64-chain:200` | 30,624 | 112,461 | 0.27 | scalar |
| `float64-chain:200` | 33,691 | 124,285 | 0.27 | scalar |
| `bytes-chain:200` | 44,438 | 177,142 | 0.25 | bytes |
| `io-control-chain:200` | 33,554 | 137,932 | 0.24 | IO control |
| `performio-apply-chain:200` | 41,902 | 125,978 | 0.33 | IO/apply |
| `stdio-chain:200` | 78,300 | 126,582 | 0.62 | near C |
| `ffi-mem-chain:200` | 160,715 | 266,888 | 0.60 | FFI memory |
| `bfile-read-chain:200` | 231,563 | 291,202 | 0.80 | near C |
| `zoo-chain:300` | 53,471 | 146,172 | 0.37 | mixed graph |
| `data-chain:300` | 38,481 | 143,959 | 0.27 | graph |
| self-host `--help` proxy | 40,438,925 | 12,548,156 | 3.22 | sink `1,985,706` both sides |

Small common rows mostly beat C. The gap that matters is self-host: Rust is
still about 3.2x slower on the proxy and does not finish the full compile within
the current time budget.

## Self-Host Compile Slice

Bounded profile from `08ef23f4` with `--step-limit 50000`; this does not produce
an output comb, but gives repeatable partial data for F2 work.

| measure | value |
|---|---:|
| parse + bounded reduce + render | 24.8 ms |
| WHNF steps | 52,190 |
| nodes before / after | 265,178 / 321,764 |
| node growth | 56,586 |
| app allocations | 56,410 |
| resolve calls | 1,249,709 |
| arg materializations | 1,569 / 6,099 nodes |
| spine rewrites | 7,833 / 47,156 extra args |
| app rewrites | 43,103 / 1,033,563 extra args |
| heap spines | 33,378 |
| max spine arity | 75 |

Hot heads in the slice are `B`, `C`, `A`, `C'B`, `O`, `S'`, `K`, `U`, `P`,
`C'`, `S`, `==`, `B'`, `Z`, and IO bind/sequence. The active signal is still
spine/update mechanics, not missing primitive coverage.

## Active Lessons

| lesson | implemented | current reading |
|---|---|---|
| one evaluator loop should own strict forcing | mostly | generic strict helper folding is done; remaining F2 work should keep moving strict eval paths into the loop |
| compact node representation matters | yes | 56 -> 32 bytes helped self-host; preserve cache locality while changing evaluator structure |
| avoid eager spine argument forcing | yes | matching C's lazier descent cut resolve calls roughly in half |
| parser allocation matters, but is no longer primary | mostly | borrowed tokens and large-input prealloc fixed Rust-only parser tax; avoid more parser work unless measurements point back there |
| app update/rebuild traffic is central | partial | app allocations, app rewrites, and extra-arg rebuild counts are the main F2 targets |
| primitive heads want better representation | not yet | narrow static-name interning regressed self-host; revisit only with a real tagged-head representation |
| GC is a performance feature | no | F5 needs explicit roots/stack protocol first |

## Current Theory

The remaining Rust/C gap is evaluator throughput. C keeps more work inside one
compact stack/goto evaluator and mutates redex/root cells directly. Rust now has
reasonable node size and parser cost, but still pays too much for
resolve/classification, transient app allocation, and extra-argument rebuilds.

## Rejected Narrow Probes

Only keep these as "do not retry alone" markers:

| probe | result |
|---|---|
| skipping outer-app rethreading / C-style continuation reuse | self-host jumped from milliseconds to seconds; resolve chains grew to 5,556 |
| first-extra-app compression only | fixed blow-up but did not improve self-host and mixed common rows |
| app-result-only tail reuse / guarded `App` writes | removed some writes, but branch/read cost or lost compression erased the gain |
| static `Prim` names for current runtime prims | scalar microbench win, self-host regression |
| evaluator first-indirection compression | fewer profiled indirections, slower wall time and self-host proxy |
| unconditional parser preallocation | helped self-host, regressed short inputs; keep the 64 KiB threshold |
| two-tier EvalSpine spill buffer | fixed heap-spine count, but 50k self-host slice slowed from 24.8 ms to 25.3 ms and proxy slowed from 40.4 ms to 41.9 ms |

## Next

| priority | work |
|---|---|
| 1 | finish F2 around app update/rebuild and remaining resolve/classification volume |
| 2 | prefer structural evaluator changes over narrow spine-storage tweaks |
| 3 | keep using bounded self-host slices for fast signal and full self-host reruns only after proxy/slice improvements |
| 4 | unblock F5 by making roots and evaluator stack ownership explicit |
