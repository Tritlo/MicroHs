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
| current runtime checkpoint | `27419eb5 Avoid eager spine argument resolves` |
| compiler scope | unchanged; keep the compiler in Haskell |
| runtime scope | Rust replacement for the C runtime/evaluator and host support |
| node layout | `Node` is 32 bytes, down from 56; cold `ForeignPtr`, `JsCall`, `Weak`, and `MutableBytes` payloads are boxed |
| parity state | benchmark/smoke rows are sink-comparable; full Rust self-host compile is not at parity |
| untracked notes | `EVALLESSONS.md` remains local working material |

## Verification

| gate | current status |
|---|---|
| Rust format/check/test/build | passed for `27419eb5`: `cargo fmt --all --check`, `cargo check -p microhs-runtime --bins --quiet`, `cargo test -p microhs-runtime --quiet`, release bin build, wasm lib build |
| `git diff --check` | passed before `27419eb5` |
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
argument resolves. They are same-machine measurements, so treat ratios and
sinks as more useful than single raw times.

| scenario | Rust ns/iter | C ns/iter | ratio | note |
|---|---:|---:|---:|---|
| `identity-chain:1000` | 34,178 | 127,404 | 0.27 | common graph |
| `arith-chain:200` | 31,413 | 110,797 | 0.28 | common scalar |
| `int64-chain:200` | 37,101 | 113,524 | 0.33 | common scalar |
| `float64-chain:200` | 41,890 | 120,040 | 0.35 | common scalar |
| `float32-chain:200` | 44,423 | 119,931 | 0.37 | common scalar |
| `bytes-chain:200` | 53,341 | 177,081 | 0.30 | common bytes |
| `cstring-pack:200` | 1,655 | 98,802 | 0.02 | bytes/FFI |
| `foreignptr-slice:200` | 1,304 | 103,410 | 0.01 | bytes/FFI |
| `unpack-chain:200` | 4,135 | 100,659 | 0.04 | bytes |
| `fromutf8-chain:200` | 5,418 | 105,330 | 0.05 | bytes |
| `array-chain:200` | 3,327 | 100,829 | 0.03 | array |
| `io-chain:200` | 51,378 | 127,424 | 0.40 | IO |
| `io-array-chain:200` | 5,820 | 98,320 | 0.06 | IO/array |
| `io-bytes-chain:200` | 1,598 | 98,566 | 0.02 | IO/bytes |
| `io-control-chain:200` | 57,909 | 134,033 | 0.43 | IO control |
| `performio-apply-chain:200` | 56,726 | 128,098 | 0.44 | IO/apply |
| `argref-chain:200` | 48,290 | 130,267 | 0.37 | args |
| `stdio-chain:200` | 106,051 | 130,689 | 0.81 | near C |
| `ffi-chain:200` | 54,712 | 166,724 | 0.33 | FFI |
| `ffi-math-chain:200` | 64,029 | 155,597 | 0.41 | FFI |
| `ffi-const-chain:200` | 55,076 | 184,116 | 0.30 | FFI |
| `ffi-mem-chain:200` | 169,672 | 244,648 | 0.69 | FFI memory |
| `bfile-read-chain:200` | 272,031 | 288,159 | 0.94 | near C |
| `mvar-chain:200` | 17,054 | 118,365 | 0.14 | sync smoke |
| `ptr-chain:200` | 96,342 | 118,472 | 0.81 | pointer |
| `rnf-chain:200` | 76,856 | 4,380,948 | 0.02 | C is slow here |
| `stableptr-chain:200` | 10,992 | 108,407 | 0.10 | stable ptr |
| `weak-chain:200` | 17,799 | 114,858 | 0.15 | weak ptr |
| `zoo-chain:300` | 66,469 | 137,948 | 0.48 | still regressed versus pre-32-byte layout |
| `data-chain:300` | 53,580 | 150,318 | 0.36 | common graph |
| self-host `--help` proxy | 51,486,221 | 11,383,627 | 4.52 | sink `1,985,706` both sides |

The 32-byte layout improved the self-host proxy from about 64.1 ms to 54.2 ms.
Removing eager spine argument resolves moves it again to about 51.5 ms and cuts
profiled resolve calls roughly in half. The `zoo` regression remains the main
warning sign for the new layout.

## Self-Host Profile

| measure | latest value | implication |
|---|---:|---|
| reductions | 297,417 | work count is stable across F2 changes |
| profiled iteration time | 76.393 ms | profile overhead; compare distributions, not wall time |
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
every argument reference. The next work should continue along that line:
representation and update mechanics together, not another narrow cache.

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
| 1 | treat 32-byte nodes plus lazy spine args as the current performance baseline |
| 2 | finish F2 around app update/rebuild and remaining resolve/classification volume |
| 3 | keep self-host proxy in comparable C `main` mode; rerun full self-host with bounded time only when the proxy improves materially |
| 4 | unblock F5 by making roots and evaluator stack ownership explicit |
