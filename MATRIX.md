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
| current runtime checkpoint | `539c52d0 Preallocate large comb parses` |
| compiler scope | unchanged; keep the compiler in Haskell |
| runtime scope | Rust replacement for the C runtime/evaluator and host support |
| node layout | `Node` is 32 bytes, down from 56; cold `ForeignPtr`, `JsCall`, `Weak`, and `MutableBytes` payloads are boxed |
| parity state | benchmark/smoke rows are sink-comparable; full Rust self-host compile is not at parity |
| untracked notes | `EVALLESSONS.md` remains local working material |

## Verification

| gate | current status |
|---|---|
| Rust format/check/test/build | passed for `539c52d0`: `cargo fmt --all --check`, `cargo check -p microhs-runtime --bins --quiet`, `cargo test -p microhs-runtime --quiet`, release bin build, wasm lib build |
| `git diff --check` | passed before `539c52d0` |
| common benchmark sinks | matching |
| rare/smoke benchmark sinks | matching; no longer tracked as a live performance table here |
| self-host `--help` proxy | sink-comparable with C when using `--c-mhsbench-mode main` |
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

Current numbers are from the 32-byte `Node` runtime after removing eager spine
argument resolves, borrowing parser token text, and preallocating parser vectors
for large `.comb` inputs. They are same-machine measurements, so treat ratios
and sinks as more useful than single raw times.

| scenario | Rust ns/iter | C ns/iter | ratio | note |
|---|---:|---:|---:|---|
| `identity-chain:1000` | 26,602 | 131,914 | 0.20 | common graph |
| `arith-chain:200` | 32,306 | 112,396 | 0.29 | common scalar |
| `int64-chain:200` | 30,624 | 112,461 | 0.27 | common scalar |
| `float64-chain:200` | 33,691 | 124,285 | 0.27 | common scalar |
| `float32-chain:200` | 40,177 | 122,903 | 0.33 | common scalar |
| `bytes-chain:200` | 44,438 | 177,142 | 0.25 | common bytes |
| `cstring-pack:200` | 1,964 | 97,686 | 0.02 | bytes/FFI |
| `foreignptr-slice:200` | 1,302 | 98,573 | 0.01 | bytes/FFI |
| `unpack-chain:200` | 4,027 | 108,382 | 0.04 | bytes |
| `fromutf8-chain:200` | 5,515 | 102,836 | 0.05 | bytes |
| `array-chain:200` | 3,114 | 107,223 | 0.03 | array |
| `io-chain:200` | 34,558 | 133,823 | 0.26 | IO |
| `io-array-chain:200` | 2,957 | 99,104 | 0.03 | IO/array |
| `io-bytes-chain:200` | 1,410 | 100,464 | 0.01 | IO/bytes |
| `io-control-chain:200` | 33,554 | 137,932 | 0.24 | IO control |
| `performio-apply-chain:200` | 41,902 | 125,978 | 0.33 | IO/apply |
| `argref-chain:200` | 37,408 | 135,844 | 0.28 | args |
| `stdio-chain:200` | 78,300 | 126,582 | 0.62 | near C |
| `ffi-chain:200` | 49,435 | 162,110 | 0.30 | FFI |
| `ffi-math-chain:200` | 57,056 | 145,349 | 0.39 | FFI |
| `ffi-const-chain:200` | 49,061 | 189,627 | 0.26 | FFI |
| `ffi-mem-chain:200` | 160,715 | 266,888 | 0.60 | FFI memory |
| `bfile-read-chain:200` | 231,563 | 291,202 | 0.80 | near C |
| `mvar-chain:200` | 14,719 | 119,867 | 0.12 | sync smoke |
| `ptr-chain:200` | 82,603 | 112,980 | 0.73 | pointer |
| `rnf-chain:200` | 63,353 | 4,406,293 | 0.01 | C is slow here |
| `stableptr-chain:200` | 8,058 | 108,394 | 0.07 | stable ptr |
| `weak-chain:200` | 16,003 | 122,508 | 0.13 | weak ptr |
| `zoo-chain:300` | 53,471 | 146,172 | 0.37 | no longer a major outlier |
| `data-chain:300` | 38,481 | 143,959 | 0.27 | common graph |
| self-host `--help` proxy | 40,438,925 | 12,548,156 | 3.22 | sink `1,985,706` both sides |

The 32-byte layout improved the self-host proxy from about 64.1 ms to 54.2 ms.
Removing eager spine argument resolves moved it to about 51.5 ms and cut
profiled resolve calls roughly in half. Borrowing parser tokens moved it again
to about 48.0 ms by cutting self-host parse from about 12.8 ms to roughly 9-10
ms per iteration. Large-input parser preallocation moves the latest comparable
self-host proxy to about 40.4 ms; small inputs keep default `Vec` growth because
unconditional preallocation regressed `data-chain`. The remaining gap is still
mostly evaluator throughput.

## Self-Host Profile

| measure | latest value | implication |
|---|---:|---|
| reductions | 297,417 | work count is stable across F2 changes |
| profiled iteration time | 76-87 ms | noisy wall time; work counts below are stable |
| nodes after run | 615,854 | graph size unchanged by loop cleanup |
| app allocations | 347,838 | allocation pressure remains central |
| arg materializations | 8,523 / 37,356 nodes | measurable, but secondary |
| spine rewrites | 62,496 / 251,284 extra args | substantial tail rebuild traffic |
| app rewrites | 226,403 / 1,300,478 extra args | strongest F2 performance signal |
| resolve calls | 2,522,928 | still much larger than reductions, but down from 5,014,674 |
| followed indirections | 135,607 | fewer calls now expose a higher useful-indirection share |
| max resolve chain | 4 | deep indirections are not the current bottleneck |

Top self-host heads are still `B`, `C`, `C'`, `P`, `C'B`, `K`, `S`, `Z`, `O`,
and `A`. The hot path is evaluator/spine/update mechanics, not smoke coverage.

## Active Lessons

| lesson | implemented | current reading |
|---|---|---|
| one evaluator loop should own strict forcing | mostly yes | generic strict helper folding is done; spine arg forcing now matches C's lazier descent |
| compact node representation matters | in progress | first useful cut is 56 -> 32 bytes; keep measuring the `zoo` tradeoff |
| parser token allocation matters | yes | primitive and float tokens are now borrowed from the input buffer |
| parser allocation shape matters | partial | large `.comb` inputs preallocate nodes/stack; small inputs do not |
| redex-root and app reuse are load-bearing | partial | next F2 work should reduce app rebuild/update traffic |
| primitive heads want tags, not strings | partial | narrow static-name interning regressed self-host; revisit only as part of a real representation change |
| GC is a performance feature, not just correctness | no | F5 needs explicit roots and stack ownership before implementation |
| smoke parity is no longer the bottleneck | yes | keep smokes for regressions, but stop expanding the matrix around them |

## Current Theory

The remaining Rust/C gap is evaluator throughput. Rust wins most small common
benchmarks, but self-host still pays for resolve/classification, transient app
allocation, and extra-argument rebuilds where C stays in one compact stack/goto
evaluator and mutates redex/root cells directly.

The 32-byte node cut improves self-host by about 15%, which supports the cache
locality lesson from `eval.c`. Avoiding eager spine argument resolves is another
clear eval.c lesson: spine descent should inspect the function chain, not force
every argument reference. The parser-token change removes a Rust-only tax, but
does not change evaluator work counts. The next work should continue with
representation and update mechanics together, not another narrow cache.

## Rejected Probes Still Worth Remembering

| probe | reason not to retry as a narrow tweak |
|---|---|
| skipping outer-app rethreading / C-style continuation reuse | self-host jumped from milliseconds to seconds; resolve chains grew to 5,556 |
| first-extra-app compression only | fixed the blow-up but did not improve self-host and mixed common rows |
| app-result-only tail reuse | removed writes but lost useful extra-arg compression |
| guarded `App` writes | branch/read cost beat avoided stores |
| static `Prim` names for current runtime prims | scalar microbench win, self-host regression |
| evaluator first-indirection compression | reduced profiled indirections, but worsened profile time from about 76 ms to 82 ms and self-host proxy from about 51.5 ms to 52.7 ms |
| unconditional parser preallocation | helped self-host, but over-allocated short inputs and regressed `data-chain`; keep the 64 KiB threshold |
| older one-off caches/layout probes | measured no clear win; only revisit as part of a wider representation/evaluator change |

## Next

| priority | work |
|---|---|
| 1 | treat 32-byte nodes plus lazy spine args plus borrowed/parser-preallocated large inputs as the current performance baseline |
| 2 | finish F2 around app update/rebuild and remaining resolve/classification volume |
| 3 | keep self-host proxy in comparable C `main` mode; rerun full self-host with bounded time only when the proxy improves materially |
| 4 | unblock F5 by making roots and evaluator stack ownership explicit |
