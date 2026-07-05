# MicroHs Rust Matrix

Updated: 2026-07-05

Working dashboard for the Rust rewrite of the MicroHs C runtime/evaluator. The
compiler stays in Haskell; Rust consumes and executes compiler-produced `.comb`.

`EVALLESSONS.md` and `NOTES.md` are local scratch material. This file tracks
only the baseline, the current bottleneck, and decisions that should steer the
next rewrite work.

## Current State

| item | state |
|---|---|
| branch | `microhs-rust`, pushed to `origin/microhs-rust` at `8933a6d1`; `split-runtime-modules` was fast-forward merged and remains as the source-organization side branch |
| committed checkpoint | `self-hosting-binary-match`; latest source-organization checkpoint is `8933a6d1` (`Split live reducer modules`), following `76ff9d3e` (`Use real runtime modules`), `b8284e1e` (`Reorganize Rust runtime tree`), and `a28d0290` (`Split runtime implementation into include files`); latest WASI self-host harness checkpoint is `c5f8ea1f` (`Add WASI self-host harnesses`); latest profiling checkpoint is `13f9d90f` (`Add evaluator force profiling counters`); latest wasm browser-host checkpoint is `61eca83b` (`Add browser host filesystem for wasm self-host`); latest wasm comparison harness checkpoint is `b49dc2ef` (`Add wasm32 runtime comparison harness`); latest runtime performance checkpoint is `d5efc37f` (`Trust hot reducer cell reads`); prior runtime checkpoint is `c969e993` (`Trust app-fun descent in reducer hot path`); prior runtime checkpoint before that is `3c04e5c1` (`Trust WHNF resolver in reducer hot path`); latest self-host harness checkpoint is `15f76b3a` (`Add self-host neutrality benchmark harness`) |
| active working tree | runtime now uses the accepted packed 8-byte `Cell` arena unconditionally; the obsolete `packed-cell` feature and 16-byte fallback were removed after a neutral A/B against `11334945`. Current source organization is real Rust modules, not `include!`: `runtime.rs` declares the public facade and internal module tree; `runtime/host.rs`, `runtime/program.rs`, `runtime/program/bfile.rs`, `runtime/program/eval.rs`, and `runtime/program/runtime_dispatch.rs` are `mod` roots with scoped imports. The live fallback reducer is no longer named legacy: `eval/reduce.rs`, `eval/spine.rs`, `eval/persistent.rs`, and `eval/fallback.rs` hold the general loop, spine helpers, persistent reducer, and runtime-primitive fallback; eval-time primitive dispatch is `eval/dispatch.rs`. Release builds use `panic = "abort"`, `lto = "thin"`, and `codegen-units = 1`. Native helpers live under `rust/microhs-runtime/tools/native/`; browser JS-FFI wasm harnesses live under `rust/microhs-runtime/tools/wasm/browser/`; the WASI self-host harness lives under `rust/microhs-runtime/tools/wasm/wasi/`. Current runtime also has committed feature-gated profiling counters, the GC-rooted read-only memory BFILE view, checked LZMA decoding, weak-pointer key/value GC semantics, the self-host/PGO harnesses, and R1 moving-GC scaffolding gated behind `cfg(any(test, feature = "moving-gc"))`: the `NodeId` remapper plus a full-heap marked-evacuation primitive; `MATRIX.md` is the live ledger and local scratch notes are present |
| compiler scope | unchanged; keep the compiler in Haskell |
| runtime scope | Rust replacement for the C runtime/evaluator and host support |
| node layout | bench prints representation sizes; current default release output is `node_size_bytes=16`, `cell_size_bytes=8`, `node_id_size_bytes=4`, `prim_size_bytes=4`. The authoritative arena is the unconditional packed `Cell { word }`: 4-bit tag, u30 App fun/arg payloads, u30 Indir/Free/Cold payloads, 60-bit small `Int`/`ThreadId`, inline `Float32`, and cold-boxed wide scalars/payloads. `Node` stays the parse/debug facade for cold boxed payloads; no 16-byte cell implementation remains in the Rust runtime |
| parity state | benchmark and smoke sinks match; Rust self-host compile produces byte-identical output. Current runtime uses unconditional 8-byte cells. Latest current-source 128M full self-host refresh after the latest profiling simplification commits is 3x byte-identical at `58.772s`, `58.657s`, `59.297s` (median `58.772s`, avg `58.909s`), same `3,659,074,736` steps, `31` GCs, high-water `138,068,132` cells, output SHA `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`, and `cell_size_bytes=8`. Latest paired accepted 128M gate remains the free-slot allocation cleanup: candidate median `58.044s` and avg `58.040s` versus same-session baseline median `57.939s` and avg `58.084s` (`+0.18%` median, `-0.08%` avg). Treat that as neutral cleanup; the previous R5 sweep free-node row is the latest source-speed claim. Most recent fairer-memory 80Mi interval median is `68.975s` with high-water `88.15M` 8-byte cells and 49 GCs. Fresh C oracle full self-host is `47.85s`; older historical gates remain recorded below |

## Verification Gates

| gate | status |
|---|---|
| Rust fmt/test/release bench build | passed after the S3 authoritative cell arena, S1 single-stack hot path, S2 GC + S1 app-only spine/head-classification worktree, rejected strict-tail/segmented-stack reverts, S3 parse-time primitive interning, S3 compact representation work, direct runtime-primitive strict dispatch, S1 stack argument binding, S1 compact stack-entry split, S1 trusted stack argument loads/static fallback names, S1 app-only eval stack, S0 layer/update profiling, feature-gated phase/profile-overhead profiling, strict Int immediate args, unchecked stack arg loads, reusable GC mark-work stack, feature-gated GC phase profiling, rejected bitmap allocator restore, S1 app-continuation descent, S1 direct app-result descent, S1 dedicated app allocation, stack scalar `SET*` redex overwrites, S1 fixed-arity `CHKARG` pop/take rewrites, direct known-app/free-cell writes, S1 strict `Int` redex-owning frames, S1 WHNF redex-owning frames, raw cell tag/trusted free-list accessors, parse-time small-int interning retest, F10 iterative serializer, F17 cold bytes views, F14 shared ForeignPtr finalizer records, F12 catch masking-state restoration, per-head/per-site eval profiling, bench representation-size output, S1 unified stack `ret`/`top` continuation, S2 parse-label non-root GC treatment, S1 allocator free-head invariant tightening, S1 identity-alias `GOIND` continuation, S1 one-word app-fun spine descent, S2 mark-time app-edge canonicalization, S1 trusted stack redex/update access, eval.c permanent compound caches, rejected low-to-high free-list reuse restore, rejected FFI/JS arity-prefix materialization restore, rejected eval.c `GOAP2` direct-push restore, app-allocation split profiling instrumentation, GCRED opportunity profiling instrumentation, F10 BFILE `IO.print`/`IO.serialize` parity, S4 young-region viability profiling, F6/F10 `IO.deserialize` parity, weak-pointer GC semantics, the app-allocation profiling-bookkeeping cold split, inline primitive cell decoding, release `panic=abort` with a checked LZMA decoder, feature-gated stack-head arity/continuation/app-shape profiling, trusted non-profile WHNF resolver, trusted app-fun descent, trusted hot reducer cell reads following eval.c's internal-heap trust rule, real runtime modules, the live reducer split/rename, feature-gated R1 moving-GC scaffolding, and stale profile-surface trims; latest `cargo fmt --check`, `cargo check`, `cargo test --lib` 43/43, `cargo check --features moving-gc`, `cargo check --features eval-phase-profile`, `cargo check --features gc-phase-profile`, release wasm checks for `wasm32-unknown-unknown` and `wasm32-wasip1`, release bench build, 10M/100M bounded gates, current `eval-phase-profile` 10M refresh, stack-head arity-profile refresh, stack-continuation transition profile, app-allocation site-shape profile, resolved app-allocation site-shape profile, trusted-WHNF 100M A/B/full gates, trusted-app-fun 8x 100M A/B/full gate, trusted-hot-cell 8x 100M A/B/full gate, latest same-session 900s full self-host refresh, rejected dense free-slot stack allocator probe, rejected direct B/B superinstruction full gate, rejected raw app-stack push bounded probe, rejected batched app-allocation accounting full gate, rejected strict-Int indirection immediate full gate, restored intrusive-free-list 100M sanity, SHA/cmp checks, and `git diff --check` passed |
| stack rewrite profiling verification | `cargo fmt`, `cargo test --manifest-path rust/microhs-runtime/Cargo.toml --lib` 41/41, wasm release `cargo check`, `eval-phase-profile` release bench build, and default release bench build passed; default 10M control after rebuild is 279.898ms at 35.83M steps/s |
| self-host neutrality harness | `bash -n rust/microhs-runtime/tools/native/bench-selfhost-neutrality.sh` passed; 10M same-source smoke with equal-length output labels was base avg 296.196ms vs candidate avg 314.510ms with identical 10,028,866 steps, sink, and high-water; balanced 100M same-source repeat (`MHS_REPEAT=4`) was base avg 2380.626ms vs candidate avg 2421.441ms (`+1.71%`) with identical 100,811,779 steps, 3 GCs, high-water 33,941,709, and `cmp_result=missing-output` by design for step-limited runs. Repeat plus alternating order is required; single-pair 10M/100M timings are too noisy |
| wasm library | release `cargo build --target wasm32-unknown-unknown` passed on current `15f76b3a`; Node `host.mjs` smoke rendered `2147483648` from `v8.4\n0\n#2147483648 }\n`, so Rust's wasm path does not inherit the C/eval.c ILP32 `intptr_t` wrap described in `WRAPBUG.md`; current `wasm32-wasip1` release bench builds and the committed pure-JS WASI shim self-host harness completes byte-identically in `135.941s` with 124 GCs and `38.420s` GC pause. Browser-path comparison now has a Node-checkable harness: `rust/microhs-runtime/tools/wasm/browser/build-browser-bench.sh` builds Rust `wasm32-unknown-unknown` plus an Emscripten ES module from `src/runtime/mhsbench.c`, `browser-compare.mjs` compares parse+WHNF+serialize with matching sinks, `browser-selfhost.mjs` tests Emscripten/browser self-hosting, and `wasi-selfhost.mjs` tests the Rust `wasm32-wasip1` CLI path. Latest 1000-iter/100-warmup Node evaluator pass matched all sinks and showed Rust faster on all eight first rows. Self-host status: Rust browser wasm completed byte-identically from the JS-backed host env/filesystem in 180.627s; fresh C/Emscripten wasm completes from MEMFS in 61.464s but output differs from native by the known eval.c wasm32 wrap/float issue; Rust `wasm32-wasip1` completes byte-identically in 135.941s |
| EVALLESSONS correctness batch | current `cargo test --manifest-path rust/microhs-runtime/Cargo.toml --lib` passes 43/43; covers F1 string escapes, F3 catchable arithmetic RTS exceptions, F4 uncaught RTS/showExn formatting and `ExitSuccess` handling, F6 shared/cyclic serializer labels plus `IO.deserialize` memory-BFILE/cycle smokes, F7 unknown prim failures, F10 iterative deep serializer plus BFILE `IO.print`/`IO.serialize` smokes, F11 mpz wire format, F12 catch handler masking/restoration smoke, F15 `putchar`/`lz77c`, F16 faithful modified UTF-8 decoding, F17 immutable bytestring views for `bssubstr`/`tailUTF8`, F18 ignored-IO shortcut depth cap, F19 compression fixture/round-trip coverage, F21 `mpz_get_d` decimal rounding, weak-pointer GC key/value reachability, and feature-gated R1 moving-GC remapper plus marked-heap evacuation coverage; F14 also has focused null-finalizer GC and direct `^&closeb` address-FFI smokes |
| common benchmark sinks | latest targeted current refresh matches C sinks on all refreshed rows |
| rare/smoke benchmark sinks | matching; no longer expanded here |
| self-host `--help` proxy | sink-comparable with C using `--c-mhsbench-mode main` |
| full self-host compile | latest current-source 128M full self-host refresh after the latest profiling simplification commits completed byte-identically in three samples: `58.772s`, `58.657s`, `59.297s` (median `58.772s`, avg `58.909s`), same `3,659,074,736` steps, `31` GCs, GC pause avg `9.043s`, high-water `138,068,132` cells, sink `661902`, `cell_size_bytes=8`, `cmp=0`, and SHA `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`. The prior accepted free-slot allocation cleanup was byte-identical in a paired 3x full self-host A/B at the 128M window: baseline median `57.939s`/avg `58.084s` versus candidate median `58.044s`/avg `58.040s`, same heap shape and output; treat it as neutral cleanup (`+0.18%` median, `-0.08%` avg), not a new source-speed claim. The previous R5 sweep free-node fast path remains the accepted speed row. Fresh C oracle full self-host completes in `47.85s` with the same output SHA; tagged checkpoint remains `self-hosting-binary-match` at `673.8s`; use `timeout 900s` for full gates and profiling while structural/codegen probes can still regress above the smaller window |

| latest live reducer split gate | committed `8933a6d1` removes the misleading live `legacy.rs`: fallback eval-loop code is now `eval/reduce.rs`, eval-spine rewriting is `eval/spine.rs`, persistent-spine reduction is `eval/persistent.rs`, fallback runtime-primitive rewriting is `eval/fallback.rs`, and eval-time primitive dispatch is `eval/dispatch.rs`. Verification passed `cargo fmt --check`, `cargo check`, `cargo test --lib` 41/41, `cargo check --features eval-phase-profile`, `cargo check --features gc-phase-profile`, release wasm checks for `wasm32-unknown-unknown` and `wasm32-wasip1`, release bench build, 900s full self-host, SHA/cmp, and `git diff --check`. Full gate was byte-identical at `74.904s`, `3,659,073,466` steps, `48.85M` steps/s, `124` GCs, `24.815s` GC pause, high-water `39,192,385`, SHA `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`, `cmp_exit=0`; source organization only, not a new performance best |
| latest stack rewrite profiling full gate | default release completed byte-identically in 83.786s with 3,659,074,831 steps, 43.67M steps/s, 125 GCs, 24.462s GC pause, high-water 38,820,704 cells, SHA `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`, `cmp_exit=0`; not a new best |
| latest self-host neutrality harness full gate | current `15f76b3a` default release completed byte-identically in 82.740s with 3,659,074,850 steps, 44.22M steps/s, 125 GCs, 24.196s GC pause, high-water 38,820,706 cells, SHA `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`, `cmp_exit=0`; tooling-only checkpoint, not a new best |
| latest app-shape diagnostics default full gate | default release with feature-gated app-allocation shape diagnostics compiled out completed byte-identically in 78.093s with 3,659,074,869 steps, 46.86M steps/s, 124 GCs, 23.616s GC pause, high-water 39,192,416 cells, SHA `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`, `cmp_exit=0`; verification only, consistent with the 77.387s/77.918s current band and not a new source checkpoint |
| latest resolved app-shape diagnostic | extended `eval-phase-profile` with app-allocation shape counters that first follow indirections for each operand and classify the resolved App fun head. `cargo fmt --check`, default `cargo check`, `cargo check --features eval-phase-profile`, an isolated `/tmp/mhs-eval-profile-resolved` release build, default release bench build, SHA/cmp, and a 900s default full gate passed. The 10M profiled self-host run had a normal pass of 304.570ms / 32.93M steps/s and a profiled pass of 16.439s; counters stayed at 11,105,979 app allocations, 8,292,661 stack app updates, 30,134,585 descent pushes, and 30,009,541 arg reads. Top resolved buckets collapse the old `Indir` split into broad App/App cases: `B.yz: App(App) App(App)=1,184,073`, `C.xz: App(App) App(App)=805,201`, `S'.zw: App(App) App(App)=504,141`, `S'.yw: App(App) App(App)=503,777`, `C'.yw: App(App) App(App)=395,092`, `C'B.yw: App(App) App(App)=278,453`, `S'.left: App(App) App(App)=221,334`, and `S.right: App(App) App(App)=203,101`; smaller direct heads such as `C'.xyw: S' App(App)=120,062`, `C'.xyw: C App(App)=116,730`, `B.yz: App(B) App(App)=29,526`, and `B.yz: I App(App)=28,434` are too narrow. Reading: following indirections rules out a hidden concentrated F2 direct-shape target; the remaining wall is the general eval.c-style app allocation/update/descent path or a larger representation/GC/codegen shift. Default full gate with this diagnostic compiled out completed byte-identically in 78.117s with 3,659,075,268 steps, 46.84M steps/s, 124 GCs, 23.416s GC pause, high-water 39,192,458 cells, SHA `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`, `cmp_exit=0`; diagnostic only, not a source checkpoint |
| latest F2 force-surface diagnostic | added `eval-phase-profile` counters for `eval_whnf_value` calls/slow paths and recursive `reduce_node_whnf` entry shapes. `cargo fmt --check`, default `cargo check`, `cargo check --features eval-phase-profile`, `cargo test --lib` 41/41, an isolated `/tmp/mhs-eval-profile-force` release build, default release bench build, 10M profile, and a 900s default full gate passed. A 10M profiled self-host run had a normal pass of 310.811ms / 32.27M steps/s and a profiled pass of 18.396s. The old native-recursive force surface is tiny relative to the hot stack loop: `eval_whnf_value` calls were `Int=25,874`, `Pointer=23,405`, `Array=23,234`, `Bytes=3,186`, `ForeignPtr=134`, but slow calls were only `Bytes=308`, `Pointer=213`, `Int=51`, `ForeignPtr=30`, `Array=5`; recursive `reduce_node_whnf` entries totaled 615, led by `App(App)=400`, `App(fp2p)=112`, `App(I)=80`, `App(bs2fp)=16`. Reading: F2's old native-stack risk is still a correctness/design item, but it is not the self-host performance wall now; the hot wall remains `stack_eval_step` app allocation/descent/update/arg traffic, with only 90 fallback eval-loop steps in 10M. Default full gate after the diagnostic compiled out was byte-identical but slow/noisy at 86.499s with 3,659,074,907 steps, 42.30M steps/s, 124 GCs, 28.416s GC pause, high-water 39,192,420, SHA `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`, `cmp_exit=0`; not a new baseline |
| latest F2 long-spine refresh and budget-split probe | reran the 10M `eval-phase-profile` force build with `--profile-top 60`. Normal pass was `365.990ms` / 27.40M steps/s; profiled pass was `17.171s`. Counters remained app-stack dominated: `11,105,967` app allocations, `8,292,661` app redex updates, `30,134,585` descent pushes, `30,009,541` arg reads, `256,660` stack-step calls, `45,574` WHNF exits, and only `90` fallback-loop steps. Top arity classes were long over-applied spines (`B@extra=1,940,011`, `C@extra=1,555,402`, `S'@extra=810,754`, `C'@extra=732,275`, `S@extra=580,185`, `C'B@extra=538,754`), with visible but narrow long-spine transitions (`B->B@319=24,664`, `B->B@320=21,317`, `S'->S'@320=13,296`, `S'->S'@319=12,196`). Tried a bounded/unbounded stack-reducer split so full self-host (`usize::MAX` limit) could compile without inner budget checks while step-limited gates kept the bounded path. `cargo fmt --check`, default `cargo check`, `cargo check --features eval-phase-profile`, `cargo test --lib` 41/41, and release bench build passed. Bounded sanity was plausible (`10M=312.658ms`, `100M=2.222s` / 45.37M steps/s / 235.367ms GC), but the 900s full gate was byte-identical and regressed to `83.900s`, `3,659,074,736` steps, 43.61M steps/s, 124 GCs, `27.058s` GC pause, high-water `39,192,402`, SHA `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`, `cmp_exit=0`; source was reverted and the default release bench rebuilt. Post-revert 100M sanity was `2.302s`, 43.79M steps/s, 3 GCs, `251.054ms` GC, high-water `33,939,308`. Rerun after suspected co-run load: restored baseline full was byte-identical at `82.421s` / 44.40M steps/s / `24.919s` GC pause; budget-split rerun was byte-identical at `78.210s` / 46.79M steps/s / `24.823s` GC pause, a same-session win but still above the committed `74.668s` best and not accepted. Reading: the unbounded budget branch is not the standalone cost, and duplicating the hot loop worsens full-gate code shape. Do not retry a budget split without a larger evaluator-loop representation change |
| latest isolated WASI cfg probe | narrowed the earlier reverted std-env/path/BFILE port so `target_os = "wasi"` selects the existing std fallback implementations while browser wasm keeps ENOSYS stubs; added a `MHS_WASI_TRACE=1` WASI-only host-call trace. Verification passed `cargo fmt --check`, native release bench build, `cargo test --lib` 41/41, `cargo check --target wasm32-wasip1 --bin mhs-rust-bench`, `cargo build --release --target wasm32-wasip1 --bin mhs-rust-bench`, Node WASI `arith-chain:1000`, native SHA/cmp, and `git diff --check`. WASI self-host now clears the old `setenv` ENOSYS blocker: default Node WASI step-limited runs pass at 200k, 400k, 500k, 600k, and 650k reductions; 650k is 46.954ms, 651,291 steps, no GC, high-water 864,769. Default Node WASI still exits 139 at 675k/700k/1M after the same host trace (`getenv MHSDIR`, `fopen` eval.c/config/source, repeated 1024-byte source reads) and after printing the first normal benchmark line, so the remaining blocker looks like wasm/Node memory-growth pressure before the normal 32M allocation-triggered GC, not another missing syscall. Forcing early GC with `MHS_GC_NODE_INTERVAL=500000` makes 675k pass at 57.543ms / 676,291 steps / 1 GC / 7.135ms GC / high-water 653,287 and 1M pass at 65.037ms / 1,001,359 steps / 2 GCs / 11.554ms GC / high-water 661,400. An out-of-tree `--initial-memory=134217728` wasm build segfaulted before the trace, so initial-memory alone is not a confirmed workaround. Native full self-host stayed in-band and byte-identical at 77.592s with 3,659,074,869 steps, 47.16M steps/s, 124 GCs, 23.314s GC pause, high-water 39,192,416, SHA `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`, `cmp_exit=0`; not a new source checkpoint yet because the WASI trace is diagnostic and full WASI still does not complete |
| latest accepted trusted WHNF resolver | committed `3c04e5c1` (`Trust WHNF resolver in reducer hot path`). The non-profile reducer path now uses a trusted WHNF resolver (`cell_trusted` plus debug assertions for bounds, `Free`, and missing indirection targets) while public `resolve` and profiled runs stay checked; this follows eval.c's hot-loop assumption that internal heap references are valid. Verification passed `cargo fmt --check`, release bench build, `cargo test --lib` 41/41, `cargo check --features eval-phase-profile`, `cargo check --features gc-phase-profile`, `git diff --check`, 10M/100M bounded gates, isolated 3x alternating 100M A/B, and 900s full self-host gates. Bounded 10M was slow/noisy at 357.626ms / 28.04M steps/s; single 100M was 2.281s / 44.20M steps/s with 3 GCs, 219.629ms GC pause, and high-water 33,939,304 cells. Isolated 3x alternating 100M A/B was base avg 2301.660ms vs candidate avg 2171.964ms (`-5.63%`) with identical 100,811,779 steps, high-water 33,939,300, and sink. Full gates were byte-identical: first candidate 77.050s / 47.49M steps/s / 23.629s GC with SHA `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`, repeat noisy at 83.241s / 43.96M steps/s / 25.242s GC; isolated full pair under noisy conditions was base 84.432s vs candidate 84.146s, and base/candidate/oracle cmp all matched. Reading: accepted as a small default-build eval.c trust-rule win; full self-host remains noisy, but the 100M A/B and best observed full gate justify checkpointing |
| latest accepted trusted app-fun descent | committed `c969e993` (`Trust app-fun descent in reducer hot path`). Added `app_fun_trusted`, using the already-trusted cell load plus debug assertions for impossible `Indir`/`Free` tags, and switched only `descend_stack_from` to it; public/cold app-fun reads stay checked. This is the same eval.c trust rule as the WHNF resolver, now applied to the one-word AP walk after resolution. Verification passed `cargo fmt --check`, release bench build, `cargo test --lib` 41/41, `cargo check --features eval-phase-profile`, `cargo check --features gc-phase-profile`, `git diff --check`, isolated 8x alternating 100M A/B, SHA/cmp, and a 900s full self-host gate. A/B results were base avg 2146.053ms vs candidate avg 2037.559ms (`-5.06%`) with identical 100,811,779 steps, 3 GCs, high-water 33,939,298, and sink 661902; the candidate stayed in a tight 2017.850-2065.275ms band while base had repeated 2.27-2.31s slow samples. Full gate completed byte-identically in 76.369s with 3,659,074,983 steps, 47.91M steps/s, 124 GCs, 23.662s GC pause, high-water 39,192,428 cells, SHA `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`, `cmp_exit=0`. Reading: accepted as the current default-release best and another eval.c-style trust win in the hot AP walk |
| latest accepted trusted hot reducer cell reads | committed `d5efc37f` (`Trust hot reducer cell reads`). Removed the now-dead checked `app_fun` helper and switched the active reducer outer descent, hot head dispatch, strict-`Int` immediate reads, `S I` check, and ready-frame current-cell read to `cell_trusted`/`app_fun_trusted`; public/cold checked APIs stay checked. Verification passed `cargo fmt --check`, release bench builds, `cargo test --lib` 41/41, `cargo check --features eval-phase-profile`, `cargo check --features gc-phase-profile`, `git diff --check`, isolated 8x alternating 100M A/B, SHA/cmp, and a 900s full self-host gate. A/B results were base avg 2050.098ms vs candidate avg 2012.507ms (`-1.83%`) with identical 100,811,779 steps, 3 GCs, high-water 33,939,300, and sink 661902; base-1 was a slow outlier, but the candidate stayed in the tighter 1984.105-2039.218ms band. Full gate completed byte-identically in 74.668s with 3,659,074,850 steps, 49.00M steps/s, 124 GCs, 23.445s GC pause, high-water 39,192,414 cells, SHA `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`, `cmp_exit=0`. Reading: accepted as the current default-release best and another eval.c internal-heap trust win; next useful work should keep removing Rust-only checked/control layers from the hot evaluator path rather than adding narrow superinstructions |
| latest rejected lazy runtime fallback-name probe | tried making runtime primitive fallback-name lookup lazy so strict runtime primitives do not eagerly call `runtime.name()` just to discard the fallback label. Checks passed (`cargo fmt --check`, release build, `cargo test --lib` 41/41, `cargo check --features eval-phase-profile`, `cargo check --features gc-phase-profile`), and isolated 8x alternating 100M A/B looked favorable at base avg 2055.140ms vs candidate avg 2015.378ms (`-1.93%`) with identical steps, high-water, and sink. The 900s full gate rejected it: byte-identical output, but 74.911s / 48.85M steps/s / 23.196s GC versus the committed `d5efc37f` best of 74.668s / 49.00M steps/s / 23.445s GC. Source was reverted; do not retry this as a standalone codegen probe |
| latest rejected direct cached-primitive accessors | tried adding direct cached accessors for eval.c-style permanent combinators (`K`, `B`, `C`, `I`, `IO.>>=`, booleans, etc.) and replacing hot literal `self.prim("...")` calls so evaluator paths bypass the string-keyed `prim(name)` cache lookup. Checks passed (`cargo fmt --check`, release build, `cargo test --lib` 41/41, `cargo check --release`, `cargo check --features eval-phase-profile`, `cargo check --features gc-phase-profile`, `git diff --check`). Isolated 8x alternating 100M A/B was mildly favorable: base avg 2211.785ms vs candidate avg 2183.147ms (`-1.29%`) with identical 100,811,779 steps, 3 GCs, high-water 33,939,310, and sink 661902. Full gates rejected it as a standalone source change: byte-identical outputs, but 74.760s and 74.676s versus the committed `d5efc37f` best of 74.668s; candidate also perturbed full step/high-water counts slightly. Source was reverted and release bench rebuilt |
| latest rejected non-saturating allocation-counter retest | retested the old accounting-only idea on the current trusted-hot-cell machine by replacing the two hot `gc_allocations_since_collect.saturating_add(1)` updates in `push_node`/`push_app_node` with plain increments. Checks passed (`cargo fmt --check`, default `cargo check`, release bench build). Bounded was mixed: 10M was noisy/slow at 329.960ms then 306.590ms with no GC, while 100M was plausible at 2221.674ms then 2237.929ms with 3 GCs and about 228-229ms GC pause. The 900s full gate rejected it: byte-identical output, 77.683s / 47.10M steps/s / 24.849s GC pause / high-water 39,192,396, SHA `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`, `cmp_exit=0`, versus the current clean refresh at 77.500s and the committed `d5efc37f` best at 74.668s. Source was reverted. Reading: per-app allocation-counter saturation is not the remaining app-allocation wall; do not retry accounting-only allocation changes without changing the allocator/cell representation itself |
| latest profiled-resolve entry probe | tried selecting plain `resolve` at `reduce_whnf` entry when no eval profile is active, instead of always entering `resolve_profiled` and bouncing on `self.profile.is_none()` at every WHNF resolver call. Cheap checks passed (`cargo fmt --check`, default `cargo check`, `cargo check --features eval-phase-profile`, release bench build), and bounded slices looked plausible: 10M was slow/noisy at 334.366ms / 29.99M steps/s, while 100M was 2.272s / 44.38M steps/s / 220.370ms GC. A 3x alternating 100M clean-base A/B was favorable but noisy: base avg 2163.665ms, candidate avg 2094.701ms; heap shape and step counts matched. Full self-host rejected it: clean `65c3be1d` archive base completed byte-identically in 77.359s with 3,659,074,888 steps, 47.30M steps/s, 124 GCs, 23.225s GC pause, high-water 39,192,418 cells, while the candidate completed byte-identically in 77.779s with 3,659,075,021 steps, 47.04M steps/s, 124 GCs, 23.185s GC pause, high-water 39,192,432 cells. Reading: the extra resolver branch is real enough to move bounded slices, but changing the WHNF entry path perturbs full-workload layout/step shape and loses to the full gate. Source was reverted, release bench rebuilt, and restored 100M sanity was 2.293s / 43.97M steps/s / 223.543ms GC |
| latest rejected fused trusted descent | tried fusing the non-profile App/Indir spine descent to match eval.c's `top` loop order: load the App cell once, push it, follow the fun, and only resolve indirections when encountered, instead of calling `resolve_for_whnf` and then `app_fun_trusted` on each App. The probe touched `descend_stack_from` and the outer `reduce_whnf_from_stack` descent loop only; the profiled resolver path kept the old checked/profiling-aware shape. Checks passed (`cargo fmt --check`, default `cargo check`, `cargo check --features eval-phase-profile`, `cargo test --lib` 41/41, release bench build). It did not clear the bounded gate: current post-`61eca83b` pre-probe 100M refreshes were already slow/noisy at 2502.299ms / 40.29M steps/s / 263.342ms GC and 2484.026ms / 40.58M steps/s / 268.563ms GC; the fused candidate was 2498.540ms / 40.35M steps/s / 258.457ms GC with the same 100,811,779 steps, 3 GCs, high-water 33,939,298, and sink 661902. Rejected without a full gate and reverted. Reading: the duplicate App-cell load around `resolve_for_whnf`/`app_fun_trusted` is not a standalone bottleneck, or the fused loop's code shape cancels it; do not retry as a narrow descent-source probe |
| latest rejected app-stack arg-cache probe | tried storing each App argument beside the app id in `EvalStack`, so `CHKARG`/`take_args` and stack rethreading could read args from the explicit stack instead of reloading App arg fields later. This directly targeted the F2/S1 arg-read bucket but doubled app-stack write/truncate traffic and changed descent push shape. Checks passed (`cargo fmt --check`, default `cargo check`, `cargo check --features eval-phase-profile`, `cargo test --lib` 41/41, release bench build). The 100M gate rejected it: candidate 2907.531ms / 34.67M steps/s / 294.369ms GC with the same 100,811,779 steps, 3 GCs, high-water 33,939,302, and sink 661902. Source was reverted; post-revert 100M sanity was 2632.384ms / 38.30M steps/s / 280.141ms GC. Reading: eval.c-style late `ARG(TOP(i))` loads are cheaper here than carrying a parallel arg stack; do not retry this as a narrow arg-read probe |
| latest accepted read-only memory BFILE view | committed `65c3be1d` (`Avoid copying large memory BFILE reads`). Added a C-shaped `openb_rd_mem` path for immutable byte buffers: buffers larger than 8 bytes become GC-rooted read-only BFILE views instead of copied `Vec`s, while tiny buffers keep the owned copy because the Rust view lookup loses on byte-at-a-time transducers. The same commit also captured the feature-gated stack-head arity, continuation-transition, and app-allocation operand-shape diagnostics that had been used to steer this pass. Verification passed `cargo fmt --check`, `cargo check`, `cargo test --lib` 41/41, `cargo check --features eval-phase-profile`, release bench build, `git diff --check`, 500/5000-iteration BFILE rows, and a 900s full self-host gate. Rejected precursor: direct plain-memory `getb` first-lookup branch was neutral on `bfile-read-chain:200` (5000 iters: 376.581us vs 376.823us baseline) and regressed `utf8-bfile-read-chain:200` (506.569us vs 476.256us), so it stayed reverted. Accepted long rows: `bfile-read-chain:200` 368.830us vs 376.823us no-view baseline; `utf8-bfile-read-chain:200` 463.311us vs 476.256us no-view baseline. Comparable 500-iter rows are `bfile-read-chain:200` 362.737us Rust / 252.583us C and `utf8-bfile-read-chain:200` 455.474us Rust / 306.454us C. Full gate completed byte-identically in 77.533s with 3,659,074,736 steps, 47.19M steps/s, 124 GCs, 23.322s GC pause, high-water 39,192,402 cells, SHA `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`, `cmp_exit=0`; this is a host-path canary gain and within the current 77s band, not a new best over 77.387s |
| latest current 900s refresh | current `d5efc37f` default release, rebuilt from the dirty diagnostics/WASI worktree with those native-default changes compiled out or equivalent, completed byte-identically in 75.196s with 3,659,074,831 steps, 48.66M steps/s, 124 GCs, 23.455s GC pause, high-water 39,192,412 cells, SHA `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`, `cmp_exit=0` against `/tmp/mhs-selfhost-hotcell-full.comb`; consistent with the accepted 74.668s best and not a new source checkpoint |
| latest 900s slow refresh | current `15f76b3a` default release completed byte-identically in 90.813s with 3,659,074,831 steps, 40.29M steps/s, 125 GCs, 26.457s GC pause, high-water 38,820,704 cells, SHA `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`, `cmp_exit=0`; after confirming `cargo build --release` was up to date, a 100M sanity ran in 2.473s with 100,811,779 steps, 40.77M steps/s, 3 GCs, 238.820ms GC pause, and high-water 33,941,633 cells. Treat as a slow/noisy refresh unless repeated, not a new accepted baseline |
| latest low-to-high free-list retest | retested the eval.c-like sweep order that rebuilds the intrusive free list high-to-low so allocation reuses low addresses first. Verification passed `cargo fmt --check`, `cargo check`, `cargo test --lib` 41/41, `MHS_GC_NODE_INTERVAL=32 cargo test --lib` 41/41, and release bench build. Candidate 100M was slower at 2.523s / 39.96M steps/s but lowered GC pause to 217.310ms; full gate was byte-identical in 84.969s with 3,659,074,926 steps, 43.06M steps/s, 125 GCs, 24.580s GC pause, high-water 38,820,714 cells, SHA `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`, `cmp_exit=0`. Rejected again because it misses both the 82.740s current refresh and the 80.927s best; runtime source was restored, release bench rebuilt, and post-revert 100M sanity was 2.508s / 40.20M steps/s / 239.207ms GC |
| latest dense free-slot stack probe | replaced the intrusive `Free(next)` chain with a side `Vec<NodeId>` free stack while preserving high-to-low reuse order and `free_nodes` accounting. Verification passed `cargo fmt --check`, `cargo check`, `cargo test --lib` 41/41, and release bench build; release `.text` shrank slightly to 1,098,244 bytes. Rejected before full because no-GC 10M regressed to 311.507ms / 32.19M steps/s, and 100M regressed to 2.558s / 39.41M steps/s with 383.043ms GC pause and a 209.251ms first collection. Source was restored to the intrusive free list, release bench rebuilt, and restored 100M sanity was 2.318s / 43.49M steps/s with 222.631ms GC pause |
| latest post-reorg 100M/profile refresh | after fast-forwarding and pushing `microhs-rust` to `b8284e1e`, the restored default release 100M self-host slice completed in 2.300s with 100,811,779 steps, 43.84M steps/s, 3 GCs, 227.270ms GC pause, high-water 33,939,545 cells, and sink 661902; this matches the pre-probe 2.300s control. The separate `/tmp/mhs-eval-profile-current` `eval-phase-profile` 10M run had a normal pass of 310.499ms / 32.30M steps/s and a profiled pass of 17.569s. Counters are unchanged in shape: 11,105,956 app allocations, 8,291,971 stack app updates, 30,134,034 descent pushes, 30,008,853 arg reads, 280,622 stack-loop iterations, 256,653 stack eval-step calls, 210,974 reduced steps, 45,589 WHNF exits, and 90 fallbacks. Top heads remain B/C/S-family (`B` 1.941M reductions, `C` 1.610M, `S'` 0.811M, `C'` 0.741M, `S` 0.723M, `C'B` 0.539M); top app-allocation sites remain `B.yz`, `C.xz`, `S'.left/yw/zw`, `C'.xyw/yw`, `S.right`, and `C'B.xz/yw` |
| previous current 10M phase-profile refresh | rebuilt `eval-phase-profile` in `/tmp/mhs-eval-profile-current` on accepted `d5efc37f`; normal pass was 341.923ms for 10,028,866 steps at 29.33M steps/s, no GC, high-water 11,259,777. Profiled pass was 16.246s for 10,028,866 steps, no GC, sink 661902. Counters remain B/C/S dominated: 11,105,987 app allocations, 8,292,661 stack app updates, 30,134,585 descent pushes, 30,009,541 arg reads, 280,614 stack-loop iterations, 256,660 stack eval-step calls, 210,996 reduced steps, 45,574 WHNF exits, and 90 fallbacks. Timers: `stack_eval_step_ms=16247.606`, app allocation 1231.228ms (`free_pop` check 194.158ms, fresh push 376.730ms), inner descent 249.963ms, arg reads 182.545ms, apply-app 151.123ms, rewrites 31.414ms, force-frame 6.847ms, profile-step 1755.567ms, profile-reduction 602.594ms, app-allocation bookkeeping 2500.095ms. Remaining rewrite opportunities are still mostly resolved/indirected: `red_i=149,682` mostly `<app_taken>` 68,470, `C'.yw` 34,519, `B.yz` 28,541, plus `red_k=95,180` mostly `<app_taken>` 62,010 and `IO.return.kx` 23,241, and `red_a=8,858`. Top head time is still B/C/S-family (`B` 1687.271ms, `S'` 1485.557ms, `C` 1357.851ms, `C'` 1035.357ms, `S` 746.450ms, `C'B` 745.803ms, `P` 441.887ms). Top resolved app-allocation shapes are broad `App(App) App(App)` buckets (`B.yz` 1,184,073, `C.xz` 805,201, `S'.zw` 504,141, `S'.yw` 503,777, `C'.yw` 395,092, `C'B.yw` 278,453), so there is still no hidden narrow F2 operand-shape target; the remaining wall is app allocation, update, descent, and arg-read mechanics |
| latest stack-head arity profile | added feature-gated `eval-phase-profile` counters for raw stack-head arity and under/exact/extra arity classes. `cargo fmt --check`, `cargo check`, `cargo check --features eval-phase-profile`, and `cargo test --lib` 41/41 pass. A 10M profiled self-host run shows the hot heads are almost all over-applied under long outer spines: `Prim:B@extra=1,940,011`, `Prim:C@extra=1,555,402`, `Prim:S'@extra=810,754`, `Prim:C'@extra=732,275`, `Prim:S@extra=580,185`, `Prim:C'B@extra=538,754`, `Prim:P@extra=483,673`, `Prim:K@extra=452,635`. Exact-arity stack hits are small by comparison (`C@exact=54,222`, `S@exact=32,301`, `A@exact=23,888`, `I@exact=23,845`, `P@exact=16,613`, `B@exact=3,030`), while raw top arities include long spines such as `B@319`, `B@320`, `B@321`, `S'@319`, and `K@315`. Reading: exact-arity-only shortcuts are too narrow; useful F2 work must consume/avoid work across the outer app context |
| latest stack-continuation transition profile | extended the feature-gated arity profile with `from->next@arity` counters after stack-loop continuations. `cargo fmt --check` and `cargo check --features eval-phase-profile` pass. A 10M profiled self-host run in `/tmp/mhs-eval-profile-transition` had a normal pass of 300.659ms / 33.36M steps/s and a profiled pass of 14.235s for 10,028,866 steps. The biggest continuation buckets are strict/scalar returns (`Prim:==->Int@0=166,848`, `Prim:u<=->Int@0=52,455`, `Prim:I->Int@0=23,365`, `Prim:chr->Int@0=23,145`) plus some B/S-family long-spine transitions (`Prim:B->Prim:B@319=24,664`, `Prim:B->Prim:B@320=21,317`, `Prim:S'->Prim:S'@320=13,296`, `Prim:S'->Prim:S'@319=12,196`). Reading: direct long-spine B/S' superinstructions are present but not broad enough by themselves; the next eval.c-shaped scalar candidate is to make strict `Int` immediate fast paths follow already-resolved indirections like eval.c's `T_BININT1` return path, not to add another exact nested-head branch |
| latest app-allocation site-shape profile | extended `eval-phase-profile` with top `site: fun-shape arg-shape` counters for app allocation operands. `cargo fmt --check`, default `cargo check`, `cargo check --features eval-phase-profile`, and an isolated `/tmp/mhs-app-shape-profile` release build pass. A 10M profiled self-host run had a normal pass of 312.671ms / 32.07M steps/s and a profiled pass of 15.187s for 10,028,866 steps. Top sites are still `B.yz=1,943,041`, `C.xz=1,609,624`, `S'.left/yw/zw=810,754` each, `C'.xyw/yw=741,069` each, and `S.right=612,486`. The new shape split is diffuse: largest buckets are `B.yz: App(App) App(App)=627,214`, `C.xz: App(App) Indir=559,567`, `B.yz: App(App) Indir=366,058`, `S'.yw: App(App) App(App)=328,170`, `C'.yw: App(App) Indir=323,191`, `S'.zw: App(App) App(App)=312,596`, and then many 80k-230k App/Indir/Int variants. Reading: app allocation remains the wall, but operand shapes are not concentrated enough for another direct special-form branch; useful work has to reduce the general app allocation/update/descent layer or change GC/layout, not specialize one shape |
| latest strict-Int indirection immediate probe | tried the scalar candidate from the continuation profile by making the active stack `IntBin` immediate fast path follow an initial `Indir` chain to an already evaluated `Int`, copying eval.c's strict `T_BININT1` return detail without compressing or allocating. `cargo fmt --check`, `cargo check`, `cargo check --features eval-phase-profile`, `cargo test --lib` 41/41, and an isolated release build in `/tmp/mhs-strict-int-indir` passed. Bounded gates looked good enough for full: 10M 293.296ms / 34.19M steps/s / no GC, high-water 11,259,769; 100M 2.243s / 44.95M steps/s / 215.303ms GC, high-water 33,939,308. The 900s full gate was byte-identical but regressed to 83.294s with 3,659,074,850 steps, 43.93M steps/s, 124 GCs, 24.765s GC pause, high-water 39,192,369 cells, SHA `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`, `cmp_exit=0`; source was reverted. Reading: strict `Int@0` transitions are real, but following indirections in this hot check adds enough full-workload code shape/branch cost to lose; do not retry as a standalone scalar helper |
| latest direct B/B superinstruction probe | tried fusing the hot `B` arm for direct `B (B a b) y z` into `a (b (y z))`, preserving allocation order for `y z` then `b (y z)` and charging two reductions. Verification passed `cargo fmt --check`, `cargo check`, `cargo check --features eval-phase-profile`, `cargo test --lib` 41/41, and an isolated release build in `/tmp/mhs-b-super`. Bounded gates looked plausible: 10M 290.630ms / 34.51M steps/s / no GC, high-water 11,259,751; 100M 2.265s / 44.50M steps/s / 222.118ms GC, high-water 33,939,290. The 900s full gate was byte-identical but regressed to 79.157s with 3,659,074,679 steps, 46.23M steps/s, 124 GCs, 23.604s GC pause, high-water 39,192,396 cells, SHA `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`, `cmp_exit=0`; source was reverted. Reading: a direct-shape nested-B fast path is still too narrow and adds enough hot-branch/code-shape cost to lose against the 77.387s/77.918s current band |
| latest raw app-stack push probe | changed `EvalStack::push_app` from `Vec::push` to an inline raw write with a cold grow path, aiming at eval.c's stack-pointer discipline and the 30.1M/10M descent-push bucket. `cargo fmt --check`, `cargo check`, `cargo test --lib` 41/41, and an isolated release build in `/tmp/mhs-stack-push` passed. Rejected before full: 10M regressed to 317.355ms / 31.60M steps/s, and 100M regressed to 2.566s / 39.29M steps/s with 244.617ms GC pause and high-water 33,939,296. Source was reverted. Reading: Rust's `Vec::push` codegen is not the isolated app-stack cost; the custom cold-grow branch/layout was worse |
| latest batched app-allocation accounting probe | tried moving hot multi-app rewrites (`S`, `S'`, `B'`, `C'`, `C'B`, and `IO.>>`) toward eval.c's `GCCHECK(k)` shape by charging `gc_allocations_since_collect` once per known app batch and using a pre-accounted app allocator. `cargo fmt --check`, `cargo check`, `cargo test --lib` 41/41, and an isolated release build in `/tmp/mhs-batch-account` passed. Bounded was mixed: 10M regressed to 305.641ms / 32.81M steps/s, while 100M looked plausible at 2.295s / 43.94M steps/s with 219.257ms GC pause. The 900s full gate was byte-identical but regressed to 78.758s with 3,659,074,793 steps, 46.46M steps/s, 124 GCs, 23.547s GC pause, high-water 39,192,408 cells, SHA `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`, `cmp_exit=0`; source was reverted. Reading: removing a few saturating counter updates is not enough, and batching can perturb allocation/counter timing without beating the full gate |
| latest accepted aborting release panic profile | committed `c62cef08` (`Use aborting release panics`). The workspace release profile now uses `panic = "abort"`, which shrinks the default release bench `.text` from 1,175,135 bytes to 1,100,876 bytes and moves the self-host gate below 80s. Because the runtime previously relied on `std::panic::catch_unwind` to convert malformed `lzma-sdk-rs` raw decoder panics into `InvalidByteString`, the commit first replaced that call with a private checked raw-LZMA decoder that uses fallible indexing/allocation and returns `InvalidByteString` on malformed streams; the dependency no longer enables its `decode` feature. Verification passed `cargo fmt --check`, `cargo check`, `cargo test --lib` 41/41, `cargo check --features eval-phase-profile`, `cargo check --features gc-phase-profile`, `cargo check --release --target wasm32-unknown-unknown`, release bench build, symbol/size checks, 10M/100M bounded gates, `git diff --check`, and two normal 900s full gates. Normal bounded gates were 10M 284.089ms / 35.30M steps/s / no GC and 100M 2.306s / 43.72M steps/s / 216.138ms GC, high-water 33,939,298 cells. Full gates were byte-identical at 78.857s and 77.387s with 3,659,074,755 / 3,659,074,774 steps, 46.40M / 47.28M steps/s, 124 GCs, 23.824s / 23.256s GC pause, high-water 39,192,404 / 39,192,406 cells, SHA `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`, `cmp_exit=0`. The initial out-of-tree `RUSTFLAGS=-Cpanic=abort` scout also completed byte-identically in 79.686s, but was not accepted until the LZMA panic-catch dependency was removed |
| latest accepted inline primitive cell decoding | committed `a82740a8` (`Inline primitive cell decoding`). `Cell::prim` and `decode_known_prim` are now `#[inline(always)]`, recovering a small source-level piece of the PGO finding that `decode_known_prim` was still an 8.2% hot-cluster function; `nm -C target/release/mhs-rust-bench` no longer emits standalone `Cell::prim` or `decode_known_prim` symbols. Verification passed `cargo fmt --check`, `cargo check`, `cargo test --lib` 41/41, release bench build, symbol check, 10M/100M bounded gates, current `eval-phase-profile` 10M refresh, `git diff --check`, and two 900s full gates. Bounded 10M was noisy/slower at 320.243ms / 31.32M steps/s with 10,028,866 steps and 11,259,759 high-water cells; bounded 100M was plausible at 2.338s / 43.13M steps/s, 3 GCs, 227.543ms GC pause, high-water 33,939,298 cells. Full gates were byte-identical at 80.309s and 80.198s with 3,659,074,755 / 3,659,074,793 steps, 45.56M / 45.63M steps/s, 124 GCs, 23.819s / 23.760s GC pause, high-water 39,192,404 / 39,192,408 cells, SHA `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`, `cmp_exit=0`. Accepted as the current default-release best, a small but repeated source gain over `4cc9879a` 80.584s / 81.336s |
| latest forced app-allocation helper inline probe | tried changing `push_app_node` from `#[inline]` to `#[inline(always)]` after default release still emitted dozens of hot `call push_app_node` sites from the stack rewrite body. Verification passed `cargo fmt --check`, `cargo check`, `cargo test --lib` 41/41, release bench build, and symbol check; `push_app_node` disappeared as a standalone symbol and `.text` grew by about 4 KiB. Bounded 10M was unchanged/noisy at 320.078ms / 31.33M steps/s, but bounded 100M regressed to 2.453s / 41.09M steps/s with 249.293ms GC pause and high-water 33,939,300 cells versus the accepted inline-decoder 100M at 2.338s / 43.13M steps/s / 227.543ms GC. Rejected without a full gate, reverted, and release bench rebuilt; reading: app allocation remains the top bucket, but force-inlining the allocator helper perturbs layout/register pressure more than it saves in call overhead |
| latest forced descend helper inline probe | tried changing `descend_stack_from` to `#[inline(always)]` to capture the PGO-shaped cross-function adjacency/inline hint; release symbol checks confirmed the standalone helper and `call descend_stack_from` sites disappeared, while `.text` grew to 1,217,847 bytes, about +42 KiB versus the restored default binary. Verification passed `cargo fmt --check`, `cargo check`, `cargo test --lib` 41/41, release bench build, and symbol/call checks. Bounded 10M regressed to 377.810ms / 26.54M steps/s with unchanged 10,028,866 steps and no GC; bounded 100M regressed to 2.486s / 40.54M steps/s with 226.284ms GC pause and high-water 33,939,300 cells versus the accepted inline-decoder 100M at 2.338s / 43.13M steps/s / 227.543ms GC. Rejected without a full gate and reverted; reading: PGO's descent win is not recoverable by blanket source inlining because the code-size/layout cost outweighs the call removal |
| latest accepted app-allocation bookkeeping cold split | committed `4cc9879a` (`Cold-split app allocation profiling bookkeeping`). The hot `app`/`app_with_site` helpers now keep only a `profile.is_some()` guard in the inline default path and move the profiling HashMap/String bookkeeping to a `#[cold] #[inline(never)]` helper, matching the `NOTES.md` codegen-debt recommendation without changing graph semantics. Verification passed `cargo fmt --check`, `cargo check`, `cargo test --lib` 41/41, release bench build, 10M/100M bounded gates, `git diff --check`, and two 900s full gates. Bounded 10M stayed at 10,028,866 steps and 11,259,767 high-water cells, running in 280.505ms / 35.75M steps/s. Bounded 100M stayed at 100,811,779 steps, ran in 2.352s / 42.87M steps/s, 3 GCs, 232.204ms GC pause, high-water 33,939,306 cells. Full gates were byte-identical at 80.584s and 81.336s with 3,659,075,040 / 3,659,075,173 steps, 45.41M / 44.99M steps/s, 124 GCs, 23.888s / 23.885s GC pause, high-water 39,192,434 / 39,192,448 cells, SHA `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`, `cmp_exit=0`. Accepted as a current-source gain versus `576d6407` 82.123s / 82.114s and as a small new best over the prior 80.927s default gate, though the repeat is in the same noise band as the old best |
| latest direct fresh-App allocation probe | added an `#[inline(always)]` fresh App helper to bypass generic `push_cell` for `Cell::app` and replace the high-water `max` with direct monotonic assignment. Verification passed `cargo fmt --check`, `cargo check`, `cargo test --lib` 41/41, and release bench build. Bounded gates looked plausible: 10M 336.697ms / 29.79M steps/s, 100M 2.477s / 40.70M steps/s / 244.844ms GC. Full gate was byte-identical but regressed to 88.846s with 3,659,074,869 steps, 41.18M steps/s, 125 GCs, 26.996s GC pause, high-water 38,820,708 cells, SHA `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`, `cmp_exit=0`; rejected and reverted, release bench rebuilt |
| latest runtime strict-action noinline probe | changed `RuntimePrim::strict_action` from `#[inline(always)]` to `#[inline(never)]` to shrink `stack_eval_step` by moving the runtime-primitive range/table classifier out of the hot evaluator body. Verification passed `cargo fmt --check`, `cargo check`, `cargo test --lib` 41/41, and release bench build. Bounded gates were misleadingly strong: 10M 280.325ms / 35.78M steps/s, 100M 2.200s / 45.83M steps/s / 222.790ms GC, high-water 33,941,623 cells. Full gate was byte-identical but regressed to 86.214s with 3,659,074,793 steps, 42.44M steps/s, 125 GCs, 26.095s GC pause, high-water 38,820,700 cells, SHA `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`, `cmp_exit=0`; rejected and reverted. Post-revert release bench rebuilt; restored 100M sanity was 2.295s / 43.93M steps/s / 229.418ms GC |
| latest constructor-time redex compression probe | tried the stack-profile hint that runtime app constructions create immediate `I`/`K`/`A` redexes. Broad `I x` plus `(K x) y`/`(A x) y` compression failed the shape gate: `cargo test --lib` was 40/41 because `B' I I 9` serialized as `((B I) 9)` instead of the byte-parity shape `((B (I I)) 9)`. Narrow K/A-only compression passed `cargo fmt --check` and `cargo test --lib` 41/41, but bounded gates were bad: 10M 370.005ms / 27.10M steps/s with only 1,355 fewer steps, 100M 2.585s / 38.98M steps/s / 231.945ms GC with 100,790,913 steps and high-water 33,939,103 cells. Rejected and reverted; release bench rebuilt, post-revert 100M sanity was 2.422s / 41.63M steps/s / 227.065ms GC |
| latest site-specific app-redex profile | added `eval-phase-profile` site labels for stack app rewrite opportunities, keeping aggregate `red_*` counters and adding `red_*@site` keys. Verification passed `cargo fmt --check`, `cargo check --features eval-phase-profile`, `cargo test --lib` 41/41, `eval-phase-profile` release bench build in `/tmp/mhs-eval-profile-target`, default release bench build, and a 900s default full gate. Feature normal 10M was 314.618ms / 31.88M steps/s; profiled 10M pass was 11.221s for 10,028,866 steps. The site split explains why broad constructor-time compression is the wrong shape: `red_i=259,585` is mostly `S.left` 109,903, `<app_taken>` 68,470, `C'.yw` 34,519, `B.yz` 28,541, and `S.right` 16,708; `red_k=95,180` is mostly `<app_taken>` 62,010, `IO.return.kx` 23,241, `B.yz` 5,457, and `P.zx` 3,637; `red_a=8,858` is all `<app_taken>`. Tiny tails remain `red_bi=33` at `C.xz` and `red_bxi=1` at `B.yz`. Default full gate with the profiling hook compiled out was byte-identical in 84.303s with 3,659,074,888 steps, 43.40M steps/s, 125 GCs, 25.557s GC pause, high-water 38,820,710 cells, SHA `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`, `cmp_exit=0`; diagnostic only, not a new best |
| latest app-taken immediate function-redex shortcut probe | tried a narrower, byte-shape-safer version of constructor-time compression only for `app_taken` results placed in function position, where direct `I`, `(K x)`, or `(A x)` would be reduced immediately by the existing stack loop. The shortcut was budget-guarded and skipped self-indirections; `cargo fmt --check`, `cargo check`, and `cargo test --lib` 41/41 passed. It did not fire in practice on the default bounded slices: 10M kept the same 10,028,866 steps but regressed to 381.639ms / 26.28M steps/s, and 100M kept the same 100,811,779 steps but regressed to 3.011s / 33.48M steps/s / 271.203ms GC. Rejected and reverted; release bench rebuilt, post-revert 100M sanity was a slow/noisy 2.808s / 35.90M steps/s / 257.610ms GC |
| latest accepted S-I stack rewrite specialization | committed `576d6407` (`Specialize S I stack rewrite`). The hot stack `S` arm now handles the site-profiled `S.left` case directly when `x` is the primitive `I` and the reduction budget can account for the immediately skipped `I z` step: `S I y z -> z (y z)`. Verification passed `cargo fmt --check`, `cargo check`, `cargo test --lib` 41/41, release bench build, 10M/100M bounded gates, and two 900s full gates. Bounded 10M stayed at 10,028,866 steps but dropped high-water from the profile baseline 11,369,662 cells to 11,259,755 cells and ran in 288.908ms / 34.71M steps/s. Bounded 100M stayed at 100,811,779 steps, ran in 2.376s / 42.43M steps/s, 3 GCs, 226.432ms GC pause, high-water 33,939,294 cells. Full gates were byte-identical at 82.123s and 82.114s with 3,659,074,717 / 3,659,074,850 steps, 44.56M steps/s, 124 GCs, 24.013s / 24.018s GC pause, high-water 39,192,400 / 39,192,414 cells, SHA `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`, `cmp_exit=0`. Accepted as a current-source gain versus `cf45800c` 84.303s and `15f76b3a` 82.740s; still misses the 80.927s all-time default best |
| latest B-I stack rewrite probe | tried the eval.c-GCRED-aligned `B x I z -> x z` specialization in the hot stack `B` arm, after `red_i@B.yz` showed 28,541 opportunities in the 10M site profile. Verification passed `cargo fmt --check`, `cargo check`, and `cargo test --lib` 41/41. Bounded 10M stayed at the accepted S-I heap shape but slowed to 298.635ms / 33.58M steps/s. Bounded 100M reduced steps slightly to 100,810,114 but regressed to 2.639s / 38.20M steps/s / 233.162ms GC. Rejected and reverted without a full gate; release bench rebuilt, post-revert 100M sanity was slow/noisy at 2.923s / 34.49M steps/s / 281.322ms GC |
| latest IO.return K continuation probe | tried the post-S-I `red_k@IO.return.kx` hint by specializing `IO.return x world K -> x` in the hot stack `IO.return` arm, charging two reductions for the skipped `IO.return` and `K x world` work. Verification passed `cargo fmt --check`, `cargo check`, `cargo test --lib` 41/41, and release bench build. Bounded 10M looked locally good at 277.055ms / 36.20M steps/s with high-water 11,236,587 cells; bounded 100M was mixed/slower at 2.566s / 39.28M steps/s, 3 GCs, 248.926ms GC pause, high-water 33,939,838 cells. The 900s full gate was byte-identical but regressed to 88.467s with 3,659,074,907 steps, 41.36M steps/s, 124 GCs, 25.405s GC pause, high-water 39,212,260 cells, SHA `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`, `cmp_exit=0`; rejected and reverted |
| latest C' direct identity probe | tried the post-S-I `red_i@C'.yw` hint by specializing direct `C' x I z w -> x w z` in the hot `C'` arm, saving the `I w` app only when `y` is directly the primitive `I`. Verification passed `cargo fmt --check`, `cargo check`, `cargo test --lib` 41/41, and release bench build. Bounded 10M barely fired: same 10,028,866 steps, only 18 fewer high-water cells, 280.431ms / 35.76M steps/s. Bounded 100M removed only 37 steps and regressed to 2.456s / 41.05M steps/s, 3 GCs, 232.887ms GC pause, high-water 33,939,842 cells. Rejected and reverted without a full gate; release bench rebuilt. Reading: like `app_taken`, the site-profiled opportunity is mostly resolved/indirected, so direct hot-branch checks just add code shape for too little graph saving |
| latest raw known-primitive dispatch probe | tried giving the hot `stack_eval_step` path a raw `CellTag::KnownPrim` fast path so ordinary B/C/S/K/etc. heads avoid constructing the intermediate `Prim`/`EvalHead` wrapper before dispatch; cold runtime/FFI/JS fallback arms stayed behind the existing `EvalHead` path. `cargo fmt --check`, default `cargo check`, `cargo check --features eval-phase-profile`, and release bench build passed. Bounded 10M was mediocre at 320.388ms / 31.30M steps/s with 10,028,866 steps and no GC; bounded 100M looked plausible at 2.085s / 48.36M steps/s, 3 GCs, 220.983ms GC pause, high-water 33,939,290. The 900s full gate rejected it: byte-identical output, 79.810s, 3,659,074,679 steps, 45.85M steps/s, 124 GCs, 26.051s GC pause, high-water 39,192,396, SHA `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`, `cmp_exit=0`. Source was reverted and the release bench rebuilt. Reading: the compiler was already doing enough here, and moving the cold arms under an explicit branch worsens full-gate code shape despite a good 100M slice |

## Open Work

| area | state |
|---|---|
| evaluator/S1/F2 | normal WHNF reduction now uses a plain app-id stack plus separate strict frames for the hot combinator, strict-primitive, and fallback primitive-helper paths; hot fixed-arity combinator and IO graph-rewrite arms pop/take stack args in eval.c `CHKARGn` style, read stack args with unchecked eval.c-style `ARG(TOP(i))` loads, trust the checked stack segment when fetching redex apps for hot rewrites/updates, use trusted non-profile WHNF resolution, trusted app-fun descent, and trusted hot reducer cell reads for internal reducer roots, test hot `Cell` tags through raw low bits, descend app spines by reading only the app fun/tag word before later `CHKARG` arg loads, overwrite the consumed app redex cell directly, continue through `GOAP`/`GOAP2`-style app results using direct `ap`/`ap2` root pushes, continue through strict force markers and K/A/I/Ord/Chr-family `GOIND` rewrites inside `stack_eval_step`, and allocate `Node::App` through a dedicated hot helper/direct trusted free-cell reuse path instead of generic `push_node`; reused app allocation now also trusts the `free_nodes > 0 => free_head is Some` invariant in release; strict `Int` binops and WHNF frames for `seq`/`IO.strict`/`isint` now pop the consumed redex before forcing and carry that redex as a frame root; `IO.strict` reuses the redex as the returned `action value` app instead of allocating a new app plus indirection; scalar strict-frame results overwrite the consumed redex cell instead of allocating a scalar node plus indirection; the hot stack path no longer emits a `StackStep::Force` handoff; strict-redex snapshots and `remaining_apps_contain` scans are zero in the 1M profile; remaining bridge hits are std-handle/control helpers |
| correctness/F batch | F1/F3/F4/F6-sharing/F6-`IO.deserialize`/F7/F10-deep-serializer/F10-BFILE-print-serialize/F11/F12-catch-masking/F14-ForeignPtr-finalizers/F15/F16/F17/F18/F19/F21 are implemented or verified in the current tree; F20's std-handle flush-per-write issue no longer exists because writes and explicit flushes are separate; weak pointers now carry eval.c-style keys, only keep values/finalizers alive when the key is marked, clear dead-key values during GC, and evaluate stored finalizer actions at a GC safe point; scheduler-backed weak finalizer thread spawning and broader scheduler/thread primitives remain later work |
| parser/S3 | primitive occurrences and parsed small `Int` literals are interned at parse time, matching eval.c's singleton primitive and permanent small-int table model more closely |
| GC/F5 | safe-point mark/sweep plus S2 allocation-pressure trigger, intrusive free list, mark-time indirection compression, reusable mark-work stack, parse-label non-root treatment, optional `gc-phase-profile` mark/sweep timings and young-region viability counters, C-style sweep-time ForeignPtr finalizers, eval.c-style weak key/value clearing, and the `moving-gc` NodeId remapper scaffold are implemented; full gates pass, but scheduler-backed weak finalizer spawning, actual minor collection, and any future selective GCRED/mark-slot semantics are still pending |
| representation/S3 | payload-out plus `NodeId` narrowing and compact runtime-primitive tags is now an authoritative 16-byte cell arena, not a mirrored side table; strict primitive classification dispatches directly from in-cell tags, cold payloads sit in a side table, and `Node` survives only as parser/debug/serialization interchange |
| JS FFI / WASI | browser wasm shim smokes pass; Rust `wasm32-wasip1` bench builds and core scenarios run under Node WASI with stubbed `mhs_js_*`. The isolated `target_os = "wasi"` env/path/native-BFILE cfg clears the old `setenv` ENOSYS blocker and reaches 650k self-host reductions by default; default Node WASI still exits 139 above that before the normal 32M allocation-triggered GC, while `MHS_GC_NODE_INTERVAL=500000` carries the same workload to 1M reductions with 2 GCs. Reading: host syscall parity moved forward; the next WASI blocker is memory-growth/GC cadence or the Node WASI engine, not env/file stubs. Keep the WASI path target-specific so native self-host stays in the 77s band |
| `IO.serialize`/`IO.deserialize`/sharing/cycles | shared app graphs and cyclic app graphs now emit C-style `:label`/`_label` output using `NodeId` labels and round-trip through the existing parser; the serializer itself is now iterative and passed a 12k-deep WHNF app-chain smoke; `IO.serialize` now writes to arbitrary BFILE destinations instead of std handles only; `IO.deserialize` now reads one comb from a BFILE, appends/remaps it into the live arena, preserves stream position after the parsed `}`, and keeps shared cycles intact |
| self-hosting | full compile succeeds and matches the C-generated comb byte-for-byte; best default-release gate is committed `d5efc37f` at 74.668s versus fresh C oracle 47.85s, about 1.56x C. The accepted trusted hot-cell signal is an 8x alternating 100M A/B, base avg 2050.098ms vs candidate avg 2012.507ms (`-1.83%`) with identical heap shape, plus the byte-identical 74.668s full gate. Prior `c969e993` trusted app-fun descent gate was 76.369s, about 1.60x C, with an 8x alternating 100M A/B at `-5.06%`. Prior `3c04e5c1` trusted WHNF resolver gate was 77.050s, with a noisy repeat at 83.241s and an isolated full base/candidate pair at 84.432s vs 84.146s. Prior clean `65c3be1d` archive best was 77.359s, about 1.62x C; previous `c62cef08` best was 77.387s. `15f76b3a` self-host-trained PGO remains the best measured execution mode at 72.387s, about 1.51x C, with a 72.160s repeat; the neutrality harness gives a balanced 100M same-source noise floor of base 2.381s vs candidate 2.421s (`+1.71%`) with identical heap shape |

Smoke-only subsystems now stay out of the live table unless they regress:
MVar, StablePtr, Weak, ForeignPtr, MD5, compression, process/env/filesystem,
BFILE codecs, and broad FFI coverage.

## Latest Common/Canary Refresh

Current default release `target/release/mhs-rust-bench`, 500 iterations with 50
warmups per row, compared against `./bin/mhsbench --mode whnf`. This is not the
self-host arbiter, but it refreshes the non-self-host canaries after the latest
F2 profiling pass.

| scenario | Rust ns/iter | C ns/iter | ratio | sink |
|---|---:|---:|---:|---|
| `identity-chain:1000` | 24,495 | 118,323 | 0.21 | match |
| `arith-chain:200` | 11,356 | 111,346 | 0.10 | match |
| `io-control-chain:200` | 29,404 | 125,892 | 0.23 | match |
| `performio-apply-chain:200` | 23,329 | 118,181 | 0.20 | match |
| `stdio-chain:200` | 73,184 | 118,909 | 0.62 | match |
| `ffi-chain:200` | 47,384 | 153,375 | 0.31 | match |
| `ffi-wide-mem-chain:100` | 171,567 | 234,705 | 0.73 | match |
| `ffi-word-mem-chain:100` | 171,751 | 262,580 | 0.65 | match |
| `ffi-ptr-mem-chain:100` | 134,733 | 228,585 | 0.59 | match |
| `ffi-strcpy-chain:100` | 233,908 | 277,829 | 0.84 | match |
| `ffi-mem-chain:200` | 314,041 | 267,950 | 1.17 | match |
| `bfile-read-chain:200` | 362,737 | 252,583 | 1.44 | match |
| `utf8-bfile-read-chain:200` | 455,474 | 306,454 | 1.49 | match |
| `zoo-chain:300` | 26,700 | 119,996 | 0.22 | match |
| `data-chain:300` | 30,450 | 137,052 | 0.22 | match |

Reading: scalar, graph, ordinary IO, and most FFI rows are now substantially
faster than C in the small harness. The committed read-only memory BFILE view
improves the BFILE rows in absolute Rust time, especially `utf8-bfile-read`, but
`ffi-mem-chain`, `bfile-read-chain`, and `utf8-bfile-read-chain` remain the
non-self-host canaries for host-buffer/BFILE work because their same-run ratios
are still above C.

## Latest Browser-Path Wasm Comparison

Node check of the browser-shaped artifacts, 2026-07-05:
`rust/microhs-runtime/tools/wasm/browser/build-browser-bench.sh` built Rust
`target/wasm32-unknown-unknown/release/microhs_runtime.wasm` and an ignored
Emscripten ES module `rust/microhs-runtime/tools/wasm/browser/browser-bench-c.mjs` from
`src/runtime/mhsbench.c` using `/home/tritlo/emsdk`. The runner is
`node rust/microhs-runtime/tools/wasm/browser/browser-compare.mjs --iters 1000 --warmup-iters 100
--json`.

Scope: this first table compares the browser cdylib/evaluator path only:
parse + WHNF reduction + MicroHs serialization, equivalent to C
`mhsbench --mode whnf`. Full self-host numbers are in the table below.

| scenario | Rust wasm ns/iter | C Emscripten ns/iter | ratio | sink |
|---|---:|---:|---:|---|
| `identity-chain:1000` | 49,828 | 540,833 | 0.09 | match |
| `arith-chain:200` | 19,538 | 467,174 | 0.04 | match |
| `io-control-chain:200` | 49,291 | 493,622 | 0.10 | match |
| `performio-apply-chain:200` | 41,081 | 449,596 | 0.09 | match |
| `ffi-mem-chain:200` | 501,184 | 729,255 | 0.69 | match |
| `bfile-read-chain:200` | 613,290 | 849,983 | 0.72 | match |
| `zoo-chain:300` | 45,474 | 537,900 | 0.08 | match |
| `data-chain:300` | 52,938 | 439,897 | 0.12 | match |

Reading: the Rust browser wasm path is ahead on the same small evaluator rows
where native Rust already beats C. The close rows are still host-buffer/BFILE
shapes (`ffi-mem-chain`, `bfile-read-chain`).

Self-host check of the same browser-shaped artifacts, 2026-07-05:

| target | command | status | wall | output |
|---|---|---|---:|---|
| Rust `wasm32-unknown-unknown` browser cdylib | `timeout 900s node rust/microhs-runtime/tools/wasm/browser/browser-selfhost.mjs --target rust` | completes from JS-backed host env/filesystem | 180.627s | output 661,784 bytes, SHA `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`, byte-identical to the reference |
| C `eval.c` Emscripten ES module | `timeout 900s node rust/microhs-runtime/tools/wasm/browser/browser-selfhost.mjs --target c` | completes from MEMFS | 76.558s | output 661,786 bytes, SHA `4ecfc72e4e9ba3e071c13791b7018b14570f717fe78db6a05c2dab2f1d05fe2a`, not byte-identical to native/reference SHA `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`; this is the known eval.c wasm32 `Int` wrap/float divergence in `WRAPBUG.md`, not a harness crash |

Reading: Rust wasm is correct but about 2.36x slower than the C wasm self-host
wall clock in Node on this run. The earlier `setenv` blocker is gone: Rust now
uses a JS-backed browser host env/filesystem and writes the compiler output
through that layer. The C wasm speed number is useful for throughput comparison,
but its output is not a correctness oracle because of `WRAPBUG.md`.

## Common Performance Snapshot

Older same-machine measurements after 32-byte nodes, lazy spine arguments,
borrowed parser tokens, and large-input parser preallocation. Ratios and sink
agreement are more useful than raw timings; this table is historical now that
the targeted current refresh above exists.

| scenario | Rust ns/iter | C ns/iter | ratio | note |
|---|---:|---:|---:|---|
| `identity-chain:1000` | 29,126 | 131,113 | 0.22 | graph |
| `arith-chain:200` | 30,662 | 112,936 | 0.27 | scalar |
| `int64-chain:200` | 33,380 | 109,178 | 0.31 | scalar |
| `float64-chain:200` | 37,281 | 129,420 | 0.29 | scalar |
| `bytes-chain:200` | 44,445 | 174,776 | 0.25 | bytes |
| `io-control-chain:200` | 48,350 | 140,716 | 0.34 | IO control |
| `performio-apply-chain:200` | 38,738 | 122,949 | 0.32 | IO/apply |
| `stdio-chain:200` | 91,552 | 129,399 | 0.71 | near C |
| `ffi-mem-chain:200` | 353,977 | 255,405 | 1.39 | regressed: long FFI spine |
| `bfile-read-chain:200` | 515,365 | 298,123 | 1.73 | regressed: long FFI/BFILE spine |
| `zoo-chain:300` | 54,668 | 145,592 | 0.38 | mixed graph |
| `data-chain:300` | 47,545 | 155,562 | 0.31 | graph |
| self-host `--help` proxy | 40,438,925 | 12,548,156 | 3.22 | sink `1,985,706` both sides |

Small scalar and mixed rows still mostly beat C. The old F2 worktree regressed
long FFI/BFILE continuation rows because the persistent path kept large spines
live across those calls; remeasure those rows after the S1 stack settles. The
full self-host path has semantic parity against the C-generated compiler comb:
Rust produces byte-identical output, and the best S1/S2/S3/F14/S1-ret-top/S2-label-root/free-head/identity-GOIND/one-word-app-fun/app-edge-canonicalization/trusted-stack-update/permanent-compound
worktree now completes in 80.927s. A fresh C oracle full self-host run completed in
47.85s, allocated 4,131,058,560 cells, performed 3,651,953,094 reductions at
76.3 Mred/s, ran 89 GCs with 11.64s total GC time, and produced the same
`29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a` output
SHA; the best Rust full gate is now 77.387s, about 1.62x slower, while the
previous accepted default checkpoint was 80.309s / 80.198s. A low-to-high
free-list reuse probe completed byte-identically in 81.622s and 81.277s,
lowering GC pause to 23.213s/23.120s but still missing the 80.927s best, so it
was reverted. The FFI/JS arity-prefix materialization retest dropped 10M
profiled materialized arg nodes from 10,522,021 to 150,988 and completed a
byte-identical full gate in 83.510s, but still missed the best and was reverted.
The current eval.c `GOAP2` direct-push retest improved the bounded 100M slice to
2.369s but regressed the byte-identical full gate to 86.784s with 25.596s GC
pause, so it was reverted; post-revert 100M sanity was 2.357s, and the latest
default rebuild refresh is 10M 358.767ms / 100M 2.354s / full 83.062s with
225.909ms 100M GC pause and 24.354s full GC pause.
The app-allocation split instrumentation full gate then completed
byte-identically in 82.949s with 44.11M steps/s and 24.396s GC pause, confirming
the profiling-only default build is correct but not a new best. The GCRED
opportunity instrumentation then found zero eval.c-style mark-reduction
opportunities on the 100M GC-profile gate, and the default full gate remained
byte-identical at 82.693s, so broad GCRED is not the next self-host throughput
lever.
The F10 BFILE `IO.print`/`IO.serialize` correctness gate completed
byte-identically in 82.518s, and the S4 young-region viability counters
completed byte-identically in 82.817s. The actual S4 allocation-set minor
collector prototype then rejected the idea as implemented: the best full gate
with lazy minor-free slots at 8M minor / 128M full completed byte-identically
in 113.490s, 32.24M steps/s, 501 GCs, 22.471s GC pause, and 17.52M high-water
cells. Lower heap and pause did not pay for the write barrier, allocation-set
tracking, and minor free-slot churn.
The `IO.deserialize` parity slice then enabled the remaining serializer-side IO
primitive: it reads one serialized comb from a BFILE, appends/remaps parsed
nodes into the live arena, preserves unread bytes after the parsed `}`, and
keeps shared cycles intact. It passed wasm/release/library gates and completed
the full self-host gate byte-identically in 82.870s with 24.037s GC pause; this
is accepted as correctness parity, not a new performance best.
A fresh `fe36f0e6` HEAD refresh after the benchmark-number audit completed
byte-identically in 82.662s, 44.27M steps/s, with 125 GCs and 23.988s GC pause.
The bounded refresh was 358.467ms at 10M and 2.513s at 100M with 219.225ms GC
pause; this is current status, not a new best versus the 80.927s permanent
compound-cache gate.
The refreshed PGO experiment on `15f76b3a` trained on the full self-host
workload and produced the first stable low-72s Rust gate: 72.387s, 50.55M
steps/s, byte-identical output, and 25.933s GC pause, with a 72.160s repeat.
This strengthens `NOTES.md`'s codegen-control thesis: mutator layout can buy
more than another small local source probe, even though GC pause stays worse
than the default-release best.
The weak-pointer GC semantics slice then closed the eval.c key/value clearing
gap without becoming a perf win: the accepted side-list version completed
byte-identically in 82.185s with 24.129s GC pause after an initial whole-arena
weak sweep regressed to 87.293s and 29.521s GC pause. Keep the eval.c lesson:
weak records need a side list; scanning the whole arena for a rare cold payload
is too expensive even when self-host allocates no weak pointers.
A 100M `eval-phase-profile` refresh on `f870b8ff` confirms the F2/S1 ranking:
117.4M app allocations dominate the measured profile, concentrated in B/C/S
rewrite sites; inner descent and arg reads are the next tier, while fallback
dispatch is still negligible. The profiled run itself took 100.810s because the
profiling bookkeeping is large, so use it for ranking, not direct timing.
The follow-up 10M stack rewrite argument/opportunity profile shows the same
B/C/S allocation wall, but with a more precise direction: runtime app
constructions frequently create immediate `I`/`K`/`A` redexes (`red_i=259,585`,
`red_k=95,180`, `red_a=8,858` on 10M), while `B I` and `B x I` are essentially
absent (`33` and `1`). That points to a selective constructor-time redex
compression probe, not broad GCRED or another B-family-only rewrite.
The self-host neutrality harness is now committed at
`rust/microhs-runtime/tools/native/bench-selfhost-neutrality.sh`. It builds base and
candidate benches in isolated target dirs, supports repeats/full gates,
alternates run order, and uses equal-length `base-*`/`cand-*` output labels so
the compiler input string does not change heap shape. The first balanced 100M
same-source control is base 2380.626ms vs candidate 2421.441ms (`+1.71%`) with
identical steps, GC count, high-water, and sink. Treat that as the current
neutral-build noise floor; single 10M/100M pairs are too noisy for decisions.
A batched free-list cache probe directly attacked the top app-allocation bucket,
but lost before a full gate: 10M regressed to 325.639ms and 100M to 3.181s.
Even an allocator change that should help reused cells can perturb the no-GC hot
path enough to fail the codegen-neutrality gate; the source was reverted and the
default release bench rebuilt.
A dead persistent-dispatcher deletion then compiled and passed the bounded
slice, but the byte-identical full gate regressed to 88.171s with 25.161s GC
pause, versus the 80.927s permanent-compound best and the 82.x current-tree
refreshes. The source was reverted, release bench rebuilt, and post-revert
bounded sanity is 10M 311.608ms / 100M 2.398s with 225.034ms 100M GC pause.
A reserve-app
allocation-accounting probe completed byte-identically but regressed to 89.330s
and was reverted. The current parser-small-int
retest repeats at 91.598s, reducing the parsed graph from 160,961 to 153,185
cells and the full high-water mark from 40,014,305 to 40,006,682 cells, but the
best observed gate is now the 80.927s eval.c permanent compound cache run. The latest accepted S1/S2/S3
slices follow eval.c's `CHKARG`/`GOAP`/`GOAP2` shape more closely:
app-producing stack rewrites stay inside `stack_eval_step`, fixed-arity hot
arms pop/take arguments before rewriting the app redex, and the direct
`ap`/`ap2` refinement pushes the known root app before descending from the
already-bound fun. The dedicated app-allocation path removes the generic
`push_node(Node::App)` dispatch from the dominant node kind, and direct
known-app/free-cell writes avoid cold-payload checks where the stack/free-list
invariant proves the old tag. The stack scalar `SET*` slice writes strict
scalar results into the consumed redex cell, and the authoritative cell arena
removes the old `Node` enum as the mutator's source of truth. Strict `Int`
frames now also own the consumed redex after popping it, so the return path no
longer replays `app_end/used` to find the `SETINT` target. WHNF frames for
`seq`/`IO.strict`/`isint` now use the same redex ownership discipline, and
`IO.strict` reuses that redex as the `action value` app. The latest raw-tag
slice makes hot `Cell` tests read the low tag bits directly and makes the
release free-list pop trust the `Free` invariant, matching eval.c's tag-word
and heap-stack trust discipline; the latest allocator invariant slice removes
the remaining release `Option` branch from the reused-allocation free-head pop.
The F14 slice adds eval.c-style shared
ForeignPtr finalizer records and runs dead records during GC sweep without
moving the self-host heap shape. The unified stack `ret`/`top` slice moves
ready strict-frame return handling into `stack_eval_step`, removes the hot
`StackStep::Force` handoff, and lets force markers plus K/A-family `GOIND`
rewrites continue in the same stack loop. The identity-alias `GOIND` slice
extends that same continuation to `I`/`Ord`/`Chr`, cutting stack-loop exits
without changing graph work. The one-word app-fun slice makes hot app descent
match eval.c's `ut = n->ufun.uutag` shape: test the AP tag and follow the fun
pointer without loading the arg word until `CHKARG` asks for it. The latest GC
slice makes app-edge marking closer to eval.c's `mark(NODEPTR *np)` model by
rewriting app fun/arg parent slots to resolved indirection targets and cached
small-int nodes where available; this lowers full high-water from 40.0M to
38.8M cells and cuts GC pause by about 1.2s. The latest trusted-stack-update
slice applies the same eval.c trust discipline to redex app access in hot
stack rewrites/updates: after arity has been checked, the app slot is loaded
directly and the helpers no longer return `Result` for impossible missing
stack entries. Strict-redex app
snapshots and remaining-app scans remain zero, and fallback helper dispatch no
longer rethreads through `EvalSpine`. Parse labels are no longer permanent GC
roots: like eval.c's parser shared table, they are only patching scaffolding, and
the label map is retained only for labels whose targets are still marked live.
The permanent-compound slice adds C's shared `combFst`, `combSnd`, `combJust`,
and `combPairUnit` shape as GC roots and cuts the full gate to 80.927s even
though the bounded node-count signal is tiny. The next target is to keep
pushing the same layer-removal pattern below 80s.

## Self-Host Compile Slice

Bounded profiles from the F2 persistent-spine worktree; these do not produce an
output comb, but give repeatable partial data for F2 work.

### 50k Slice

| measure | value |
|---|---:|
| parse + bounded reduce + render | 16.0 ms |
| WHNF steps | 50,855 |
| nodes before / after | 265,178 / 319,235 |
| node growth | 54,057 |
| app allocations | 53,881 |
| resolve calls | 3,117 |
| arg materializations | 540 / 4,099 nodes |
| spine rewrites | 417 / 2,214 extra args |
| app rewrites | 0 / 0 extra args |
| heap spines | 32 |
| max spine arity | 76 |

### 1M Slice

| measure | value |
|---|---:|
| parse + bounded reduce + render | 211.7 ms |
| WHNF steps | 1,001,383 |
| nodes before / after | 265,178 / 1,368,741 |
| node growth | 1,103,563 |
| app allocations | 1,102,669 |
| resolve calls | 693,475 |
| arg materializations | 5,866 / 1,332,505 nodes |
| spine rewrites | 3,194 / 665,163 extra args |
| app rewrites | 0 / 0 extra args |
| heap spines | 2,691 |
| max spine arity | 612 |
| max resolve chain | 25 |

Hot heads in the 1M slice are `B`, `C`, `S'`, `C'`, `S`, `C'B`, `P`, `K`, `U`,
`A`, `==`, `J`, `O`, `I`, `Z`, `u<=`, `R`, and IO bind. Persistent dispatch
inside strict frames removed the previous app-rewrite/resolve explosion. A
100M clean bounded run now takes 18.37 s for 100,811,774 WHNF steps
(~5.49M steps/s).

### 1M Profile Slice, Dispatch + Allocation Counters

New profiling counters record primitive-dispatch probes/hits and allocation
kinds. Profile-mode runtime is allowed to be slower; the signal is the shape.

| measure | value |
|---|---:|
| profile total | 331.0 ms |
| WHNF steps | 1,001,383 |
| node growth | 1,103,573 |
| runtime `App` allocations | 1,102,669 |
| runtime `Prim` allocations | 623 |
| runtime non-small `Int` allocations | 52 |
| strict int-binop probes / hits | 52,889 / 46,941 |
| fallback array-op probes / hits | 2,974 / 2,587 |
| fallback bytes-op probes / hits | 2,974 / 259 |
| fallback bytes-unop probes / hits | 3,191 / 152 |
| fallback foreign-ptr-unop probes / hits | 3,191 / 138 |

Reading: F8/P3 is real and should be fixed using `eval.c`'s parse-time
name-to-tag approach, but this slice is still dominated by P2/P1 graph
allocation/update traffic: almost every new runtime node is an `App`.

### S0 Evaluator-Layer Profile Slice

After `NOTES.md` landed, profiling was extended to count the current evaluator
layers that eval.c avoids with one stack: strict-redex app snapshots, remaining
spine scans, persistent-to-fallback transitions, and frame pushes. This is
instrumentation, not a performance win; the full gate remains byte-identical
but regresses from the 511.9s best to 524.6s.

1M self-host profile slice, 32M allocation trigger:

| measure | value |
|---|---:|
| non-profile parse+reduce+render | 155.0 ms |
| profile total | 318.8 ms |
| WHNF steps | 1,001,383 |
| nodes before / after | 160,961 / 1,264,552 |
| runtime `App` allocations | 1,102,687 |
| spine arity observations | 263,734,046 |
| persistent forces | 51,591 |
| persistent fallbacks / fallback eval-loop steps | 3,281 / 3,281 |
| strict-redex snapshots / app ids copied | 45,473 / 12,001,065 |
| remaining-app scans / app ids scanned | 110,621 / 30,072,895 |
| eval-frame pushes | 98,532 |
| eval-frame push kinds | `Int`: 93,887; `Whnf`: 4,645 |

100M self-host slice control with the profiling counters compiled in but
disabled: 11.87s, 8.49M steps/s, 3 GCs, 313 ms GC pause, 34,138,013 high-water
nodes. Full self-host gate: 524.6s, 3,659,075,035 WHNF steps, 125 GCs, 57.0s
GC pause, 43,527,524 high-water nodes, `cmp_exit=0`.

Reading: the remaining cost is exactly where `NOTES.md` said it would be. The
current persistent-spine machine spends measurable work copying app ids into
strict continuations and scanning remaining continuations to avoid self-cycles.
Do not chase another local `remaining_apps_contain` cache; the next meaningful
change is the eval.c-style single stack where the continuation is the stack.

### 10M Profile Slice, App Allocation Sites

App-site profiling was added after rejecting the F8 enum and side-table probes.
A 10M self-host main slice shows the app allocation traffic is mostly ordinary
combinator rewrite payloads, not an unlabeled runtime helper leak:

| measure | value |
|---|---:|
| non-profile slice | 2,729.2 ms |
| profile total | 4,269.8 ms |
| WHNF steps | 10,028,861 |
| runtime `App` allocations | 11,215,896 |
| arg materializations / nodes | 49,968 / 20,865,785 |
| spine rewrites / extra args | 26,633 / 10,370,168 |
| max spine arity | 2,795 |

Top app allocation sites:

| site | allocations |
|---|---:|
| `B.yz` | 1,943,041 |
| `C.xz` | 1,609,624 |
| `S'.left` / `S'.yw` / `S'.zw` | 810,754 each |
| `C'.xyw` / `C'.yw` | 741,069 each |
| `S.left` / `S.right` | 612,486 each |
| `C'B.xz` / `C'B.yw` | 538,754 each |
| `P.zx` | 500,285 |

Reading: these are the apps `eval.c` also has to build for the same
combinator reductions. The remaining Rust-only pressure is less "mystery app
allocation" and more the cost of carrying/rethreading large spines and the
32-byte arena cell/cache footprint.

### Fallback Prefix Materialization Probe

The fallback primitive path used to materialize the entire spine into
`scratch_args` before trying low-arity helper families. The helper set needs at
most four head arguments (`A.write`, `bswrite`, `Wknewfin`), so the current
probe materializes only a four-argument prefix while still using the full
spine length for reduction/rethreading.

| run | before | prefix materialization | result |
|---|---:|---:|---|
| 1M non-profile slice | 204.5 ms | 202.3 ms | small win |
| 10M non-profile slice | 2,729.2 ms | 2,626.1 ms | small slice win |
| 10M profile total | 4,269.8 ms | 4,290.0 ms | profile overhead/noise |
| 10M materialized arg nodes | 20,865,785 | 10,522,018 | expected direction |

The full self-host gate immediately before this probe, with app-site profiling
compiled in but no prefix cap, completed in 716.7s and produced byte-identical
output (`cmp_exit=0`). The prefix cap completed the same gate in 673.8s with
byte-identical output, a 4.8% full-run win and useful evidence, but not the
structural step needed for the 120s target.

### Rejected F8 Variant-Prim Probe

An enum-variant `Prim::{IntBin, IntUn, Int64Bin, ...}` shape was tested after
the profiling chunk. It removed the hot strict int-binop probe cascade in the
1M profile slice, but it regressed both the slice and the full gate:

| run | before | enum-variant F8 | result |
|---|---:|---:|---|
| 1M non-profile slice | 211.7 ms | 248.2 ms | rejected |
| 1M profile total | 331.0 ms | 367.7 ms | rejected |
| full self-host, 16M GC | 707.8s | 770.1s | rejected |
| full self-host output | byte-identical | byte-identical | semantics ok |

A sparse `NodeId -> PrimDispatch` side table was also tested. It preserved
`Node` layout and removed the same strict int-binop probe cascade, but still
regressed the 1M slice to 220.5 ms and the full self-host gate to 731.3s
(`cmp_exit=0`). The useful lesson is narrower than "do not tag primops": F8 is
real, but not currently the dominant lever; a successful tag dispatch probably
needs an in-node compact id that does not grow the cell or add hash lookups.

## GC Scaffold

The current worktree has a first non-moving mark/sweep collector:

- collection only runs at the top of `reduce_whnf_from`, between evaluator
  steps;
- dead nodes are tombstoned as `Free(next)` and reused through an intrusive
  free-list;
- live roots include the current root, original root, stable pointers,
  cached prims, small ints, world, argument array, strict-frame stack,
  persistent/eval spines, scratch spine vectors, and node-backed pointer
  targets reachable through live `ForeignPtr`/`Ptr`/`FunPtr` values;
- parse labels are not roots; after each mark pass the public label map retains
  only labels whose targets are still live, matching eval.c's `shared_table`
  being freed after parsing;
- weak values/finalizers are currently marked conservatively; this keeps the
  first pass safe but does not implement C's weak/finalizer semantics yet;
- default trigger is every 32M runtime allocations since the previous
  collection; `MHS_GC_NODE_INTERVAL=N` can force smaller intervals for smokes,
  and `0` disables the collector;
- mark bitmap storage is reused between collections and bench output now logs
  per-GC pause time, live/free/arena node counts, freed nodes, allocation
  pressure, and total GC pause time.

Forced-GC checks after the scaffold:

| check | result |
|---|---|
| `MHS_GC_NODE_INTERVAL=32 cargo test --workspace` | passed |
| `MHS_GC_NODE_INTERVAL=32` on both pointer reproducers | passed |
| self-host main slice, 50k limit, `MHS_GC_NODE_INTERVAL=1024` | passed; 50,855 steps, sink `661902` |
| self-host main slice, 1M limit, `MHS_GC_NODE_INTERVAL=65536` | passed; 1,001,383 steps, 284.0 ms parse+reduce+render, sink `661902` |
| full self-host gate, `MHS_GC_NODE_INTERVAL=4194304` | passed; 3,659,074,769 steps, 716.8s, sink `661902` |
| full self-host gate, `MHS_GC_NODE_INTERVAL=16777216` | passed; 3,659,075,092 steps, 707.8s, sink `661902` |
| full self-host gate, `MHS_GC_NODE_INTERVAL=16777216`, fallback prefix cap | passed; 3,659,074,902 steps, 673.8s, sink `661902` |
| S2 full gate, `MHS_GC_NODE_INTERVAL=16777216` | passed; 3,659,074,940 steps, 748.9s, 250 GCs, 108.6s GC pause, high-water 27.3M nodes |
| S2 full gate, `MHS_GC_NODE_INTERVAL=33554432` | passed; 3,659,074,902 steps, 727.8s, 125 GCs, 58.3s GC pause, high-water 43.6M nodes |
| S1 strict-redex app-only gate, `MHS_GC_NODE_INTERVAL=33554432` | passed; 3,659,074,997 steps, 578.4s, 125 GCs, 60.7s GC pause, high-water 43.6M nodes |
| S1 persistent-spine app-only gate, `MHS_GC_NODE_INTERVAL=33554432` | passed; 3,659,075,206 steps, 566.7s, 125 GCs, 57.5s GC pause, high-water 43.6M nodes |
| S1 persistent head-classification gate, `MHS_GC_NODE_INTERVAL=33554432` | passed; 3,659,075,016 steps, 539.6s, 125 GCs, 57.8s GC pause, high-water 43.6M nodes |
| S1 EvalSpine head-classification gate, `MHS_GC_NODE_INTERVAL=33554432` | passed; 3,659,075,092 steps, 525.2s, 125 GCs, 57.6s GC pause, high-water 43.6M nodes |
| S0 evaluator-layer profile counters gate, `MHS_GC_NODE_INTERVAL=33554432` | passed; 3,659,075,035 steps, 524.6s, 125 GCs, 57.0s GC pause, high-water 43.5M nodes, `cmp_exit=0` |
| S2 mark-time indirection compression gate, `MHS_GC_NODE_INTERVAL=33554432` | passed; 3,659,075,111 steps, 493.0s, 125 GCs, 33.3s GC pause, high-water 40.0M nodes, `cmp_exit=0` |
| S1 compact stack-entry split gate, `MHS_GC_NODE_INTERVAL=33554432` | passed; 3,659,075,035 steps, 169.4s, 125 GCs, 30.7s GC pause, high-water 40.0M nodes, `cmp_exit=0` |
| S1 trusted stack args/static fallback names gate, `MHS_GC_NODE_INTERVAL=33554432` | passed; 3,659,075,111 steps, 158.5s, 125 GCs, 29.4s GC pause, high-water 40.0M nodes, `cmp_exit=0` |
| S1 app-only eval stack gate, `MHS_GC_NODE_INTERVAL=33554432` | passed; 3,659,074,959 steps, 151.4s, 125 GCs, 29.1s GC pause, high-water 40.0M nodes, `cmp_exit=0` |
| S0 feature-gated phase profiling/default rebuild gate, `MHS_GC_NODE_INTERVAL=33554432` | passed; 3,659,075,187 steps, 149.2s, 125 GCs, 28.8s GC pause, high-water 40.0M nodes, `cmp_exit=0` |
| S1 strict Int immediate-arg fast path gate, `MHS_GC_NODE_INTERVAL=33554432` | passed; 3,659,075,092 steps, 147.5s, 125 GCs, 29.0s GC pause, high-water 40.0M nodes, `cmp_exit=0` |
| S0 stack-head phase profile/default rebuild gate, `MHS_GC_NODE_INTERVAL=33554432` | passed; 3,659,075,168 steps, 150.1s, 125 GCs, 30.2s GC pause, high-water 40.0M nodes, `cmp_exit=0`; not a new best |
| S1 unchecked stack arg loads gate, `MHS_GC_NODE_INTERVAL=33554432` | passed; 3,659,075,168 steps, 145.8s, 125 GCs, 30.1s GC pause, high-water 40.0M nodes, `cmp_exit=0`; new best |
| S2 reusable GC mark-work stack gate, `MHS_GC_NODE_INTERVAL=33554432` | passed; 3,659,074,921 steps, 143.9s, 125 GCs, 29.5s GC pause, high-water 40.0M nodes, `cmp_exit=0`; new best |
| S0 feature-gated GC phase profiling/default gate, `MHS_GC_NODE_INTERVAL=33554432` | passed; 3,659,075,016 steps, 143.7s, 125 GCs, 30.2s GC pause, high-water 40.0M nodes, `cmp_exit=0`; new best/no-regression |
| S2 post-bitmap-reject intrusive free-list restore gate, `MHS_GC_NODE_INTERVAL=33554432` | passed; 3,659,075,263 steps, 144.9s, 125 GCs, 29.5s GC pause, high-water 40.0M nodes, `cmp_exit=0`; restored accepted band but not a new best |
| S1 C-style app-continuation gate, `MHS_GC_NODE_INTERVAL=33554432` | passed; 3,659,074,940 steps, 137.6s, 125 GCs, 30.0s GC pause, high-water 40.0M nodes, `cmp_exit=0`; new best |
| S1 direct `ap`/`ap2` app-result descent gate, `MHS_GC_NODE_INTERVAL=33554432` | passed; 3,659,074,978 steps, 131.6s, 125 GCs, 30.2s GC pause, high-water 40.0M nodes, `cmp_exit=0`; new best |
| S1 dedicated `Node::App` allocation gate, `MHS_GC_NODE_INTERVAL=33554432` | passed; latest repeat best 3,659,074,940 steps, 120.708s, 125 GCs, 28.548s GC pause, high-water 40.0M nodes, `cmp_exit=0`; new best, still just above 120s |
| S0 sub-timer profiling/default rebuild gate, `MHS_GC_NODE_INTERVAL=33554432` | passed; 3,659,075,111 steps, 122.341s, 125 GCs, 28.891s GC pause, high-water 40.0M nodes, `cmp_exit=0`; not a new best, default build stays in accepted band |
| F6 serializer labels/default rebuild gate, `MHS_GC_NODE_INTERVAL=33554432` | passed; 3,659,074,959 steps, 124.030s, 125 GCs, 29.381s GC pause, high-water 40.0M nodes, `cmp_exit=0`; not a new best, serializer parity change stays correct |
| S1 compact profile-head frame gate, `MHS_GC_NODE_INTERVAL=33554432` | passed; 3,659,075,092 steps, 120.167s, 125 GCs, 30.588s GC pause, high-water 40.0M nodes, `cmp_exit=0`; new best. Repeat output file also byte-matches the oracle, but its timing stdout was lost with the dead tool session |
| S0 profile-overhead instrumentation/default full gate, `MHS_GC_NODE_INTERVAL=33554432` | passed; 3,659,075,244 steps, 118.951s, 30.76M steps/s, 125 GCs, 29.398s GC pause, high-water 39,958,383 nodes, `cmp_exit=0`; first logged sub-120s gate, byte-identical |
| S3 authoritative cell arena gate, `MHS_GC_NODE_INTERVAL=33554432` | passed; 3,659,074,997 steps, 105.859s, 34.57M steps/s, 125 GCs, 28.928s GC pause, high-water 40,014,305 cells, `cmp_exit=0`; new best, byte-identical, 2.21x C |
| S1 fixed-arity `CHKARG` pop/take gate, `MHS_GC_NODE_INTERVAL=33554432` | passed; 3,659,074,978 steps, 99.022s, 36.95M steps/s, 125 GCs, 28.724s GC pause, high-water 40,014,303 cells, `cmp_exit=0`; new best, byte-identical, 2.07x C |
| S1 strict `Int` redex-owning frame gate, `MHS_GC_NODE_INTERVAL=33554432` | passed; 3,659,074,959 steps, 95.345s, 38.38M steps/s, 125 GCs, 27.975s GC pause, high-water 40,014,301 cells, `cmp_exit=0`; new best, byte-identical, 1.99x C |
| S1 WHNF redex-owning frame gate, `MHS_GC_NODE_INTERVAL=33554432` | passed; 3,659,074,978 steps, 93.202s, 39.26M steps/s, 125 GCs, 27.492s GC pause, high-water 40,014,303 cells, `cmp_exit=0`; new best, byte-identical, 1.95x C |
| S1 raw cell tag/trusted free-list gate, `MHS_GC_NODE_INTERVAL=33554432` | passed; 3,659,074,997 steps, 91.503s, 39.99M steps/s, 125 GCs, 26.637s GC pause, high-water 40,014,305 cells, `cmp_exit=0`; new best, byte-identical, 1.91x C |
| Restored default full rerun after rejected probes, `MHS_GC_NODE_INTERVAL=33554432` | passed; noisy 103.363s, 3,659,075,016 steps, 35.40M steps/s, 125 GCs, 28.544s GC pause, high-water 40,014,307 cells, `cmp_exit=0`; byte-identical but not baseline-worthy |
| S3 parse-time small-int interning retest gate, `MHS_GC_NODE_INTERVAL=33554432` | passed; first 92.508s, repeat 91.598s, 3,659,075,187 steps, 39.95M steps/s, 125 GCs, 26.632s GC pause, high-water 40,006,682 cells, `cmp_exit=0`; byte-identical and effectively tied with the 91.503s best |
| S1 direct inner `GOAP2` descent probe, `MHS_GC_NODE_INTERVAL=33554432` | rejected; 1M/100M won, but full gates were 131.8s and 133.2s vs then-best 131.6s, byte-identical; reverted |

GC interval sweep on the 100M self-host main slice:

| `MHS_GC_NODE_INTERVAL` | time | steps/s | collections | freed nodes | high-water nodes |
|---:|---:|---:|---:|---:|---:|
| `0` | 19.44s | 5.19M | 0 | 0 | 117,763,780 |
| `4,194,304` | 16.15s | 6.24M | 7 | 114,433,273 | 29,625,311 |
| `16,777,216` | 16.13s | 6.25M | 3 | 99,289,532 | 50,596,826 |
| `67,108,864` | 16.66s | 6.05M | 1 | 66,690,767 | 67,374,042 |

Before S2, the 16M arena-length trigger was the best reading. After S2, the
32M allocation trigger is the best measured current default, but it still loses
wall time to the tagged 673.8s checkpoint because the collector now runs much
more often.

### S2 Heap-Loop Fix

`NOTES.md` identified the key GC bug: the old trigger used arena length, so
free-list reuse did not advance the next trigger and the arena could still grow
by another 16M slots per GC cycle. The S2 worktree triggers on allocations since
the last collection, uses an intrusive `Free(next)` list, reuses the mark bitmap,
and prints pause/event stats.

100M self-host main slices:

| interval | time | steps/s | collections | GC pause | high-water nodes |
|---:|---:|---:|---:|---:|---:|
| old 16M arena trigger | 16.13s | 6.25M | 3 | not logged | 50,596,826 |
| S2 16M allocation trigger | 15.22s | 6.62M | 7 | 489 ms | 17,611,149 |
| S2 32M allocation trigger | 15.02s | 6.71M | 3 | 333 ms | 34,237,713 |
| S2 64M allocation trigger | 16.56s | 6.09M | 1 | 221 ms | 67,374,042 |
| S2 128M allocation trigger | 18.03s | 5.59M | 0 | 0 ms | 117,763,792 |
| S1 strict-redex app-only, 32M allocation trigger | 12.56s | 8.03M | 3 | 316 ms | 34,237,717 |
| S1 persistent-spine app-only, 32M allocation trigger | 12.82s | 7.87M | 3 | 315 ms | 34,237,738 |
| S1 persistent head classification, 32M allocation trigger | 11.91s | 8.47M | 3 | 328 ms | 34,237,719 |
| S1 EvalSpine head classification, 32M allocation trigger | 11.97s | 8.42M | 3 | 332 ms | 34,237,727 |
| S3 parse-time primitive interning, 32M allocation trigger | 11.48s | 8.78M | 3 | 340 ms | 34,138,007 |
| S0 evaluator-layer profile counters, 32M allocation trigger | 11.87s | 8.49M | 3 | 313 ms | 34,138,013 |
| S2 mark-time indirection compression, 32M allocation trigger | 11.67s | 8.64M | 3 | 315 ms | 34,050,047 |
| rejected PersistentSpine contiguous `Vec`, 32M allocation trigger | 13.33s | 7.56M | 3 | 356 ms | 34,237,730 |
| rejected strict-redex tail ownership, 32M allocation trigger | 15.31s | 6.59M | 3 | 311 ms | 34,237,736 |
| rejected segmented persistent strict stack, 32M allocation trigger | 13.64s | 7.39M | 3 | 320 ms | 34,237,754 |
| rejected hybrid app/marker stack splice, no GC | 241.6 ms for 1M | 4.15M | 0 | 0 ms | 1,264,542 |
| rejected direct scalar redex overwrite, 32M allocation trigger | 12.32s | 8.18M | 3 | 340 ms | 34,049,608 |
| S1 stack argument binding, 32M allocation trigger | 5.08s | 19.84M | 3 | 297 ms | 34,050,047 |
| S1 compact stack-entry split, 32M allocation trigger | 4.78s | 21.09M | 3 | 308 ms | 34,050,263 |

Full-gate S2 result:

| interval | time | steps/s | collections | GC pause | high-water nodes | output |
|---:|---:|---:|---:|---:|---:|---|
| 16M | 748.9s | 4.89M | 250 | 108.6s | 27,261,315 | byte-identical |
| 32M | 727.8s | 5.03M | 125 | 58.3s | 43,626,866 | byte-identical |

The S2 heap loop works as a memory fix and exposes useful GC telemetry, but it
does not buy the 600s gate. It also exposed the expected F2/F5 root issue:
allocation-triggered collection inside nested `reduce_node_whnf` calls could
collect outer evaluator roots that only lived on the native Rust stack. The
current guard collects only in the outermost `reduce_whnf_from`; the proper fix
is still S1's single-stack evaluator.

This is intentionally not full C GC parity yet: no finalizer execution, no weak
clearing, no GCRED, and no collection inside helper allocations. The useful
property is narrower: bounded self-host graph shapes survive safe-point
collection and can reuse arena slots without moving live `NodeId`s.

### S2 Mark-Time Indirection Compression

`eval.c` compresses indirection chains during mark. The Rust collector now does
the same limited version at GC time: when mark sees an `Indir`, it follows the
chain to a live non-indirection target, rewrites the marked entry to that final
target, and marks the target. This is deliberately not the rejected
per-evaluation first-indirection compression probe; it only runs while the GC is
already walking live nodes.

Checks before the full gate:

| check | result |
|---|---|
| `cargo fmt --manifest-path rust/microhs-runtime/Cargo.toml` | passed |
| `cargo check --manifest-path rust/microhs-runtime/Cargo.toml` | passed |
| `MHS_GC_NODE_INTERVAL=32 cargo test --workspace` | passed; 34 tests |
| `cargo build --release --manifest-path rust/microhs-runtime/Cargo.toml --bin mhs-rust-bench` | passed |

100M self-host main slice, 32M allocation trigger:

| run | time | steps/s | collections | GC pause | high-water nodes |
|---|---:|---:|---:|---:|---:|
| S0 profiled-control baseline | 11.87s | 8.49M | 3 | 313 ms | 34,138,013 |
| post-revert sanity baseline | 11.66s | 8.64M | 3 | not separately logged | not separately logged |
| mark-time indirection compression | 11.67s | 8.64M | 3 | 315 ms | 34,050,047 |

Full self-host gate:

```text
timeout 900s env MHS_GC_NODE_INTERVAL=33554432 target/release/mhs-rust-bench \
  --input /tmp/mhs-selfhost.comb --mode main --warmup-iters 0 --iters 1 -- \
  ./bin/mhs -i -imhs -isrc -ilib MicroHs.Main -o/tmp/mhs-selfhost-rust-out-gc-indir-compress.comb
```

```text
parse_reduce_render_total_ms: 492957.459
whnf_steps_per_iter: 3659075111.0
whnf_steps_per_s: 7422699.6
step_limited_iters: 0
serialize_sink: 661902
gc_collections: 125
gc_freed_nodes_total: 4192827425
gc_last_live_nodes: 1637697
gc_last_free_nodes: 38319606
gc_high_water_nodes: 39957303
gc_current_nodes: 39957303
gc_current_free_nodes: 24170937
gc_last_pause_ms: 171.783
gc_total_pause_ms: 33251.710
```

The generated output matched the C-generated input exactly:

```text
sha256(/tmp/mhs-selfhost.comb) = 29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a
sha256(/tmp/mhs-selfhost-rust-out-gc-indir-compress.comb) = 29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a
cmp_exit=0
```

Reading: keep this slice. It cuts the current best full gate from 511.9s to
493.0s, drops total GC pause from ~58.4s to 33.3s, and lowers high-water arena
size from ~43.5M to 40.0M nodes. The full-gate win is much larger than the 100M
slice signal because the later self-host phases have longer indirection chains
and higher live sets. This is still not the 120s lever; it just removes a C GC
trick the Rust collector was missing.

### S1 Eval.c-Style Single-Stack Hot Path

`NOTES.md` called out the remaining gap as mechanical cost per step: Rust was
still carrying `PersistentSpine`, `EvalSpine`, `StrictRedex::Spine`, separate
strict frames, and rethread scans where eval.c has one stack and a `ret:` path.
The current S1 slice routes normal WHNF reduction through one stack of app-cell
ids plus strict marker entries. The hot combinator, strict-primitive, and
fallback primitive-helper paths no longer snapshot strict redex app ids, scan
remaining apps, or rethread through `EvalSpine`. A tiny bridge remains for
std-handle/control helpers whose representation is not yet eval.c-shaped.

Important implementation note: the first version used the right stack shape but
computed the current app-segment base by reverse-scanning the stack. That was a
2x regression on the 1M slice. Keeping the app segment base as an explicit stack
field is load-bearing and matches the point of the eval.c representation.

Checks before the full gate:

| check | result |
|---|---|
| `cargo fmt --manifest-path rust/microhs-runtime/Cargo.toml` | passed |
| `MHS_GC_NODE_INTERVAL=32 cargo test --workspace` | passed; 34 tests |
| `cargo build --release --manifest-path rust/microhs-runtime/Cargo.toml --bin mhs-rust-bench` | passed |
| `git diff --check` | passed |

Self-host main slices, 32M allocation trigger:

| run | time | steps/s | collections | GC pause | high-water nodes |
|---|---:|---:|---:|---:|---:|
| accepted pre-stack baseline | 11.67s | 8.64M | 3 | 315 ms | 34,050,047 |
| stack with scanned app base | 330.7 ms for 1M | 3.03M | 0 | 0 ms | 1,264,542 |
| stack with explicit app base, 1M | 92.0 ms | 10.89M | 0 | 0 ms | 1,264,552 |
| stack with explicit app base, 100M | 6.71s | 15.03M | 3 | 309 ms | 34,050,236 |
| stack-native fallback primitive helpers, 1M | 90.1 ms | 11.11M | 0 | 0 ms | 1,264,556 |
| stack-native fallback primitive helpers, 100M | 6.01s | 16.76M | 3 | 305 ms | 34,050,275 |

1M profile slice after the explicit app-base fix:

| measure | value |
|---|---:|
| non-profile parse+reduce+render | 98.6 ms |
| profile total | 248.0 ms |
| WHNF steps | 1,001,383 |
| runtime `App` allocations | 1,102,703 |
| strict-redex snapshots / app ids copied | 0 / 0 |
| remaining-app scans / app ids scanned | 0 / 0 |
| persistent fallbacks / fallback eval-loop steps | 90 / 90 |
| remaining stack fallback heads | `IO.stdout` 83, `IO.getArgRef` 2, `IO.stderr` 2, `IO.stdin` 2, `catch` 1 |
| eval-frame pushes | 98,532 |

Full self-host gate:

```text
timeout 900s env MHS_GC_NODE_INTERVAL=33554432 target/release/mhs-rust-bench \
  --input /tmp/mhs-selfhost.comb --mode main --warmup-iters 0 --iters 1 -- \
  ./bin/mhs -i -imhs -isrc -ilib MicroHs.Main -o/tmp/mhs-selfhost-rust-out-stack-helper.comb
```

```text
parse_reduce_render_total_ms: 200439.107
whnf_steps_per_iter: 3659075016.0
whnf_steps_per_s: 18255294.9
step_limited_iters: 0
serialize_sink: 661902
gc_collections: 125
gc_freed_nodes_total: 4192827123
gc_last_live_nodes: 1637988
gc_last_free_nodes: 38320086
gc_high_water_nodes: 39958074
gc_current_nodes: 39958074
gc_current_free_nodes: 24171536
gc_last_pause_ms: 165.613
gc_total_pause_ms: 32597.681
```

The generated output matched the C-generated input exactly:

```text
sha256(/tmp/mhs-selfhost.comb) = 29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a
sha256(/tmp/mhs-selfhost-rust-out-stack-helper.comb) = 29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a
cmp_exit=0
```

Reading: keep this slice. It is the first replacement-loop result that beats
the accepted machine instead of reproducing the rejected hybrid failure: full
gate falls from 493.0s to 200.4s, while preserving the exact 3.659B-step
semantic profile and byte-identical output. The fallback primitive-helper move
cuts old-loop bridge hits from 3,281/1M to 90/1M and removes heap `EvalSpine`
use in the 1M profile. This is still not the 120s target. The next S1 work
should combine the stack path with in-cell primitive tags/compact cells. The
remaining std-handle/control-helper bridge is too small to justify another fat
enum shape; handle it when std handles can be represented without perturbing the
hot `Node` layout. Do not retry stack storage variants that make app-base lookup
implicit; the explicit base field is essential.

Rejected follow-up inside this slice: directly treating `IO.stdin`/`IO.stdout`/
`IO.stderr` as stack WHNF and copying `IO.getArgRef`/`catch`/`catchr` into the
stack match made the self-host run stop after only 3,973 steps. C mutates the
std handles into permanent runtime objects with `mk_std` during initialization,
so Rust needs a real representation change before those bridge hits move.

Second rejected follow-up: adding a `Node::StdHandle` runtime-object variant and
parsing/allocating std handles through it was semantically fine in the 1M
profile: the run still hit 1,001,383 steps and fallback heads dropped from 90/1M
to `IO.getArgRef` 2 + `catch` 1. It regressed 100M slices to 6.94s / 6.27s
against the 6.01s accepted helper-dispatch slice, likely from changing the hot
`Node` enum shape/codegen. After revert, 100M returned to 6.21s. Do not add a
fat enum std-handle variant; fold std handles into the compact-cell/tag work.

### Post-`NOTES.md` Refresh

After `NOTES.md` landed, the live conclusion is stricter: the work count already
matches C, so stop treating F8/P3 as a dispatch-cache micro-probe. The
unfinished `StrictPrim` enum-tag experiment was removed before measurement
because it was another "tag added to the fat node" shape. The next valid F8/S3
attempt needs to move the representation itself: smaller hot cells and primitive
identity in the cell/tag payload.

Fresh slice and gate numbers, same 32M allocation trigger:

| run | time | steps/s | collections | GC pause | high-water nodes | note |
|---|---:|---:|---:|---:|---:|---|
| 1M non-profile | 79.7 ms | 12.56M | 0 | 0 ms | 1,264,546 | sink `661902` |
| 1M profile | 228.2 ms | n/a | 0 | 0 ms | 1,264,546 | 0 strict snapshots/scans; 90 stack fallback heads |
| 100M slice | 6.11s | 16.49M | 3 | 300 ms | 34,050,235 | sink `661902` |
| full self-host rerun | 215.9s | 16.95M | 125 | 34.44s | 39,958,062 | byte-identical, `cmp_exit=0` |

The full rerun command was:

```text
timeout 900s env MHS_GC_NODE_INTERVAL=33554432 target/release/mhs-rust-bench \
  --input /tmp/mhs-selfhost.comb --mode main --warmup-iters 0 --iters 1 -- \
  ./bin/mhs -i -imhs -isrc -ilib MicroHs.Main -o/tmp/mhs-selfhost-rust-out-latest.comb
```

```text
parse_reduce_render_total_ms: 215918.244
whnf_steps_per_iter: 3659074902.0
whnf_steps_per_s: 16946575.9
step_limited_iters: 0
serialize_sink: 661902
gc_collections: 125
gc_freed_nodes_total: 4192827127
gc_last_live_nodes: 1637974
gc_last_free_nodes: 38320088
gc_high_water_nodes: 39958062
gc_current_nodes: 39958062
gc_current_free_nodes: 24171684
gc_last_pause_ms: 174.398
gc_total_pause_ms: 34435.916
```

```text
sha256(/tmp/mhs-selfhost.comb) = 29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a
sha256(/tmp/mhs-selfhost-rust-out-latest.comb) = 29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a
cmp_exit=0
```

At this point bench output printed the representation sizes that mattered for
the next slice:

```text
node_size_bytes: 32
node_id_size_bytes: 8
prim_size_bytes: 24
```

Reading: this was the accepted S1/S2/S3 machine before payload-out. The 215.9s rerun is slower
than the 200.4s best but in the same band and byte-identical. `Vec<bool>` is a
packed bitmap in Rust, so the GC mark-storage note is already satisfied by the
current `gc_marked` storage; GC still costs ~34s of this gate, and the next
section tests the 32-byte cell/string-backed primitive-head part of the theory.

### S3 Payload-Out 24-Byte Node Slice

The first representation slice after the post-`NOTES.md` refresh moves cold
vector/string payloads out of the hot `Node` enum and changes `Prim::Other` to a
boxed string. This keeps hot `App`, scalar, pointer, and known-primitive payloads
direct, but boxes `Bytes`, `BigInt`, `Array`, FFI names, JS wrapper names,
function pointers, and ticks. It is not the final eval.c cell: `NodeId` is still
8 bytes and primitive identity is still not in-cell. It does, however, make the
measured hot cell smaller instead of adding another side structure.

Checks:

| check | result |
|---|---|
| `cargo check --manifest-path rust/microhs-runtime/Cargo.toml` | passed |
| `cargo fmt --manifest-path rust/microhs-runtime/Cargo.toml` | passed |
| `cargo build --release --manifest-path rust/microhs-runtime/Cargo.toml --bin mhs-rust-bench` | passed |
| `MHS_GC_NODE_INTERVAL=32 cargo test --workspace` | passed; 34 tests |
| debug/release `cargo build --target wasm32-unknown-unknown --manifest-path rust/microhs-runtime/Cargo.toml` | passed |
| Node wasm import smoke | rendered `42` |
| `git diff --check` | passed |

Representation-size smoke:

```text
node_size_bytes: 24
node_id_size_bytes: 8
prim_size_bytes: 16
```

Fresh slice and gate numbers, same 32M allocation trigger:

| run | before payload-out | payload-out | result |
|---|---:|---:|---|
| `Node` size | 32 bytes | 24 bytes | keep |
| `Prim` size | 24 bytes | 16 bytes | keep |
| 1M non-profile | 79.7 ms | 78.4 ms | small win/noise |
| 1M profile | 228.2 ms | 228.0 ms | neutral; shape unchanged |
| 100M slice | 6.11s / 16.49M steps/s | 5.92s / 17.03M steps/s | win |
| 100M GC pause | 300 ms | 294 ms | small win |
| full self-host | 215.9s / 16.95M steps/s | 203.6s / 17.97M steps/s | win vs latest rerun, near 200.4s best |
| full self-host GC pause | 34.44s | 31.09s | win |
| full self-host output | byte-identical | byte-identical | `cmp_exit=0` |

Full gate command:

```text
timeout 900s env MHS_GC_NODE_INTERVAL=33554432 target/release/mhs-rust-bench \
  --input /tmp/mhs-selfhost.comb --mode main --warmup-iters 0 --iters 1 -- \
  ./bin/mhs -i -imhs -isrc -ilib MicroHs.Main -o/tmp/mhs-selfhost-rust-out-node24.comb
```

```text
parse_reduce_render_total_ms: 203630.166
whnf_steps_per_iter: 3659074902.0
whnf_steps_per_s: 17969218.3
step_limited_iters: 0
serialize_sink: 661902
gc_collections: 125
gc_freed_nodes_total: 4192827127
gc_last_live_nodes: 1637974
gc_last_free_nodes: 38320088
gc_high_water_nodes: 39958062
gc_current_nodes: 39958062
gc_current_free_nodes: 24171684
gc_last_pause_ms: 157.339
gc_total_pause_ms: 31090.655
```

```text
sha256(/tmp/mhs-selfhost.comb) = 29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a
sha256(/tmp/mhs-selfhost-rust-out-node24.comb) = 29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a
cmp_exit=0
```

Reading: keep this slice. It validates the `NOTES.md` representation thesis with
a real size drop and a full-gate win against the latest rerun, though not a new
absolute best. The next 120s lever is probably the harder half of the cell work:
`NodeId` to `u32` plus primitive tags small enough for a 16-byte hot `Node`.
That has a larger blast radius because the runtime has roughly 90 direct arena
index sites using `id.0`.

### S3 Compact 16-Byte Node Slice

This slice finishes the representation move that the payload-out slice set up:
`NodeId` is narrowed from `usize` to `u32`, non-`KnownPrim` runtime primitive
heads become compact `RuntimePrim(u16)` tags backed by one static name table,
and cold FFI/JS/funptr name payloads move from fat `Box<str>` to thin
`Box<String>` pointers. Unknown primitive names are now rejected at parse time;
the old test-only ability to construct an impossible unknown `Node::Prim` was
removed, while the recognized-but-unsupported runtime primitive check
(`IO.fork`, etc.) remains covered.

Checks:

| check | result |
|---|---|
| `cargo check --manifest-path rust/microhs-runtime/Cargo.toml` | passed |
| `cargo fmt --manifest-path rust/microhs-runtime/Cargo.toml` | passed |
| `cargo build --release --manifest-path rust/microhs-runtime/Cargo.toml --bin mhs-rust-bench` | passed |
| `MHS_GC_NODE_INTERVAL=32 cargo test --workspace` | passed; 34 tests |
| debug/release `cargo build --target wasm32-unknown-unknown --manifest-path rust/microhs-runtime/Cargo.toml` | passed |
| Node wasm import smoke | rendered `42` |
| `git diff --check` | passed |

Representation-size smoke:

```text
node_size_bytes: 16
node_id_size_bytes: 4
prim_size_bytes: 4
```

Fresh slice and gate numbers, same 32M allocation trigger:

| run | 24-byte payload-out | 16-byte compact node | result |
|---|---:|---:|---|
| `Node` size | 24 bytes | 16 bytes | keep |
| `NodeId` size | 8 bytes | 4 bytes | keep |
| `Prim` size | 16 bytes | 4 bytes | keep |
| 1M non-profile | 78.4 ms | 65.5 ms | win |
| 1M profile | 228.0 ms | 216.7 ms | win; shape unchanged |
| 100M slice | 5.92s / 17.03M steps/s | 5.45s / 18.50M steps/s | win |
| 100M GC pause | 294 ms | 295 ms | neutral |
| full self-host | 203.6s / 17.97M steps/s | 195.4s / 18.73M steps/s | new best |
| full self-host GC pause | 31.09s | 29.34s | win |
| full self-host output | byte-identical | byte-identical | `cmp_exit=0` |

Full gate command:

```text
timeout 900s env MHS_GC_NODE_INTERVAL=33554432 target/release/mhs-rust-bench \
  --input /tmp/mhs-selfhost.comb --mode main --warmup-iters 0 --iters 1 -- \
  ./bin/mhs -i -imhs -isrc -ilib MicroHs.Main -o/tmp/mhs-selfhost-rust-out-node16.comb
```

```text
parse_reduce_render_total_ms: 195391.409
whnf_steps_per_iter: 3659074902.0
whnf_steps_per_s: 18726897.6
step_limited_iters: 0
serialize_sink: 661902
gc_collections: 125
gc_freed_nodes_total: 4192827127
gc_last_live_nodes: 1637974
gc_last_free_nodes: 38320088
gc_high_water_nodes: 39958062
gc_current_nodes: 39958062
gc_current_free_nodes: 24171684
gc_last_pause_ms: 150.654
gc_total_pause_ms: 29344.510
```

```text
sha256(/tmp/mhs-selfhost.comb) = 29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a
sha256(/tmp/mhs-selfhost-rust-out-node16.comb) = 29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a
cmp_exit=0
```

Reading: keep this slice. It is the first full gate under 200s and the first
Rust representation that is genuinely eval.c-sized on the hot `App`/scalar/prim
path. The remaining primitive-dispatch profile still shows the old string
cascade counters because dispatch currently converts `RuntimePrim` tags back to
names before classifying strict/fallback actions. The next slice below addresses
that directly; it is a small cleanup win, not the main 120s lever.

### S3 Direct RuntimePrim Strict Dispatch Slice

Runtime primitive heads already live in the compact cell as `RuntimePrim(u16)`.
This slice stops converting that tag back to a string for strict primitive
classification. Strict integer/float/bytes/conversion groups now dispatch
through inlined tag ranges and small op tables; fallback helpers still request
the primitive name only when they actually need string-keyed runtime helper
dispatch or an error.

The first non-inlined version regressed the 1M slice from ~65ms to ~77ms, which
is a useful warning: direct tags only help here if the classifier compiles down
to the same kind of local branch as eval.c's tag switch. The kept version marks
the tiny lookup helper and `RuntimePrim::strict_action` as `#[inline(always)]`.

Checks:

| check | result |
|---|---|
| `cargo check --manifest-path rust/microhs-runtime/Cargo.toml` | passed |
| `cargo fmt --manifest-path rust/microhs-runtime/Cargo.toml` | passed |
| `cargo build --release --manifest-path rust/microhs-runtime/Cargo.toml --bin mhs-rust-bench` | passed |
| `MHS_GC_NODE_INTERVAL=32 cargo test --workspace` | passed; 34 tests |
| debug/release `cargo build --target wasm32-unknown-unknown --manifest-path rust/microhs-runtime/Cargo.toml` | passed |
| Node wasm import smoke | rendered `42` |
| `git diff --check` | passed |

Representation-size smoke:

```text
node_size_bytes: 16
node_id_size_bytes: 4
prim_size_bytes: 4
```

Fresh slice and gate numbers, same 32M allocation trigger:

| run | 16-byte compact node | direct `RuntimePrim` dispatch | result |
|---|---:|---:|---|
| 1M non-profile | 65.5 ms | 65.2 ms | tiny win |
| 1M profile | 216.7 ms | 215.1 ms | tiny win |
| 100M slice | 5.45s / 18.50M steps/s | 5.48s / 18.41M steps/s | neutral/slight regression |
| 100M GC pause | 295 ms | 306 ms | noise/slight regression |
| full self-host | 195.4s / 18.73M steps/s | 192.4s / 19.02M steps/s | new best |
| full self-host GC pause | 29.34s | 29.81s | slight regression; mutator still wins |
| full self-host output | byte-identical | byte-identical | `cmp_exit=0` |

Full gate command:

```text
timeout 900s env MHS_GC_NODE_INTERVAL=33554432 target/release/mhs-rust-bench \
  --input /tmp/mhs-selfhost.comb --mode main --warmup-iters 0 --iters 1 -- \
  ./bin/mhs -i -imhs -isrc -ilib MicroHs.Main -o/tmp/mhs-selfhost-rust-out-directprim-inline.comb
```

```text
parse_reduce_render_total_ms: 192378.478
whnf_steps_per_iter: 3659075111.0
whnf_steps_per_s: 19020189.6
step_limited_iters: 0
serialize_sink: 661902
gc_collections: 125
gc_freed_nodes_total: 4192827118
gc_last_live_nodes: 1638004
gc_last_free_nodes: 38320082
gc_high_water_nodes: 39958086
gc_current_nodes: 39958086
gc_current_free_nodes: 24171413
gc_last_pause_ms: 152.155
gc_total_pause_ms: 29807.190
```

```text
sha256(/tmp/mhs-selfhost.comb) = 29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a
sha256(/tmp/mhs-selfhost-rust-out-directprim-inline.comb) = 29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a
cmp_exit=0
```

Reading: keep this slice as cleanup under S3. It confirms `NOTES.md`'s warning
that local probes inside the current machine are small and codegen-sensitive.
The full gate win is real, but the 100M slice says this is not the 120s path.
The next credible work is back in S1/F2 territory: app/update traffic and the
remaining redex-result path, with eval.c's `GOAP`/`GOAP2`/`SET*` discipline as
the guide.

### S1 Stack Argument Binding Slice

The stack evaluator still read arguments with repeated `arg!(i)` calls inside
each combinator arm. This slice binds hot stack arguments once per arm, closer
to eval.c's `CHKARGn` shape. Only the stack evaluator was changed; the older
persistent fallback path still uses its existing `arg!` accesses.

An earlier probe in the same area, "raw app allocation when profiling is
disabled", was rejected before this slice: it tried to bypass `app_with_site`
when not profiling, but the 1M slice regressed from ~65.1ms to 68.1ms.

Checks:

| check | result |
|---|---|
| `cargo check --manifest-path rust/microhs-runtime/Cargo.toml` | passed |
| `cargo fmt --manifest-path rust/microhs-runtime/Cargo.toml` | passed |
| `cargo build --release --manifest-path rust/microhs-runtime/Cargo.toml --bin mhs-rust-bench` | passed |
| `MHS_GC_NODE_INTERVAL=32 cargo test --workspace` | passed; 34 tests |
| debug/release `cargo build --target wasm32-unknown-unknown --manifest-path rust/microhs-runtime/Cargo.toml` | passed |
| Node wasm import smoke | rendered `42` |
| `git diff --check` | passed |

Fresh slice and gate numbers, same 32M allocation trigger:

| run | direct `RuntimePrim` dispatch | stack argument binding | result |
|---|---:|---:|---|
| 1M non-profile | 65.1 ms | 65.0 ms | neutral |
| 100M slice | 5.48s / 18.41M steps/s | 5.08s / 19.84M steps/s | win |
| 100M GC pause | 306 ms | 297 ms | small win/noise |
| full self-host | 192.4s / 19.02M steps/s | 191.2s / 19.13M steps/s | new best |
| full self-host GC pause | 29.81s | 30.73s | worse; mutator wins anyway |
| full self-host output | byte-identical | byte-identical | `cmp_exit=0` |

Full gate command:

```text
timeout 900s env MHS_GC_NODE_INTERVAL=33554432 target/release/mhs-rust-bench \
  --input /tmp/mhs-selfhost.comb --mode main --warmup-iters 0 --iters 1 -- \
  ./bin/mhs -i -imhs -isrc -ilib MicroHs.Main -o/tmp/mhs-selfhost-rust-out-argbind.comb
```

```text
parse_reduce_render_total_ms: 191234.683
whnf_steps_per_iter: 3659074921.0
whnf_steps_per_s: 19133950.3
step_limited_iters: 0
serialize_sink: 661902
gc_collections: 125
gc_freed_nodes_total: 4192827129
gc_last_live_nodes: 1637972
gc_last_free_nodes: 38320092
gc_high_water_nodes: 39958064
gc_current_nodes: 39958064
gc_current_free_nodes: 24171662
gc_last_pause_ms: 147.603
gc_total_pause_ms: 30727.554
```

```text
sha256(/tmp/mhs-selfhost.comb) = 29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a
sha256(/tmp/mhs-selfhost-rust-out-argbind.comb) = 29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a
cmp_exit=0
```

Reading: keep this slice. It is small, but it points in the right direction:
bind app-cell arguments once in the stack machine instead of repeating stack
index arithmetic and node matches inside each rewrite arm. The 100M/full split
again says full self-host is the real benchmark; even a strong bounded slice can
shrink to a one-second full-gate win once GC noise and phase mix are included.

Historical rejected follow-up: I retried eval.c-style scalar redex overwrite
only in the accepted stack-frame return path after the stack evaluator and
compact-cell work had landed. The probe wrote
`Int`/`Int64`/`Float64`/`Float32`/conversion scalar results directly into the
consumed app cell and left boolean/order results on the existing
permanent-combinator indirection path. It still regressed the 1M sanity slice,
so it was reverted before any larger gate:

| run | kept arg-binding source | stack-only scalar overwrite | result |
|---|---:|---:|---|
| 1M self-host slice | ~65.0 ms kept band | 66.2 ms | rejected |
| post-revert 1M sanity | 61.2 ms | n/a | back in accepted band |

Reading at that point: `SETINT`/`SETDBL` was still the C shape to aim for, but
not as a narrow branch in that strict-frame return path. A later stack scalar
`SET*` retest, after more S1 app/update work, is accepted only as a noise-band
alignment win; it does not change the next structural target.

### S1 Compact Stack-Entry Split

The update-profile counters showed the accepted stack path already reuses redex
cells for app results: the 1M profile had 824,100 stack app updates, 2,651,745
consumed app entries, and zero stack app-update allocations. WHNF/fallback
rethreading was also not the bottleneck: 4,560 rethread calls but only 11 app
cells rethreaded. The expensive remaining stack shape was more basic: every hot
app entry lived in `Vec<EvalStackEntry>`, whose size is forced by the largest
strict-frame payload and profile `Option<String>`.

This slice keeps the logical eval.c-style stack but splits storage: the hot
entry vector is now only `App(NodeId)` or `Frame`, while strict-frame payloads
live in a side LIFO vector and are rooted by GC alongside the app entries.

Checks:

| check | result |
|---|---|
| `cargo fmt --manifest-path rust/microhs-runtime/Cargo.toml` | passed |
| `cargo check --manifest-path rust/microhs-runtime/Cargo.toml` | passed |
| `cargo build --release --manifest-path rust/microhs-runtime/Cargo.toml --bin mhs-rust-bench` | passed |
| `MHS_GC_NODE_INTERVAL=32 cargo test --workspace` | passed; 34 tests |
| debug/release `cargo build --target wasm32-unknown-unknown --manifest-path rust/microhs-runtime/Cargo.toml` | passed |
| Node `host.mjs` wasm smoke | rendered `42` |
| `git diff --check` | passed |

Fresh slice and gate numbers, same 32M allocation trigger:

| run | stack argument binding | compact stack entries | result |
|---|---:|---:|---|
| 1M non-profile | ~65.0 ms kept band | 57.4 ms | win |
| 100M slice | 5.08s / 19.84M steps/s | 4.78s / 21.09M steps/s | win |
| full self-host | 191.2s / 19.13M steps/s | 169.4s / 21.60M steps/s | new best |
| full self-host GC pause | 30.73s | 30.70s | neutral |
| full self-host output | byte-identical | byte-identical | `cmp_exit=0` |

Full gate command:

```text
timeout 900s env MHS_GC_NODE_INTERVAL=33554432 target/release/mhs-rust-bench \
  --input /tmp/mhs-selfhost.comb --mode main --warmup-iters 0 --iters 1 -- \
  ./bin/mhs -i -imhs -isrc -ilib MicroHs.Main -o/tmp/mhs-selfhost-rust-out-compact-entry.comb
```

```text
parse_reduce_render_total_ms: 169424.987
whnf_steps_per_iter: 3659075035.0
whnf_steps_per_s: 21597021.2
step_limited_iters: 0
serialize_sink: 661902
gc_collections: 125
gc_freed_nodes_total: 4192827128
gc_last_live_nodes: 1637985
gc_last_free_nodes: 38320091
gc_high_water_nodes: 39958076
gc_current_nodes: 39958076
gc_current_free_nodes: 24171517
gc_last_pause_ms: 148.831
gc_total_pause_ms: 30702.037
```

```text
sha256(/tmp/mhs-selfhost.comb) = 29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a
sha256(/tmp/mhs-selfhost-rust-out-compact-entry.comb) = 29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a
cmp_exit=0
```

Reading: keep this slice. It is exactly the kind of structural win `NOTES.md`
was pointing at: not fewer reductions, not less graph allocation, but cheaper
per-step stack traffic. The post-split profile still reports 263.1M
spine-arity entries per 1M steps, 1.10M app allocations, 174k stack rewrites,
and 824k stack app updates, so the next 120s work should keep attacking the
spine descent/app argument load path rather than scalar result allocation.

### S1 Trusted Stack Args + Static Runtime Fallback Names

The first post-compact-entry profile corrected one stale interpretation. The
263.1M number is the arity histogram mass, not real app-cell loads. New
profile counters on the 1M self-host slice show the actual stack work:

| counter | value |
|---|---:|
| stack descent pushes | 3,009,528 |
| stack arg reads | 2,994,601 |
| stack arg batches | 887,603 |
| stack app updates | 824,100 |
| stack rewrites | 174,404 |
| stack rethreaded apps | 11 |

Reading: app descent and `ARG(TOP(i))` loads are real but not the whole 120s
gap by themselves. The next cost is dispatch/update machinery around those
loads. This slice removes two Rust-only checks/allocations from that path:

- runtime primitive fallback names are now static `&'static str` values, and
  the stack evaluator only keeps one when the primitive has no strict action;
- stack app entries are trusted after spine descent, so stack argument helpers
  return `NodeId` directly instead of `Result<NodeId, EvalError>`.

Checks:

| check | result |
|---|---|
| `cargo fmt --manifest-path rust/microhs-runtime/Cargo.toml` | passed |
| `cargo check --manifest-path rust/microhs-runtime/Cargo.toml` | passed |
| `git diff --check` | passed |
| `cargo build --release --manifest-path rust/microhs-runtime/Cargo.toml --bin mhs-rust-bench` | passed |
| `MHS_GC_NODE_INTERVAL=32 cargo test --workspace` | passed; 34 tests |
| debug/release `cargo build --target wasm32-unknown-unknown --manifest-path rust/microhs-runtime/Cargo.toml` | passed |
| Node `host.mjs` wasm smoke | rendered `42` |

Fresh slice and gate numbers, same 32M allocation trigger:

| run | compact stack entries | trusted args/static names | result |
|---|---:|---:|---|
| 1M non-profile | 57.4 ms | 57.1 ms | flat/no regression |
| 100M slice | 4.78s / 21.09M steps/s | 4.46s / 22.60M steps/s | win |
| full self-host | 169.4s / 21.60M steps/s | 158.5s / 23.09M steps/s | new best |
| full self-host GC pause | 30.70s | 29.40s | small win |
| full self-host output | byte-identical | byte-identical | `cmp_exit=0` |

Full gate command:

```text
timeout 900s env MHS_GC_NODE_INTERVAL=33554432 target/release/mhs-rust-bench \
  --input /tmp/mhs-selfhost.comb --mode main --warmup-iters 0 --iters 1 -- \
  ./bin/mhs -i -imhs -isrc -ilib MicroHs.Main -o/tmp/mhs-selfhost-rust-out-trusted-stack-arg.comb
```

```text
parse_reduce_render_total_ms: 158498.978
whnf_steps_per_iter: 3659075111.0
whnf_steps_per_s: 23085796.3
step_limited_iters: 0
serialize_sink: 661902
gc_collections: 125
gc_freed_nodes_total: 4192827118
gc_last_live_nodes: 1638004
gc_last_free_nodes: 38320082
gc_high_water_nodes: 39958086
gc_current_nodes: 39958086
gc_current_free_nodes: 24171413
gc_last_pause_ms: 149.398
gc_total_pause_ms: 29400.795
```

```text
sha256(/tmp/mhs-selfhost.comb) = 29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a
sha256(/tmp/mhs-selfhost-rust-out-trusted-stack-arg.comb) = 29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a
cmp_exit=0
```

Reading: keep this slice. The win is still mechanical, not a work-count
change. It also narrows the next target: the stack evaluator still round-trips
every reduction through `StackStep`, which is closer to the remaining eval.c
gap than another scalar-result retry. A narrower trusted-update-entry cleanup
was tried after this slice and rejected.

Phase timing is now available behind the explicit `eval-phase-profile` Cargo
feature. Build it only for profile runs, then rebuild the default release bench
before any gate:

```text
cargo build --release --bin mhs-rust-bench --features eval-phase-profile
timeout 180s env MHS_GC_NODE_INTERVAL=33554432 target/release/mhs-rust-bench \
  --input /tmp/mhs-selfhost.comb --mode main --step-limit 1000000 \
  --warmup-iters 0 --iters 1 --profile --profile-top 25 -- \
  ./bin/mhs -i -imhs -isrc -ilib MicroHs.Main -o/tmp/mhs-selfhost-rust-1m-phase-feature-profile.comb
```

It is high-overhead in absolute terms, but useful as a ranking signal. After
the strict Int immediate-arg fast path, the 1M self-host profile is:

| profiled stack phase | time |
|---|---:|
| stack eval step | 199.6 ms |
| app descent | 29.1 ms |
| root resolve | 19.0 ms |
| GC check | 18.2 ms |
| ready-frame check/return | 3.9 ms |
| WHNF/fallback handling | 0.3 ms |

Counts from the same run: 1,051,729 stack-loop iterations, 1,003,441
`stack_eval_step` calls, 961,160 reduced steps, 37,345 force steps, 4,846 WHNF
steps, and 90 fallback steps. The feature build's profile total was 362.1ms.
Before the strict Int fast path, this same feature profile was 403.3ms total
with 220.9ms in `stack_eval_step`, 51,591 force steps, and 7.1ms in
ready-frame check/return. After rebuilding the default bench without the
feature before the strict Int gate, the normal 1M/100M slices were 52.9ms and
4.14s/4.25s.

The same feature now also has a deliberately higher-overhead per-head timing
view inside `stack_eval_step`. Latest 1M head-timing profile:

| stack-step head | time |
|---|---:|
| `B` | 19.3 ms |
| `S'` | 16.2 ms |
| `C` | 13.9 ms |
| `C'` | 11.3 ms |
| `S` | 9.5 ms |
| `C'B` | 9.1 ms |
| `P` | 4.4 ms |
| `A.read` | 2.7 ms |
| `==` | 2.7 ms |
| `O` | 2.3 ms |

This run had `profile_total_ms=416.089` and `profile_stack_eval_step_ms=254.199`,
so use it only for ranking. Reading: the remaining time is spread across the
standard combinator rewrite/app-allocation arms. A batch two-app allocation
probe based on this ranking was neutral/slower and rejected below.

Default full gate after the stack-head phase-profile/default rebuild
(then-best remained the strict Int 147.5s gate):

```text
timeout 900s env MHS_GC_NODE_INTERVAL=33554432 target/release/mhs-rust-bench \
  --input /tmp/mhs-selfhost.comb --mode main --warmup-iters 0 --iters 1 -- \
  ./bin/mhs -i -imhs -isrc -ilib MicroHs.Main -o/tmp/mhs-selfhost-rust-out-head-profile-default.comb
```

```text
parse_reduce_render_total_ms: 150055.732
whnf_steps_per_iter: 3659075168.0
whnf_steps_per_s: 24384774.4
gc_collections: 125
gc_total_pause_ms: 30249.606
gc_high_water_nodes: 39958092
sha256(input/output): 29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a
cmp_exit=0
```

Reading: the remaining mutator gap is inside the combinator/strict dispatch
and rewrite arms, not in spine descent, resolve chains, WHNF rethreading, or
GC checks. Local code-shape probes around that dispatch were still losers; the
next evaluator change needs to remove a real layer or change the update/app
allocation shape, not just rearrange matches.

### S1 Strict Int Immediate-Arg Fast Path

`eval.c` checks strict integer arguments in-place before pushing the
`BININT2`/`BININT1` marker path: if the second argument is already an `Int`, it
skips directly to the first-argument marker; if both are already `Int`, it jumps
straight to the arithmetic result. The Rust stack path now mirrors that narrow
case for `IntBin` instead of always pushing a strict frame.

Slices:

| run | before | strict Int immediate | result |
|---|---:|---:|---|
| 1M non-profile | 53.0 ms | 52.9 ms | flat/small win |
| 100M slice | 4.50s / 4.22s | 4.14s / 4.25s | win/noise-band win |
| full self-host | 149.2s / 24.53M steps/s | 147.5s / 24.81M steps/s | new best |

Full gate:

```text
timeout 900s env MHS_GC_NODE_INTERVAL=33554432 target/release/mhs-rust-bench \
  --input /tmp/mhs-selfhost.comb --mode main --warmup-iters 0 --iters 1 -- \
  ./bin/mhs -i -imhs -isrc -ilib MicroHs.Main -o/tmp/mhs-selfhost-rust-out-intbin-immediate.comb
```

```text
parse_reduce_render_total_ms: 147471.070
whnf_steps_per_iter: 3659075092.0
whnf_steps_per_s: 24812155.3
gc_collections: 125
gc_total_pause_ms: 29018.124
gc_high_water_nodes: 39958084
sha256(input/output): 29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a
cmp_exit=0
```

Reading: keep this slice. It is not a match reshuffle; it removes real strict
frame traffic in the same place eval.c avoids it. Next likely siblings are
only worth trying where the profile shows enough strict-frame traffic; `Int`
binops are hot, while `Int` unops are almost absent in the 1M profile.
Post-acceptance 1M profiles are in the same band: latest non-profile
`parse_reduce_render_total_ms=54.051`, `profile_total_ms=208.510`,
`persistent_forces` 37,345, `eval_frame_pushes` 52,933, and
`profile_eval_frame_push_kinds` `Int=48,288`, `Whnf=4,645`.

### S1 Unchecked Stack Arg Loads

The accepted stack path already maintains eval.c's invariant that the active app
segment contains only application cells. `eval.c` reads arguments as raw
`ARG(TOP(i))` loads after `CHKARGn`; Rust still paid safe vector indexing and a
`Node::App` discriminant check in every `EvalStack::arg_at_app`. This slice keeps
the invariant at the stack boundary and makes stack argument loads unchecked in
that one helper.

Slices:

| run | strict Int baseline | unchecked stack args | result |
|---|---:|---:|---|
| 1M non-profile | 54.2 ms latest sanity | 50.9 ms / 52.3 ms | win/noise-band win |
| 100M slice | 4.14s / 4.25s accepted band | 4.03s / 4.30s | noisy but promising |
| full self-host | 147.5s / 24.81M steps/s | 145.8s / 25.10M steps/s | new best |

Full gate:

```text
timeout 900s env MHS_GC_NODE_INTERVAL=33554432 target/release/mhs-rust-bench \
  --input /tmp/mhs-selfhost.comb --mode main --warmup-iters 0 --iters 1 -- \
  ./bin/mhs -i -imhs -isrc -ilib MicroHs.Main -o/tmp/mhs-selfhost-rust-out-unchecked-stack-args.comb
```

```text
parse_reduce_render_total_ms: 145760.452
whnf_steps_per_iter: 3659075168.0
whnf_steps_per_s: 25103346.8
gc_collections: 125
gc_total_pause_ms: 30052.299
gc_high_water_nodes: 39958092
sha256(input/output): 29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a
cmp_exit=0
```

Validation after the unsafe hot-path change:

| check | result |
|---|---|
| `cargo fmt --manifest-path rust/microhs-runtime/Cargo.toml` | passed |
| `cargo check --manifest-path rust/microhs-runtime/Cargo.toml` | passed |
| `cargo check --manifest-path rust/microhs-runtime/Cargo.toml --features eval-phase-profile` | passed |
| `cargo test --workspace` | passed; 34 tests |
| `cargo build --release --manifest-path rust/microhs-runtime/Cargo.toml --bin mhs-rust-bench` | passed |
| debug/release `cargo build --target wasm32-unknown-unknown --manifest-path rust/microhs-runtime/Cargo.toml` | passed |
| Node `host.mjs` wasm smoke | rendered `42` |
| `git diff --check` | passed |

Reading: keep this slice. It is the first post-Int change to move the full gate
again, and it follows Lennart's evaluator invariant directly: after the stack
contains app pointers, argument reads are trusted. The feature-timed profile is
too high-overhead to rank this particular win; the default slices and full gate
are the useful evidence. The next sibling to try is the redex/update side of
the same invariant, but previous trusted-update probes warn that app-cell writes
need to be tested against the full gate, not accepted on 1M/100M alone.

### S0 Broad Eval-Profile Gate Probe (Rejected)

A fresh 10M phase-profile slice on the current accepted worktree confirmed that
the remaining hot cost is inside the combinator rewrite step, not the fallback
bridge or WHNF return path:

| measure | value |
|---|---:|
| 10M non-profile slice, phase-feature binary | 509.0 ms |
| 10M profile total | 4,188.7 ms |
| WHNF steps | 10,028,861 |
| `stack_eval_step` calls | 10,049,395 |
| `stack_eval_step` profiled time | 2,584.7 ms |
| stack descent / resolve / ready-frame time | 303.9 ms / 193.0 ms / 38.8 ms |
| stack descent pushes | 30,244,480 |
| stack arg reads / batches | 30,119,438 / 9,372,735 |
| stack app updates / app cells used | 8,292,659 / 26,749,026 |
| stack rewrites / rewrite apps | 1,711,077 / 3,495,001 |
| stack rethreads / app cells moved | 44,963 / 11 |
| persistent fallbacks / fallback loop steps | 90 / 90 |
| eval-frame pushes | 550,492 (`Int`: 504,396; `Whnf`: 45,048; `Bytes`: 1,048) |

Because normal builds still carried general profiling branches in hot helpers,
a broader `eval-profile` feature gate was tested: `eval-phase-profile` depended
on it, default `--profile` errored clearly, and hot `self.profile.is_some()`
checks compiled to false. The slices looked useful but the full gate rejected
the shape:

| run | accepted unchecked args | broad eval-profile gate | result |
|---|---:|---:|---|
| 1M non-profile | 50.9 ms / 52.3 ms accepted band | 52.3 ms | neutral |
| 100M slice | 4.03s / 4.30s accepted band | 3.77s / 3.78s | promising slice win |
| full self-host | 145.8s / 25.10M steps/s | 152.7s / 23.97M steps/s | rejected |
| full self-host GC pause | 30.05s | 33.74s | worse |
| full self-host output | byte-identical | byte-identical | `cmp_exit=0` |

Full rejected gate:

```text
parse_reduce_render_total_ms: 152656.978
whnf_steps_per_iter: 3659075035.0
whnf_steps_per_s: 23969261.6
gc_collections: 125
gc_total_pause_ms: 33735.363
gc_high_water_nodes: 39958076
sha256(input/output): 29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a
cmp_exit=0
```

Reading: reverted. This is the same lesson as the older const-specialized
profile-vs-normal step probe: code-shape changes that look cleaner and win on
bounded slices can still lose the whole compile. Do not retry broad profile
gating as an isolated lever; keep profiling available and spend the next S1 cut
on a real evaluator-layer deletion.

### S2 Reusable GC Mark-Work Stack

`eval.c` reuses the evaluator stack as the GC mark stack. The Rust collector
already reused the mark bitmap but allocated and dropped a fresh `Vec<NodeId>`
worklist for every collection. This slice keeps a retained `gc_mark_work`
scratch vector in `Program`, clears it before/after marking, and reuses its
capacity across collections. It does not change roots, sweep policy, or free
list shape.

Slices:

| run | unchecked stack args | reusable mark work | result |
|---|---:|---:|---|
| 1M non-profile | 51.4 ms latest sanity | 51.6 ms | neutral |
| 100M slice | 4.03s / 4.30s accepted band | 4.06s, 299 ms GC pause | neutral |
| 500M slice | not rerun | 17.87s, 16 GCs, 1.99s GC pause, 28.18M steps/s | enough GC signal for full gate |
| full self-host | 145.8s / 25.10M steps/s | 143.9s / 25.43M steps/s | new best |
| full self-host GC pause | 30.05s | 29.46s | small win |
| full self-host output | byte-identical | byte-identical | `cmp_exit=0` |

Full gate:

```text
timeout 900s env MHS_GC_NODE_INTERVAL=33554432 target/release/mhs-rust-bench \
  --input /tmp/mhs-selfhost.comb --mode main --warmup-iters 0 --iters 1 -- \
  ./bin/mhs -i -imhs -isrc -ilib MicroHs.Main -o/tmp/mhs-selfhost-rust-out-gc-work.comb
```

```text
parse_reduce_render_total_ms: 143913.656
whnf_steps_per_iter: 3659074921.0
whnf_steps_per_s: 25425487.9
gc_collections: 125
gc_total_pause_ms: 29460.250
gc_high_water_nodes: 39958064
sha256(input/output): 29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a
cmp_exit=0
```

Validation after the GC scratch-stack change:

| check | result |
|---|---|
| `cargo fmt --manifest-path rust/microhs-runtime/Cargo.toml` | passed |
| `cargo check --manifest-path rust/microhs-runtime/Cargo.toml` | passed |
| `cargo check --manifest-path rust/microhs-runtime/Cargo.toml --features eval-phase-profile` | passed |
| `cargo test --workspace` | passed; 34 tests |
| `cargo build --release --manifest-path rust/microhs-runtime/Cargo.toml --bin mhs-rust-bench` | passed |
| debug/release `cargo build --target wasm32-unknown-unknown --manifest-path rust/microhs-runtime/Cargo.toml` | passed |
| Node `host.mjs` wasm smoke | rendered `42` |
| `git diff --check` | passed |

Reading: keep this slice. It is not the 120s lever by itself, but it follows
eval.c's reusable mark-stack discipline and moves the full gate without adding
mutator complexity. The next large win still has to delete evaluator work inside
the combinator/strict rewrite step.

### Rejected S1 Int First-Immediate Carry Probe

I tried extending the accepted strict-`Int` immediate fast path one step further:
when the first operand was already an `Int` but the second still needed forcing,
the frame stored the first operand as an `i64` and computed directly when the
second returned. This deleted many `Int` frame pushes in the 10M profile, but it
did not move wall time once the whole compiler ran.

| run | accepted reusable-GC-work | first-immediate carry | result |
|---|---:|---:|---|
| 1M slice | 51.4/51.6 ms band | 52.1 ms | no win |
| 100M slice | 4.03/4.30s band | 4.01s / 4.06s | noise-band |
| 500M slice | 17.87s / 28.18M steps/s | 17.86s / 28.20M steps/s | neutral |
| 10M profile `Int` frame pushes | 504,396 | 328,117 | fewer frames, not enough |
| full self-host | 143.9s / 25.43M steps/s | 145.6s / 25.14M steps/s | rejected |
| full self-host GC pause | 29.46s | 29.68s | slightly worse |
| full self-host output | byte-identical | byte-identical | `cmp_exit=0` |

Full rejected gate:

```text
timeout 900s env MHS_GC_NODE_INTERVAL=33554432 target/release/mhs-rust-bench \
  --input /tmp/mhs-selfhost.comb --mode main --warmup-iters 0 --iters 1 -- \
  ./bin/mhs -i -imhs -isrc -ilib MicroHs.Main -o/tmp/mhs-selfhost-rust-out-int-first.comb
```

```text
parse_reduce_render_total_ms: 145571.176
whnf_steps_per_iter: 3659074959.0
whnf_steps_per_s: 25135985.4
gc_collections: 125
gc_total_pause_ms: 29675.676
gc_high_water_nodes: 39958068
sha256(input/output): 29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a
cmp_exit=0
```

Reading: rejected and reverted. The accepted eval.c-style immediate case stays:
when the second operand is already `Int`, the stack path skips straight to the
first-operand marker and computes directly if both operands are already `Int`.
Carrying a scalar first operand across a later force removes marker traffic, but
the extra frame shape and dispatch do not pay on the full self-host gate.

Post-revert checks:

| check | result |
|---|---|
| `cargo fmt --manifest-path rust/microhs-runtime/Cargo.toml` | passed |
| `cargo check --manifest-path rust/microhs-runtime/Cargo.toml` | passed |
| `git diff --check` | passed |
| `cargo build --release --manifest-path rust/microhs-runtime/Cargo.toml --bin mhs-rust-bench` | passed |
| 1M self-host sanity after release rebuild | first cold run 63.1 ms, repeat 52.1 ms, back in accepted band |

Fresh 10M phase-profile snapshot on the reverted accepted tree:

| measure | value |
|---|---:|
| 10M non-profile slice, phase-feature binary | 490.6 ms |
| 10M profile total | 4,209.5 ms |
| WHNF steps | 10,028,861 |
| `stack_eval_step` calls | 10,049,395 |
| `stack_eval_step` profiled time | 2,608.3 ms |
| stack descent / resolve / ready-frame time | 301.8 ms / 192.6 ms / 39.9 ms |
| stack descent pushes | 30,244,480 |
| stack arg reads / batches | 30,119,438 / 9,372,735 |
| stack app updates / app cells used | 8,292,659 / 26,749,026 |
| stack rewrites / rewrite apps | 1,711,077 / 3,495,001 |
| stack rethreads / app cells moved | 44,963 / 11 |
| persistent fallbacks / fallback loop steps | 90 / 90 |
| eval-frame pushes | 550,492 (`Int`: 504,396; `Whnf`: 45,048; `Bytes`: 1,048) |

Top profiled `stack_eval_step` heads:

| head | profiled time |
|---|---:|
| `S'` | 187.2 ms |
| `B` | 172.5 ms |
| `C` | 148.2 ms |
| `C'` | 120.3 ms |
| `S` | 99.9 ms |
| `C'B` | 86.0 ms |
| `P` | 47.3 ms |
| `==` | 32.5 ms |
| `A.read` | 28.2 ms |
| `O` | 19.5 ms |

Reading: the profile is back to the accepted pre-probe shape. The remaining F2
target is still the hot combinator/update arms inside `stack_eval_step`, not
strict-frame traffic, fallback bridge work, descent, or resolve.

### S0 Feature-Gated GC Phase Profiling

`NOTES.md` called out that total GC pause alone was too coarse. I added an
explicit `gc-phase-profile` feature that records mark/reachability time and
sweep/free-list rebuild time per collection. Keeping those fields and timers in
the default build was tested and rejected: the full gate stayed byte-identical
but regressed to about 154.8-154.9s. With the fields compiled out unless the
feature is enabled, the default gate returned to the accepted band and nudged
the best full self-host gate to 143.7s.

| run | result |
|---|---:|
| always-on GC phase timing full gate | 154.9s / 23.62M steps/s, byte-identical, rejected |
| default 1M after `cfg` gating | 52.8 ms |
| default 100M after `cfg` gating | 3.99s / 25.24M steps/s, 312 ms GC |
| default full self-host after `cfg` gating | 143.7s / 25.46M steps/s, byte-identical, `cmp_exit=0` |
| `gc-phase-profile` 100M slice | 4.16s / 24.25M steps/s, 291 ms GC |
| `gc-phase-profile` 100M GC split | 72.7 ms mark, 190.9 ms sweep |

Default full gate:

```text
timeout 900s env MHS_GC_NODE_INTERVAL=33554432 target/release/mhs-rust-bench \
  --input /tmp/mhs-selfhost.comb --mode main --warmup-iters 0 --iters 1 -- \
  ./bin/mhs -i -imhs -isrc -ilib MicroHs.Main -o/tmp/mhs-selfhost-rust-gcphase-cfg-full.comb
```

```text
parse_reduce_render_total_ms: 143746.283
whnf_steps_per_iter: 3659075016.0
whnf_steps_per_s: 25455093.1
gc_collections: 125
gc_total_pause_ms: 30168.429
gc_high_water_nodes: 39958074
sha256(input/output): 29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a
cmp_exit=0
```

Feature-profile 100M slice:

```text
cargo build --release --manifest-path rust/microhs-runtime/Cargo.toml \
  --bin mhs-rust-bench --features gc-phase-profile
timeout 300s env MHS_GC_NODE_INTERVAL=33554432 target/release/mhs-rust-bench \
  --input /tmp/mhs-selfhost.comb --mode main --step-limit 100000000 \
  --warmup-iters 0 --iters 1 -- \
  ./bin/mhs -i -imhs -isrc -ilib MicroHs.Main -o/tmp/mhs-selfhost-rust-gcphase-feature-100m.comb
```

```text
parse_reduce_render_total_ms: 4157.809
whnf_steps_per_iter: 100811774.0
whnf_steps_per_s: 24246371.5
gc_collections: 3
gc_total_pause_ms: 291.364
gc_total_mark_ms: 72.732
gc_total_sweep_ms: 190.918
gc_high_water_nodes: 34050272
```

Reading: this is profiling, not a mutator optimization. The useful new fact is
that sweep/free-list rebuild is the larger measured GC bucket on the 100M slice.
That led to the eval.c-style bitmap allocator probe below; it cut measured GC
pause but lost more time in the allocation path, so the intrusive free list
stays for now. Do not retry the previously rejected preserve-list,
compact-free-stack, or bitmap shapes as standalone GC tweaks.

### Rejected S2 Free Bitmap Allocator

I tried replacing the intrusive `Node::Free(next)` list with an eval.c-style
free bitmap scanned on allocation. To preserve Rust drop semantics, the probe
only tombstoned dead slots whose variants own payloads (`Bytes`, `Array`,
boxed FFI/JS/weak/finalizer objects, etc.); ordinary dead app/scalar slots were
represented by free bits instead of overwritten during sweep.

| run | result |
|---|---:|
| bitmap 1M slice | 53.4-53.9 ms, no GC |
| bitmap 100M slices | 4.17-4.19s / 24.1M steps/s, 266-268 ms GC |
| bitmap full self-host | 150.6s / 24.29M steps/s, 125 GCs, 28.1s GC, byte-identical |
| bitmap fast-pop retry | 1M regressed to 56.2 ms; 100M regressed to 4.27s despite 272 ms GC |
| intrusive restore sanity | 1M 52.5 ms; 100M 4.10s; full 144.9s, byte-identical |

Default full bitmap gate:

```text
parse_reduce_render_total_ms: 150637.990
whnf_steps_per_iter: 3659075149.0
whnf_steps_per_s: 24290520.2
gc_collections: 125
gc_total_pause_ms: 28104.134
gc_high_water_nodes: 39958090
sha256(input/output): 29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a
cmp_exit=0
```

Restored intrusive-list full gate after re-adding the `free_nodes == 0` fast
return in `pop_free_node`:

```text
parse_reduce_render_total_ms: 144876.962
whnf_steps_per_iter: 3659075263.0
whnf_steps_per_s: 25256432.9
gc_collections: 125
gc_total_pause_ms: 29535.654
gc_high_water_nodes: 39958100
sha256(input/output): 29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a
cmp_exit=0
```

Reading: the bitmap did what it was meant to do inside GC, reducing full-gate
pause from about 30.2s to 28.1s, but it cost roughly 6.9s wall time against
the 143.7s best. The first revert accidentally exposed another allocation-path
lesson: even the no-free path needs the cheap `free_nodes == 0` guard before
touching free-list head state. Standalone GC structure changes are not the
120s lever; keep working in the evaluator/update path unless a future compact
cell/allocator redesign changes the cost model.

Validation after rejecting/restoring the bitmap probe:

| check | result |
|---|---|
| `cargo fmt --manifest-path rust/microhs-runtime/Cargo.toml` | passed |
| `cargo check --manifest-path rust/microhs-runtime/Cargo.toml` | passed |
| `cargo check --manifest-path rust/microhs-runtime/Cargo.toml --features gc-phase-profile` | passed |
| `cargo check --manifest-path rust/microhs-runtime/Cargo.toml --features eval-phase-profile,gc-phase-profile` | passed |
| `MHS_GC_NODE_INTERVAL=32 cargo test --workspace` | passed; 34 tests |
| `cargo build --release --manifest-path rust/microhs-runtime/Cargo.toml --bin mhs-rust-bench` | passed |
| debug/release `cargo build --target wasm32-unknown-unknown --manifest-path rust/microhs-runtime/Cargo.toml` | passed |
| Node `host.mjs` wasm smoke | rendered `42` |
| `git diff --check` | passed |

Validation after the feature-gated instrumentation:

| check | result |
|---|---|
| `cargo fmt --manifest-path rust/microhs-runtime/Cargo.toml` | passed |
| `cargo check --manifest-path rust/microhs-runtime/Cargo.toml` | passed |
| `cargo check --manifest-path rust/microhs-runtime/Cargo.toml --features gc-phase-profile` | passed |
| `cargo check --manifest-path rust/microhs-runtime/Cargo.toml --features eval-phase-profile,gc-phase-profile` | passed |
| `cargo test --workspace` | passed; 34 tests |
| `cargo build --release --manifest-path rust/microhs-runtime/Cargo.toml --bin mhs-rust-bench` | passed; rebuilt default after feature run |
| debug/release `cargo build --target wasm32-unknown-unknown --manifest-path rust/microhs-runtime/Cargo.toml` | passed |
| Node `host.mjs` wasm smoke | rendered `42` |
| `git diff --check` | passed |

### Current S1 Phase Profile After Bitmap Revert

After restoring the intrusive free list, I reran the 1M self-host profile in
both the default binary and the `eval-phase-profile` release binary. The profile
confirms that the GC allocator was a detour: the hot time is still inside
`stack_eval_step` and specifically the combinator rewrite/app-allocation arms.

| measure | value |
|---|---:|
| default 1M slice | 52.4 ms / 19.09M steps/s |
| phase-profile 1M non-profile run | 59.1 ms / 16.93M steps/s |
| phase-profile profiled run | 434.3 ms |
| WHNF steps | 1,001,383 |
| app allocations | 1,102,701 |
| stack app updates / updated apps | 824,100 / 2,651,745 |
| stack rewrites / rewrite apps | 174,404 / 357,222 |
| strict-redex snapshots / remaining-app scans | 0 / 0 |
| fallback eval-loop steps | 90 |

Phase timing inside the 1M profiled run:

| stack phase | time |
|---|---:|
| `stack_eval_step` | 268.7 ms |
| descent | 30.7 ms |
| resolve | 19.4 ms |
| GC check | 18.3 ms |
| ready-frame check | 4.0 ms |
| WHNF finish | 0.3 ms |

Top `stack_eval_step` head timings:

| head | time |
|---|---:|
| `B` | 20.0 ms |
| `S'` | 16.7 ms |
| `C` | 14.5 ms |
| `C'` | 11.9 ms |
| `S` | 10.1 ms |
| `C'B` | 9.4 ms |
| `P` | 4.7 ms |
| `==` | 3.1 ms |
| `A.read` | 2.9 ms |
| `O` | 2.4 ms |

Top app allocation sites are still the normal combinator payloads:
`B.yz` 200,239; `C.xz` 156,026; each `S'` site 72,271; each `C'` site
72,229; each `S` site 59,402; each `C'B` site 57,660; `P.zx` 47,698.

Reading: the accepted S1 invariants are holding: no strict-redex snapshots, no
remaining-app scans, and only 90 fallback steps per 1M. The next credible
performance slice has to change the representation or update path for the hot
combinator/app result arms. Isolated app allocation helpers, scalar `SET*`,
direct head dispatch, and trusted-update-entry cleanups are already rejected as
standalone probes.

### S1 C-Style App Continuation

`eval.c`'s `GOAP`/`GOAP2` rewrites do not return to the top-level driver after
building an app result; they jump to `ap`/`ap2` and immediately descend through
the produced app cells. Rust previously rewrote the redex, returned
`StackStep::Reduced`, and then paid the outer loop's GC check, resolve,
ready-frame check, and descent before the next reduction. This slice keeps
app-producing stack reductions inside `stack_eval_step`: after updating the
redex it descends through the produced app spine and continues the evaluator
loop with the same stack.

Implementation notes:

| point | current shape |
|---|---|
| app-producing rewrites | continue inside `stack_eval_step` after `apply_stack_app` |
| reduction accounting | `StackStep::{Reduced,Force,Whnf,Fallback}` carries accumulated reductions |
| stack rethreading | `Whnf`/`Fallback` carry the current head so outer rethreading does not use a stale head |
| GC safe point | still at the outer driver; continuation can overshoot by one app-producing chain, matching the C-style direction |

Benchmarks, 32M allocation trigger:

| run | previous accepted band | app-continuation | result |
|---|---:|---:|---|
| 1M slice | 52.1-52.4 ms | 50.132 ms / 19.98M steps/s | win |
| 100M slice | 4.10s restored / 3.99s best-band | 3.760s / 26.81M steps/s | win |
| 100M GC pause | ~286-312 ms | 285.630 ms | neutral |
| full self-host | 143.7s / 25.46M steps/s | 137.6s / 26.59M steps/s | new best |
| full GC pause | ~30.2s | 29.952s | neutral/slight win |
| full output | byte-identical | byte-identical | `cmp_exit=0` |

Full gate command:

```text
timeout 900s env MHS_GC_NODE_INTERVAL=33554432 target/release/mhs-rust-bench \
  --input /tmp/mhs-selfhost.comb --mode main --warmup-iters 0 --iters 1 -- \
  ./bin/mhs -i -imhs -isrc -ilib MicroHs.Main -o/tmp/mhs-selfhost-rust-appcont-full.comb
```

Full gate output:

| measure | value |
|---|---:|
| parse/reduce/render | 137601.016 ms |
| WHNF steps | 3,659,074,940 |
| WHNF steps/s | 26,591,918 |
| step-limited iters | 0 |
| serialize sink | 661,902 |
| GC collections | 125 |
| GC total pause | 29,951.796 ms |
| GC high-water nodes | 39,958,351 |

Checks:

| check | result |
|---|---|
| `cargo fmt --manifest-path rust/microhs-runtime/Cargo.toml` | passed |
| `cargo check --manifest-path rust/microhs-runtime/Cargo.toml` | passed |
| `cargo build --release --manifest-path rust/microhs-runtime/Cargo.toml --bin mhs-rust-bench` | passed |
| `env MHS_GC_NODE_INTERVAL=32 cargo test --workspace` | passed; 34 tests |
| `cargo check --manifest-path rust/microhs-runtime/Cargo.toml --features gc-phase-profile` | passed |
| `cargo check --manifest-path rust/microhs-runtime/Cargo.toml --features eval-phase-profile,gc-phase-profile` | passed |
| debug/release `cargo build --target wasm32-unknown-unknown --manifest-path rust/microhs-runtime/Cargo.toml` | passed |
| Node `host.mjs` wasm smoke | rendered `42` |
| post-rejected-force revert `cargo check` + release bench build | passed |
| post-rejected-force 1M sanity | 52.161 ms / 19.20M steps/s, sink `661902` |

Reading: this is the accepted version of the `GOAP`/`GOAP2` continuation idea.
The earlier rejected "stack `GOAP` continuation return" still returned control
to the outer driver and paid enough extra bookkeeping to lose at full scale.
This version stays in the stack evaluator, preserves the current stack, and
carries reduction/head metadata only when it finally has to hand control back.

Fresh 10M `eval-phase-profile` snapshot on the accepted app-continuation tree:

| measure | value |
|---|---:|
| non-profile phase-feature slice | 450.396 ms / 22.27M steps/s |
| profile total | 2,982.814 ms |
| WHNF steps | 10,028,861 |
| `stack_eval_step` calls | 1,756,737 |
| stack loop iterations | 2,262,181 |
| stack steps: reduced / force / WHNF / fallback | 1,340,423 / 370,656 / 45,568 / 90 |
| eval-frame pushes | 550,492 |
| app allocations | 11,215,904 |
| stack app updates / updated apps | 8,292,659 / 26,749,026 |
| stack descent pushes / arg reads | 30,244,480 / 30,119,438 |
| strict-redex snapshots / remaining-app scans | 0 / 0 |

Phase timing inside that profiled run:

| stack phase | time |
|---|---:|
| `stack_eval_step` | 2,632.306 ms |
| descent | 44.731 ms |
| resolve | 44.522 ms |
| GC check | 39.316 ms |
| ready-frame check | 37.810 ms |
| WHNF finish | 2.980 ms |

Top `stack_eval_step` head timings after app-continuation:

| head | time |
|---|---:|
| `S'` | 186.723 ms |
| `B` | 172.601 ms |
| `C` | 147.136 ms |
| `C'` | 120.774 ms |
| `S` | 98.630 ms |
| `C'B` | 86.758 ms |
| `P` | 47.494 ms |
| `==` | 30.962 ms |
| `A.read` | 26.330 ms |
| `O` | 19.209 ms |

Reading: app-continuation reduced the outer driver boundary, but the profile is
still overwhelmingly inside the hot combinator/update arms. Strict force
returns are visible at 370k/10M, so I tried the eval.c-looking sibling: after
pushing a strict marker, continue inside `stack_eval_step` and add an internal
ready-frame check for immediate `ret:` handling. It was correct but not faster:

| run | app-continuation baseline | strict-force continuation | result |
|---|---:|---:|---|
| 1M slice | 50.132 ms best / 52.1 ms band | 51.414 ms | neutral/slower than best |
| 100M slice | 3.760s / 26.81M steps/s | 3.759s / 26.82M steps/s | neutral |
| 100M GC pause | 285.630 ms | 273.569 ms | small GC win |
| full self-host | 137.6s / 26.59M steps/s | 138.1s / 26.49M steps/s | rejected |
| full GC pause | 29.952s | 29.218s | GC win, mutator loss dominates |
| full output | byte-identical | byte-identical | `cmp_exit=0` |

Full rejected gate: 3,659,074,978 steps, 125 GCs, high-water 39,958,355 nodes,
`parse_reduce_render_total_ms=138133.620`, `gc_total_pause_ms=29218.336`.

#### Direct `ap`/`ap2` App-Result Descent

The accepted app-continuation slice still used generic descent on the just
rewritten root app: resolve the root, match it as `App`, push it, then follow
the fun. `eval.c`'s labels are narrower: `GOAP` writes the root app and jumps
to `ap`, while `GOAP2` writes the root app and jumps to `ap2`; both labels
already know the root is an app and push it directly before continuing from
its fun. This slice binds the app-result `fun`/`arg` once, updates the redex,
pushes the known root app directly, and calls generic descent only from the
already-bound fun. It is the same continuation path, but with the first
root-app resolve/match removed.

Benchmarks, 32M allocation trigger:

| run | app-continuation | direct app-result descent | result |
|---|---:|---:|---|
| 1M slice | 50.132 ms / 19.98M steps/s | 48.512 ms / 20.64M steps/s | win |
| 100M slice | 3.760s / 26.81M steps/s | 3.605s / 27.96M steps/s | win |
| 100M GC pause | 285.630 ms | 313.933 ms | worse GC/noise; mutator wins |
| full self-host | 137.6s / 26.59M steps/s | 131.6s / 27.80M steps/s | new best |
| full GC pause | 29.952s | 30.207s | slight regression/noise |
| full output | byte-identical | byte-identical | `cmp_exit=0` |

Full gate command:

```text
timeout 900s env MHS_GC_NODE_INTERVAL=33554432 target/release/mhs-rust-bench \
  --input /tmp/mhs-selfhost.comb --mode main --warmup-iters 0 --iters 1 -- \
  ./bin/mhs -i -imhs -isrc -ilib MicroHs.Main -o/tmp/mhs-selfhost-rust-directapp-full.comb
```

Full gate output:

| measure | value |
|---|---:|
| parse/reduce/render | 131618.326 ms |
| WHNF steps | 3,659,074,978 |
| WHNF steps/s | 27,800,650 |
| step-limited iters | 0 |
| serialize sink | 661,902 |
| GC collections | 125 |
| GC total pause | 30,206.636 ms |
| GC high-water nodes | 39,958,355 |

Checks:

| check | result |
|---|---|
| `cargo fmt --manifest-path rust/microhs-runtime/Cargo.toml` | passed |
| `cargo check --manifest-path rust/microhs-runtime/Cargo.toml` | passed |
| `cargo test --manifest-path rust/microhs-runtime/Cargo.toml` | passed; 34 tests |
| `cargo build --release --manifest-path rust/microhs-runtime/Cargo.toml --bin mhs-rust-bench` | passed |
| `cargo check --manifest-path rust/microhs-runtime/Cargo.toml --features gc-phase-profile` | passed |
| `cargo check --manifest-path rust/microhs-runtime/Cargo.toml --features eval-phase-profile,gc-phase-profile` | passed |
| debug/release `cargo build --target wasm32-unknown-unknown --manifest-path rust/microhs-runtime/Cargo.toml` | passed |
| Node `host.mjs` wasm smoke | rendered `42` |

Fresh 10M `eval-phase-profile` snapshot:

| measure | app-continuation | direct app-result descent |
|---|---:|---:|
| non-profile phase-feature slice | 450.396 ms | 445.627 ms |
| profile total | 2,982.814 ms | 2,952.809 ms |
| `stack_eval_step` profiled time | 2,632.306 ms | 2,603.692 ms |
| descent / resolve | 44.731 ms / 44.522 ms | 45.187 ms / 43.848 ms |
| `stack_eval_step` calls | 1,756,737 | 1,756,737 |
| stack descent pushes / arg reads | 30,244,480 / 30,119,438 | 30,244,480 / 30,119,438 |
| app allocations | 11,215,904 | 11,215,908 |
| strict-redex snapshots / remaining-app scans | 0 / 0 | 0 / 0 |

Top `stack_eval_step` head timings after direct app-result descent:

| head | time |
|---|---:|
| `S'` | 182.050 ms |
| `B` | 172.894 ms |
| `C` | 144.534 ms |
| `C'` | 118.959 ms |
| `S` | 98.742 ms |
| `C'B` | 86.155 ms |
| `P` | 47.364 ms |
| `==` | 31.526 ms |
| `A.read` | 25.193 ms |
| `O` | 19.792 ms |

Reading: this validates the narrow part of Lennart's `ap`/`ap2` labels that the
previous app-continuation slice had not copied. The app counts and stack push
counts did not change; the win comes from deleting generic work on a value
whose shape is already known after `GOAP`/`GOAP2`. This is worth keeping even
though GC pause moved slightly against it.

#### Dedicated App Allocation Path

A fresh 10M profile after direct app-result descent showed the hot allocation
shape clearly: `profile_app_allocations=11,215,904`, while non-app node
allocation was tiny by comparison (`Int=3,032`, `Prim=5,031`). Top app sites
were normal combinator rewrite payloads (`B.yz`, `C.xz`, `S'`, `C'`, `S`,
`C'B`, `P`, `O`, and IO bind/return), so scalar result allocation was not the
lever. This slice keeps the generic `push_node` path for cold node kinds but
routes `app()`/`app_with_site()` through a dedicated `push_app_node(fun, arg)`
helper, while preserving app allocation/site counters.

Benchmarks, 32M allocation trigger:

| run | direct app-result descent | dedicated app allocation | result |
|---|---:|---:|---|
| 1M slice | 48.512 ms / 20.64M steps/s | 47.430 ms / 21.11M steps/s | win |
| 100M slice | 3.605s / 27.96M steps/s | 3.305s / 30.50M steps/s | win |
| 100M GC pause | 313.933 ms | 273.656 ms | win |
| full self-host | 131.6s / 27.80M steps/s | 121.0s / 30.25M steps/s | new best |
| full repeat | n/a | 120.9s / 30.27M steps/s | repeat best at the time |
| current default rebuild full | n/a | 120.708s / 30.31M steps/s | fresh best after correctness/profile refresh |
| full GC pause | 30.207s | 28.981s / 29.028s repeat | win |
| full output | byte-identical | byte-identical | `cmp_exit=0` |

Full gate command:

```text
timeout 900s env MHS_GC_NODE_INTERVAL=33554432 target/release/mhs-rust-bench \
  --input /tmp/mhs-selfhost.comb --mode main --warmup-iters 0 --iters 1 -- \
  ./bin/mhs -i -imhs -isrc -ilib MicroHs.Main -o/tmp/mhs-selfhost-rust-appallocfast-full.comb
```

Full gate outputs:

| measure | first full | repeat full | current full |
|---|---:|---:|---:|
| parse/reduce/render | 120,970.491 ms | 120,881.824 ms | 120,708.340 ms |
| WHNF steps | 3,659,075,035 | 3,659,075,168 | 3,659,074,940 |
| WHNF steps/s | 30,247,666 | 30,269,854 | 30,313,356 |
| step-limited iters | 0 | 0 | 0 |
| serialize sink | 661,902 | 661,902 | 661,902 |
| GC collections | 125 | 125 | 125 |
| GC total pause | 28,981.220 ms | 29,027.546 ms | 28,548.026 ms |
| GC high-water nodes | 39,958,361 | 39,958,375 | 39,958,351 |

Output identity:

| artifact | sha256 |
|---|---|
| `/tmp/mhs-selfhost.comb` | `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a` |
| `/tmp/mhs-selfhost-rust-appallocfast-full.comb` | `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a` |
| `/tmp/mhs-selfhost-rust-appallocfast-full-repeat.comb` | `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a` |
| `/tmp/mhs-selfhost-rust-current-full.comb` | `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a` |

Checks:

| check | result |
|---|---|
| `cargo fmt --manifest-path rust/microhs-runtime/Cargo.toml` | passed |
| `cargo check --manifest-path rust/microhs-runtime/Cargo.toml` | passed |
| `cargo test --manifest-path rust/microhs-runtime/Cargo.toml` | passed; 34 tests |
| `cargo build --release --manifest-path rust/microhs-runtime/Cargo.toml --bin mhs-rust-bench` | passed |
| `cargo check --manifest-path rust/microhs-runtime/Cargo.toml --features gc-phase-profile` | passed |
| `cargo check --manifest-path rust/microhs-runtime/Cargo.toml --features eval-phase-profile,gc-phase-profile` | passed |
| debug/release `cargo build --target wasm32-unknown-unknown --manifest-path rust/microhs-runtime/Cargo.toml` | passed |
| Node `host.mjs` wasm smoke | rendered `42` from `target/wasm32-unknown-unknown/release/microhs_runtime.wasm` |
| `git diff --check` | passed |

Reading: the dedicated app-allocation slice was the first accepted
post-direct-app change that moved the full gate by about ten seconds. The win
matches the profile: almost every hot
allocation is an app, so deleting generic node-allocation dispatch from apps is
a broad eval.c-shaped change rather than another narrow scalar tweak. The
post-profile-overhead baseline later crossed the 120s line at 118.951s; the
next target remains app/update/allocation mechanics and widening the margin under 2x C,
with care because the earlier free-list pop and direct-inner-`GOAP2` probes won
bounded slices but lost the full gate.

Post-acceptance current profile, 10M self-host slice. Refreshed 2026-07-04
after the EVALLESSONS correctness-batch verification and the rejected
parallel-cell-shell revert:

| measure | value |
|---|---:|
| non-profile parse/reduce/render, phase-feature binary | 447.387 ms |
| profile total | 2,805.703 ms |
| stack eval step time | 2,458.582 ms |
| GC check / resolve / ready-frame / descent | 39.524 / 43.790 / 36.505 / 44.827 ms |
| stack eval step calls | 1,756,737 |
| stack app updates | 8,292,659 |
| app allocations | 11,215,900 |
| stack descent pushes / arg reads | 30,244,480 / 30,119,438 |
| strict snapshots / remaining-app scans / fallback steps | 0 / 0 / 90 |

Top phase-profile head times remain the normal app-building combinators:

| head | time |
|---|---:|
| `S'` | 161.036 ms |
| `B` | 146.996 ms |
| `C` | 128.617 ms |
| `C'` | 102.301 ms |
| `S` | 86.161 ms |
| `C'B` | 72.822 ms |
| `P` | 40.692 ms |
| `==` | 31.902 ms |
| `A.read` | 25.980 ms |

Sub-timed stack profile, 10M self-host slice. This uses finer timers inside the
hot stack macros, so the profiled binary is slower than the phase-only profile;
read these as relative buckets, not default-path wall time.

| measure | value |
|---|---:|
| non-profile parse/reduce/render, sub-timed phase binary | 462.758 ms |
| profile total | 4,262.752 ms |
| stack eval step time | 3,906.503 ms |
| GC check / resolve / ready-frame / descent / WHNF finish | 39.419 / 43.520 / 49.744 / 45.520 / 5.010 ms |
| arg reads / app allocation / app update / rewrite | 183.317 / 332.689 / 151.127 / 31.708 ms |
| force-frame push / inner app-result descent | 7.509 / 247.209 ms |
| stack eval step calls | 1,756,737 |
| stack app updates / stack rewrites | 8,292,659 / 1,711,077 |
| app allocations | 11,215,890 |
| stack descent pushes / arg reads | 30,244,480 / 30,119,438 |
| strict snapshots / remaining-app scans / fallback steps | 0 / 0 / 90 |

Top sub-timed head buckets:

| head | time |
|---|---:|
| `B` | 368.183 ms |
| `S'` | 308.868 ms |
| `C` | 302.269 ms |
| `C'` | 212.670 ms |
| `S` | 176.152 ms |
| `C'B` | 154.497 ms |
| `P` | 94.159 ms |
| `K` | 45.850 ms |
| `==` | 45.719 ms |
| `O` | 38.589 ms |

Latest same-session controls after the post-app-allocation probe batch:

| run | parse/reduce/render | WHNF steps/s | GC pause | reading |
|---|---:|---:|---:|---|
| accepted shape, 1M control | 47.923 ms | 20.90M | 0 ms | current release rebuild before marker-stack probe |
| accepted shape, 100M control | 3.738s / 3.542s repeat | 26.97M / 28.46M | 273.6 / 276.6 ms | noisy but establishes same-session band |
| eval-phase-profile 10M slice | 545.924 ms non-profile, 3,199.594 ms profile | 18.37M non-profile | 0 ms | `stack_eval_step=2,815.844ms`, app allocations `11,215,900`, stack app updates `8,292,659`, descent/arg reads `30,244,480/30,119,438` |
| post-revert accepted shape, 1M sanity | 45.335 ms | 22.09M | 0 ms | accepted shape restored after rejected marker-stack probe |
| post-revert accepted shape, 100M sanity | 3.440s | 29.31M | 273.7 ms | faster than the marker-stack repeat |
| post-revert accepted shape after cell-shell probe, 1M/100M | 49.865 ms / 3.732s | 20.08M / 27.01M | 0 / 287.3 ms | accepted shape restored after rejected parallel-cell shell |
| current default rebuild after correctness refresh, 1M/100M | 47.388 ms / 3.519s | 21.13M / 28.65M | 0 / 281.7 ms | default release rebuilt after the phase-profile run; still accepted band |
| default rebuild after sub-timer patch, 1M/100M | 45.727 ms / 3.484s | 21.90M / 28.94M | 0 / 273.8 ms | default binary compiled without `eval-phase-profile`; still accepted band while full gate runs |
| default full after sub-timer patch | 122.341s | 29.91M | 28.891s | byte-identical, `cmp_exit=0`; no new best versus then-best 120.708s |
| `target-cpu=native` control, 1M/100M | 47.427 ms / 3.504s | 21.11M / 28.77M | 0 / 310.2 ms | rejected as a build-flag lever; worse than same-session default band |
| F6 serializer labels default rebuild, 1M/100M | 47.229 ms / 3.508s | 21.20M / 28.73M | 0 / 291.5 ms | serializer-only correctness change; bounded evaluator path remains accepted band while full gate runs |
| F6 serializer labels full gate | 124.030s | 29.50M | 29.381s | byte-identical, `cmp_exit=0`; no new best versus then-best 120.708s |
| F10 iterative serializer, deep smoke + 1M/100M + full | deep 12k app chain 1.082ms; 40.278 ms / 2.598s; full 92.575s | deep 0 steps; 24.86M / 38.80M; full 39.53M | 0 / 231.236 ms; full 26.559s | accepted correctness slice: `serialize_program` now uses an explicit postorder work stack instead of recursive `serialize_comb_into`, removing the old 10k depth cap; full gate byte-identical but not a new performance best |
| F17 cold `BytesView`, 1M/100M + full | 33.043 ms avg over 3 / 2.571s; full 93.574s, noisy repeat 100.859s | 30.31M / 39.21M; full 39.10M, repeat 36.28M | 0 / 239.022 ms; full 26.857s, repeat 28.375s | accepted parity slice, not a self-host win: `bssubstr`/`tailUTF8` create cold immutable view nodes over existing byte nodes, writes materialize a copy, outputs byte-match; bounded self-host stays in band but full does not beat F10/raw-tag gates |
| F14 ForeignPtr finalizers, 1M/100M + full | 29.672 ms avg over 3 / 2.582s; full 90.237s | 33.75M / 39.04M; full 40.55M | 0 / 226.466 ms; full 26.169s | accepted correctness slice and new best: `ForeignPtr` nodes now share side-table finalizer records, `fp+` aliases the same record, `fpfin` records `free`/`closeb`/null descriptors, and GC sweep runs dead records inline; byte-identical |
| S1 unified stack `ret`/`top`, 10M/100M + full | 293.691 ms / 2.552s; full 89.141s | 34.15M / 39.50M; full 41.05M | 0 / 235.017 ms; full 26.352s | accepted eval.c-shaped stack-loop slice and new best: ready strict-frame return handling now lives inside `stack_eval_step`, hot strict force markers and K/A-family `GOIND` rewrites continue in the same stack loop, the normal stack path no longer emits `StackStep::Force`, and output is byte-identical |
| S2 parse-label non-root GC, 10M/100M + full | 334.999 ms / 2.587s; full 88.623s | 29.94M / 38.97M; full 41.29M | 0 / 221.196 ms; full 25.826s | accepted eval.c root-model slice and new best: parser labels are no longer permanent GC roots, the label map retains only still-live targets after mark, the full gate is byte-identical, and GC pause drops about 0.53s versus ret/top |
| S1 allocator free-head invariant, 100M repeats + full | 2.450s / 2.463s; full 86.619s | 41.14M / 40.93M; full 42.24M | 212.439 / 213.868 ms; full 25.625s | accepted eval.c-shaped allocator slice and new best: reused app allocation now treats `free_nodes > 0` as proof that `free_head` is present, removing the remaining release `Option` branch from `pop_free_node`; output is byte-identical |
| S1 identity-alias `GOIND` continuation, profile/100M/full | 2.481s / 2.403s; full 86.500s | 40.63M / 41.95M; full 42.30M | 217.576 / 212.838 ms; full 25.920s | accepted eval.c-shaped continuation slice and new best: `I`/`Ord`/`Chr` rewrites now continue inside `stack_eval_step`; 10M phase profile stack loop iterations dropped 613k -> 281k, step calls 530k -> 257k, reduced exits 484k -> 211k; output is byte-identical |
| trusted hot cell accessors, 100M repeats + full | 2.375s / 2.462s / 2.485s; full 86.518s | 42.45M / 40.95M / 40.57M; full 42.29M | 212.545 / 216.341 / 218.713 ms; full 26.072s | rejected and reverted: private hot `cell`/`cell_at` and app/free setters used debug-checked unchecked release indexing; output was byte-identical, but full self-host regressed versus the 86.500s identity-`GOIND` best |
| generic `app()` attribution, phase profile + 100M | feature normal 307.169ms / profiled 9248.425ms; 100M 2.404s | feature normal 32.65M/s; 100M 41.94M | 0; 100M 217.773ms | profiling-only: generic helper app allocations are now visible as `<generic app()>`; they are only 99,874/11,215,916 app allocations and 3.285ms allocation timing in 10M, so generic `app()` is not the next lever; default 100M stayed in the accepted band |
| S1 one-word app-fun spine descent, profile/100M/full | default-profile normal 334.812ms; phase feature normal 324.987ms / profiled 9256.176ms; 100M 2.424s; full 85.189s | default normal 29.95M; phase normal 30.86M; 100M 41.58M; full 42.95M | 0; 100M 210.284ms; full 25.508s | accepted eval.c-shaped hot descent slice and new best: stack descent reads only `Cell.word0` to test/extract app fun before `CHKARG` arg loads; full output is byte-identical |
| redundant high-water field removal, 100M repeats | 2.739s / 2.481s; restore 2.460s | 36.81M / 40.63M; restore 40.97M | 231.048 / 219.347ms; restore 212.381ms | rejected and reverted before full: arena length is monotonic, but removing the separate `gc_high_water_nodes` field/update worsened 100M, likely from codegen/layout effects; restored app-fun baseline |
| raw free-head sentinel, 100M repeats | 2.484s / 2.458s; restore 2.473s | 40.59M / 41.01M; restore 40.76M | 210.365 / 206.813ms; restore 215.567ms | rejected and reverted before full: replacing `free_head: Option<NodeId>` with a raw `u32::MAX` sentinel did not produce a clean bounded win over the accepted free-head invariant shape, so this stays a local representation/codegen probe rather than a structural allocator change |
| current accepted profile refresh + keep-redex stack probe | profile normal 310.853ms / profiled 2061.766ms; keep-redex 10M 294.551ms, 100M 2.451s, full 86.813s; restore 100M 2.427s | profile normal 32.26M; keep-redex 34.05M / 41.13M / full 42.15M; restore 41.53M | keep-redex 100M 217.672ms; full 26.285s; restore 213.226ms | rejected and reverted: keeping the rewritten redex app slot on the stack avoids a physical pop/push for `GOAP`-style app rewrites and wins the no-GC 10M slice, but the byte-identical full gate regressed versus 85.189s and raised GC pause |
| WHNF-frame return-in-stack-loop probe | control 100M 2.413s; probe 10M 330.105ms, 100M 2.375s, full 85.593s; post-revert 100M 2.432s | control 41.77M; probe 30.38M / 42.45M / full 42.75M; post-revert 41.44M | control 100M 210.644ms; probe 100M 216.210ms; full 25.806s; post-revert 217.562ms | rejected and reverted: moving the remaining `StackStep::Whnf` strict-frame finish/rethread path into `stack_eval_step` matched eval.c's `RET` shape and won the noisy 100M slice, but the byte-identical full gate regressed versus the 85.189s app-fun best and raised GC pause |
| S2 mark-time app-edge canonicalization, 10M/100M/full | 323.168ms / 2.371s; full 83.752s | 31.03M / 42.52M; full 43.69M | 0 / 224.430ms; full 24.312s | accepted and new best: GC mark now rewrites app fun/arg parent slots to resolved indirection targets and cached small-int nodes where available, mirroring eval.c's pointer-to-slot mark shape; full high-water drops to 38,820,708 cells, last live to 1,289,534, and output is byte-identical |
| S1 trusted stack redex/update access, profile + 10M/100M/full | profile control 10M 311.419ms / profiled 2007.345ms; slice 10M 295.019ms, 100M 2.371s, full 82.154s | control 32.20M; slice 33.99M / 42.52M / full 44.54M | 100M 222.867ms; full 23.913s | accepted and new best: hot stack rewrite/app-update helpers now trust the checked app segment when loading the redex app and return plain `NodeId` instead of `Result`; output is byte-identical and full high-water stays in the app-edge-canonicalized 38.8M-cell shape |
| eval.c permanent compound caches, 10M/100M/full | 10M 320.240ms, 100M 2.390s, full 80.927s | 31.32M / 42.17M / full 45.21M | 100M 220.223ms; full 23.793s | accepted and new best: caches C permanent compounds `fst`, `snd`, `Just`, and `Pair Unit` as GC roots; bounded node-count signal is tiny but full self-host wins byte-identically, suggesting reduced cold helper construction/GC shape still matters |
| current-tree refresh after `68bf5347`, 10M/100M/full | 10M 363.915ms; 100M 2.709s then 2.523s repeat; full 84.503s | 27.56M; 37.22M then 39.96M; full 43.30M | 100M 243.985ms then 227.782ms; full 24.426s | no code change and not a new baseline; confirms byte-identical self-host after the catch-up commit, but the run was noisier/slower than the 80.927s best |
| current default/gc-profile refresh after rejected probes | default 10M 358.767ms; default 100M 2.354s; default full 83.062s; `gc-phase-profile` 100M 2.379s | 27.95M; 42.83M; full 44.05M; gc-profile 42.38M | default 100M 225.909ms; full 24.354s; gc-profile 222.787ms (`75.562ms` mark / `118.929ms` sweep) | normal release was rebuilt after profiling-feature runs; current bounded shape is back in the accepted 42M/s band and the full gate is byte-identical, about 1.74x the 47.85s C oracle, but not a new best versus 80.927s |
| current `eval-phase-profile` refresh, 10M | feature normal 388.542ms / profiled 10,153.891ms | normal 25.81M | 0 | profile buckets: app alloc 374.275ms, inner descent 334.197ms, arg reads 200.161ms, app updates 162.946ms, rewrites 35.837ms, force frames 7.708ms; top head times still `B`, `C`, `S'`, `C'`, `S`, `C'B`; fallback remains 90 steps |
| F10 BFILE `IO.print`/`IO.serialize` parity | smoke `IO.print IO.stdout #5` emits `#5`; `IO.serialize IO.stdout #5` emits `v8.4\n0\n#5 }`; default 10M 319.983ms; default 100M 2.352s; full 82.518s | 31.34M; 42.86M; full 44.34M | default 100M 222.716ms; full 24.157s | accepted correctness slice, not a new performance best: `IO.print`/`IO.serialize` now evaluate the first argument as a BFILE pointer and write through `write_bfile_bytes`; `IO.print` uses an iterative prefix/parenthesized graph printer instead of depth-capped debug render. The first un-cold build regressed bounded 10M/100M to 486.032ms/2.792s, so the printer/serializer entrypoints are cold/noinline. Full output is byte-identical with SHA `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`, `cmp_exit=0` |
| F6/F10 `IO.deserialize` parity | 10M 310.521ms; 100M 2.757s then 2.381s repeat; full 82.870s | 10M 32.30M; 100M 36.57M then 42.33M; full 44.15M | 100M 241.264ms then 221.811ms; full 24.037s | accepted correctness slice, not a new performance best: `IO.deserialize` is now a supported known primitive, reads one comb from the BFILE without consuming trailing bytes, parses with the existing comb parser, appends/remaps parsed nodes into the live arena with placeholders for cycles, and tests memory-BFILE stream preservation plus shared cycles. Full output is byte-identical with SHA `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`, `cmp_exit=0` |
| current `fe36f0e6` HEAD benchmark refresh | 10M 358.467ms; 100M 2.513s; full 82.662s | 10M 27.98M; 100M 40.12M; full 44.27M | 100M 219.225ms; full 23.988s | no code change: release bench was already up to date, full output is byte-identical with SHA `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`, `cmp_exit=0`; current HEAD is about 1.73x the 47.85s C oracle and still not a new best versus the 80.927s permanent-compound run |
| self-host-trained PGO build | current `15f76b3a` training full gate 116.525s; PGO 10M 278.053ms; PGO 100M 2.027s; PGO full 72.387s then 72.160s repeat | training 31.40M; PGO 10M 36.07M; PGO 100M 49.74M; PGO full 50.55M then 50.71M | training full 29.187s; PGO 100M 264.968ms; PGO full 25.933s then 25.906s | benchmark-only codegen-control win and current best measured execution mode: built with `-Cprofile-generate`, trained on the full self-host, merged with toolchain `llvm-profdata`, then built with `-Cprofile-use`; both current PGO full outputs are byte-identical with SHA `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`, `cmp_exit=0`. `rust/microhs-runtime/tools/native/build-selfhost-pgo.sh` captures the build recipe and passes `bash -n`; PGO improves mutator throughput enough to beat default release despite higher GC pause |
| stack cold-head outline probe | 10M 330.518ms; 100M 2.440s; full 84.731s | 10M 30.34M; 100M 41.32M; full 43.18M | 100M 224.774ms; full 24.337s | rejected and reverted: moving the `stack_eval_step` FFI/JS cold-head detection and materialize/call logic into `#[cold] #[inline(never)]` helpers kept output byte-identical and bounded rows plausible, but the full gate regressed versus both the 80.927s default best and the 82.662s current refresh. This confirms the hot-path layout is fragile enough that manual source outlining needs a full-gate proof |
| weak pointer GC semantics | initial arena-scan full 87.293s; side-list 10M 285.847ms; side-list 100M 2.398s; side-list full 82.185s | initial full 41.92M; side-list 35.08M / 42.03M / full 44.52M | initial full 29.521s; side-list 100M 228.175ms; side-list full 24.129s | accepted correctness slice, not a new performance best: weak nodes now carry keys, `Wknew`/`Wknewfin` pass key/value in eval.c order, weak values/finalizers are not marked strongly, the weak table marks value/finalizer only when the key is live, and dead-key finalizer actions stay rooted while evaluated synchronously at the GC safe point. The side list avoids the rejected whole-arena weak scan; scheduler-backed finalizer thread spawning remains pending. Full output SHA `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`, `cmp_exit=0` |
| `f870b8ff` 100M eval-phase profile refresh | feature-build normal 100M 2.763s; profiled 100M 100.810s; post-default-rebuild 10M sanity 288.092ms | feature normal 36.48M; profiled 1.00M; default sanity 34.81M | feature normal 230.101ms; profiled 231.197ms | profiling-only: current hot ranking remains app allocation first, then inner descent and arg reads. Profile counters: 117,411,800 app allocations, 306,704,110 stack arg reads, 308,147,022 stack descent pushes, 85,374,342 app updates, 15,091,021 stack rewrites. Timers: app allocation 11.555s (`free_pop` 2.072s, reused write 1.453s, fresh push 1.079s), inner descent 2.897s, arg reads 1.829s, apply-app 1.493s. Top app allocation sites by count/time are B.yz 24.76M/2.427s, C.xz 13.54M/1.340s, S' left/yw/zw 9.83M each/~0.96s each, C' xyw/yw 7.70M each/~0.75s each, P.zx 6.13M/0.603s, and C'B xz/yw 5.01M each/~0.50s each. Profile bookkeeping itself is large, so this is an attribution refresh, not a candidate gate |
| stack rewrite argument/opportunity profiling | profiled 10M 11.353s; default rebuild 10M 279.898ms; default full 83.786s | profiled 0.88M; default 10M 35.83M; full 43.67M | profiled 0; full 24.462s | profiling-only: added top-N argument-shape counters for hot B/C/S-family stack rewrites and exact eval.c-style app-redex opportunity counters for stack app construction/update. On 10M, top allocation sites remain B.yz 1,943,041, C.xz 1,609,624, S' sites 810,754 each, C' sites 741,069 each, S sites 612,486 each, C'B sites 538,754 each, and P.zx 500,286. Top argument shapes are mostly `App(App)`/`Indir`; exact opportunities are `red_i=259,585`, `red_k=95,180`, `red_a=8,858`, `red_bi=33`, `red_bxi=1`. Reading: runtime creates many immediate I/K/A redexes that mark-time GCRED misses; broad B-family GCRED remains unsupported by the data. Full output SHA `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`, `cmp_exit=0`; not a new best |
| self-host neutrality harness | 10M same-source smoke base avg 296.196ms vs candidate avg 314.510ms; balanced 100M same-source repeat base avg 2.381s vs candidate avg 2.421s; current full refresh 82.740s | 10M avg 33.89M vs 32.15M; 100M avg 42.38M vs 41.74M; full 44.22M | 100M base/candidate 230.355ms/231.773ms; full 24.196s | tooling-only: `15f76b3a` adds `bench-selfhost-neutrality.sh`, a repeatable base/candidate build+run harness for codegen-neutrality gates. It passed `bash -n`, uses equal-length output labels, alternates order, reports averages, and treats step-limited missing output as expected. Same-source 100M control drift is `+1.71%` with identical 100,811,779 steps, 3 GCs, high-water 33,941,709, and sink 661902; single 10M/100M pairs are too noisy. Full gate output SHA `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`, `cmp_exit=0`; not a new best |
| wasm `#2147483648` int-width smoke | release wasm build passed; Node `host.mjs` render smoke printed `2147483648` | n/a | n/a | audit from `WRAPBUG.md`: C/eval.c uses `value_t = intptr_t`, so C wasm32/ILP32 wraps `#2147483648` to `#-2147483648`; Rust parses `#` literals as `i64`, stores `Node::Int(i64)` in the two-word `Cell`, and the wasm export renders through the same `Program::render` path. Current smoke command instantiated `target/wasm32-unknown-unknown/release/microhs_runtime.wasm` and rendered `v8.4\n0\n#2147483648 }\n`, confirming this specific wrap is not inherited by Rust wasm |
| WASI std env/path/BFILE cfg probe | Node WASI `arith-chain:1000` ran; self-host 100k and 200k step-limited runs passed after enabling std env/path/BFILE on `target_os = "wasi"`; 300k+ release runs crashed; native full gates regressed to 89.420s and 88.934s | 100k WASI 5.44M; 200k 8.98M; 300k O2 8.48M; native full 40.92M / 41.14M | native full 26.281s / 26.259s | rejected and reverted: widening the existing `not(target_arch = "wasm32")` NativeFile/std-handle cfgs plus env/path helpers let the WASI bench build and run farther than the old ENOSYS blocker, but full WASI still segfaulted before 1M reductions. 16MiB/32MiB wasm stack did not move the 300k release crash; debug and release+debug-assertions reached 300k but also crashed at 1M; `-Copt-level=2` reached 300k but not 1M. The native 100M sanity stayed normal at 2.429s, but two byte-identical native full gates regressed badly versus the current 82.740s refresh, so the source probe was reverted. After rebuilding the reverted source, 100M sanity was 2.606s / 38.68M with the same 100,811,779 steps and sink 661902. Next WASI work should add target-specific code without touching the native hot cfg surface, or first find the wasm crash with a smaller reproducer |
| modest `EvalStack` pre-size probe | baseline 10M 343.834ms, candidate 10M 315.318ms; baseline 100M 2.812s / 2.784s, candidate 100M 2.467s / 2.621s; candidate full 82.779s | candidate 10M 31.81M; candidate 100M 40.87M / 38.47M; full 44.20M | candidate 100M 230.520ms / 238.592ms; full 24.399s | rejected and reverted: pre-sizing the active `EvalStack` with 1024 app slots and 16 frame slots was the closest Rust analogue to C's preallocated managed stack without a huge arena reserve. Bounded runs were noisy and order-sensitive; the byte-identical full gate missed both the 80.927s default best and the 82.740s current refresh, so Vec growth is not the current self-host bottleneck. Source was reverted |
| raw trivial app compression probe | broad first attempt failed `reduces_partial_arity_specializations` by changing exact graph shape (`((B (I I)) 9)` became `((B I) 9)`); narrowed non-`_under` stack-site attempt passed unit tests but 10M regressed to 308.159ms; post-revert 10M sanity 285.163ms | probe 32.54M; post-revert 35.17M | 0 | rejected and reverted before 100M/full: constructor-time `I`/`K`/`A` compression is semantically tempting because the profile sees transient redexes, but even the raw/non-resolving narrowed form adds enough branch/codegen cost to lose the no-GC gate and can disturb exact graph-shape parity at partial-arity sites. Default release bench was rebuilt after revert |
| targeted `S I` shortcut probe | 10M 285.920ms; 100M 2.437s; post-revert 10M 286.580ms | 10M 35.08M; 100M 41.36M; post-revert 34.99M | 100M 230.668ms | rejected and reverted before full: replacing only `S I y z`'s `I z` inner app with `z` avoids the broad compression branch and preserves partial-arity graph tests, but bounded gates were neutral-to-slower and high-water did not improve. The runtime cost/codegen perturbation still beats the saved transient redexes. Default release bench was rebuilt after revert |
| batched free-list cache probe | probe 10M 325.639ms; probe 100M 3.181s; post-revert 10M 300.611ms | probe 10M 30.80M; probe 100M 31.69M; post-revert 33.36M | probe 100M 272.206ms | rejected and reverted before full: cached up to 64 free indices from the intrusive free list to avoid reading the next free cell on every reused allocation, preserving order within each batch. The no-GC 10M slice regressed immediately, so the extra field/refill branch/codegen shape is not neutral even before free-cell reuse matters. Default release bench was rebuilt after revert |
| S4 young-region viability profiling | default 100M 2.436s; default full 82.817s; `gc-phase-profile` 32M 100M 2.961s; `gc-phase-profile` 8M 100M 2.787s | default 100M 41.38M; full 44.18M; profile 32M 34.04M; profile 8M 36.17M | default 100M 252.515ms; full 24.153s; profile 32M 444.085ms (`83.662ms` mark / `119.606ms` sweep); profile 8M 831.859ms (`345.701ms` mark / `161.857ms` sweep) | profiling-only: allocation-set counters show 32M cadence allocates 100.66M slots across 3 GCs with only 401k live and 2,052 old-to-young sources; 8M cadence allocates 117.44M slots across 14 GCs with 1.04M live and 5,088 old-to-young sources. Remembered-set pressure is tiny, but at 32M the allocation set is nearly the whole arena, so S4 needs a real allocation-set/remembered-set minor collector at a smaller cadence, not just a contiguous young range or another full-GC interval tweak |
| S4 allocation-set minor collector prototype | feature-off 100M 2.893s; eager 8M/128M 100M 3.281s; lazy-free 8M/128M 100M 3.189s; lazy-free 32M/128M 100M 3.951s; lazy-free 8M/128M full 113.490s; post-revert default 100M 2.492s | feature-off 34.84M; eager 8M 30.73M; lazy 8M 31.61M; lazy 32M 25.51M; full 32.24M; post-revert 40.45M | feature-off 422.761ms; eager 8M 698.827ms; lazy 8M 510.464ms; lazy 32M 466.006ms; full 22.471s; post-revert 223.100ms | rejected and reverted: the minor collector lowered full high-water to 17.52M cells and GC pause below the 80.927s best, but the mutator/codegen cost of allocation tracking, write barriers, young marking, and minor free-slot reuse crushed throughput; future S4 work needs a different design, not this allocation-set collector |
| dead persistent-dispatcher deletion cleanup | probe 10M 328.314ms; probe 100M 2.415s; probe full 88.171s; post-revert 10M 311.608ms; post-revert 100M 2.398s | probe 10M 30.55M; probe 100M 41.74M; probe full 41.50M; post-revert 32.18M / 42.04M | probe 100M 225.750ms; probe full 25.161s; post-revert 100M 225.034ms | rejected and reverted: `PersistentSpine`, `persistent_eval_step`, old `EvalFrameStack` strict markers, and the `whnf_frames=true` branch are unreachable from current callers, but deleting ~1.7k stale lines changed release codegen/layout enough to miss the full gate badly; output SHA stayed `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`, `cmp_exit=0` |
| app-allocation split profiling | no-GC profile 10M normal 341.152ms / profiled 10,013.545ms; forced-GC profile 10M normal 365.741ms / profiled 10,079.068ms; default 100M sanity 2.407s; full 82.949s | no-GC normal 29.40M; forced-GC normal 27.42M; default 100M 41.89M; full 44.11M | forced-GC 10M 158.373ms profile GC; default 100M 227.031ms; full 24.396s | profiling-only: `push_app_node` now splits fresh vs reused app cells under `eval-phase-profile`; forced-GC 10M shows 9.79M reused vs 1.43M fresh allocations; default full gate is byte-identical but not a new best versus 80.927s |
| GCRED opportunity profiling | `gc-phase-profile` 100M 2.450s; default 100M 2.448s; default full 82.693s | gc-profile 41.15M; default 41.17M; full 44.25M | gc-profile 100M 230.533ms (`85.521ms` mark / `117.058ms` sweep); default 100M 229.873ms; full 24.388s | profiling-only: counts eval.c mark-time reductions (`I`, `K`, `A`, `B I`, `B x I`, `C'B x I`, `C (C x)`, `C' I`, `C'B ((B C) P)`, and `C op`) under `gc-phase-profile`; all counters were zero on the 100M slice, and full output is byte-identical, so broad GCRED is not worth implementing blindly for self-host |
| parsed primitive-cache seed probe | 10M 294.711ms, 100M 2.406s, full 82.228s; post-revert 100M 2.413s | 34.03M / 41.90M / full 44.50M; post-revert 41.78M | 100M 226.987ms; full 24.050s; post-revert 226.325ms | rejected and reverted: seeding the existing fixed `PrimCache` from parsed primitive singletons matched eval.c's permanent primitive-node discipline and lowered high-water/live by three cells, but the byte-identical full gate still missed the 80.927s compound-cache best |
| top-20 phase profile after const retest | feature normal 325.266ms / profiled 9,941.427ms | normal 30.83M | 0 | profile shape unchanged: app allocations 11,215,886, B/C/S-family heads dominate; top app-site timers `B.yz` 204.440ms, `C.xz` 169.467ms, `S'` sites ~85ms each; top inner-descent head timers `C` 49.631ms, `B` 46.765ms, `S'` 23.616ms; app-allocation total is inflated to 1,181.770ms by nested split timers |
| const-specialized unprofiled stack-step retest | 10M 283.594ms, 100M 2.391s, full 84.112s; post-revert 100M 2.456s | 35.36M / 42.16M / full 43.50M; post-revert 41.04M | 100M 225.126ms; full 24.038s; post-revert 223.461ms | rejected and reverted: splitting `stack_eval_step`/descent into profiled and unprofiled monomorphs won the no-GC 10M slice, but the 100M slice was not a clean win and the byte-identical full gate missed both 80.927s best and latest 82.949s instrumentation-only refresh |
| GC-check fast predicate probe | 10M 278.245ms, 100M 2.345s / 2.358s, full 81.769s / 82.593s; post-revert 100M 2.443s | 36.04M, 42.99M / 42.76M, full 44.75M / 44.30M; post-revert 41.27M | 100M 223.655ms / 225.921ms; full 23.866s / 23.963s; post-revert 226.790ms | rejected and reverted: gating the root-set-heavy GC call behind an inline threshold predicate gave strong bounded wins, but two byte-identical full gates still missed the 80.927s best, so the full arbiter keeps the existing helper-call shape |
| reserve-app allocation accounting probe | 10M 325.618ms, 100M 2.522s, full 89.330s; post-revert 100M 2.414s | 30.80M / 39.98M / full 40.96M; post-revert 41.77M | 100M 226.151ms; full 25.613s; post-revert 227.933ms | rejected and reverted: batching `gc_allocations_since_collect` for multi-app rewrite arms matched eval.c's `GCCHECK(k)` accounting shape superficially, but it slowed the byte-identical full gate badly; the remaining gap is not per-app counter increment overhead |
| bitmap free-map allocator probe | gc-phase control 100M 2.342s; probe 10M 315.636ms, 100M 2.633s, word-cache 100M 2.597s, default 100M 2.610s; post-revert 100M 2.369s | control 43.05M; probe 31.77M / 38.28M / word-cache 38.83M / default 38.63M; post-revert 42.55M | control 223.614ms with 76.457ms mark and 118.550ms sweep; word-cache 217.195ms with 77.268ms mark and 111.804ms sweep; default probe 218.904ms; post-revert 221.099ms | rejected and reverted before full: replacing the intrusive free list with a C-style free bitmap cut sweep only slightly, but per-allocation bitmap scanning lost much more mutator time; C's bitmap allocator is not a drop-in win in this Rust cell layout |
| low-to-high free-list reuse order probe | 10M 292.599ms, 100M 2.429s / 2.415s, full 81.622s / 81.277s; post-revert 100M 2.415s | 34.28M / 41.50M / 41.74M / full 44.83M / 45.02M; post-revert 41.74M | 100M 213.628ms / 214.200ms; full 23.213s / 23.120s; post-revert 225.493ms | rejected and reverted: allocating from low addresses first like eval.c lowered GC pause and gave a strong 10M signal, but two byte-identical full gates still missed the 80.927s best |
| FFI/JS arity-prefix materialization retest | control 10M 283.818ms, 100M 2.451s; probe 10M 275.978ms, 100M 2.444s, full 83.510s; post-revert 100M 2.641s | control 35.34M / 41.13M; probe 36.34M / 41.25M / full 43.82M; post-revert 38.17M | control 100M 224.693ms; probe 100M 227.102ms, full 23.937s; post-revert 228.836ms | rejected and reverted: capped FFI/JS scratch-arg copying to the declared arity prefix and dropped 10M profiled materialized arg nodes from 10,522,021 to 150,988, but the byte-identical full gate still missed the 80.927s compound-cache best |
| eval.c `GOAP2` direct-push retest | 10M 319.832ms, 100M 2.369s, full 86.784s; post-revert 100M 2.357s | 31.36M / 42.55M / full 42.16M; post-revert 42.77M | 100M 225.702ms; full 25.596s; post-revert 229.471ms | rejected and reverted: pushing the reused outer redex plus freshly allocated inner app directly matched eval.c's `ap2` continuation and won the bounded 100M slice, but the byte-identical full gate regressed versus the 80.927s compound-cache best and raised full GC pause |
| eval.c `GOPAIR` runtime-return shape probe | unoutlined 10M 331.645ms, 100M 2.772s; cold/noinline 10M 330.161ms, 100M 2.367s, full 82.600s; post-revert 100M 2.462s | cold/noinline 30.38M / 42.60M / full 44.30M; post-revert 40.95M | cold/noinline 100M 227.171ms, full 23.502s; post-revert 223.856ms | rejected and reverted: returning pair/unit-pair shapes from runtime/FFI helpers let the stack path overwrite the consumed redex as the outer pair app, but the byte-identical full gate regressed versus the 80.927s compound-cache best and high-water rose to 39,491,643 cells |
| focused stack app-site profile split | probe 10M 340.971ms, 100M 2.422s, full 85.721s; post-revert 100M 2.420s | probe 29.41M / 41.62M / full 42.69M; post-revert 41.66M | probe 100M 226.159ms, full 25.503s; post-revert 220.124ms | rejected and reverted: using the stack-loop `profiling` flag to bypass `app_with_site` in non-profile app-site allocations was byte-identical but slowed the full self-host versus the 82.154s trusted-stack best; this confirms the remaining app-allocation cost is not just profile-site branch bookkeeping |
| current-tree F2/GC refresh + direct known-head dispatch probe | current 32M 100M control 2.461s; 64M 100M 2.978s; phase-profile 10M normal 377.119ms/profiled 9783.203ms; direct known-head dispatch 10M 325.601ms, 100M 2.396s, full 85.951s; post-revert 100M 2.609s noisy | direct probe 30.80M / 42.08M / full 42.57M; 64M 100M 33.85M | direct full 25.135s; 64M 100M 162.254ms | rejected and reverted: widening the GC window bloated the bounded arena and lost; replacing stack `EvalHead` dispatch with a direct `Prim::Known` fast match won the noisy 100M slice but lost the byte-identical full gate versus 80.927s, so the remaining F2 cost is not this enum shape |
| eval.c `GOAP2` allocation-order probe | 10M 385.102ms, 100M 2.428s, full 82.827s; post-revert 100M 2.454s | 26.04M / 41.51M / full 44.18M; post-revert 41.07M | 100M 226.885ms; full 24.186s; post-revert 236.630ms | rejected and reverted: ordering independent `S'`/`B'`/`C'B` inner app allocations closer to eval.c's `GOAP2` textual order stayed byte-identical but lost the full gate versus 80.927s, so remaining F2 work is not just local independent-app allocation order |
| eval.c low-bit app tag encoding probe | 10M 396.531ms, 100M 2.498s, full 92.637s; post-revert 100M 2.397s | 25.29M / 40.35M / full 39.50M; post-revert 42.06M | 100M 233.133ms; full 26.366s; post-revert 225.599ms | rejected and reverted: changing `Cell.word0` to use low bit 0 for app and low bit 1 for non-app tags matched eval.c's tag-word idea more closely, but in this Rust `NodeId` encoding it slowed both bounded and full gates despite byte-identical output |
| manual `Vec<u64>` GC mark bitmap probe | 100M 2.375s, full 84.578s; post-revert 100M 2.412s | 42.44M / full 43.26M; post-revert 41.80M | 100M 222.774ms; full 25.491s; post-revert 225.890ms | rejected and reverted: replacing `Vec<bool>` with a manual `Vec<u64>` mark bitmap slightly helped the bounded GC slice, but the byte-identical full gate lost versus 80.927s and raised full GC pause, so Rust's packed `Vec<bool>` proxy is not the standalone GC bottleneck |
| broader GC slot canonicalization probe | 10M 322.993ms, 100M 2.498s, full 88.717s; post-revert 100M 2.400s | 31.05M / 40.36M / full 41.24M; post-revert 42.01M | 100M 227.337ms; full 25.603s; post-revert 221.299ms | rejected and reverted: extending canonicalization to program root/stable slots and cold `Weak`/`MVar`/`BytesView`/`Array` children was byte-identical but added mark work without improving heap shape beyond the App-edge slice; full regressed badly versus 83.752s |
| compact profile-head frame, 1M/100M | 44.841 ms / 3.370s | 22.33M / 29.91M | 0 / 312.5 ms | stores profile heads in frames as `NodeId` instead of `Option<String>`; bounded win despite noisier GC |
| compact profile-head frame full gate | 120.167s | 30.45M | 30.588s | byte-identical, `cmp_exit=0`; then-best before the later 118.951s baseline. Repeat output file also byte-matches the oracle, but timing stdout was lost with the dead tool session |
| pre-cell accepted 10M profile | 444.086 ms normal / 2075.421 ms profiled | 22.58M normal | 0 | 10,028,861 steps; 11,215,862 app allocations; only 90 fallback eval-loop steps; top heads still `B`, `C`, `S'`, `C'`, `S`, `C'B`, `P` |
| pre-cell `eval-phase-profile` 10M | 405.877 ms normal / 4486.071 ms profiled | 24.71M feature-build normal | 0 | `stack_eval_step_ms=4133.693`; sub-buckets: app alloc 318.706ms, inner descent 252.713ms, arg reads 185.781ms, app update 149.973ms, rewrites 31.210ms, force frames 7.248ms; head-time leaders `B` 360.115ms, `S'` 303.501ms, `C` 298.851ms |
| independent app-pair batch probe, 1M/100M | 46.637 ms / 3.433s | 21.47M / 29.36M | 0 / 307.4 ms | rejected and reverted; batching independent inner app allocations in `S`, `S'`, `B'`, `C'B`, and `IO.>>` preserved allocation order but worsened both bounded gates |
| F21 `mpz_get_d` decimal rounding, 1M/100M | 46.725 ms / 3.360s | 21.43M / 30.00M | 0 / 293.1 ms | accepted cold correctness fix; `to_f64` now uses decimal parsing like C `strtod`, behind `#[cold] #[inline(never)]`; 37/37 tests pass |
| pre-cell profile-overhead split, 10M + default 1M/100M | 452.398 ms feature normal / 6152.657 ms profiled; default 46.080 ms / 3.395s | 22.17M feature normal; default 21.73M / 29.69M | 0 / 287.988 ms | accepted profiling-only instrumentation; `stack_eval_step=5788.409ms`, visible profile bookkeeping is 2491.876ms (`profile_step` 712.890, reductions 541.194, per-head timing 545.343, app-site bookkeeping 692.449), so the profile bucket is distorted while the default bounded gate stays accepted |
| profile-overhead default full gate | 118.951s | 30.76M | 29.398s | byte-identical, `cmp_exit=0`; first logged sub-120s run, likely a clean baseline/noise win rather than a hot-path change because the instrumentation is cfg-gated out of default builds |
| refreshed `gc-phase-profile` full gate | 118.651s | 30.84M | 28.948s | byte-identical, `cmp_exit=0`; feature build split total GC into 19.134s mark and 9.687s sweep, so GC is still material but not the whole remaining gap |
| stack scalar `SET*` redex overwrite, 1M/100M + full | 47.172 ms / 3.387s; full 118.739s | 21.23M / 29.77M; full 30.82M | 0 / 312.636 ms; full 29.958s | accepted as eval.c-aligned but noise-band: scalar strict results overwrite the consumed app cell instead of allocating `Int`/`Int64`/`Float32`/`Float64`/conversion result nodes plus an indirection; byte-identical, `cmp_exit=0`, but not a standalone route below 90s |
| authoritative cell arena, 1M/100M + full | 43.148 ms / 3.252s; full 105.859s | 23.21M / 31.00M; full 34.57M | 0 / 282.461 ms; full 28.928s | accepted structural S3/S1 win: `Program` now stores 16-byte `Cell`s as the authoritative arena, with App/Indir/Free/Prim/scalar tags in-cell and cold payloads in a side table; byte-identical, `cmp_exit=0`, 2.21x C |
| fixed-arity `CHKARG` pop/take + direct app/free writes, 1M/100M + full | 46.012 ms first 1M, 37.379 ms post-rebuild 1M / 2.811s; full 99.022s | 21.76M then 26.79M / 35.87M; full 36.95M | 0 / 268.580 ms; full 28.724s | accepted S1 cell-era rewrite: hot fixed-arity stack arms pop/take args before rewriting the app redex, direct known-app/free-cell writes bypass cold-payload checks, and output remains byte-identical; 100M/full are clear wins despite a noisy first 1M |
| strict `Int` redex-owning frames, 1M/100M + full | 40.177 ms first 1M, 37.466 ms post-rebuild 1M / 2.858s; full 95.345s | 24.92M then 26.73M / 35.27M; full 38.38M | 0 / 257.328 ms; full 27.975s | accepted eval.c `BININT`/`SETINT` shape: strict `Int` primitives now pop the consumed app segment before forcing, carry the redex as the frame root, and write the result directly into that cell; bounded 100M was slightly slower but the full gate won decisively and crossed 2x C |
| WHNF redex-owning frames, 1M/100M + full | 41.776 ms / 2.684s; full 93.202s | 23.97M / 37.56M; full 39.26M | 0 / 246.571 ms; full 27.492s | accepted eval.c-shaped strict WHNF frame slice: `seq`/`isint`/`IO.strict` now pop the consumed redex before forcing; `IO.strict` reuses the redex app instead of allocating `action value` plus an indirection; byte-identical |
| raw cell tag + trusted free-list accessors, 1M/100M + full | 38.141 ms / 2.551s; full 91.503s | 26.25M / 39.51M; full 39.99M | 0 / 232.620 ms; full 26.637s | accepted eval.c-shaped accessor slice: hot cell tests use raw tag bits for App/Prim/scalar/Indir paths, resolve uses raw tag bits, and free-list pop trusts the Free invariant in release; byte-identical |
| current parser-small-int `eval-phase-profile` refresh, 10M | 354.424 ms normal / 6529.172 ms profiled | 28.30M feature-build normal | 0 | profile shape unchanged: 10,028,861 steps; `stack_eval_step=6149.873ms`; app allocation 347.147ms, inner descent 242.082ms, arg reads 188.616ms, app updates 155.561ms, rewrites 34.340ms, force frames 8.388ms; top heads `B`, `S'`, `C`, `C'`, `S`, `C'B`, `P` |
| numeric known-head stack dispatch probe, 1M/100M | 39.156 ms / 2.667s | 25.57M / 37.80M | 0 / 239.951 ms | rejected and reverted; carrying raw `u16` known-prim codes through `stack_eval_step` avoided enum decode but worsened both bounded gates versus the fresh same-session 37.802 ms / 2.619s baseline; post-revert 1M sanity was 36.604ms |
| raw GC mark/sweep tag loop probe, `gc-phase-profile` 100M | 2.647s | 38.08M | 236.916 ms | rejected and reverted; raw tag tests in mark/indirection compression plus inline sweep free-list rebuild worsened the same-session GC-profile control, 230.779 ms pause with 73.508 ms mark / 128.855 ms sweep -> 236.916 ms pause with 73.721 ms mark / 135.083 ms sweep |
| parse-time small-int interning retest, 1M/100M + full | 37.045 ms / 2.614s, 2.608s repeat; full 92.508s, 91.598s repeat | 27.03M / 38.56M, 38.66M; full repeat 39.95M | 0 / 233.801 ms, 231.241 ms; full repeat 26.632s | accepted post-cell retest: parsed nodes 160,961 -> 153,185 and full high-water 40,014,305 -> 40,006,682; byte-identical; ties but does not beat the 91.503s best |
| eval-time first-indirection compression retest, 1M/100M + full | 38.671 ms / 2.556s, 2.618s repeat; full 98.036s | 25.90M / 39.44M, 38.51M; full 37.32M | 0 / 224.174 ms, 227.321 ms; full 27.551s | rejected and reverted; matching eval.c's `SETINDIR(on,n)` in `resolve_for_whnf` was byte-identical but repeated the old local/completion mismatch: mixed bounded results and full gate regressed badly versus 91.598s current repeat |
| stack `Bytes` redex-owning frame extension, 1M/100M + full | 39.538 ms / 2.580s; full 92.679s | 25.33M / 39.07M; full 39.48M | 0 / 234.680 ms; full 26.895s | rejected and reverted; extending the accepted strict-redex ownership pattern to the remaining profiled `Bytes` frame was byte-identical but did not beat the parser-small-int 91.598s repeat or 91.503s best |
| direct `Node::{Known,Runtime}` primitive-tag split, 1M/100M | 45.490 ms / 3.494s | 22.01M / 28.85M | 0 / 277.9 ms | rejected and reverted; moving `Prim::{Known,Runtime}` into top-level `Node` variants helped 1M noise but lost the 100M slice despite a shorter `-o` path/fewer nodes; post-revert exact-output-path counters match the pre-probe control |

Bounded self-host controls should keep the `-o/tmp/...` argument stable when
comparing allocation counters: the compiler sees that path as a guest argument,
so changing its length can move node counts by a few dozen before the step
limit.

S3 authoritative cell arena slice:

| item | result |
|---|---|
| change | `Program` stores `Vec<Cell>` plus `cold_nodes`; `Cell` owns App/Indir/Free/Prim/scalar tags and payload words, while boxed payloads (`Bytes`, `Array`, `Ffi`, `ForeignPtr`, `Weak`, `MVar`, etc.) live in a cold side table; `Node` remains the parse/debug/serialization interchange type |
| rationale | this is the `NOTES.md` S3 shape rather than the rejected mirrored side table: hot stack args/descent/update/GC/resolve read the single authoritative cell representation |
| verification | `cargo fmt --manifest-path rust/microhs-runtime/Cargo.toml`; `cargo check --manifest-path rust/microhs-runtime/Cargo.toml`; `cargo check --manifest-path rust/microhs-runtime/Cargo.toml --features eval-phase-profile`; `cargo check --manifest-path rust/microhs-runtime/Cargo.toml --features gc-phase-profile`; `cargo check --release --manifest-path rust/microhs-runtime/Cargo.toml --target wasm32-unknown-unknown`; `cargo test --manifest-path rust/microhs-runtime/Cargo.toml --lib` (37/37); `cargo build --release --manifest-path rust/microhs-runtime/Cargo.toml`; `git diff --check` |
| bounded gate | 1M `43.148ms`, 1,001,383 steps, 23.21M steps/s, no GC; 100M `3.252s`, 100,811,774 steps, 31.00M steps/s, 3 GCs, 282.461ms GC pause, sink `661902` |
| full gate | `timeout 900s env MHS_GC_NODE_INTERVAL=33554432 target/release/mhs-rust-bench --input /tmp/mhs-selfhost.comb --mode main --warmup-iters 0 --iters 1 -- ./bin/mhs -i -imhs -isrc -ilib MicroHs.Main -o/tmp/mhs-selfhost-rust-out-stack-cells.comb` completed in `105.859s`, 3,659,074,997 steps, 34.57M steps/s, 125 GCs, 28.928s GC pause, high-water 40,014,305 cells |
| output | `/tmp/mhs-selfhost.comb` and `/tmp/mhs-selfhost-rust-out-stack-cells.comb` both SHA256 `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`; `cmp -s` exit 0 |
| post-cell eval profile | 10M feature-build non-profile run `359.989ms`, 10,028,861 steps, 27.86M steps/s; profiled run `6595.516ms`; `stack_eval_step_ms=6216.518`, with sub-buckets app allocation `353.863ms`, inner descent `242.621ms`, arg reads `199.845ms`, app update `153.427ms`, ready frame `63.566ms`, descent `50.289ms`, resolve `45.144ms`; top timed heads `B` 472.976ms, `S'` 432.093ms, `C` 391.986ms, `C'` 292.017ms, `S` 246.657ms, `C'B` 211.883ms |
| post-cell GC profile | 100M `gc-phase-profile` run `3.225s`, 100,811,774 steps, 31.26M steps/s, 3 GCs, 268.773ms total GC pause split into 86.890ms mark and 153.969ms sweep |
| default rebuild sanity | after the profile builds, default release was rebuilt and reran 1M at `43.120ms`, 1,001,383 steps, 23.22M steps/s |
| reading | real structural win at the time: 118.739s -> 105.859s, later superseded by the `CHKARG` pop/take slice at 99.022s; the refreshed profile says to work on app allocation, inner descent, arg reads, and app updates in the cell-backed stack machine |

S1 fixed-arity `CHKARG` pop/take slice:

| item | result |
|---|---|
| change | fixed-arity hot stack combinator and IO graph-rewrite arms now take arguments and pop the consumed app segment in one eval.c `CHKARGn`-style operation, then write the app redex directly and continue through the known root app; stack/spine app rewrites and free-list app allocation use direct known-app/free-cell writes instead of the generic cold-payload-dropping setter |
| rationale | this is the cell-era version of Lennart's `CHKARG`/`GOAP` discipline: args are read through app cells, the consumed stack segment is popped before the graph rewrite, and app/free slots with proven tags do not pay generic node update dispatch |
| verification | `cargo fmt --manifest-path rust/microhs-runtime/Cargo.toml --check`; `cargo check --manifest-path rust/microhs-runtime/Cargo.toml`; `cargo check --manifest-path rust/microhs-runtime/Cargo.toml --features eval-phase-profile`; `cargo check --manifest-path rust/microhs-runtime/Cargo.toml --features gc-phase-profile`; `cargo test --manifest-path rust/microhs-runtime/Cargo.toml --lib` (37/37); `cargo build --release --manifest-path rust/microhs-runtime/Cargo.toml --bin mhs-rust-bench`; `cargo check --release --manifest-path rust/microhs-runtime/Cargo.toml --target wasm32-unknown-unknown` |
| bounded gate | first 1M `46.012ms`, then post-profile default rebuild sanity 1M `37.379ms`; 100M `2.811s`, 100,811,774 steps, 35.87M steps/s, 3 GCs, 268.580ms GC pause, sink `661902` |
| full gate | `timeout 900s env MHS_GC_NODE_INTERVAL=33554432 target/release/mhs-rust-bench --input /tmp/mhs-selfhost.comb --mode main --warmup-iters 0 --iters 1 -- ./bin/mhs -i -imhs -isrc -ilib MicroHs.Main -o/tmp/mhs-selfhost-rust-out-chkarg-pop.comb` completed in `99.022s`, 3,659,074,978 steps, 36.95M steps/s, 125 GCs, 28.724s GC pause, high-water 40,014,303 cells |
| output | `/tmp/mhs-selfhost.comb` and `/tmp/mhs-selfhost-rust-out-chkarg-pop.comb` both SHA256 `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`; `cmp -s` exit 0 |
| post-slice eval profile | 10M `eval-phase-profile` feature-build non-profile control `356.367ms`, 10,028,861 steps, 28.14M steps/s; profiled run `6627.678ms`; `stack_eval_step_ms=6244.885`, with sub-buckets app allocation `366.150ms`, inner descent `248.244ms`, arg reads `196.625ms`, app update `159.793ms`, ready frame `64.311ms`, descent `49.422ms`, resolve `45.912ms`; top timed heads `B` 480.983ms, `S'` 441.790ms, `C` 391.637ms, `C'` 299.007ms, `S` 243.891ms, `C'B` 213.244ms |
| reading | accepted: 105.859s -> 99.022s full gate, byte-identical. The first 1M run was noisy/cold, while 100M/full moved decisively. The profile ranking barely changed; at this point the remaining 95.7s target was still in normal app allocation, descent, arg reads, and app updates, not in a newly exposed rare path. The later strict-`Int` redex frame slice crossed that target |

S1 strict `Int` redex-owning frame slice:

| item | result |
|---|---|
| change | strict `Int` primitive stack arms now use `take_args1`/`take_args2`: they read args through app cells, pop the consumed app segment before forcing, store the consumed redex `NodeId` directly in `StackIntFrame`, mark that redex as a GC root, and overwrite it directly when the forced value returns |
| rationale | this is the eval.c `BININT2`/`BININT1`/`SETINT` shape for the Rust stack machine: the redex root is owned by the strict marker, not rediscovered later from `app_end/used`; unlike the earlier scalar-only `SET*` slice, this removes the consumed app stack segment before strict forcing |
| verification | `cargo fmt --manifest-path rust/microhs-runtime/Cargo.toml --check`; `cargo check --manifest-path rust/microhs-runtime/Cargo.toml`; `cargo check --manifest-path rust/microhs-runtime/Cargo.toml --features eval-phase-profile`; `cargo check --manifest-path rust/microhs-runtime/Cargo.toml --features gc-phase-profile`; `cargo test --manifest-path rust/microhs-runtime/Cargo.toml --lib` (37/37); `cargo build --release --manifest-path rust/microhs-runtime/Cargo.toml --bin mhs-rust-bench`; `cargo check --release --manifest-path rust/microhs-runtime/Cargo.toml --target wasm32-unknown-unknown` |
| bounded gate | first 1M `40.177ms`, 1,001,383 steps, 24.92M steps/s, no GC; 100M `2.858s`, 100,811,774 steps, 35.27M steps/s, 3 GCs, 257.328ms GC pause; post-profile default rebuild 1M `37.466ms`, 26.73M steps/s |
| full gate | `timeout 900s env MHS_GC_NODE_INTERVAL=33554432 target/release/mhs-rust-bench --input /tmp/mhs-selfhost.comb --mode main --warmup-iters 0 --iters 1 -- ./bin/mhs -i -imhs -isrc -ilib MicroHs.Main -o/tmp/mhs-selfhost-rust-out-int-redex.comb` completed in `95.345s`, 3,659,074,959 steps, 38.38M steps/s, 125 GCs, 27.975s GC pause, high-water 40,014,301 cells |
| output | `/tmp/mhs-selfhost.comb` and `/tmp/mhs-selfhost-rust-out-int-redex.comb` both SHA256 `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`; `cmp -s` exit 0 |
| post-slice eval profile | 10M `eval-phase-profile` feature-build non-profile control `316.693ms`, 10,028,861 steps, 31.67M steps/s; profiled run `6242.324ms`; `stack_eval_step_ms=5880.416`, with sub-buckets app allocation `332.689ms`, inner descent `235.919ms`, arg reads `183.931ms`, app update `149.905ms`, ready frame `59.668ms`, descent `48.287ms`, resolve `42.748ms`; top timed heads `B` 449.754ms, `S'` 409.878ms, `C` 371.999ms, `C'` 275.319ms, `S` 231.274ms, `C'B` 200.047ms |
| reading | accepted: 99.022s -> 95.345s full gate, byte-identical, and now just under 2x the fresh 47.85s C oracle. The 100M bounded slice was slightly slower, so this is another reminder that full self-host phase mix is authoritative. The remaining profile ranking is still app allocation, inner app-result descent, arg reads, and app updates |

S1 WHNF redex-owning frame slice:

| item | result |
|---|---|
| change | `seq`, `IO.strict`, and `isint` now use `take_args1`/`take_args2`, pop the consumed app segment before strict forcing, store the consumed redex in `StackWhnfFrame`, mark that redex as a GC root, and finish by writing through that redex |
| rationale | this extends the accepted eval.c `BININT`/`SETINT` discipline to strict WHNF frames: the frame owns the redex it will update, instead of replaying `app_end/used` after the forced value returns |
| verification | `cargo fmt --manifest-path rust/microhs-runtime/Cargo.toml`; `cargo fmt --manifest-path rust/microhs-runtime/Cargo.toml --check`; `cargo check --manifest-path rust/microhs-runtime/Cargo.toml`; `cargo test --manifest-path rust/microhs-runtime/Cargo.toml --lib` (37/37); `cargo check --manifest-path rust/microhs-runtime/Cargo.toml --features eval-phase-profile`; `cargo check --manifest-path rust/microhs-runtime/Cargo.toml --features gc-phase-profile`; `cargo build --release --manifest-path rust/microhs-runtime/Cargo.toml --bin mhs-rust-bench`; `cargo check --release --manifest-path rust/microhs-runtime/Cargo.toml --target wasm32-unknown-unknown`; `git diff --check` |
| bounded gate | 1M `41.776ms`, 1,001,383 steps, 23.97M steps/s, no GC; 100M `2.684s`, 100,811,774 steps, 37.56M steps/s, 3 GCs, 246.571ms GC pause, sink `661902` |
| full gate | `timeout 900s env MHS_GC_NODE_INTERVAL=33554432 target/release/mhs-rust-bench --input /tmp/mhs-selfhost.comb --mode main --warmup-iters 0 --iters 1 -- ./bin/mhs -i -imhs -isrc -ilib MicroHs.Main -o/tmp/mhs-selfhost-rust-out-whnf-redex.comb` completed in `93.202s`, 3,659,074,978 steps, 39.26M steps/s, 125 GCs, 27.492s GC pause, high-water 40,014,303 cells |
| output | `/tmp/mhs-selfhost.comb` and `/tmp/mhs-selfhost-rust-out-whnf-redex.comb` both SHA256 `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`; `cmp -s` exit 0 |
| post-slice eval profile | 10M `eval-phase-profile` feature-build non-profile control `338.631ms`, 10,028,861 steps, 29.62M steps/s; profiled run `6227.382ms`; `stack_eval_step_ms=5868.815`, with sub-buckets app allocation `337.533ms`, inner descent `232.274ms`, arg reads `179.880ms`, app update `149.306ms`, ready frame `59.858ms`, descent `46.652ms`, resolve `42.425ms`, WHNF finish `6.687ms`; top timed heads `B` 450.616ms, `S'` 414.950ms, `C` 371.455ms, `C'` 276.775ms, `S` 230.028ms, `C'B` 201.033ms |
| reading | accepted: 95.345s -> 93.202s full gate, byte-identical, and now 1.95x the fresh 47.85s C oracle. The feature-build non-profile 10M control regressed versus the strict-`Int` control, so full self-host remains the arbiter. The remaining profile ranking is still app allocation, inner app-result descent, arg reads, and app updates |

S1 raw cell tag/trusted free-list accessor slice:

| item | result |
|---|---|
| change | `Cell::tag_bits`/`has_tag` are used by hot App/Prim/scalar/cold accessors; `resolve`/`resolve_profiled` use raw low tag bits for `Indir`/`Free`; `pop_free_node` loads the free cell once and keeps the `Free` check as a debug assertion |
| rationale | eval.c tests tag bits and trusts internal heap/stack invariants; this targets app allocation/free-list reuse, descent/dispatch/resolve tag tests, and scalar probes without changing graph semantics |
| verification | `cargo fmt --manifest-path rust/microhs-runtime/Cargo.toml --check`; `cargo check --manifest-path rust/microhs-runtime/Cargo.toml`; `cargo test --manifest-path rust/microhs-runtime/Cargo.toml --lib` (37/37); `cargo check --manifest-path rust/microhs-runtime/Cargo.toml --features eval-phase-profile`; release bench rebuilt; full gate byte-identical |
| bounded gate | 1M `38.141ms`, 1,001,383 steps, 26.25M steps/s, no GC; 100M `2.551s`, 100,811,774 steps, 39.51M steps/s, 3 GCs, 232.620ms GC pause, sink `661902` |
| full gate | `timeout 900s env MHS_GC_NODE_INTERVAL=33554432 target/release/mhs-rust-bench --input /tmp/mhs-selfhost.comb --mode main --warmup-iters 0 --iters 1 -- ./bin/mhs -i -imhs -isrc -ilib MicroHs.Main -o/tmp/mhs-selfhost-rust-out-rawtag-free.comb` completed in `91.503s`, 3,659,074,997 steps, 39.99M steps/s, 125 GCs, 26.637s GC pause, high-water 40,014,305 cells |
| output | `/tmp/mhs-selfhost.comb` and `/tmp/mhs-selfhost-rust-out-rawtag-free.comb` both SHA256 `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`; `cmp -s` exit 0 |
| post-slice eval profile | 10M `eval-phase-profile` feature-build non-profile control `342.654ms`, 10,028,861 steps, 29.27M steps/s; profiled run `6223.983ms`; `stack_eval_step_ms=5866.274`, with sub-buckets app allocation `332.034ms`, inner descent `229.743ms`, arg reads `181.551ms`, app update `149.669ms`, rewrites `32.520ms`, force frames `7.839ms`, ready frame `58.923ms`, descent `46.427ms`, resolve `42.334ms`, WHNF finish `6.371ms`; top timed heads `B` 451.703ms, `S'` 409.536ms, `C` 370.923ms, `C'` 275.602ms, `S` 229.256ms, `C'B` 198.599ms |
| reading | accepted: 93.202s -> 91.503s full gate, byte-identical, and now 1.91x the fresh 47.85s C oracle. The first raw-tag-only gate was 92.106s; adding raw resolve and trusted free-list pop moved the full gate to 91.503s. The feature-build 10M control is noisy, so full self-host remains the arbiter. The remaining profile ranking is still app allocation, inner app-result descent, arg reads, and app updates; the next target is sub-90s |

Rejected eval-time first-indirection compression retest:

| item | result |
|---|---|
| change | made `resolve_for_whnf` short-circuit the entry indirection after walking a chain, matching eval.c's `SETINDIR(on, n)` shape in the evaluator rather than only during GC mark |
| bounded gate | 1M regressed to `38.671ms`; 100M first was `2.556s` with 224.174ms GC pause, but repeat was `2.618s` with 227.321ms GC pause, so bounded evidence was mixed |
| full gate | `98.036s`, 3,659,075,092 steps, 37.32M steps/s, 125 GCs, 27.551s GC pause, high-water 40,006,672 cells |
| output | `/tmp/mhs-selfhost.comb` and `/tmp/mhs-selfhost-rust-out-resolve-compress.comb` both SHA256 `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`; `cmp -s` exit 0 |
| reading | rejected and reverted. The cell-era retest confirms the old "evaluator first-indirection compression" warning still holds: fewer/shorter chains are not a standalone win because the extra write perturbs the full self-host path |

Rejected S1 stack `Bytes` redex-owning frame extension:

| item | result |
|---|---|
| change | tried extending the accepted strict `Int`/WHNF redex-frame discipline to stack `Bytes` binops: `BytesBin` used `take_args2`, popped the consumed app segment before forcing, stored the consumed redex in `StackBytesFrame`, marked that redex as a GC root, and wrote the bytes result through the frame-owned redex |
| bounded gate | 1M `39.538ms`, 1,001,383 steps, 25.33M steps/s, no GC; 100M `2.580s`, 100,811,774 steps, 39.07M steps/s, 3 GCs, 234.680ms GC pause |
| full gate | `92.679s`, 3,659,074,997 steps, 39.48M steps/s, 125 GCs, 26.895s GC pause, high-water 40,006,662 cells |
| output | `/tmp/mhs-selfhost.comb` and `/tmp/mhs-selfhost-rust-out-bytes-redex.comb` both SHA256 `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`; `cmp -s` exit 0 |
| reading | rejected and reverted. The successful redex-frame idea does not generalize by simply covering the much rarer `Bytes` strict frame: bounded 100M was only competitive, and the full gate lost to both the current parser-small-int repeat and the raw-tag best; post-revert release rebuild 1M sanity returned to `37.371ms` |

F14 ForeignPtr finalizer records:

| item | result |
|---|---|
| change | `ForeignPtrNode.finalizer` now stores a shared side-table record index instead of a Haskell finalizer node; new ForeignPtrs allocate a record, `fp+` copies the same record index, and `fpfin` evaluates the `FunPtr` once into `free`, `closeb`, or null |
| rationale | this matches eval.c's `forptr`/`final` split: offsets share the same finalizer record, changing a finalizer after slicing updates the shared record, and GC runs dead ForeignPtr finalizers inline during sweep rather than keeping a Haskell node alive forever |
| scope | C-compatible host descriptors are `&free`, `&closeb`, `toFunPtr #0`, and parsed `;0`; arbitrary raw nonzero function pointers remain unsupported and fail at `fpfin` rather than crashing at sweep time |
| verification | `cargo fmt --manifest-path rust/microhs-runtime/Cargo.toml --check`; `cargo check --manifest-path rust/microhs-runtime/Cargo.toml`; `cargo check --manifest-path rust/microhs-runtime/Cargo.toml --features eval-phase-profile`; `cargo check --manifest-path rust/microhs-runtime/Cargo.toml --features gc-phase-profile`; `cargo check --release --manifest-path rust/microhs-runtime/Cargo.toml --target wasm32-unknown-unknown`; `cargo test --manifest-path rust/microhs-runtime/Cargo.toml --lib` (37/37); release bench rebuilt |
| focused smoke | `MHS_GC_NODE_INTERVAL=1 target/release/mhs-rust-bench --input /tmp/mhs-f14-null-finalizer-gc.comb --mode whnf --warmup-iters 0 --iters 1` reduced the existing null-finalizer `fpfin` shape with 1 GC, 4 freed nodes, and no sweep crash; `target/release/mhs-rust-bench --input /tmp/mhs-f14-addr-closeb.comb --mode whnf --warmup-iters 0 --iters 1` verifies direct zero-arity `^&closeb` address FFI is accepted as a finalizer descriptor |
| bounded gate | 1M average over 3: `29.672ms`, 1,001,388 steps/iter, 33.75M steps/s, no GC; 100M `2.582s`, 100,811,779 steps, 39.04M steps/s, 3 GCs, 226.466ms GC pause, sink `661902` |
| full gate | `MHS_GC_NODE_INTERVAL=33554432 timeout 900s target/release/mhs-rust-bench --input /tmp/mhs-selfhost.comb --mode main --warmup-iters 0 --iters 1 -- ./bin/mhs -i -imhs -isrc -ilib MicroHs.Main -o/tmp/mhs-selfhost-rust-f14-final.comb` completed in `90.237s`, 3,659,074,888 steps, 40.55M steps/s, 125 GCs, 26.169s GC pause, high-water 40,006,650 cells |
| output | `/tmp/mhs-selfhost.comb` and `/tmp/mhs-selfhost-rust-f14-final.comb` both SHA256 `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`; `cmp -s` exit 0 |
| reading | accepted. This closes F14 for C-style ForeignPtr finalizers and unexpectedly nudges the full gate from the 91.503s raw-tag best to 90.237s without changing reduction count materially. Weak pointer finalizer semantics are still separate: C spawns Haskell weak finalizers as threads, while Rust still marks weak values/finalizers conservatively |

S1 unified stack `ret`/`top` continuation:

| item | result |
|---|---|
| change | `stack_eval_step` now handles ready strict-frame returns before dispatching a head, so strict force markers descend in the same stack loop instead of returning `StackStep::Force`; K/A/KK/KA/K2/K3/K4 indirection rewrites now continue like eval.c `GOIND(x); goto top` |
| rationale | the guarded GOIND-only probe proved the missing piece: without C-style `ret:` handling inside the stack loop, a continued GOIND can reach an `Int` frame result and fall into the wrong WHNF path. Moving ready strict-frame return handling into the loop makes the continuation structural instead of a guard |
| verification | `cargo fmt --manifest-path rust/microhs-runtime/Cargo.toml --check`; `cargo check --manifest-path rust/microhs-runtime/Cargo.toml`; `cargo test --manifest-path rust/microhs-runtime/Cargo.toml --lib` (37/37); `cargo check --manifest-path rust/microhs-runtime/Cargo.toml --features eval-phase-profile`; `cargo check --manifest-path rust/microhs-runtime/Cargo.toml --features gc-phase-profile`; `cargo check --release --manifest-path rust/microhs-runtime/Cargo.toml --target wasm32-unknown-unknown`; `git diff --check`; release bench rebuilt |
| bounded gate | 10M `293.691ms`, 10,028,866 steps, 34.15M steps/s, no GC; 100M `2.552s`, 100,811,779 steps, 39.50M steps/s, 3 GCs, 235.017ms GC pause |
| full gate | `MHS_GC_NODE_INTERVAL=33554432 timeout 900s target/release/mhs-rust-bench --input /tmp/mhs-selfhost.comb --mode main --warmup-iters 0 --iters 1 -- ./bin/mhs -i -imhs -isrc -ilib MicroHs.Main -o/tmp/mhs-selfhost-rust-ret-top.comb` completed in `89.141s`, 3,659,074,850 steps, 41.05M steps/s, 125 GCs, 26.352s GC pause, high-water 40,010,793 cells |
| output | `/tmp/mhs-selfhost-rust-ret-top.comb` and `/tmp/mhs-selfhost-rust-f14-final.comb` both SHA256 `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`; `cmp -s` exit 0 |
| reading | accepted and new best: 90.237s -> 89.141s, byte-identical, about 1.86x the 47.85s C oracle. The useful part is not GOIND alone; it is moving another eval.c `ret`/`top` layer inside the single stack loop |

S2 parse-label non-root GC treatment:

| item | result |
|---|---|
| change | parse labels are no longer marked as GC roots; after `mark_reachable`, the public label map retains only entries whose target node was marked live |
| rationale | eval.c's `shared_table` is parse scaffolding and is freed immediately after `parse_top`; references and cycles are already patched into the graph as direct targets or indirections, so the graph should own reachability rather than the label map |
| verification | `cargo fmt --manifest-path rust/microhs-runtime/Cargo.toml --check`; `cargo check --manifest-path rust/microhs-runtime/Cargo.toml`; `cargo test --manifest-path rust/microhs-runtime/Cargo.toml --lib` (37/37); `cargo check --manifest-path rust/microhs-runtime/Cargo.toml --features eval-phase-profile`; `cargo check --manifest-path rust/microhs-runtime/Cargo.toml --features gc-phase-profile`; `cargo check --release --manifest-path rust/microhs-runtime/Cargo.toml --target wasm32-unknown-unknown`; `git diff --check`; release bench rebuilt |
| bounded gate | 10M `334.999ms`, 10,028,866 steps, 29.94M steps/s, no GC; 100M `2.587s`, 100,811,779 steps, 38.97M steps/s, 3 GCs, 221.196ms GC pause, last live 599,396 nodes |
| full gate | `MHS_GC_NODE_INTERVAL=33554432 timeout 900s target/release/mhs-rust-bench --input /tmp/mhs-selfhost.comb --mode main --warmup-iters 0 --iters 1 -- ./bin/mhs -i -imhs -isrc -ilib MicroHs.Main -o/tmp/mhs-selfhost-rust-nolabelroot.comb` completed in `88.623s`, 3,659,074,926 steps, 41.29M steps/s, 125 GCs, 25.826s GC pause, last live 1,573,832 nodes, high-water 40,008,403 cells |
| output | `/tmp/mhs-selfhost-rust-nolabelroot.comb`, `/tmp/mhs-selfhost-rust-ret-top.comb`, and `/tmp/mhs-selfhost.comb` all SHA256 `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`; both `cmp -s` checks exit 0 |
| reading | accepted and new best: 89.141s -> 88.623s, byte-identical, about 1.85x the 47.85s C oracle. The bounded mutator time is noisy/slower, but the full arbiter wins through a smaller root set and 25.826s total GC pause |

S1 allocator free-head invariant tightening:

| item | result |
|---|---|
| change | `pop_free_node` now treats `free_nodes > 0` as proof that `free_head` is present in release, using `unreachable_unchecked` for the impossible empty-head case, and `pop_free_node`/`push_app_node` are marked inline on the reused-allocation hot path |
| rationale | eval.c relies on heap/free-list invariants in the allocator hot path. The Rust app allocator was already dedicated and tag-trusting, but still carried one release `Option` branch before every reused free-cell pop; every B/C/S-family app allocation hits this path after the first GC |
| verification | `cargo fmt --manifest-path rust/microhs-runtime/Cargo.toml --check`; `cargo check --manifest-path rust/microhs-runtime/Cargo.toml`; `cargo test --manifest-path rust/microhs-runtime/Cargo.toml --lib` (37/37); `cargo check --manifest-path rust/microhs-runtime/Cargo.toml --features eval-phase-profile`; `cargo check --manifest-path rust/microhs-runtime/Cargo.toml --features gc-phase-profile`; `cargo check --release --manifest-path rust/microhs-runtime/Cargo.toml --target wasm32-unknown-unknown`; `git diff --check`; release bench rebuilt |
| bounded control | current accepted 100M control before the slice: `2.533s`, 100,811,779 steps, 39.80M steps/s, 3 GCs, 215.248ms GC pause |
| bounded gate | 100M `2.450s`, 41.14M steps/s, 3 GCs, 212.439ms GC pause; repeat `2.463s`, 40.93M steps/s, 3 GCs, 213.868ms GC pause |
| full gate | `MHS_GC_NODE_INTERVAL=33554432 timeout 900s target/release/mhs-rust-bench --input /tmp/mhs-selfhost.comb --mode main --warmup-iters 0 --iters 1 -- ./bin/mhs -i -imhs -isrc -ilib MicroHs.Main -o/tmp/mhs-selfhost-rust-freehead-full.comb` completed in `86.619s`, 3,659,074,964 steps, 42.24M steps/s, 125 GCs, 25.625s GC pause, last live 1,573,828 nodes, high-water 40,008,407 cells |
| output | `/tmp/mhs-selfhost-rust-freehead-full.comb` and `/tmp/mhs-selfhost.comb` both SHA256 `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`; `cmp` exit 0 |
| rejected siblings | merging `IO.strict`/`seq`/`isint` into the main known-head match regressed 100M to `2.561s` versus the `2.533s` control; a direct in-cell code fast path for hot B/C/S heads regressed 100M to `2.690s`; both were reverted before full gates |
| reading | accepted and new best: 88.623s -> 86.619s, byte-identical, about 1.81x the 47.85s C oracle. This is still an app-allocation bucket win rather than a GC-policy change; the next work should keep removing allocator/update/descent overhead in the B/C/S family |

S1 identity-alias `GOIND` continuation:

| item | result |
|---|---|
| change | `I`/`Ord`/`Chr` rewrites now use a `rewrite_continue_reductions` path that applies the stack rewrite, records the reduction, and immediately descends from the result inside `stack_eval_step` instead of returning `StackStep::Reduced` to the outer driver |
| rationale | eval.c handles `T_I` as `CHKARG1; GOIND(x)` and keeps running at `top`; after the accepted `ret`/`top` work, Rust can safely extend the same continuation discipline to the identity-alias heads. This removes a real remaining return-to-driver path rather than changing helper shape |
| verification | `cargo fmt --manifest-path rust/microhs-runtime/Cargo.toml --check`; `cargo check --manifest-path rust/microhs-runtime/Cargo.toml`; `cargo test --manifest-path rust/microhs-runtime/Cargo.toml --lib` (37/37); `cargo check --release --manifest-path rust/microhs-runtime/Cargo.toml --target wasm32-unknown-unknown`; `cmp -s /tmp/mhs-selfhost.comb /tmp/mhs-selfhost-rust-goind-id-full.comb`; release bench rebuilt after the profile run |
| bounded control | fresh accepted 100M repeat before the slice: `2.511s`, 100,811,779 steps, 40.14M steps/s, 3 GCs, 212.301ms GC pause |
| profile check | same 10M `eval-phase-profile` run before/after: feature non-profile `330.457ms` -> `303.405ms`; profile total `9314.208ms` -> `9216.881ms`; stack loop iterations `613,264` -> `280,614`; stack eval-step calls `529,637` -> `256,660`; `StackStep::Reduced` exits `483,973` -> `210,996` |
| bounded gate | 100M `2.481s`, 40.63M steps/s, 3 GCs, 217.576ms GC pause; repeat `2.403s`, 41.95M steps/s, 3 GCs, 212.838ms GC pause |
| full gate | `MHS_GC_NODE_INTERVAL=33554432 timeout 900s target/release/mhs-rust-bench --input /tmp/mhs-selfhost.comb --mode main --warmup-iters 0 --iters 1 -- ./bin/mhs -i -imhs -isrc -ilib MicroHs.Main -o/tmp/mhs-selfhost-rust-goind-id-full.comb` completed in `86.500s`, 3,659,074,964 steps, 42.30M steps/s, 125 GCs, 25.920s GC pause, last live 1,558,374 nodes, high-water 40,021,286 cells |
| output | `/tmp/mhs-selfhost-rust-goind-id-full.comb` and `/tmp/mhs-selfhost.comb` both SHA256 `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`; `cmp` exit 0 |
| reading | accepted and new best: 86.619s -> 86.500s, byte-identical, about 1.81x the 47.85s C oracle. The profile counters prove the intended layer was removed, but the full win is small and GC pause rose slightly; the remaining high-value buckets are still app allocation, inner descent, arg reads, app updates, and GC pause |

S1 one-word app-fun spine descent:

| item | result |
|---|---|
| change | hot stack spine descent now uses `app_fun`, which reads only `Cell.word0` to test the App tag and extract the fun `NodeId`; argument words stay untouched until the later `CHKARG`/`take_args` arg loads. This is used in `descend_stack_from` and the outer stack descent loop |
| rationale | eval.c's descent reads `n->ufun.uutag`, checks AP, pushes the app, and follows the fun pointer; it does not load `ARG(n)` during descent. The previous generic-helper attribution showed hidden `app()` calls are too small to matter, so the useful remaining descent probe is the actual word traffic in the AP walk |
| verification | `cargo fmt --manifest-path rust/microhs-runtime/Cargo.toml --check`; `cargo check --manifest-path rust/microhs-runtime/Cargo.toml`; `cargo test --manifest-path rust/microhs-runtime/Cargo.toml --lib` (37/37); `cargo check --release --manifest-path rust/microhs-runtime/Cargo.toml --target wasm32-unknown-unknown`; `cmp -s /tmp/mhs-selfhost.comb /tmp/mhs-selfhost-rust-appfun-full.comb`; SHA256 matches `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a` |
| default-profile 10M | normal pass `334.812ms`, 10,028,866 steps, 29.95M steps/s, no GC; profiled pass `2000.764ms`; 11,215,906 app allocations; 8,292,661 stack app updates; 30,244,488 stack descent pushes; 30,119,444 stack arg reads; 90 fallback eval-loop steps; `<generic app()>` is only 99,864 app allocations |
| phase-profile 10M | feature-build normal pass `324.987ms`; profiled pass `9256.176ms`; `stack_eval_step_ms=9232.961`; sub-buckets: app alloc 346.997ms, inner descent 301.984ms, arg reads 183.331ms, app updates 147.863ms, rewrites 33.338ms, force frames 6.713ms |
| top phase buckets | stack head time remains ordinary combinator traffic: `B` 869.376ms, `C` 727.888ms, `S'` 713.956ms, `C'` 498.642ms, `S` 411.348ms, `C'B` 362.609ms; app allocation sites remain `B.yz`, `C.xz`, `S'.zw`/`S'.left`/`S'.yw`, `C'.xyw`/`C'.yw`, `S.right`/`S.left`, and `C'B` payloads |
| bounded gate | 100M `2.424s`, 100,811,779 steps, 41.58M steps/s, 3 GCs, 210.284ms GC pause |
| full gate | `timeout 900s env MHS_GC_NODE_INTERVAL=33554432 target/release/mhs-rust-bench --input /tmp/mhs-selfhost.comb --mode main --warmup-iters 0 --iters 1 -- ./bin/mhs -i -imhs -isrc -ilib MicroHs.Main -o/tmp/mhs-selfhost-rust-appfun-full.comb` completed in `85.189s`, 3,659,074,926 steps, 42.95M steps/s, 125 GCs, 25.508s GC pause, last live 1,558,370 nodes, high-water 40,021,282 cells |
| output | `/tmp/mhs-selfhost-rust-appfun-full.comb` and `/tmp/mhs-selfhost.comb` both SHA256 `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`; `cmp` exit 0 |
| reading | accepted and new best: 86.500s -> 85.189s, byte-identical, about 1.78x the 47.85s C oracle. The 100M bounded slice was not a clean win, so the full self-host gate remains the arbiter; this confirms that eval.c's one-word AP walk matters even after the larger continuation changes. The post-slice profile ranking is still app allocation, inner descent, arg reads, and app updates, so the next useful change needs to remove work from those buckets rather than chase generic helper calls |

S2 mark-time app-edge canonicalization:

| item | result |
|---|---|
| change | GC marking now canonicalizes App fun/arg edges before pushing children: an edge to an indirection is rewritten to the resolved target, and an edge to an `Int` in the `-10..255` cache range is rewritten to the canonical small-int node when that cache entry exists |
| rationale | eval.c's `mark(NODEPTR *np)` receives a parent slot and can rewrite it while marking; the previous Rust mark worklist compressed the indirection node but usually left the parent App edge pointing at the old node. This slice copies the pointer-to-slot behavior for App children without changing mutator dispatch |
| verification | `cargo fmt --manifest-path rust/microhs-runtime/Cargo.toml --check`; `cargo check --manifest-path rust/microhs-runtime/Cargo.toml`; `cargo test --manifest-path rust/microhs-runtime/Cargo.toml --lib` (37/37); `cargo check --release --manifest-path rust/microhs-runtime/Cargo.toml --target wasm32-unknown-unknown`; `git diff --check`; `cmp -s /tmp/mhs-selfhost.comb /tmp/mhs-selfhost-gc-canon-full.comb`; SHA256 matches `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a` |
| bounded control | accepted post-WHNF-revert 100M control `2.432s`, 100,811,779 steps, 41.44M steps/s, 3 GCs, 217.562ms GC pause; earlier same-session control before the WHNF probe was `2.413s`, 41.77M steps/s, 210.644ms GC pause |
| bounded gate | 10M `323.168ms`, 31.03M steps/s, no GC; 100M `2.371s`, 100,811,779 steps, 42.52M steps/s, 3 GCs, 224.430ms GC pause, last live 467,177 nodes, high-water 33,941,631 cells |
| full gate | `timeout 900s env MHS_GC_NODE_INTERVAL=33554432 target/release/mhs-rust-bench --input /tmp/mhs-selfhost.comb --mode main --warmup-iters 0 --iters 1 -- ./bin/mhs -i -imhs -isrc -ilib MicroHs.Main -o/tmp/mhs-selfhost-gc-canon-full.comb` completed in `83.752s`, 3,659,074,869 steps, 43.69M steps/s, 125 GCs, 24.312s GC pause, last live 1,289,534 nodes, high-water 38,820,708 cells |
| output | `/tmp/mhs-selfhost-gc-canon-full.comb` and `/tmp/mhs-selfhost.comb` both SHA256 `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`; `cmp` exit 0 |
| reading | accepted and new best: 85.189s -> 83.752s, byte-identical, about 1.75x the 47.85s C oracle. The 100M slice showed the intended heap-shape change before the full gate: live nodes and high-water dropped even though GC pause rose locally. Full run wins through lower retained graph/high-water and lower total GC pause, so the next GC-side candidate is extending the same parent-slot model beyond App children rather than adding another local stack-loop continuation |

S1 trusted stack redex/update access:

| item | result |
|---|---|
| change | `EvalStack` now has a debug-checked `app_unchecked` accessor used by hot stack rewrite/app-update helpers after arity has already proven the app segment is present. `apply_stack_rewrite`, `apply_stack_frame_rewrite`, `apply_stack_frame_value`, and `apply_stack_app` now return plain `NodeId` instead of `Result<NodeId, EvalError>`; the cold rethread path keeps its checked `stack_entry_app` handling |
| rationale | this is the same eval.c trust rule as the existing unchecked `ARG(TOP(i))` loads: once `CHKARGn` has succeeded, missing redex app slots are internal corruption, not a recoverable runtime error. The previous helpers still built impossible `DanglingIndirection` errors on the hot path |
| verification | `cargo fmt --manifest-path rust/microhs-runtime/Cargo.toml --check`; `cargo check --manifest-path rust/microhs-runtime/Cargo.toml`; `cargo build --release --manifest-path rust/microhs-runtime/Cargo.toml --bin mhs-rust-bench`; `cargo test --manifest-path rust/microhs-runtime/Cargo.toml --lib` (37/37); `cargo check --release --manifest-path rust/microhs-runtime/Cargo.toml --target wasm32-unknown-unknown`; `cmp -s /tmp/mhs-selfhost.comb /tmp/mhs-selfhost-rust-out-trusted-stack.comb`; SHA256 matches `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a` |
| profile/control | current-tree 10M profile control before the slice: normal `311.419ms`, 10,028,866 steps, 32.20M steps/s; profiled `2007.345ms`; app allocations 11,215,900, stack app updates 8,292,661, stack arg reads 30,119,444, fallback eval-loop steps 90 |
| bounded gate | 10M `295.019ms`, 10,028,866 steps, 33.99M steps/s, no GC; 100M `2.371s`, 100,811,779 steps, 42.52M steps/s, 3 GCs, 222.867ms GC pause, last live 467,197 nodes, high-water 33,941,651 cells |
| full gate | `timeout 900s env MHS_GC_NODE_INTERVAL=33554432 target/release/mhs-rust-bench --input /tmp/mhs-selfhost.comb --mode main --warmup-iters 0 --iters 1 -- ./bin/mhs -i -imhs -isrc -ilib MicroHs.Main -o/tmp/mhs-selfhost-rust-out-trusted-stack.comb` completed in `82.154s`, 3,659,075,040 steps, 44.54M steps/s, 125 GCs, 23.913s GC pause, last live 1,289,554 nodes, high-water 38,820,726 cells |
| output | `/tmp/mhs-selfhost-rust-out-trusted-stack.comb` and `/tmp/mhs-selfhost.comb` both SHA256 `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`; `cmp` exit 0 |
| reading | accepted and new best: 83.752s -> 82.154s, byte-identical, about 1.72x the 47.85s C oracle. This is not a new algorithm, but it removes a real Rust-only layer from app-update/rewrite traffic and confirms the eval.c trust discipline is still paying after the larger S1/S2/S3 changes |

eval.c permanent compound caches:

| item | result |
|---|---|
| change | `Program` now has a `CompoundCache` for C-style permanent `combFst` (`U K`), `combSnd` (`U A`), `combJust` (`Z U`), and `combPairUnit` (`P I`). The cached app nodes are marked as GC roots, and `fst`/`snd`/`just`/`unit_pair` reuse them instead of rebuilding the compound prefix each time |
| rationale | eval.c allocates these below `heap_start` in `init_nodes`, so helper constructors avoid both repeated graph allocation and later collection. This is a broader eval.c representation decision, not another app-site bookkeeping tweak |
| verification | `cargo fmt --manifest-path rust/microhs-runtime/Cargo.toml --check`; `cargo check --manifest-path rust/microhs-runtime/Cargo.toml`; `cargo build --release --manifest-path rust/microhs-runtime/Cargo.toml --bin mhs-rust-bench`; `cargo test --manifest-path rust/microhs-runtime/Cargo.toml --lib` (37/37); `cargo check --release --manifest-path rust/microhs-runtime/Cargo.toml --target wasm32-unknown-unknown`; `git diff --check`; `cmp -s /tmp/mhs-selfhost.comb /tmp/mhs-selfhost-rust-compound-cache-full.comb`; SHA256 matches `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a` |
| bounded gate | 10M `320.240ms`, 10,028,866 steps, 31.32M steps/s, no GC, high-water 11,369,696 cells; 100M `2.390s`, 100,811,779 steps, 42.17M steps/s, 3 GCs, 220.223ms GC pause, last live 467,199 nodes, high-water 33,941,653 cells |
| full gate | `timeout 900s env MHS_GC_NODE_INTERVAL=33554432 target/release/mhs-rust-bench --input /tmp/mhs-selfhost.comb --mode main --warmup-iters 0 --iters 1 -- ./bin/mhs -i -imhs -isrc -ilib MicroHs.Main -o/tmp/mhs-selfhost-rust-compound-cache-full.comb` completed in `80.927s`, 3,659,075,078 steps, 45.21M steps/s, 125 GCs, 23.793s GC pause, last live 1,289,560 nodes, high-water 38,820,730 cells |
| output | `/tmp/mhs-selfhost-rust-compound-cache-full.comb` and `/tmp/mhs-selfhost.comb` both SHA256 `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`; `cmp` exit 0 |
| reading | accepted and new best: 82.154s -> 80.927s, byte-identical, about 1.69x the 47.85s C oracle. The bounded node-count signal is tiny because the cached compounds are cold-helper prefixes, but full self-host wins; keep following eval.c's permanent-node decisions when they remove graph construction rather than attribution overhead |

Current-tree benchmark refresh after `68bf5347`:

| item | result |
|---|---|
| release build | `cargo build --release --manifest-path rust/microhs-runtime/Cargo.toml --bin mhs-rust-bench` was already up to date |
| 10M bounded gate | `timeout 180s env MHS_GC_NODE_INTERVAL=33554432 target/release/mhs-rust-bench --input /tmp/mhs-selfhost.comb --mode main --step-limit 10000000 --warmup-iters 0 --iters 1 -- ./bin/mhs -i -imhs -isrc -ilib MicroHs.Main -o/tmp/mhs-selfhost-rust-refresh-10m.comb` completed in `363.915ms`, 10,028,866 steps, 27.56M steps/s, no GC, high-water 11,369,682 cells |
| 100M bounded gate | first run `2.709s`, 100,811,779 steps, 37.22M steps/s, 3 GCs, `243.985ms` GC pause, high-water 33,941,639 cells; repeat `2.523s`, 39.96M steps/s, 3 GCs, `227.782ms` GC pause, high-water 33,941,653 cells |
| full gate | `timeout 900s env MHS_GC_NODE_INTERVAL=33554432 target/release/mhs-rust-bench --input /tmp/mhs-selfhost.comb --mode main --warmup-iters 0 --iters 1 -- ./bin/mhs -i -imhs -isrc -ilib MicroHs.Main -o/tmp/mhs-selfhost-rust-refresh-full.comb` completed in `84.503s`, 3,659,074,945 steps, 43.30M steps/s, 125 GCs, `24.426s` GC pause, last live 1,289,545 nodes, high-water 38,820,716 cells |
| output | `/tmp/mhs-selfhost-rust-refresh-full.comb` SHA256 `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`; `cmp -s` against `/tmp/mhs-selfhost-rust-compound-cache-full.comb` exits 0 |
| reading | same committed tree remains semantically good but the refresh is slower/noisier than the 80.927s best. Treat 80.927s as the accepted best and 84.503s as the latest same-tree sanity rerun, not a regression from code changes |

Current bounded/profile refresh after rejected probes:

| item | result |
|---|---|
| default release rebuild | `cargo build --release --manifest-path rust/microhs-runtime/Cargo.toml --bin mhs-rust-bench` rebuilt the normal non-profile binary after the profiling-feature runs |
| default 10M bounded gate | `timeout 180s env MHS_GC_NODE_INTERVAL=33554432 target/release/mhs-rust-bench --input /tmp/mhs-selfhost.comb --mode main --step-limit 10000000 --warmup-iters 0 --iters 1 -- ./bin/mhs -i -imhs -isrc -ilib MicroHs.Main -o/tmp/mhs-selfhost-rust-refresh-current-10m.comb` completed in `358.767ms`, 10,028,866 steps, 27.95M steps/s, no GC, high-water 11,369,698 cells |
| default 100M bounded gate | `timeout 240s env MHS_GC_NODE_INTERVAL=33554432 target/release/mhs-rust-bench --input /tmp/mhs-selfhost.comb --mode main --step-limit 100000000 --warmup-iters 0 --iters 1 -- ./bin/mhs -i -imhs -isrc -ilib MicroHs.Main -o/tmp/mhs-selfhost-rust-refresh-current-100m.comb` completed in `2.354s`, 100,811,779 steps, 42.83M steps/s, 3 GCs, `225.909ms` GC pause, last live 467,201 nodes, high-water 33,941,655 cells |
| default full gate | `timeout 900s env MHS_GC_NODE_INTERVAL=33554432 target/release/mhs-rust-bench --input /tmp/mhs-selfhost.comb --mode main --warmup-iters 0 --iters 1 -- ./bin/mhs -i -imhs -isrc -ilib MicroHs.Main -o/tmp/mhs-selfhost-rust-current-full.comb` completed in `83.062s`, 3,659,074,945 steps, 44.05M steps/s, 125 GCs, `24.354s` GC pause, last live 1,289,545 nodes, high-water 38,820,716 cells |
| full output | `/tmp/mhs-selfhost-rust-current-full.comb` SHA256 `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`; `cmp -s` against `/tmp/mhs-selfhost-rust-compound-cache-full.comb` exits 0 |
| `gc-phase-profile` 100M bounded gate | `2.379s`, 100,811,779 steps, 42.38M steps/s, 3 GCs, `222.787ms` GC pause, `75.562ms` mark, `118.929ms` sweep, last live 467,191 nodes, high-water 33,941,645 cells |
| reading | the current normal bounded run is back in the accepted 42M/s band after rebuilding without profile features, and the current full gate is byte-identical at about 1.74x the fresh C oracle. The GC split is still sweep-heavy on the 100M slice, but allocator/bitmap/reuse-order probes already showed that transplanting eval.c's bitmap/free-order pieces alone does not beat the full gate |

Current `eval-phase-profile` refresh after rejected probes:

| measure | value |
|---|---:|
| feature normal 10M parse/reduce/render | 388.542 ms |
| profiled 10M total | 10,153.891 ms |
| WHNF steps | 10,028,866 |
| stack eval step time | 10,126.146 ms |
| GC check / resolve / ready-frame / descent / WHNF finish | 5.567 / 5.869 / 5.249 / 6.424 / 7.344 ms |
| arg reads / app allocation / app update / rewrite | 200.161 / 374.275 / 162.946 / 35.837 ms |
| inner app-result descent / force-frame push | 334.197 / 7.708 ms |
| stack loop iterations / step calls | 280,614 / 256,660 |
| app allocations | 11,215,906 |
| stack app updates / stack rewrites | 8,292,661 / 1,711,080 |
| stack descent pushes / arg reads | 30,244,488 / 30,119,444 |
| strict snapshots / remaining-app scans / fallback steps | 0 / 0 / 90 |
| top head-time buckets | `B` 957.298ms, `C` 785.915ms, `S'` 778.137ms, `C'` 543.174ms, `S` 454.485ms, `C'B` 396.316ms |
| top app-allocation sites | `B.yz` 1,943,041; `C.xz` 1,609,624; `S'.left`/`S'.yw`/`S'.zw` 810,754 each; `C'.xyw`/`C'.yw` 741,069 each |
| reading | the profile shape is unchanged after the rejected probes: standard app-building combinators dominate, with app allocation and inner descent still ahead of arg reads and app updates. The remaining fallback bridge is too small to matter; the next useful slice has to remove work from the B/C/S app-build/update/descent path, not generic helper dispatch |

App-allocation split profile:

| item | result |
|---|---|
| change | added `eval-phase-profile`-only counters/timers inside `push_app_node` for fresh vs reused app cells, free-list pop, reused-cell write, and fresh arena push; default builds compile these fields and timers out |
| verification | `cargo fmt --manifest-path rust/microhs-runtime/Cargo.toml`; `cargo check --manifest-path rust/microhs-runtime/Cargo.toml`; `cargo check --manifest-path rust/microhs-runtime/Cargo.toml --features eval-phase-profile`; `cargo test --manifest-path rust/microhs-runtime/Cargo.toml --lib` (37/37); `cargo build --release --manifest-path rust/microhs-runtime/Cargo.toml --bin mhs-rust-bench --features eval-phase-profile`; default release bench rebuilt; `git diff --check` |
| no-GC 10M profile | feature normal `341.152ms`, profiled `10,013.545ms`, 10,028,866 steps, no GC, app allocations 11,215,904; split: reused 0, fresh 11,215,904, free-pop timer 195.741ms, fresh-push timer 343.861ms |
| forced-GC 10M profile | `MHS_GC_NODE_INTERVAL=1048576`; feature normal `365.741ms`, profiled `10,079.068ms`, 10,028,866 steps, 10 GCs, `158.373ms` profiled GC pause, high-water 1,582,309 cells; split: reused 9,787,060, fresh 1,428,850, free-pop timer 201.884ms, reused-write timer 170.092ms, fresh-push timer 45.348ms |
| default 100M sanity | after rebuilding without `eval-phase-profile`, 100M completed in `2.407s`, 100,811,779 steps, 41.89M steps/s, 3 GCs, `227.031ms` GC pause, last live 467,213 nodes, high-water 33,941,667 cells |
| default full gate | after rebuilding without `eval-phase-profile`, full self-host completed in `82.949s`, 3,659,075,211 steps, 44.11M steps/s, 125 GCs, `24.396s` GC pause, last live 1,289,576 nodes, high-water 38,820,744 cells |
| output | `/tmp/mhs-selfhost-rust-alloc-profile-default-full.comb` and `/tmp/mhs-selfhost.comb` both SHA256 `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`; `cmp` exit 0 |
| reading | the split confirms that after GC the hot app allocator is overwhelmingly the reused-cell path, but the nested `Instant` probes inflate `profile_stack_app_alloc_ms` and should not be treated as absolute cost. The next allocator-side candidate must remove real reused-cell work without adding a slower allocation search; the bitmap allocator and low-to-high reuse probes already showed that free-list policy alone is not enough |

GCRED opportunity profile:

| item | result |
|---|---|
| change | added `gc-phase-profile`-only counters for eval.c's mark-time reduction opportunities: `I x`, `K x y`, `A x y`, `B I`, `B x I`, `C'B x I`, `C (C x)`, `C' I`, `C'B ((B C) P)`, and `C op -> flip(op)`. This is instrumentation only; it does not rewrite graph cells |
| rationale | eval.c's GCRED/`flip_ops` machinery is a larger representation-level idea than another local app-result hop, but C also disables broad GCRED after the initial parse because it is "rarely a win". Before porting reducers, measure whether the self-host graph actually contains the shapes on normal GC boundaries |
| verification | `cargo fmt --manifest-path rust/microhs-runtime/Cargo.toml`; `cargo check --manifest-path rust/microhs-runtime/Cargo.toml`; `cargo check --manifest-path rust/microhs-runtime/Cargo.toml --features gc-phase-profile`; release builds with and without `gc-phase-profile`; full output SHA256 `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`; `cmp_exit=0` |
| `gc-phase-profile` 100M gate | `2.450s`, 100,811,779 steps, 41.15M steps/s, 3 GCs, `230.533ms` GC pause split as `85.521ms` mark and `117.058ms` sweep, last live 467,197 nodes, high-water 33,941,651 cells |
| GCRED counters | all zero on the 100M gate: `I`, `K`, `A`, `B I`, `B x I`, `C'B x I`, `C (C x)`, `C' I`, `C'B ((B C) P)`, and `C op` |
| default 100M sanity | after rebuilding without `gc-phase-profile`, 100M completed in `2.448s`, 100,811,779 steps, 41.17M steps/s, 3 GCs, `229.873ms` GC pause, last live 467,197 nodes, high-water 33,941,651 cells |
| default full gate | full self-host completed in `82.693s`, 3,659,075,059 steps, 44.25M steps/s, 125 GCs, `24.388s` GC pause, last live 1,289,558 nodes, high-water 38,820,728 cells |
| output | `/tmp/mhs-selfhost-rust-gcred-default-full.comb` and `/tmp/mhs-selfhost.comb` both SHA256 `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`; `cmp` exit 0 |
| reading | do not implement broad GCRED for self-host throughput right now. The specific eval.c mark-reduction shapes are absent at the normal 100M collection boundaries, so the remaining performance gap is still the B/C/S-family app-build/update/descent path plus material but already-profiled GC pause |

S4 young-region viability profile:

| item | result |
|---|---|
| change | added `gc-phase-profile`-only allocation-set counters: every allocated slot since the previous GC is recorded, then after normal full marking the collector reports allocated slots, live/dead allocated slots, and live old objects with direct old-to-young edges. Also added a debug-only `arg_from_app` invariant check to match the surrounding trusted stack accessors |
| rationale | NOTES.md points at the only unburned GC bet: reduce sweep/mark scope with a non-moving young region while preserving the fast intrusive free-list allocation path. These counters test whether a remembered set would be small and whether allocation-set sweep scope can be materially smaller than the arena before changing collection semantics |
| verification | `cargo fmt --manifest-path rust/microhs-runtime/Cargo.toml --check`; `cargo check --manifest-path rust/microhs-runtime/Cargo.toml`; `cargo check --manifest-path rust/microhs-runtime/Cargo.toml --features gc-phase-profile`; `cargo test --manifest-path rust/microhs-runtime/Cargo.toml --lib` 37/37; `cargo check --release --manifest-path rust/microhs-runtime/Cargo.toml --target wasm32-unknown-unknown`; release builds with and without `gc-phase-profile`; full output SHA256 `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`; `cmp_exit=0` |
| `gc-phase-profile` 32M 100M gate | `2.961s`, 100,811,779 steps, 34.04M steps/s, 3 GCs, `444.085ms` GC pause split as `83.662ms` mark and `119.606ms` sweep; allocated-slot totals: 100,663,928 slots, 401,497 live, 100,262,431 dead, 2,052 old-to-young sources, 2,534 old-to-young edges |
| `gc-phase-profile` 8M 100M gate | `2.787s`, 100,811,779 steps, 36.17M steps/s, 14 GCs, `831.859ms` GC pause split as `345.701ms` mark and `161.857ms` sweep; allocated-slot totals: 117,441,968 slots, 1,036,436 live, 116,405,532 dead, 5,088 old-to-young sources, 6,684 old-to-young edges |
| default 100M sanity | after rebuilding without `gc-phase-profile`, 100M completed in `2.436s`, 100,811,779 steps, 41.38M steps/s, 3 GCs, `252.515ms` GC pause, last live 467,197 nodes, high-water 33,941,651 cells |
| default full gate | after rebuilding without `gc-phase-profile`, full self-host completed in `82.817s`, 3,659,075,154 steps, 44.18M steps/s, 125 GCs, `24.153s` GC pause, last live 1,289,567 nodes, high-water 38,820,738 cells |
| output | `/tmp/mhs-selfhost-rust-s4-profile-default-full.comb` and `/tmp/mhs-selfhost-rust-compound-cache-full.comb` both SHA256 `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`; `cmp` exit 0 |
| reading | S4 is plausible only as a real allocation-set/remembered-set minor collector. At the current 32M full-GC cadence the allocated set is nearly as large as the arena, so a minor sweep would not buy enough by itself. At 8M, the remembered set is still tiny, but normal full marking dominates because every collection still marks old live data; the next experiment should implement minor marking from roots plus remembered old sources and traverse only young objects, with periodic full collection for old garbage |

Rejected S4 allocation-set minor collector prototype:

| item | result |
|---|---|
| change | added a feature-gated `minor-gc` prototype with `MHS_GC_MINOR_NODE_INTERVAL`, allocation-set tracking, old-to-young write barriers around the central setters plus array/MVar writes, minor marking from roots and remembered old sources, and a periodic full collection using `MHS_GC_NODE_INTERVAL` |
| rationale | this was the direct test of the S4 viability counters: shrink the sweep set and reduce high-water memory while staying non-moving and preserving stable `NodeId`s |
| verification | `cargo fmt --manifest-path rust/microhs-runtime/Cargo.toml`; `cargo check --manifest-path rust/microhs-runtime/Cargo.toml`; `cargo check --manifest-path rust/microhs-runtime/Cargo.toml --features minor-gc`; `cargo check --manifest-path rust/microhs-runtime/Cargo.toml --features minor-gc,gc-phase-profile`; `cargo test --manifest-path rust/microhs-runtime/Cargo.toml --lib --features minor-gc` 37/37; `cargo check --release --manifest-path rust/microhs-runtime/Cargo.toml --target wasm32-unknown-unknown --features minor-gc`; release bench build with `minor-gc` |
| bounded eager-free gates | 8M minor / 32M full: `3.176s`, 31.75M steps/s, 14 GCs, `740.739ms` GC pause, high-water 9,807,193 cells; 8M minor / 128M full: `3.281s`, 30.73M steps/s, 14 GCs, `698.827ms` GC pause, high-water 11,031,592 cells; 16M minor / 128M full: `3.425s`, 29.43M steps/s, 7 GCs, `641.138ms` GC pause, high-water 18,595,419 cells; 32M minor / 128M full: `3.587s`, 28.11M steps/s, 3 GCs, `506.720ms` GC pause, high-water 34,230,733 cells |
| feature-off overhead | `minor-gc` feature build with no minor interval and `MHS_GC_NODE_INTERVAL=33554432` completed the 100M sanity in `2.893s`, 34.84M steps/s, 3 GCs, `422.761ms` GC pause, showing codegen/accounting overhead even with minor collection disabled |
| phase-profile eager gate | `minor-gc,gc-phase-profile`, 8M minor / 128M full, completed 100M in `4.206s`, 23.97M steps/s, 14 GCs, `738.006ms` GC pause split as `178.104ms` mark and `547.298ms` sweep; eager tombstoning/free-list writes dominated the minor path |
| bounded lazy-free gates | lazy minor-free 8M minor / 128M full completed 100M in `3.189s`, 31.61M steps/s, 14 GCs, `510.464ms` GC pause, high-water 11,031,602 cells; lazy minor-free 32M minor / 128M full completed 100M in `3.951s`, 25.51M steps/s, 3 GCs, `466.006ms` GC pause |
| full gate | best variant, lazy minor-free 8M minor / 128M full, completed in `113.490s`, 3,659,075,040 steps, 32.24M steps/s, 501 GCs, `22.471s` GC pause, high-water 17,520,394 cells, current nodes 17,520,394, current free nodes 12,712,186, last live 3,728, last free 15,701,535 |
| output | `/tmp/mhs-selfhost-rust-minorgc-lazy-full.comb` and `/tmp/mhs-selfhost-rust-compound-cache-full.comb` both SHA256 `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`; `cmp` exit 0 |
| post-revert sanity | after reverting the prototype and rebuilding the default release bench, 100M completed in `2.492s`, 100,811,779 steps, 40.45M steps/s, 3 GCs, `223.100ms` GC pause, last live 467,209 nodes, high-water 33,941,663 cells |
| reading | rejected and reverted. The probe proved GC can cut high-water and pause, but it also proved this allocation-set design taxes the mutator too much: tracking every allocation, checking barriers on hot writes, scanning roots for every minor, and maintaining a secondary free-slot path loses far more than the collector saves. Do not retry this exact S4 design; a future GC attempt needs a lower-tax region/bump or otherwise different shape |

Rejected parsed primitive-cache seed probe:

| item | result |
|---|---|
| change | seeded the existing fixed `PrimCache` from parser-interned primitive nodes during `Program::new`, so runtime calls like `self.prim("K")` could reuse the parsed singleton instead of allocating a duplicate cached primitive node |
| rationale | eval.c has one permanent cell per primitive tag below `heap_start`. Rust already interns primitive names while parsing, but the runtime cache starts empty; this tested the remaining gap without replacing the hot fixed-field cache with a generic map |
| verification | `cargo fmt --manifest-path rust/microhs-runtime/Cargo.toml`; `cargo check --manifest-path rust/microhs-runtime/Cargo.toml`; `cargo check --manifest-path rust/microhs-runtime/Cargo.toml --features gc-phase-profile`; `cargo check --manifest-path rust/microhs-runtime/Cargo.toml --features eval-phase-profile`; release bench build; full output SHA256 `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`; `cmp_exit=0`; probe reverted and release bench rebuilt |
| bounded gate | 10M `294.711ms`, 10,028,866 steps, 34.03M steps/s, no GC, high-water 11,369,691 cells; 100M `2.406s`, 100,811,779 steps, 41.90M steps/s, 3 GCs, `226.987ms` GC pause, last live 467,194 nodes, high-water 33,941,648 cells |
| full gate | full self-host completed in `82.228s`, 3,659,075,078 steps, 44.50M steps/s, 125 GCs, `24.050s` GC pause, last live 1,289,555 nodes, high-water 38,820,725 cells |
| output | `/tmp/mhs-selfhost-rust-primcache-seed-full.comb` and `/tmp/mhs-selfhost.comb` both SHA256 `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`; `cmp` exit 0 |
| post-revert sanity | after restoring the empty startup `PrimCache` and rebuilding release, 100M completed in `2.413s`, 100,811,779 steps, 41.78M steps/s, 3 GCs, `226.325ms` GC pause, last live 467,213 nodes, high-water 33,941,667 cells |
| reading | rejected and reverted. The probe saved exactly the expected three live/high-water cells and improved the noisy current full sanity, but it still missed the accepted 80.927s best. The missing eval.c primitive permanence is not a standalone self-host lever at this point |

Current top-20 app/build/descent profile:

| item | result |
|---|---|
| command | `timeout 240s env MHS_GC_NODE_INTERVAL=33554432 target/release/mhs-rust-bench --input /tmp/mhs-selfhost.comb --mode main --step-limit 10000000 --warmup-iters 0 --iters 1 --profile --profile-top 20 -- ./bin/mhs -i -imhs -isrc -ilib MicroHs.Main -o/tmp/mhs-selfhost-profile-top20.comb` |
| normal feature run | `325.266ms`, 10,028,866 steps, 30.83M steps/s, no GC, high-water 11,369,676 cells |
| profiled run | `9,941.427ms`, 10,028,866 steps, no GC, app allocations 11,215,886, stack app updates 8,292,661, stack arg reads 30,119,444, stack descent pushes 30,244,488, fallback loop steps 90 |
| phase buckets | stack eval step 9,922.473ms, arg reads 179.785ms, app allocation 1,181.770ms, app updates 144.770ms, rewrites 32.800ms, force frames 6.553ms, inner descent 292.027ms; app-allocation total is inflated by the nested fresh/reused split timers |
| allocation split | no-GC profile path reused 0 cells and freshly allocated 11,215,886 cells; free-pop timer 193.644ms, fresh-push timer 325.372ms |
| top heads by step time | `B` 994.329ms, `S'` 882.571ms, `C` 830.391ms, `C'` 599.493ms, `S` 498.152ms, `C'B` 435.079ms, `P` 263.503ms |
| top app-allocation sites by timer | `B.yz` 204.440ms, `C.xz` 169.467ms, `S'.zw` 85.696ms, `S'.yw` 85.274ms, `S'.left` 84.898ms, `C'.xyw` 78.054ms, `C'.yw` 77.923ms, `S.left` 65.702ms, `S.right` 63.839ms |
| top apply-app heads | `B` 32.850ms, `C` 27.334ms, `S'` 14.484ms, `C'` 12.721ms, `S` 10.932ms, `C'B` 9.211ms, `P` 8.656ms |
| top inner-descent heads | `C` 49.631ms, `B` 46.765ms, `S'` 23.616ms, `P` 18.825ms, `C'` 16.867ms, `S` 16.667ms, `K` 15.049ms, `C'B` 13.585ms |
| reading | the next useful F2 slice is not another broad GOAP2/profile branch probe. `B` and `C` dominate but have different shapes: `B` is mostly one fresh arg app plus redex overwrite, while `C` and the S/C' families also feed fresh-app funs into descent. A candidate needs either a representation-level app construction change or a scoped B/C-family layer removal with a full gate, not a generic app-site attribution bypass |

Rejected const-specialized unprofiled stack-step retest:

| item | result |
|---|---|
| change | made `stack_eval_step`, `descend_stack_from`, and `reduce_whnf_from_stack` const-generic over profiling mode, dispatching to `<false>` for normal execution and `<true>` only when `Program::enable_profile()` was active; reverted after the full gate |
| rationale | default self-host still carries `self.profile.is_some()` profile plumbing through the hot B/C/S stack loop; this retested the old profile-vs-normal split on the current compact-cell/permanent-compound-cache machine |
| verification | probe: `cargo fmt --manifest-path rust/microhs-runtime/Cargo.toml --check`; `cargo check --manifest-path rust/microhs-runtime/Cargo.toml`; `cargo check --manifest-path rust/microhs-runtime/Cargo.toml --features eval-phase-profile`; `cargo test --manifest-path rust/microhs-runtime/Cargo.toml --lib` (37/37); `cargo build --release --manifest-path rust/microhs-runtime/Cargo.toml --bin mhs-rust-bench`; full output SHA256 matched `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`; `cmp -s /tmp/mhs-selfhost.comb /tmp/mhs-selfhost-const-unprofiled-full.comb` exited 0. Restore: const-generic diff removed; `cargo fmt --manifest-path rust/microhs-runtime/Cargo.toml --check`; `cargo check --manifest-path rust/microhs-runtime/Cargo.toml`; release bench rebuilt |
| bounded gate | 10M `283.594ms`, 10,028,866 steps, 35.36M steps/s, no GC, high-water 11,369,690 cells; 100M `2.391s`, 100,811,779 steps, 42.16M steps/s, 3 GCs, `225.126ms` GC pause, last live 467,193 nodes, high-water 33,941,647 cells |
| full gate | `84.112s`, 3,659,075,021 steps, 43.50M steps/s, 125 GCs, `24.038s` GC pause, last live 1,289,552 nodes, high-water 38,820,724 cells |
| output | `/tmp/mhs-selfhost-const-unprofiled-full.comb` and `/tmp/mhs-selfhost.comb` both SHA256 `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`; `cmp` exit 0 |
| post-revert sanity | after restoring the non-generic stack loop and rebuilding release, 100M completed in `2.456s`, 100,811,779 steps, 41.04M steps/s, 3 GCs, `223.461ms` GC pause, last live 467,195 nodes, high-water 33,941,649 cells |
| reading | rejected and reverted. The no-GC 10M signal says profile branches still perturb small slices, but the 100M/full gates repeat the older lesson: broad code-shape duplication is not a self-host lever by itself. Future F2 work needs to remove real app construction/update/descent work or change the representation, not only split profiled and unprofiled loop bodies |

Rejected GC-check fast predicate probe:

| item | result |
|---|---|
| change | replaced the hot-loop `maybe_collect_garbage_between_steps(...)` call with an inline `should_collect_garbage_between_steps()` predicate at the old-loop and stack-loop call sites, only passing the root-set arguments when a collection was due |
| rationale | the phase profile had shown a visible GC-check bucket, and eval.c keeps the no-GC hot path light; this tested whether the Rust helper call and root-set argument setup were still material |
| verification | `cargo fmt --manifest-path rust/microhs-runtime/Cargo.toml --check`; `cargo check --manifest-path rust/microhs-runtime/Cargo.toml`; `cargo test --manifest-path rust/microhs-runtime/Cargo.toml --lib` (37/37); `cargo build --release --manifest-path rust/microhs-runtime/Cargo.toml --bin mhs-rust-bench`; both full outputs SHA256-match `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`; probe reverted and release bench rebuilt |
| bounded gate | 10M `278.245ms`, 10,028,866 steps, 36.04M steps/s, no GC, high-water 11,369,680 cells; 100M `2.345s` / repeat `2.358s`, 100,811,779 steps, 42.99M / 42.76M steps/s, 3 GCs, `223.655ms` / `225.921ms` GC pause |
| full gate | first full `81.769s`, 3,659,074,926 steps, 44.75M steps/s, 125 GCs, `23.866s` GC pause, last live 1,289,543 nodes, high-water 38,820,714 cells; repeat `82.593s`, 3,659,075,059 steps, 44.30M steps/s, 125 GCs, `23.963s` GC pause, last live 1,289,558 nodes, high-water 38,820,728 cells |
| post-revert sanity | after restoring the helper-call shape and rebuilding release, 100M completed in `2.443s`, 100,811,779 steps, 41.27M steps/s, 3 GCs, `226.790ms` GC pause, last live 467,205 nodes, high-water 33,941,659 cells |
| reading | rejected and reverted. The bounded gates were strong, but full self-host did not beat the 80.927s accepted best across two byte-identical runs. The full phase mix still rules; do not accept this GC-check split without coupling it to a larger evaluator-loop change |

Rejected reserve-app allocation accounting probe:

| item | result |
|---|---|
| change | added a reserved app-allocation helper and used it in multi-app rewrite arms (`S`, `S'`, `B'`, `C'`, `C'B`, `IO.>>`, tuple prefixes) so the GC allocation counter increments once per arm instead of once per `push_app_node`, while preserving app allocation order and graph shape |
| rationale | eval.c reserves cells once per rewrite with `GCCHECK(k)` and then allocates with `new_ap`; this tested whether Rust's per-app `gc_allocations_since_collect` update was still material in the hot B/C/S app-build path |
| verification | `cargo fmt --manifest-path rust/microhs-runtime/Cargo.toml --check`; `cargo check --manifest-path rust/microhs-runtime/Cargo.toml`; `cargo test --manifest-path rust/microhs-runtime/Cargo.toml --lib` (37/37); `cargo build --release --manifest-path rust/microhs-runtime/Cargo.toml --bin mhs-rust-bench`; full output SHA256 `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`; `cmp -s` against the compound-cache output exits 0; probe reverted and release bench rebuilt |
| bounded gate | 10M `325.618ms`, 10,028,866 steps, 30.80M steps/s, no GC, high-water 11,369,692 cells; 100M `2.522s`, 100,811,779 steps, 39.98M steps/s, 3 GCs, `226.151ms` GC pause, high-water 33,941,649 cells |
| full gate | `89.330s`, 3,659,075,040 steps, 40.96M steps/s, 125 GCs, `25.613s` GC pause, last live 1,289,554 nodes, high-water 38,820,726 cells |
| post-revert sanity | after restoring the baseline helper shape and rebuilding release, 100M completed in `2.414s`, 100,811,779 steps, 41.77M steps/s, 3 GCs, `227.933ms` GC pause, last live 467,209 nodes, high-water 33,941,663 cells |
| reading | rejected and reverted. Batching the allocation counter is the wrong slice: it does not remove the expensive work identified by the phase profile and makes the full gate worse. Keep following eval.c's larger mutator shape (`GOAP`/`GOAP2`, stack descent, redex reuse) rather than accounting-only similarities |

Rejected bitmap free-map allocator probe:

| item | result |
|---|---|
| control | current tree rebuilt with `gc-phase-profile`; 100M completed in `2.342s`, 100,811,779 steps, 43.05M steps/s, 3 GCs, `223.614ms` GC pause, `76.457ms` mark, `118.550ms` sweep, high-water 33,941,639 cells |
| change | replaced the intrusive free-list pop/rebuild path with a C-style free bitmap populated at sweep; reused cells kept stale tags in release after dead cold payload cleanup, and allocation scanned free-map words with `trailing_zeros` |
| refinement | added a one-word allocation cache so each free-map word is loaded once and then consumed locally for up to 64 reused allocations |
| verification | `cargo fmt --manifest-path rust/microhs-runtime/Cargo.toml --check`; `cargo check --manifest-path rust/microhs-runtime/Cargo.toml`; `cargo test --manifest-path rust/microhs-runtime/Cargo.toml --lib` (37/37); `MHS_GC_NODE_INTERVAL=32 cargo test --manifest-path rust/microhs-runtime/Cargo.toml --lib` (37/37); release builds with and without `gc-phase-profile`; probe reverted and release bench rebuilt |
| bounded gate | initial bitmap 10M `315.636ms`, 10,028,866 steps, 31.77M steps/s, no GC, high-water 11,369,690 cells; initial bitmap 100M `2.633s`, 38.28M steps/s, 3 GCs, `217.439ms` GC pause, `77.116ms` mark, `112.056ms` sweep |
| word-cache gate | 100M `2.597s`, 38.83M steps/s, 3 GCs, `217.195ms` GC pause, `77.268ms` mark, `111.804ms` sweep |
| default gate | default release 100M `2.610s`, 38.63M steps/s, 3 GCs, `218.904ms` GC pause |
| post-revert sanity | after restoring the intrusive free-list allocator and rebuilding release, 100M completed in `2.369s`, 42.55M steps/s, 3 GCs, `221.099ms` GC pause, high-water 33,941,661 cells |
| reading | rejected and reverted before full self-host. The profile confirmed sweep is material, but this C-inspired bitmap allocator was the wrong transplant: it saved only about 6-7ms of 100M GC pause while losing about 240ms in the default bounded mutator. A future sweep attack needs to remove sweep writes without adding per-allocation bitmap scan cost, or change both allocator and cell/free representation together |

Rejected low-to-high free-list reuse order probe:

| item | result |
|---|---|
| change | changed sweep to rebuild the intrusive free list by iterating dead cells high-to-low, so the prepended free-list head reuses the lowest free slot first |
| rationale | Rust's sweep had been iterating low-to-high and prepending each free cell, which made allocation reuse high addresses downward after GC. C's `free_map` scanner restarts at `heap_start` and reuses low addresses upward; this tested the reuse-order half without the bitmap allocator's per-allocation scan cost |
| verification | `cargo fmt --manifest-path rust/microhs-runtime/Cargo.toml --check`; `cargo check --manifest-path rust/microhs-runtime/Cargo.toml`; `cargo test --manifest-path rust/microhs-runtime/Cargo.toml --lib` (37/37); `MHS_GC_NODE_INTERVAL=32 cargo test --manifest-path rust/microhs-runtime/Cargo.toml --lib` (37/37); `cargo build --release --manifest-path rust/microhs-runtime/Cargo.toml --bin mhs-rust-bench`; full output SHA256 `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`; `cmp -s` against `/tmp/mhs-selfhost-rust-compound-cache-full.comb` exits 0; probe reverted and release bench rebuilt |
| bounded gate | 10M `292.599ms`, 10,028,866 steps, 34.28M steps/s, no GC, high-water 11,369,684 cells; 100M `2.429s`, 100,811,779 steps, 41.50M steps/s, 3 GCs, `213.628ms` GC pause, last live 467,187 nodes, high-water 33,941,641 cells |
| bounded repeat | 100M `2.415s`, 100,811,779 steps, 41.74M steps/s, 3 GCs, `214.200ms` GC pause, last live 467,201 nodes, high-water 33,941,655 cells |
| full gate | first full completed in `81.622s`, 3,659,074,964 steps, 44.83M steps/s, 125 GCs, `23.213s` GC pause, last live 1,289,547 nodes, high-water 38,820,718 cells, current free 26,177,414 cells |
| full repeat | repeat full completed in `81.277s`, 3,659,075,097 steps, 45.02M steps/s, 125 GCs, `23.120s` GC pause, last live 1,289,561 nodes, high-water 38,820,732 cells, current free 26,177,177 cells |
| post-revert sanity | after restoring the baseline low-to-high sweep iteration and rebuilding release, 100M completed in `2.415s`, 100,811,779 steps, 41.74M steps/s, 3 GCs, `225.493ms` GC pause, last live 467,211 nodes, high-water 33,941,665 cells |
| reading | rejected and reverted. Reusing low addresses first like eval.c improved GC pause and the 10M bounded slice, but two byte-identical full gates still missed the 80.927s compound-cache best. Allocation order alone is not enough; future allocator work needs a larger graph-shape or mutator-path change, not just reuse order |

Rejected FFI/JS arity-prefix materialization retest:

| item | result |
|---|---|
| change | FFI/JS helper dispatches materialized only the demanded head-argument prefix into `scratch_args`: `ffi_arity(name) + world` for known FFI calls, `tags.len()` for JS calls, and two args for JS wrappers. The old full-spine materialization helpers were restored after the rejection |
| rationale | the 10M `eval-phase-profile` refresh showed fallback eval-loop steps were only 90, but FFI/JS/fallback materialization still copied 10,522,021 node ids from 49,970 materializations. This tested whether the old long-spine FFI/BFILE symptom was still a material self-host cost |
| verification | probe: `cargo fmt --manifest-path rust/microhs-runtime/Cargo.toml --check`; `cargo check --manifest-path rust/microhs-runtime/Cargo.toml`; `cargo test --manifest-path rust/microhs-runtime/Cargo.toml --lib` (37/37); release builds with and without `eval-phase-profile`; full output SHA256 matched `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`; `cmp -s /tmp/mhs-selfhost.comb /tmp/mhs-selfhost-ffi-prefix-full-current.comb` exited 0. Restore: runtime diff returned clean; `cargo fmt --manifest-path rust/microhs-runtime/Cargo.toml --check`; `cargo check --manifest-path rust/microhs-runtime/Cargo.toml`; `cargo test --manifest-path rust/microhs-runtime/Cargo.toml --lib` (37/37); release bench rebuilt |
| same-session controls | 10M `283.818ms`, 10,028,866 steps, 35.34M steps/s, no GC, high-water 11,369,672 cells; 100M `2.451s`, 100,811,779 steps, 41.13M steps/s, 3 GCs, `224.693ms` GC pause, high-water 33,941,629 cells |
| bounded gate | 10M `275.978ms`, 10,028,866 steps, 36.34M steps/s, no GC, high-water 11,369,678 cells; 100M `2.444s`, 100,811,779 steps, 41.25M steps/s, 3 GCs, `227.102ms` GC pause, last live 467,181 nodes, high-water 33,941,635 cells |
| profile delta | `profile_arg_materializations` stayed at 49,970, while `profile_arg_materialized_nodes` fell from 10,522,021 to 150,988; profiled total was noisy/slower at `10,106.562ms` versus `9,730.435ms`, and the main buckets stayed app allocation, inner descent, arg reads, and app updates |
| full gate | `timeout 900s env MHS_GC_NODE_INTERVAL=33554432 target/release/mhs-rust-bench --input /tmp/mhs-selfhost.comb --mode main --warmup-iters 0 --iters 1 -- ./bin/mhs -i -imhs -isrc -ilib MicroHs.Main -o/tmp/mhs-selfhost-ffi-prefix-full-current.comb` completed in `83.510s`, 3,659,075,059 steps, 43.82M steps/s, 125 GCs, `23.937s` GC pause, last live 1,289,558 nodes, high-water 38,820,728 cells |
| output | `/tmp/mhs-selfhost-ffi-prefix-full-current.comb` and `/tmp/mhs-selfhost.comb` both SHA256 `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`; `cmp` exit 0 |
| post-revert sanity | after restoring full materialization and rebuilding release, 100M completed in `2.641s`, 100,811,779 steps, 38.17M steps/s, 3 GCs, `228.836ms` GC pause, last live 467,205 nodes, high-water 33,941,659 cells |
| reading | rejected and reverted. The profile counter moved exactly as intended, and the short 10M/100M gates were neutral-to-slightly-better, but the full gate still missed the accepted 80.927s compound-cache best. This confirms the huge materialized-node counter is mostly cold/FFI continuation bookkeeping now; the remaining F2 cost is still the B/C/S app-build/update/descent path rather than FFI/JS argument copying |

Rejected eval.c `GOAP2` direct-push retest:

| item | result |
|---|---|
| change | added a hot stack-loop `app2_taken` continuation for GOAP2-shaped rewrites: update the consumed redex to point at the freshly allocated inner app, push both the outer redex app and inner app directly, then continue from the inner fun. Applied only to arms that already built a fresh inner app as the new outer fun (`S`, `S'`, `B'`, `C`, `C'`, `P`, `R`, `O`, `C'B`, `Tag`, and IO graph rewrites), preserving the current per-arm allocation order |
| rationale | eval.c's `GOAP2` goes through `ap2`, pushing the outer app and inner app without re-running a generic app descent over the inner app. The current profile still has a large inner-descent bucket, and the older direct-inner-`GOAP2` probe was from a much earlier machine, before compact cells, one-word app-fun descent, trusted stack updates, app-edge canonicalization, and permanent compound roots |
| verification | probe: `cargo fmt --manifest-path rust/microhs-runtime/Cargo.toml --check`; `cargo check --manifest-path rust/microhs-runtime/Cargo.toml`; `cargo test --manifest-path rust/microhs-runtime/Cargo.toml --lib` (37/37); `cargo build --release --manifest-path rust/microhs-runtime/Cargo.toml --bin mhs-rust-bench`; `git diff --check`; full output SHA256 matched `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`; `cmp -s /tmp/mhs-selfhost.comb /tmp/mhs-selfhost-goap2-direct-full.comb` exited 0. Restore: runtime diff returned clean; `cargo fmt --manifest-path rust/microhs-runtime/Cargo.toml --check`; `cargo check --manifest-path rust/microhs-runtime/Cargo.toml`; `cargo test --manifest-path rust/microhs-runtime/Cargo.toml --lib` (37/37); release bench rebuilt |
| bounded gate | 10M `319.832ms`, 10,028,866 steps, 31.36M steps/s, no GC, high-water 11,369,682 cells; 100M `2.369s`, 100,811,779 steps, 42.55M steps/s, 3 GCs, `225.702ms` GC pause, last live 467,185 nodes, high-water 33,941,639 cells |
| full gate | `timeout 900s env MHS_GC_NODE_INTERVAL=33554432 target/release/mhs-rust-bench --input /tmp/mhs-selfhost.comb --mode main --warmup-iters 0 --iters 1 -- ./bin/mhs -i -imhs -isrc -ilib MicroHs.Main -o/tmp/mhs-selfhost-goap2-direct-full.comb` completed in `86.784s`, 3,659,074,945 steps, 42.16M steps/s, 125 GCs, `25.596s` GC pause, last live 1,289,545 nodes, high-water 38,820,716 cells |
| output | `/tmp/mhs-selfhost-goap2-direct-full.comb` and `/tmp/mhs-selfhost.comb` both SHA256 `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`; `cmp` exit 0 |
| post-revert sanity | after restoring the accepted continuation and rebuilding release, 100M completed in `2.357s`, 100,811,779 steps, 42.77M steps/s, 3 GCs, `229.471ms` GC pause, last live 467,195 nodes, high-water 33,941,649 cells |
| reading | rejected and reverted. This is the second broad direct-inner-`GOAP2` attempt to win bounded slices but lose the full gate. The likely issue is code shape and phase mix rather than semantics: the graph and step count stay stable, but full GC pause rises and the full mutator loses. Do not retry this broad direct-push shape alone; any future `GOAP2` work needs to be coupled to a larger stack/descent representation change or scoped by profiling to a specific head family |

eval.c `GOPAIR` runtime-return shape probe:

| item | result |
|---|---|
| change | runtime/FFI helper returns were changed from `(used, node)` to a `Node`/`Pair`/`UnitPair` enum, and the stack evaluator used `Pair`/`UnitPair` to overwrite the consumed redex as the outer `(P result) world` app instead of allocating that outer app and making the redex an indirection to it |
| rationale | eval.c's `GOPAIR` and `GOPAIRUNIT` keep the current IO/runtime redex as the outer pair app. This tested whether the remaining cold helper return path was still paying a meaningful allocation/update tax after the accepted compound-cache slice |
| verification | `cargo fmt --manifest-path rust/microhs-runtime/Cargo.toml --check`; `cargo check --manifest-path rust/microhs-runtime/Cargo.toml`; `cargo build --release --manifest-path rust/microhs-runtime/Cargo.toml --bin mhs-rust-bench`; `cmp -s /tmp/mhs-selfhost.comb /tmp/mhs-selfhost-rust-gopair-cold-full.comb`; SHA256 matches `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`; probe reverted, release bench rebuilt, and post-revert 100M sanity passed |
| unoutlined bounded gate | 10M `331.645ms`, 10,028,866 steps, 30.24M steps/s, no GC; 100M `2.772s`, 100,811,779 steps, 36.37M steps/s, 3 GCs, 235.236ms GC pause, high-water 33,941,396 cells |
| cold/noinline bounded gate | 10M `330.161ms`, 10,028,866 steps, 30.38M steps/s, no GC; 100M `2.367s`, 100,811,779 steps, 42.60M steps/s, 3 GCs, 227.171ms GC pause, high-water 33,941,406 cells |
| full gate | `timeout 900s env MHS_GC_NODE_INTERVAL=33554432 target/release/mhs-rust-bench --input /tmp/mhs-selfhost.comb --mode main --warmup-iters 0 --iters 1 -- ./bin/mhs -i -imhs -isrc -ilib MicroHs.Main -o/tmp/mhs-selfhost-rust-gopair-cold-full.comb` completed in `82.600s`, 3,659,075,021 steps, 44.30M steps/s, 124 GCs, 23.502s GC pause, last live 1,442,142 nodes, high-water 39,491,643 cells |
| output | `/tmp/mhs-selfhost-rust-gopair-cold-full.comb` and `/tmp/mhs-selfhost.comb` both SHA256 `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`; `cmp` exit 0 |
| post-revert sanity | after reverting the enum/direct-pair return shape and rebuilding release, 100M ran in `2.462s`, 100,811,779 steps, 40.95M steps/s, 3 GCs, 223.856ms GC pause, last live 467,207 nodes, high-water 33,941,661 cells |
| reading | rejected and reverted. Outlining the cold enum path made the 100M slice competitive, but the full gate lost 80.927s -> 82.600s and retained a larger graph. The remaining runtime-helper return path is not broad enough to justify adding a new return-shape enum in the hot evaluator module; future `GOPAIR` work should target known IO cases directly or be coupled to a larger continuation/descent change |

Focused stack app-site profile split:

| item | result |
|---|---|
| change | In the stack evaluator only, the `app_site!` macro used the already-computed `profiling` flag to call `push_app_node` directly when profiling was disabled, falling back to `app_with_site` only for profile runs |
| rationale | this tested whether the hottest app allocations were paying a meaningful Rust-only cost for profile-site branch/bookkeeping even in default non-profile runs. It deliberately avoided the earlier rejected broad eval-profile feature gate and left generic `app()` plus the cold persistent path unchanged |
| verification | `cargo fmt --manifest-path rust/microhs-runtime/Cargo.toml --check`; `cargo build --release --manifest-path rust/microhs-runtime/Cargo.toml --bin mhs-rust-bench`; `cmp -s /tmp/mhs-selfhost.comb /tmp/mhs-selfhost-rust-appsite-split-full.comb`; SHA256 matches `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`; probe reverted and release bench rebuilt |
| bounded gate | 10M `340.971ms`, 10,028,866 steps, 29.41M steps/s, no GC; 100M `2.422s`, 100,811,779 steps, 41.62M steps/s, 3 GCs, 226.159ms GC pause, last live 467,197 nodes, high-water 33,941,651 cells |
| full gate | `timeout 900s env MHS_GC_NODE_INTERVAL=33554432 target/release/mhs-rust-bench --input /tmp/mhs-selfhost.comb --mode main --warmup-iters 0 --iters 1 -- ./bin/mhs -i -imhs -isrc -ilib MicroHs.Main -o/tmp/mhs-selfhost-rust-appsite-split-full.comb` completed in `85.721s`, 3,659,075,059 steps, 42.69M steps/s, 125 GCs, 25.503s GC pause, last live 1,289,558 nodes, high-water 38,820,728 cells |
| output | `/tmp/mhs-selfhost-rust-appsite-split-full.comb` and `/tmp/mhs-selfhost.comb` both SHA256 `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`; `cmp` exit 0 |
| post-revert sanity | after reverting the macro split and rebuilding release, 100M ran in `2.420s`, 100,811,779 steps, 41.66M steps/s, 3 GCs, 220.124ms GC pause, last live 467,209 nodes, high-water 33,941,663 cells |
| reading | rejected and reverted. The full gate regressed versus the 82.154s trusted-stack best even though the output was byte-identical, so the hot app-allocation bucket is not mostly the `app_with_site` profile branch. Next app-allocation work needs to remove real allocation/update/descent structure, not just bypass attribution machinery |

Current-tree F2/GC refresh and direct known-head dispatch probe:

| item | result |
|---|---|
| refresh controls | current default 32M 100M control ran in `2.461s`, 100,811,779 steps, 40.97M steps/s, 3 GCs, `221.015ms` GC pause, high-water 33,941,639 cells. A 64M GC interval 100M control regressed to `2.978s`, 33.85M steps/s, 1 GC, `162.254ms` GC pause, high-water 67,262,126 cells, so the wider window is not a good default for this heap shape |
| phase-profile refresh | with `eval-phase-profile`, 10M normal pass took `377.119ms`; profiled pass took `9783.203ms`, with 10,028,866 steps, 11,215,920 app allocations, 8,292,661 stack app updates, 30,244,488 stack descent pushes, 30,119,444 stack arg reads, `stack_step_force=0`, `stack_step_whnf=45,574`, `stack_step_fallback=90`, and no strict-redex snapshots or remaining-spine scans |
| current top buckets | `stack_eval_step_ms=9760.296`; app allocation `384.012ms`, inner descent `330.130ms`, arg reads `191.086ms`, app updates `154.700ms`, rewrites `35.021ms`, force-frame pushes `6.697ms`. Top heads remain `B` 914.701ms, `C` 769.561ms, `S'` 748.069ms, `C'` 526.869ms, `S` 433.282ms, `C'B` 382.770ms. Top app sites remain `B.yz`, `C.xz`, `S'.zw`/`S'.yw`/`S'.left`, `C'.yw`/`C'.xyw`, `S.left`/`S.right`, and `C'B` |
| probe | changed only the stack evaluator's dispatch shape so `Prim::Known` went directly to the known-combinator match, with runtime/FFI/JS/WHNF handling outside the common `EvalHead` enum path |
| probe verification | `cargo fmt --manifest-path rust/microhs-runtime/Cargo.toml --check`; `cargo check --manifest-path rust/microhs-runtime/Cargo.toml`; `cargo build --release --manifest-path rust/microhs-runtime/Cargo.toml --bin mhs-rust-bench`; `cmp -s /tmp/mhs-selfhost.comb /tmp/mhs-selfhost-rust-direct-dispatch-full.comb`; SHA256 matches `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`; probe reverted and release bench rebuilt |
| probe bounded gate | 10M `325.601ms`, 10,028,866 steps, 30.80M steps/s, no GC, high-water 11,369,698 cells; 100M `2.396s`, 100,811,779 steps, 42.08M steps/s, 3 GCs, `224.833ms` GC pause, high-water 33,941,655 cells |
| probe full gate | `timeout 900s env MHS_GC_NODE_INTERVAL=33554432 target/release/mhs-rust-bench --input /tmp/mhs-selfhost.comb --mode main --warmup-iters 0 --iters 1 -- ./bin/mhs -i -imhs -isrc -ilib MicroHs.Main -o/tmp/mhs-selfhost-rust-direct-dispatch-full.comb` completed in `85.951s`, 3,659,075,097 steps, 42.57M steps/s, 125 GCs, `25.135s` GC pause, last live 1,289,561 nodes, high-water 38,820,732 cells |
| output | `/tmp/mhs-selfhost-rust-direct-dispatch-full.comb` and `/tmp/mhs-selfhost.comb` both SHA256 `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`; `cmp` exit 0 |
| post-revert sanity | after reverting the direct dispatch shape and rebuilding release, 100M ran in a noisy `2.609s`, 100,811,779 steps, 38.64M steps/s, 3 GCs, `249.729ms` GC pause, last live 467,211 nodes, high-water 33,941,665 cells |
| reading | rejected and reverted. The bounded 100M win did not survive the full self-host arbiter: the full gate lost 80.927s -> 85.951s and raised GC pause. The common `EvalHead` enum shape is not the remaining F2/S1 lever; the remaining work is still the broader app allocation/update/descent mechanics in the B/C/S family |

eval.c `GOAP2` allocation-order probe:

| item | result |
|---|---|
| change | In the stack `take_args!` hot arms for `S'`, `B'`, and `C'B`, reordered only the independent inner app allocations to match eval.c's textual `GOAP2` order more closely: `S'` allocated `yw`, outer-left, then `zw`; `B'` allocated `xy` before `zw`; `C'B` allocated `xz` before `yw` |
| rationale | eval.c's rewrite macros are the current oracle. This tested whether the full-gate gap was partly heap locality or GC-shape fallout from allocating independent payload apps in a different order, without changing reduction count or graph semantics |
| verification | probe build passed `cargo fmt --manifest-path rust/microhs-runtime/Cargo.toml --check`, `cargo check --manifest-path rust/microhs-runtime/Cargo.toml`, and release bench build; the full output matched `/tmp/mhs-selfhost.comb` byte-for-byte. After rejection, only the three hot-arm order changes were reverted, and fmt/check/release build passed again |
| bounded gate | 10M `385.102ms`, 10,028,866 steps, 26.04M steps/s, no GC, high-water 11,369,690 cells; 100M `2.428s`, 100,811,779 steps, 41.51M steps/s, 3 GCs, `226.885ms` GC pause, high-water 33,941,647 cells |
| full gate | `timeout 900s env MHS_GC_NODE_INTERVAL=33554432 target/release/mhs-rust-bench --input /tmp/mhs-selfhost.comb --mode main --warmup-iters 0 --iters 1 -- ./bin/mhs -i -imhs -isrc -ilib MicroHs.Main -o/tmp/mhs-selfhost-rust-goap2-order-full.comb` completed in `82.827s`, 3,659,075,021 steps, 44.18M steps/s, 125 GCs, `24.186s` GC pause, last live 1,289,552 nodes, high-water 38,820,724 cells |
| output | `/tmp/mhs-selfhost-rust-goap2-order-full.comb` and `/tmp/mhs-selfhost.comb` both SHA256 `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`; `cmp` exit 0 |
| post-revert sanity | after reverting the allocation-order probe and rebuilding release, 100M ran in `2.454s`, 100,811,779 steps, 41.07M steps/s, 3 GCs, `236.630ms` GC pause, last live 467,205 nodes, high-water 33,941,659 cells |
| reading | rejected and reverted. The probe preserved semantics and essentially the same heap high-water mark, but full self-host lost 80.927s -> 82.827s and raised GC pause. Matching eval.c's allocation order for these independent inner apps is not enough; the next F2 change needs to remove broader app allocation/update/descent work rather than reshuffle the same allocations |

eval.c low-bit app tag encoding probe:

| item | result |
|---|---|
| change | Changed the cell tag encoding so app cells used low bit `0` in `word0` and stored the fun id with a one-bit shift, while non-app cells used low bit `1` plus the logical tag. `app_fun` then tested one bit instead of the previous byte tag and extracted the fun with `>> 1` instead of `>> 8` |
| rationale | eval.c's hot app walk distinguishes applications by low tag bits on the first word. This tested whether the Rust cell's byte-tag encoding was still costing the descent/app-field buckets after the accepted one-word app-fun slice |
| correctness | the first attempt compiled but failed 31/37 lib tests because `Cell::prim` still matched old hard-coded primitive tag numbers. After switching primitive decoding to the new `CellTag` encoding, `cargo fmt --manifest-path rust/microhs-runtime/Cargo.toml --check`, `cargo check --manifest-path rust/microhs-runtime/Cargo.toml`, and `cargo test --manifest-path rust/microhs-runtime/Cargo.toml --lib` passed 37/37; release bench build also passed |
| bounded gate | 10M `396.531ms`, 10,028,866 steps, 25.29M steps/s, no GC, high-water 11,369,688 cells; 100M `2.498s`, 100,811,779 steps, 40.35M steps/s, 3 GCs, `233.133ms` GC pause, high-water 33,941,645 cells |
| full gate | `timeout 900s env MHS_GC_NODE_INTERVAL=33554432 target/release/mhs-rust-bench --input /tmp/mhs-selfhost.comb --mode main --warmup-iters 0 --iters 1 -- ./bin/mhs -i -imhs -isrc -ilib MicroHs.Main -o/tmp/mhs-selfhost-rust-lowbit-tag-full.comb` completed in `92.637s`, 3,659,075,002 steps, 39.50M steps/s, 125 GCs, `26.366s` GC pause, last live 1,289,550 nodes, high-water 38,820,722 cells |
| output | `/tmp/mhs-selfhost-rust-lowbit-tag-full.comb` and `/tmp/mhs-selfhost.comb` both SHA256 `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`; `cmp` exit 0 |
| post-revert sanity | after restoring the accepted byte-tag encoding and rebuilding release, 100M ran in `2.397s`, 100,811,779 steps, 42.06M steps/s, 3 GCs, `225.599ms` GC pause, last live 467,207 nodes, high-water 33,941,661 cells |
| reading | rejected and reverted. The low-bit scheme is closer to eval.c superficially, but it is not the same as eval.c's pointer-tag representation: Rust still carries integer `NodeId`s, and this encoding worsened app descent/codegen enough to lose both bounded and full gates. Do not retry tag-word encoding in isolation; future representation work needs to remove a real surrounding layer, not just change the bit packing |

Manual `Vec<u64>` GC mark bitmap probe:

| item | result |
|---|---|
| change | Replaced the collector's packed `Vec<bool>` mark map with a manual `MarkBits { words: Vec<u64>, len }`, using direct word/mask set and test operations while keeping the existing intrusive free-list sweep and foreign-finalizer mark vector unchanged |
| rationale | eval.c's collector is explicitly bitmap-based, and the current best full gate still spends `23.793s` in GC versus C's `11.64s`. This tested whether Rust's `Vec<bool>` proxy access was a material standalone mark/sweep cost |
| verification | `cargo fmt --manifest-path rust/microhs-runtime/Cargo.toml`; `cargo check --manifest-path rust/microhs-runtime/Cargo.toml`; `cargo test --manifest-path rust/microhs-runtime/Cargo.toml --lib` passed 37/37; release bench build passed; full output matched `/tmp/mhs-selfhost.comb` byte-for-byte |
| bounded gate | 100M `2.375s`, 100,811,779 steps, 42.44M steps/s, 3 GCs, `222.774ms` GC pause, last live 467,187 nodes, high-water 33,941,641 cells |
| full gate | `timeout 900s env MHS_GC_NODE_INTERVAL=33554432 target/release/mhs-rust-bench --input /tmp/mhs-selfhost.comb --mode main --warmup-iters 0 --iters 1 -- ./bin/mhs -i -imhs -isrc -ilib MicroHs.Main -o/tmp/mhs-selfhost-rust-markbits-full.comb` completed in `84.578s`, 3,659,074,964 steps, 43.26M steps/s, 125 GCs, `25.491s` GC pause, last live 1,289,547 nodes, high-water 38,820,718 cells |
| output | `/tmp/mhs-selfhost-rust-markbits-full.comb` and `/tmp/mhs-selfhost.comb` both SHA256 `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`; `cmp` exit 0 |
| post-revert sanity | after restoring the accepted `Vec<bool>` mark map and rebuilding release, 100M ran in `2.412s`, 100,811,779 steps, 41.80M steps/s, 3 GCs, `225.890ms` GC pause, last live 467,211 nodes, high-water 33,941,665 cells |
| reading | rejected and reverted. The 100M pause moved in the expected direction, but the full gate lost 80.927s -> 84.578s and raised total GC pause. The remaining GC gap is not simply `Vec<bool>` proxy overhead; future GC work needs to change sweep/free-list shape or graph reduction, not only the mark-bit container |

S0 post-`ret`/`top` profile refresh:

| item | result |
|---|---|
| default-profile 10M | normal pass `362.949ms`, 10,028,866 steps, 27.63M steps/s, no GC; profiled pass `2279.382ms` with the same counter shape as ret/top: 11,215,864 app allocations, 8,292,661 stack app updates, 30,244,488 stack descent pushes, 30,119,444 stack arg reads, 0 strict-redex snapshots, 0 remaining-app scans, 90 fallback eval-loop steps |
| phase-profile 10M | feature-build normal pass `350.898ms`; profiled pass `9124.624ms`; `stack_step_force=0`, `stack_eval_step_ms=9044.713`, app allocation `342.872ms`, inner descent `280.568ms`, arg reads `179.659ms`, app updates `144.796ms`, rewrites `32.631ms`, force-frame pushes `6.495ms` |
| top phase buckets | stack head time is still ordinary combinator traffic: `B` 856.666ms, `C` 720.303ms, `S'` 700.005ms, `C'` 489.466ms, `S` 409.087ms, `C'B` 356.196ms; app allocation sites are still `B.yz`, `C.xz`, `S'.zw`/`S'.yw`/`S'.left`, `C'.yw`/`C'.xyw`, `S.left`/`S.right`, and `C'B` payloads |
| reading | `ret`/`top` really removed the `StackStep::Force` handoff, but the mutator ranking did not change: the next S1 work still has to remove app allocation/update/descent work from the B/C/S family, not chase cold fallback helpers |

S0 post-F14 per-head/per-site profile refresh:

| item | result |
|---|---|
| change | `eval-phase-profile` now attributes stack arg-read, app-update, rewrite, force-frame, inner-descent, and app-allocation-site timings to the current head/site; default builds do not carry these maps |
| verification | `cargo fmt --manifest-path rust/microhs-runtime/Cargo.toml --check`; `cargo check --manifest-path rust/microhs-runtime/Cargo.toml`; `cargo check --manifest-path rust/microhs-runtime/Cargo.toml --features eval-phase-profile`; release profile bench rebuilt |
| 10M control | feature-build non-profile control `348.009ms`, 10,028,866 steps, 28.82M steps/s, no GC, high-water 11,369,678 cells |
| 10M profile | `9319.195ms`, 10,028,866 steps, node growth 11,216,493, app allocations 11,215,888, no GC |
| global stack buckets | `stack_eval_step=8962.590ms`; app allocation `349.631ms`, inner descent `241.949ms`, arg reads `181.886ms`, app updates `146.164ms`, rewrites `33.155ms`, force-frame pushes `7.729ms` |
| top heads | overall stack time: `B` 863.341ms, `C` 726.565ms, `S'` 720.488ms, `C'` 499.122ms, `S` 409.661ms, `C'B` 365.538ms |
| top app sites | allocation timing: `B.yz` 59.869ms, `C.xz` 49.490ms, `S'.zw` 27.144ms, `S'.left` 25.314ms, `S'.yw` 24.960ms, `C'.yw` 23.667ms, `C'.xyw` 22.953ms |
| default full rerun | after rebuilding without `eval-phase-profile`, the 900s self-host gate completed byte-identically in `99.607s`, 3,659,074,945 steps, 36.74M steps/s, 125 GCs, `28.426s` GC pause, high-water 40,006,656 cells; this is a noisy verification rerun, not a new baseline |
| reading | the finer profile does not reveal a hidden cold leak: app allocation, arg reads, app updates, and inner descent are all dominated by the same `B`/`C`/`S`/`C'`/`C'B` combinator traffic. Remaining work needs to remove a broader Rust-side layer around those eval.c-shaped rewrites, not specialize one more allocation site |

F12 catch masking-state restoration:

| item | result |
|---|---|
| change | `catch_result` now captures the old masking state before running the action; on exception it sets the current mask to `mask_interruptible` (`1`) for the handler and returns the eval.c-shaped graph `IO.>>= (handler exn) (B' IO.>> (IO.setmaskingstate old) IO.return) world` |
| rationale | eval.c's `catchr` does not directly run the handler result; it rebuilds the handler/restoration continuation in graph so the handler observes interruptible masking and the saved mask is restored afterward |
| verification | `cargo fmt --manifest-path rust/microhs-runtime/Cargo.toml --check`; `cargo check --manifest-path rust/microhs-runtime/Cargo.toml`; `cargo test --manifest-path rust/microhs-runtime/Cargo.toml --lib` (37/37); `cargo check --manifest-path rust/microhs-runtime/Cargo.toml --features eval-phase-profile`; `cargo check --manifest-path rust/microhs-runtime/Cargo.toml --features gc-phase-profile`; `cargo check --release --manifest-path rust/microhs-runtime/Cargo.toml --target wasm32-unknown-unknown`; release `mhs-rust` and `mhs-rust-bench` rebuilt |
| focused smokes | `target/release/mhs-rust` renders `1` for `IO.performIO (catch (raise #7) (K IO.getmaskingstate))`; after `IO.setmaskingstate #2`, `IO.>> (catch (raise #7) (K (IO.return #42))) IO.getmaskingstate` renders `2` |
| full gate | `MHS_GC_NODE_INTERVAL=33554432 timeout 900s target/release/mhs-rust-bench --input /tmp/mhs-selfhost.comb --mode main --warmup-iters 0 --iters 1 -- ./bin/mhs -i -imhs -isrc -ilib MicroHs.Main -o/tmp/mhs-selfhost-rust-f12-mask.comb` completed in `98.076s`, 3,659,074,869 steps, 37.31M steps/s, 125 GCs, `27.937s` GC pause, high-water 40,006,648 cells |
| output | `/tmp/mhs-selfhost.comb` and `/tmp/mhs-selfhost-rust-f12-mask.comb` both SHA256 `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`; `cmp -s` exit 0 |
| reading | accepted as cold correctness parity. The full gate is byte-identical but noisy/slower than the 90.237s F14 best, so it does not reset the performance baseline |

F17 cold `BytesView` slice:

| item | result |
|---|---|
| change | added a cold `Node::BytesView { base, offset, len }` payload for immutable bytestring slices; `bssubstr` and `tailUTF8` now return views over immutable `Bytes`, flatten nested views, and still copy mutable-byte slices; `bytes()`, pointer reads, rendering, and serialization read the visible slice; writes through a view materialize it back to an ordinary `Bytes` node before mutation |
| rationale | this matches eval.c's O(1) `addForPtr` view idea for the direct `bssubstr`/`tailUTF8` path without changing every `Node::Bytes` allocation. A first broad attempt changed all `Bytes` payloads to shared `Arc`/`Rc` buffers and regressed 100M self-host slices to 2.67-3.13s, so the accepted shape keeps the existing `Box<Vec<u8>>` representation and adds a separate cold view node only when slicing |
| verification | `cargo fmt --manifest-path rust/microhs-runtime/Cargo.toml --check`; `cargo test --manifest-path rust/microhs-runtime/Cargo.toml --lib` (37/37); `cargo check --manifest-path rust/microhs-runtime/Cargo.toml --features eval-phase-profile`; `cargo check --release --manifest-path rust/microhs-runtime/Cargo.toml --target wasm32-unknown-unknown`; release bench rebuilt; `git diff --check`; full gates byte-identical |
| bounded gate | final-source 1M average over 3: `33.043ms`, 1,001,383 steps/iter, 30.31M steps/s, no GC; final-source 100M `2.571s`, 100,811,774 steps, 39.21M steps/s, 3 GCs, 239.022ms GC pause; earlier same-slice 100M runs were 2.562s and 2.692s |
| byte rows | `bytes-chain:10000` final-source `57.626ms`/iter, 173.5k steps/s; `foreignptr-slice:100000` final-source `371.191us`/iter, 2.7k steps/s. These are sanity rows only; the new direct F17 path is `bssubstr`/`tailUTF8`, not the `fp+` fallback |
| full gate | first full run `93.574s`, 3,659,074,864 steps, 39.10M steps/s, 125 GCs, 26.857s GC pause, high-water 40,006,648 cells; repeat `100.859s`, 3,659,074,997 steps, 36.28M steps/s, 125 GCs, 28.375s GC pause, high-water 40,006,662 cells |
| output | `/tmp/mhs-selfhost.comb`, `/tmp/mhs-selfhost-rust-f17-view.comb`, and `/tmp/mhs-selfhost-rust-f17-view-repeat.comb` all SHA256 `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`; `cmp -s` exit 0 for both full outputs |
| reading | keep as an F17 parity slice, not as a self-host performance win. Bounded self-host stays in the accepted band, but the full gate does not beat the latest F10 gate or the raw-tag best. Do not retry the broad shared-buffer `Bytes` representation without a bigger cold-payload/layout reason |

S3 parse-time small-int interning retest:

| item | result |
|---|---|
| change | parser shares parsed `Node::Int` literals in `-10..255` through a per-parse table; `##` `Int64` literals remain fresh `Node::Int64` cells |
| rationale | eval.c has a permanent small-int table; the old pre-cell rejection no longer applies after the 16-byte cell arena and in-cell primitive tags changed the representation |
| verification | `cargo fmt --manifest-path rust/microhs-runtime/Cargo.toml --check`; `cargo check --manifest-path rust/microhs-runtime/Cargo.toml`; `cargo test --manifest-path rust/microhs-runtime/Cargo.toml --lib` (37/37); `cargo check --manifest-path rust/microhs-runtime/Cargo.toml --features eval-phase-profile`; `cargo check --manifest-path rust/microhs-runtime/Cargo.toml --features gc-phase-profile`; `cargo check --release --manifest-path rust/microhs-runtime/Cargo.toml --target wasm32-unknown-unknown`; release bench rebuilt; `git diff --check`; full gates byte-identical |
| bounded gate | 1M `37.045ms`, 1,001,383 steps, 27.03M steps/s, no GC; 100M `2.614s`, 100,811,774 steps, 38.56M steps/s, 3 GCs, 233.801ms GC pause; 100M repeat `2.608s`, 38.66M steps/s, 231.241ms GC pause |
| full gate | first full run `92.508s`, 3,659,075,054 steps, 39.55M steps/s, 125 GCs, 26.963s GC pause, high-water 40,006,668 cells; repeat `91.598s`, 3,659,075,187 steps, 39.95M steps/s, 125 GCs, 26.632s GC pause, high-water 40,006,682 cells |
| output | `/tmp/mhs-selfhost.comb` and `/tmp/mhs-selfhost-rust-out-parse-smallint-repeat.comb` both SHA256 `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`; `cmp -s` exit 0 |
| reading | old pre-cell rejection cleared: the repeated 100M slice no longer regresses, the full gate ties the 91.503s best within noise, and live/high-water cells drop. Keep it as C-shaped parser representation work, but it is not a new speed lever by itself |

F21 `mpz_get_d` decimal-rounding slice:

| item | result |
|---|---|
| change | `MpzValue::to_f64` now converts through decimal bytes and Rust's decimal parser, matching C's `strtod`-style one-step rounding instead of Horner accumulation in `f64` |
| regression value | `-299228957055072645483636` previously rounds one ULP differently under Horner accumulation; the new unit test asserts the decimal-parser bit pattern |
| code shape | conversion is `#[cold] #[inline(never)]` so decimal parsing stays out of the hot evaluator layout |
| tests | `cargo test --manifest-path rust/microhs-runtime/Cargo.toml --lib` passes 37/37; `cargo check --manifest-path rust/microhs-runtime/Cargo.toml --features gc-phase-profile` passes |
| bounded gate | 1M `46.725ms`; 100M `3.360s`, 3 GCs, `293.1ms` GC pause, sink `661902`; no full gate because this is a cold mpz correctness path and the bounded self-host gate stayed in the accepted band |

S1 compact profile-head frame slice:

| item | result |
|---|---|
| change | stack and fallback strict/WHNF frames now store `ProfileHead = Option<NodeId>` instead of `Option<String>` |
| rationale | default runs push `None`, but the old frame enum still paid the 24-byte string-option payload in every frame; profile keys are now materialized only in cold profile-recording code |
| tests | `cargo test --manifest-path rust/microhs-runtime/Cargo.toml --lib` passes 36/36 |
| full output | `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`, byte-identical |
| reading | unlike the boxed-profile-head probe, this removes payload from the default frame shape; first full gate was a small new best at the time. The repeat output file byte-matches the oracle, but its timing stdout was lost with the dead tool session. It was later superseded by the 118.739s stack scalar `SET*` gate and then the 105.859s authoritative-cell gate |

F6 serializer sharing/cycle parity slice:

| item | result |
|---|---|
| change | `IO.serialize` now does a sharing-discovery pass and emits C-style `:label` definitions plus `_label` references for repeated shareable nodes |
| shareable nodes | app, array, foreign pointer, bigint, bytes, mutable bytes |
| label identity | `NodeId.index()`; compatible with the existing parser's arbitrary numeric labels |
| cyclic app graph | `v8.4\n1\nK _0 @ :0 }\n` serializes without recursion failure and parses back |
| shared app graph | repeated app subgraph serializes with one label definition and one reference |
| remaining serializer work | none for the current F6/F10 parity target; BFILE `IO.print`/`IO.serialize` behavior and `IO.deserialize` are now covered by later parity slices |
| tests | `cargo test --manifest-path rust/microhs-runtime/Cargo.toml --lib` passes 36/36 |

F10 iterative/deep serializer slice:

| item | result |
|---|---|
| change | replaced recursive `serialize_comb_into(root, depth, ...)` with an explicit postorder work stack that emits app children, array elements, app markers, array markers, and labels in the same postfix order |
| depth smoke | generated `/tmp/mhs-deep-app.comb` as a 12,000-deep WHNF app chain headed by `#0`; `target/release/mhs-rust-bench --input /tmp/mhs-deep-app.comb --mode whnf --warmup-iters 0 --iters 1` completed in `1.082ms`, 0 WHNF steps, sink `48130`, and 12,001 live nodes; this would have hit the old `depth > 10_000` serializer cap |
| verification | `cargo fmt --manifest-path rust/microhs-runtime/Cargo.toml --check`; serializer-focused tests 4/4; `cargo test --manifest-path rust/microhs-runtime/Cargo.toml --lib` 37/37; `cargo check --manifest-path rust/microhs-runtime/Cargo.toml`; `cargo check --manifest-path rust/microhs-runtime/Cargo.toml --features eval-phase-profile`; `cargo check --manifest-path rust/microhs-runtime/Cargo.toml --features gc-phase-profile`; `cargo build --release --manifest-path rust/microhs-runtime/Cargo.toml --bin mhs-rust-bench`; `cargo check --release --manifest-path rust/microhs-runtime/Cargo.toml --target wasm32-unknown-unknown`; `git diff --check` |
| bounded gate | 1M `40.278ms`, 1,001,383 steps, 24.86M steps/s, no GC; 100M `2.598s`, 100,811,774 steps, 38.80M steps/s, 3 GCs, 231.236ms GC pause |
| full gate | `92.575s`, 3,659,075,073 steps, 39.53M steps/s, 125 GCs, 26.559s GC pause, high-water 40,006,670 cells |
| output | `/tmp/mhs-selfhost.comb` and `/tmp/mhs-selfhost-rust-out-iter-serializer.comb` both SHA256 `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`; `cmp -s` exit 0 |
| reading | accepted as correctness/F10 progress. This is not a speed lever: the current full gate remains near the parser-small-int noise band and does not beat the 91.503s raw-tag best |

F10 BFILE `IO.print`/`IO.serialize` parity slice:

| item | result |
|---|---|
| change | `IO.print` and `IO.serialize` now evaluate their first argument as a BFILE pointer and write through the existing BFILE writer; `IO.print` no longer uses depth-capped debug render and instead emits prefix/parenthesized graph output with sharing labels for repeated nodes |
| stdout smokes | `IO.print IO.stdout #5` emits `#5`; `IO.serialize IO.stdout #5` emits `v8.4\n0\n#5 }` before the benchmark trailer |
| verification | `cargo fmt --manifest-path rust/microhs-runtime/Cargo.toml --check`; `cargo check --manifest-path rust/microhs-runtime/Cargo.toml`; `cargo build --release --manifest-path rust/microhs-runtime/Cargo.toml --bin mhs-rust-bench`; `cargo test --manifest-path rust/microhs-runtime/Cargo.toml --lib` 37/37; `cargo check --release --manifest-path rust/microhs-runtime/Cargo.toml --target wasm32-unknown-unknown`; `git diff --check` |
| bounded gate | 10M `319.983ms`, 10,028,866 steps, 31.34M steps/s, no GC; 100M `2.352s`, 100,811,779 steps, 42.86M steps/s, 3 GCs, 222.716ms GC pause |
| full gate | `82.518s`, 3,659,074,983 steps, 44.34M steps/s, 125 GCs, 24.157s GC pause, last live 1,289,548 nodes, high-water 38,820,720 cells |
| output | `/tmp/mhs-selfhost-rust-f10-bfile-full.comb` and `/tmp/mhs-selfhost-rust-compound-cache-full.comb` both SHA256 `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`; `cmp -s` exit 0 |
| reading | accepted as correctness/F10 progress, not a new performance best. Marking the new printer and existing serializer cold/noinline matters: the first normal build with the new printer in the same hot codegen neighborhood regressed 10M/100M to 486.032ms/2.792s before the cold split restored bounded performance to the current band |

Two follow-up checks after this accepted slice:

| probe | result |
|---|---|
| trusted `apply_stack_app` redex lookup | rejected; 1M improved to 46.220 ms, but 100M regressed to 3.501s / 28.79M steps/s, so reverted |
| GC interval sweep on 100M | 16M solo was best bounded reading at 3.265s / 30.87M steps/s; 24M 3.552s, 32M solo control 3.483s, 48M 3.943s; full 16M gate regressed to 151.8s with 250 GCs and 51.8s GC pause, so keep 32M as the current full-gate default |
| C-style app-stack `POP` via `Vec::set_len` | rejected/no full gate; 1M was 46.389 ms and 100M repeated at 3.433s / 3.441s, only a small same-session win versus a noisy 3.488s control and still worse than the accepted 3.305s app-allocation reading, so reverted |
| fixed-heap-style node arena pre-reserve | rejected; `MHS_NODE_RESERVE=41943040` made 1M look fast at 43.641 ms, but 100M regressed to 3.517s, so the opt-in reserve hook was removed |
| `Node #[repr(u8)]` discriminant hint | rejected; `Node` stayed 16 bytes, 1M was neutral at 46.321 ms, and 100M regressed to 3.482s / 28.95M steps/s, so reverted |
| compact marker word in app stack | rejected after full gate; put frame markers into the 4-byte app stack stream with frame payloads still side-stored, giving 1M 46.545ms and 100M 3.490s/3.479s, but the full self-host gate regressed to 122.798s with 30.170s GC pause versus the 120.882s accepted gate, despite byte-identical output |
| parallel App/Indir/Free cell shell | rejected/no full gate; added a synchronized 12-byte `Cell` side arena and moved app/indir/free hot reads to it while keeping `Node` as the compatibility shell; 1M regressed to 61.183ms and 100M to 4.428s with 640.3ms GC pause, so reverted |
| direct primitive-tag node variants | rejected/no full gate; split `Node::Prim(Prim::{Known,Runtime})` into direct `Node::{Known,Runtime}` variants as a small authoritative tag-word slice, but 100M regressed to 3.494s / 28.85M steps/s despite fewer nodes from a shorter output path, so restored the nested `Prim` shape |

Rejected parallel cell-shell bounded gate:

| measure | 1M | 100M |
|---|---:|---:|
| parse/reduce/render | 61.183 ms | 4,427.626 ms |
| WHNF steps/s | 16.37M | 22.77M |
| GC collections | 0 | 3 |
| GC total pause | 0 ms | 640.283 ms |
| node / cell / node-id / prim size | 16 / 12 / 4 / 4 bytes | 16 / 12 / 4 / 4 bytes |
| result | rejected | rejected |

Rejected compact marker-stack full gate:

| measure | value |
|---|---:|
| parse/reduce/render | 122,798.138 ms |
| WHNF steps | 3,659,074,978 |
| WHNF steps/s | 29,797,479 |
| step-limited iters | 0 |
| serialize sink | 661,902 |
| GC collections | 125 |
| GC total pause | 30,169.809 ms |
| GC high-water nodes | 39,958,355 |
| output | byte-identical; SHA `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a` |

Rejected 16M full gate:

| measure | value |
|---|---:|
| parse/reduce/render | 151,787.888 ms |
| WHNF steps | 3,659,074,902 |
| WHNF steps/s | 24,106,501 |
| step-limited iters | 0 |
| serialize sink | 661,902 |
| GC collections | 250 |
| GC total pause | 51,784.364 ms |
| GC high-water nodes | 24,037,356 |
| output | byte-identical; SHA `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a` |

#### Rejected Direct Inner `GOAP2` Descent

After the accepted direct root-app push, I tried the next literal `eval.c`
`ap2` step: when a rewrite had just allocated the inner app, push both the
rewritten root and that known inner app before descending from the already-bound
inner fun. This covered clean `GOAP2` shapes such as `S`, `S'`, `C`, `C'`, `P`,
`R`, `O`, `C'B`, `TAG`, `IO.performIO`, `IO.>>=`, `IO.>>`, and `IO.return`.
Tuple chains were deliberately left alone because their inner app is built by a
loop rather than a single local allocation.

Benchmarks, 32M allocation trigger:

| run | direct root app-result descent | direct inner `GOAP2` descent | result |
|---|---:|---:|---|
| 1M slice | 48.512 ms / 20.64M steps/s | 46.958 ms / 21.32M steps/s | bounded win |
| 100M slice | 3.605s / 27.96M steps/s | 3.467s / 29.08M steps/s | bounded win |
| 100M GC pause | 313.933 ms | 271.938 ms | win |
| full self-host | 131.6s / 27.80M steps/s | 131.8s / 27.77M steps/s | not a new best |
| full repeat | n/a | 133.2s / 27.47M steps/s | rejected |
| full output | byte-identical | byte-identical | `cmp_exit=0` |

Full rejected gates:

| measure | first full | repeat full |
|---|---:|---:|
| parse/reduce/render | 131760.711 ms | 133222.640 ms |
| WHNF steps | 3,659,074,997 | 3,659,075,130 |
| GC collections | 125 | 125 |
| GC total pause | 29,875.016 ms | 30,223.918 ms |
| GC high-water nodes | 39,958,357 | 39,958,371 |

Checks:

| check | result |
|---|---|
| `cargo fmt --manifest-path rust/microhs-runtime/Cargo.toml` | passed |
| `cargo check --manifest-path rust/microhs-runtime/Cargo.toml` | passed |
| `cargo test --manifest-path rust/microhs-runtime/Cargo.toml` | passed; 34 tests |
| `cargo build --release --manifest-path rust/microhs-runtime/Cargo.toml --bin mhs-rust-bench` | passed |
| full output compare | byte-identical on both full runs |
| post-revert release rebuild | passed |
| post-revert 1M sanity | 48.732 ms / 20.55M steps/s, back in accepted band |

Reading: this is a useful false positive. Removing one more generic descent
step from clean `GOAP2` rewrites helps the first 100M steps, but the complete
self-host phase mix does not improve, and the repeat regresses. Keep the
accepted root-app push; do not retry direct inner `GOAP2` descent alone.

### S1 App-Only Eval Stack

`NOTES.md`'s fresh warning was mostly stale about the old persistent-spine
machine, but still right about the direction: remove Rust-only layers from the
step machine instead of reshuffling matches. This slice changes the stack
representation from `Vec<EvalStackEntry>` with `App`/`Frame` tags to a plain
`Vec<NodeId>` for app cells plus the existing separate frame payload stack.
That makes the hot stack segment closer to eval.c's app-pointer stack:
`ARG(TOP(i))` no longer pays an enum tag check or stores frame sentinels in
the app vector.

Checks:

| check | result |
|---|---|
| `cargo fmt --manifest-path rust/microhs-runtime/Cargo.toml` | passed |
| `cargo check --manifest-path rust/microhs-runtime/Cargo.toml` | passed |
| `git diff --check` | passed |
| `cargo build --release --manifest-path rust/microhs-runtime/Cargo.toml --bin mhs-rust-bench` | passed |
| `cargo test --manifest-path rust/microhs-runtime/Cargo.toml` | passed; 34 tests |
| debug/release `cargo build --target wasm32-unknown-unknown --manifest-path rust/microhs-runtime/Cargo.toml` | passed |
| Node `host.mjs` wasm smoke | rendered `42` |

Fresh slice and gate numbers, same 32M allocation trigger:

| run | trusted args/static names | app-only eval stack | result |
|---|---:|---:|---|
| 1M non-profile | 57.1 ms | 53.1 ms / 52.1 ms profile-smoke control | win |
| 100M slice | 4.46s / 22.60M steps/s | 4.26s / 23.68M steps/s, repeat 4.42s | win |
| 1M profile total | 217.8 ms | 210.6 ms | small win, same profile counts |
| full self-host | 158.5s / 23.09M steps/s | 151.4s / 24.16M steps/s | new best |
| full self-host GC pause | 29.40s | 29.13s | flat/small win |
| full self-host output | byte-identical | byte-identical | `cmp_exit=0` |

Full gate command:

```text
timeout 900s env MHS_GC_NODE_INTERVAL=33554432 target/release/mhs-rust-bench \
  --input /tmp/mhs-selfhost.comb --mode main --warmup-iters 0 --iters 1 -- \
  ./bin/mhs -i -imhs -isrc -ilib MicroHs.Main -o/tmp/mhs-selfhost-rust-out-app-stack.comb
```

```text
parse_reduce_render_total_ms: 151432.314
whnf_steps_per_iter: 3659074959.0
whnf_steps_per_s: 24163105.3
step_limited_iters: 0
serialize_sink: 661902
gc_collections: 125
gc_freed_nodes_total: 4192827108
gc_last_live_nodes: 1637998
gc_last_free_nodes: 38320070
gc_high_water_nodes: 39958068
gc_current_nodes: 39958068
gc_current_free_nodes: 24171593
gc_last_pause_ms: 148.837
gc_total_pause_ms: 29129.253
```

```text
sha256(/tmp/mhs-selfhost.comb) = 29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a
sha256(/tmp/mhs-selfhost-rust-out-app-stack.comb) = 29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a
cmp_exit=0
```

Reading: keep this slice. The win is exactly the eval.c lesson in smaller
form: app stack entries are app ids, not tagged records. The remaining S1
work should keep removing layers around the same loop. A profile-vs-normal
const-specialized `stack_eval_step` looked plausible on slices but regressed
the full gate, so code-shape duplication alone is still not enough.

### S1 App-Only Strict Redex Slice

`NOTES.md` landed after the S2 work and confirmed the main diagnosis: the Rust
runtime does essentially the same reduction/allocation work as eval.c, but pays
too much per step. The first S1 slice makes strict continuations closer to
eval.c: `StrictRedex::Spine` stores only the app-cell ids, not a parallel copy
of all remaining args. When the strict result returns, the reducer reads each
argument back out of the saved app cell and rethreads through those cells.

Checks before the full gate:

| check | result |
|---|---|
| `cargo check --manifest-path rust/microhs-runtime/Cargo.toml` | passed |
| `MHS_GC_NODE_INTERVAL=32 cargo test --workspace` | passed; 34 tests |
| `cargo build --release --manifest-path rust/microhs-runtime/Cargo.toml --bin mhs-rust-bench` | passed |

100M self-host main slice, 32M allocation trigger:

| run | time | steps/s | collections | GC pause | high-water nodes |
|---|---:|---:|---:|---:|---:|
| S2 baseline | 15.02s | 6.71M | 3 | 333 ms | 34,237,713 |
| S1 app-only strict redex | 12.56s | 8.03M | 3 | 316 ms | 34,237,717 |

Full self-host gate:

```text
timeout 900s env MHS_GC_NODE_INTERVAL=33554432 target/release/mhs-rust-bench \
  --input /tmp/mhs-selfhost.comb --mode main --warmup-iters 0 --iters 1 -- \
  ./bin/mhs -i -imhs -isrc -ilib MicroHs.Main -o/tmp/mhs-selfhost-rust-out-s1-appsonly.comb
```

```text
parse_reduce_render_total_ms: 578419.251
whnf_steps_per_iter: 3659074997.0
whnf_steps_per_s: 6325991.0
step_limited_iters: 0
serialize_sink: 661902
gc_collections: 125
gc_freed_nodes_total: 4192274258
gc_last_live_nodes: 2295069
gc_last_free_nodes: 41331806
gc_high_water_nodes: 43626875
gc_current_nodes: 43626875
gc_current_free_nodes: 27183281
gc_total_pause_ms: 60746.722
```

The generated output matched the C-generated input exactly:

```text
sha256(/tmp/mhs-selfhost.comb) = 29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a
sha256(/tmp/mhs-selfhost-rust-out-s1-appsonly.comb) = 29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a
cmp_exit=0
```

This is the first post-tag gate under 600s, but it is still not the 120s path.
The next structural slice tested the same app-only invariant in
`PersistentSpine`, the ordinary fast path.

### S1 App-Only PersistentSpine Slice

`PersistentSpine` now stores only app-cell ids. `arg(i)` reads the argument back
from the saved app cell, and GC marks only the app cells because marking an app
recursively marks its fun and arg. This removes the persistent path's parallel
args deque, but it is not a pure win on the bounded slice: with the current
32-byte `Node`, every arg read now reloads and matches the arena cell. The full
gate still improved, likely because the whole program weights spine retention
and GC pressure differently than the first 100M steps.

Checks before the full gate:

| check | result |
|---|---|
| `MHS_GC_NODE_INTERVAL=32 cargo test --workspace` | passed; 34 tests |
| `cargo build --release --manifest-path rust/microhs-runtime/Cargo.toml --bin mhs-rust-bench` | passed |
| `git diff --check` | passed |

100M self-host main slice, 32M allocation trigger:

| run | time | steps/s | collections | GC pause | high-water nodes |
|---|---:|---:|---:|---:|---:|
| S1 strict-redex app-only baseline | 12.56s | 8.03M | 3 | 316 ms | 34,237,717 |
| S1 persistent-spine app-only | 12.82s | 7.87M | 3 | 315 ms | 34,237,738 |

Full self-host gate:

```text
timeout 900s env MHS_GC_NODE_INTERVAL=33554432 target/release/mhs-rust-bench \
  --input /tmp/mhs-selfhost.comb --mode main --warmup-iters 0 --iters 1 -- \
  ./bin/mhs -i -imhs -isrc -ilib MicroHs.Main -o/tmp/mhs-selfhost-rust-out-s1-persistent-appsonly.comb
```

```text
parse_reduce_render_total_ms: 566682.591
whnf_steps_per_iter: 3659075206.0
whnf_steps_per_s: 6457010.1
step_limited_iters: 0
serialize_sink: 661902
gc_collections: 125
gc_freed_nodes_total: 4192274221
gc_last_live_nodes: 2295128
gc_last_free_nodes: 41331769
gc_high_water_nodes: 43626897
gc_current_nodes: 43626897
gc_current_free_nodes: 27182980
gc_total_pause_ms: 57513.332
```

The generated output matched the C-generated input exactly:

```text
sha256(/tmp/mhs-selfhost.comb) = 29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a
sha256(/tmp/mhs-selfhost-rust-out-s1-persistent-appsonly.comb) = 29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a
cmp_exit=0
```

Reading: keep this slice for now because the full gate improved from 578.4s to
566.7s, but do not infer that app-only spines alone are the 120s lever. The
remaining large move is still eval.c's whole shape: one contiguous stack, direct
in-cell tags, no hot `Node` clones, and eventually compact cells.

### S1 Persistent Head Classification Slice

`persistent_eval_step` no longer clones the whole head node just to dispatch.
Known primitive heads are copied as `KnownPrim`; unknown primitive names are
classified while borrowed into a small `StrictPrimitiveAction`; FFI/JS payloads
still clone because they are not the hot combinator/strict-primitive path. This
keeps the existing profile probe counters by replaying the same strict-dispatch
probe order from the classified action.

Checks before the full gate:

| check | result |
|---|---|
| `MHS_GC_NODE_INTERVAL=32 cargo test --workspace` | passed; 34 tests |
| `cargo build --release --manifest-path rust/microhs-runtime/Cargo.toml --bin mhs-rust-bench` | passed |
| `git diff --check` | passed |

100M self-host main slice, 32M allocation trigger:

| run | time | steps/s | collections | GC pause | high-water nodes |
|---|---:|---:|---:|---:|---:|
| S1 persistent-spine app-only baseline | 12.82s | 7.87M | 3 | 315 ms | 34,237,738 |
| S1 persistent head classification | 11.91s | 8.47M | 3 | 328 ms | 34,237,719 |

Full self-host gate:

```text
timeout 900s env MHS_GC_NODE_INTERVAL=33554432 target/release/mhs-rust-bench \
  --input /tmp/mhs-selfhost.comb --mode main --warmup-iters 0 --iters 1 -- \
  ./bin/mhs -i -imhs -isrc -ilib MicroHs.Main -o/tmp/mhs-selfhost-rust-out-s1-headclass.comb
```

```text
parse_reduce_render_total_ms: 539639.511
whnf_steps_per_iter: 3659075016.0
whnf_steps_per_s: 6780591.4
step_limited_iters: 0
serialize_sink: 661902
gc_collections: 125
gc_freed_nodes_total: 4192274241
gc_last_live_nodes: 2295087
gc_last_free_nodes: 41331790
gc_high_water_nodes: 43626877
gc_current_nodes: 43626877
gc_current_free_nodes: 27183240
gc_total_pause_ms: 57787.419
```

The generated output matched the C-generated input exactly:

```text
sha256(/tmp/mhs-selfhost.comb) = 29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a
sha256(/tmp/mhs-selfhost-rust-out-s1-headclass.comb) = 29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a
cmp_exit=0
```

Reading: this confirms the `NOTES.md` claim that hot head cloning was real.
It is still a local fix around the old machine, not the final eval.c shape.
The next likely S1/S3 slice is either moving the remaining `EvalSpine` fallback
off head clones/parallel args, or starting the compact in-cell tag path that
makes app-only arg loads cheap instead of just less duplicative.

### S1 EvalSpine Head Classification Slice

The fallback/general `EvalSpine` path now uses the same classification idea as
`PersistentSpine`: FFI/JS payloads still clone because they are cold and owned
by call helpers, but known primitive heads copy a `KnownPrim`, and unknown
strict primitive names are classified while borrowed before the runtime decides
whether it needs a fallback helper name. This removes the main
`self.nodes[head].clone()` in `eval_loop_step`; the remaining head clones are
in small IO shortcut helpers.

Checks before the full gate:

| check | result |
|---|---|
| `MHS_GC_NODE_INTERVAL=32 cargo test --workspace` | passed; 34 tests |
| `cargo build --release --manifest-path rust/microhs-runtime/Cargo.toml --bin mhs-rust-bench` | passed |
| `git diff --check` | passed |

100M self-host main slice, 32M allocation trigger:

| run | time | steps/s | collections | GC pause | high-water nodes |
|---|---:|---:|---:|---:|---:|
| S1 persistent head-classification baseline | 11.91s | 8.47M | 3 | 328 ms | 34,237,719 |
| S1 EvalSpine head classification | 11.97s | 8.42M | 3 | 332 ms | 34,237,727 |

Full self-host gate:

```text
timeout 900s env MHS_GC_NODE_INTERVAL=33554432 target/release/mhs-rust-bench \
  --input /tmp/mhs-selfhost.comb --mode main --warmup-iters 0 --iters 1 -- \
  ./bin/mhs -i -imhs -isrc -ilib MicroHs.Main -o/tmp/mhs-selfhost-rust-out-s1-evalheadclass.comb
```

```text
parse_reduce_render_total_ms: 525189.184
whnf_steps_per_iter: 3659075092.0
whnf_steps_per_s: 6967156.2
step_limited_iters: 0
serialize_sink: 661902
gc_collections: 125
gc_freed_nodes_total: 4192274237
gc_last_live_nodes: 2295099
gc_last_free_nodes: 41331786
gc_high_water_nodes: 43626885
gc_current_nodes: 43626885
gc_current_free_nodes: 27183140
gc_total_pause_ms: 57593.037
```

The generated output matched the C-generated input exactly:

```text
sha256(/tmp/mhs-selfhost.comb) = 29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a
sha256(/tmp/mhs-selfhost-rust-out-s1-evalheadclass.comb) = 29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a
cmp_exit=0
```

Reading: keep this slice. The bounded 100M slice was neutral/slightly worse,
but the full gate improved from 539.6s to 525.2s. The next meaningful move is
less likely to be another clone cleanup and more likely to be either the
single-stack strict-continuation shape or S3's compact tag/cell representation.

### S3 Parse-Time Primitive Interning Slice

`eval.c` does not allocate one primitive node per `.comb` occurrence: the parser
pushes references to permanent primitive singleton nodes. The Rust parser now
keeps a parse-local primitive table and reuses one `NodeId` per primitive name
inside a parsed program. This is not the final in-cell tag representation, but
it removes a Rust-only source of duplicate `Prim` cells and moves the input
graph closer to the C model.

Checks before/after the full gate:

| check | result |
|---|---|
| `cargo check --manifest-path rust/microhs-runtime/Cargo.toml` | passed |
| `MHS_GC_NODE_INTERVAL=32 cargo test --workspace` | passed; 34 tests |
| `cargo build --release --manifest-path rust/microhs-runtime/Cargo.toml --bin mhs-rust-bench` | passed |
| debug/release `cargo build --target wasm32-unknown-unknown --manifest-path rust/microhs-runtime/Cargo.toml` | passed |
| Node `host.mjs` wasm smoke | rendered `42` |
| `git diff --check` | passed |

1M self-host profile slice:

| measure | baseline | primitive interning |
|---|---:|---:|
| parsed nodes before reduce | 265,178 | 160,961 |
| nodes after 1M slice | 1,368,741 | 1,264,564 |
| non-profile parse+reduce+render | 211.7 ms | 152.2 ms |
| profile total | 331.0 ms | 297.7 ms |
| WHNF steps | 1,001,383 | 1,001,383 |

100M self-host main slice, 32M allocation trigger:

| run | time | steps/s | collections | GC pause | high-water nodes |
|---|---:|---:|---:|---:|---:|
| S1 EvalSpine head-classification baseline | 11.97s | 8.42M | 3 | 332 ms | 34,237,727 |
| S3 parse-time primitive interning | 11.48s | 8.78M | 3 | 340 ms | 34,138,007 |

Full self-host gate:

```text
timeout 900s env MHS_GC_NODE_INTERVAL=33554432 target/release/mhs-rust-bench \
  --input /tmp/mhs-selfhost.comb --mode main --warmup-iters 0 --iters 1 -- \
  ./bin/mhs -i -imhs -isrc -ilib MicroHs.Main -o/tmp/mhs-selfhost-rust-out-prim-intern.comb
```

```text
parse_reduce_render_total_ms: 511949.525
whnf_steps_per_iter: 3659074997.0
whnf_steps_per_s: 7147335.5
step_limited_iters: 0
serialize_sink: 661902
gc_collections: 125
gc_freed_nodes_total: 4192269393
gc_last_live_nodes: 2195717
gc_last_free_nodes: 41331803
gc_high_water_nodes: 43527520
gc_current_nodes: 43527520
gc_current_free_nodes: 27183278
gc_total_pause_ms: 58410.276
```

The generated output matched the C-generated input exactly:

```text
sha256(/tmp/mhs-selfhost.comb) = 29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a
sha256(/tmp/mhs-selfhost-rust-out-prim-intern.comb) = 29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a
cmp_exit=0
```

Reading: keep this slice. It is a real C-shape win because it reduces the
initial graph and live set without adding evaluator-side machinery. It still
only buys ~2.5% on the full gate, so the 120s target continues to require the
S1/S3 vertical slice: compact cells, in-cell primitive tags, and one evaluator
stack.

### Superseded S3 Parse-Time Small-Int Interning Probe

`eval.c` also has a permanent `intTable` for `-10..255`, so I tested sharing
parsed `Node::Int` literals in that range. The first 1M slice looked tempting
because the initial graph shrank, but repeated 100M slices regressed. The probe
was reverted in the old `Node` enum machine. The post-cell retest is now
accepted with no repeated-100M regression; keep this section as the historical
reason it waited for compact cells instead of retrying inside the old evaluator.

| run | primitive interning | small-int parser interning | result |
|---|---:|---:|---|
| parsed nodes before reduce | 160,961 | 153,185 | smaller graph |
| nodes after 1M slice | 1,264,552 | 1,256,782 | smaller graph |
| best 1M non-profile slice | 155.0 ms | 143.5 ms | local win |
| repeated 100M slice | 11.48s best / 11.87s profiled-control | 12.22s, then 12.60s | rejected |

Reading: this old result was representation-specific. After compact cells, the
same eval.c-shaped small-int sharing lowers parsed/live cells and ties the full
gate instead of regressing.

### Rejected S1 Moved Persistent-Spine Redex Probe

I tested removing the persistent-path strict-redex app snapshot by moving the
whole `PersistentSpine` into the strict continuation and rethreading directly
from that moved spine on return. This attacks the 12.0M app-id copy counted by
S0 profiling, but repeats the core locality problem from the earlier tail
ownership probe: the hot persistent-spine buffer stops being the single owner.
The probe was reverted.

| run | baseline/profiled-control | moved persistent spine | result |
|---|---:|---:|---|
| 1M non-profile slice | 155.0 ms | 168.3 ms | rejected |
| 1M profile total | 318.8 ms | 324.0 ms | rejected |
| 100M self-host slice | 11.87s / 8.49M steps/s | 13.35s / 7.55M steps/s | rejected |
| post-revert 100M sanity | 11.66s / 8.64M steps/s | n/a | back in accepted band |

Reading: avoiding one copy is not enough if the continuation ceases to be one
stable stack. The next S1 attempt should replace the frame/redex/spine split
with one stack-entry representation, not move ownership between those objects.

### Rejected S1 PersistentSpine Contiguous Vec Probe

I tested replacing `PersistentSpine`'s `VecDeque` with a contiguous `Vec` in
descent order, while preserving the existing head-order API by indexing from
the end. This looked closer to eval.c's contiguous AP stack, but in the current
Rust representation it made both the 100M slice and full gate worse. The probe
was reverted; `PersistentSpine` is back to `VecDeque`.

| run | baseline | contiguous `Vec` | result |
|---|---:|---:|---|
| 100M self-host slice | 11.97s / 8.42M steps/s | 13.33s / 7.56M steps/s | rejected |
| full self-host gate | 525.2s / 6.97M steps/s | 612.0s / 5.98M steps/s | rejected |
| full self-host output | byte-identical | byte-identical | semantics ok |

Reading: do not retry a storage-only contiguous stack inside the current
32-byte-node machine. eval.c's stack shape probably needs to land with the
larger S1/S3 package: direct stack interpreter, in-cell tags, and compact cells.

### Rejected S1 Strict-Redex Tail Ownership Probe

I tested shrinking `StrictRedex::Spine` to the consumed redex app plus only the
tail app cells that need rethreading. On the persistent path, the probe moved
the tail out of `PersistentSpine` instead of cloning the full app snapshot. That
was semantically safe, but mechanically worse: it saved one copy while throwing
away the hot persistent spine buffer on strict forces. The probe was reverted.

| run | baseline | tail ownership | result |
|---|---:|---:|---|
| 100M self-host slice | 11.97s / 8.42M steps/s | 15.31s / 6.59M steps/s | rejected |
| post-revert 100M sanity | 11.87s / 8.49M steps/s | n/a | back in accepted band |
| full self-host gate | 525.2s / 6.97M steps/s | 773.3s / 4.73M steps/s | rejected |
| full self-host GC pause | 57.6s | 61.9s | not the main loss |
| full self-host output | byte-identical | byte-identical | semantics ok |

Reading: eval.c's win is not "move spine ownership between objects"; it is that
there is only one stack object. Do not retry tail-moving or split ownership in
the current evaluator. The next S1 attempt should keep app entries and strict
markers in the same persistent stack.

### Rejected S1 Segmented PersistentSpine Probe

I tested a closer eval.c-style persistent stack: `PersistentSpine` had an active
prefix plus inactive suspended tails, and strict frames from the persistent fast
path stored only `root`, the consumed redex app, and `tail_len`. Nested strict
forces passed the forced-GC tests and the full gate was byte-identical, but the
extra active-segment bookkeeping taxed every persistent spine operation. The
probe was reverted.

| run | baseline | segmented stack | result |
|---|---:|---:|---|
| 100M self-host slice | 11.97s / 8.42M steps/s | 13.64s / 7.39M steps/s | rejected |
| post-revert 100M sanity | 12.33s / 8.18M steps/s | n/a | back near accepted band |
| full self-host gate | 525.2s / 6.97M steps/s | 619.3s / 5.91M steps/s | rejected |
| full self-host GC pause | 57.6s | 57.6s | not the main loss |
| full self-host output | byte-identical | byte-identical | semantics ok |

Reading: a segmented overlay on top of the old `PersistentSpine`/`EvalFrameStack`
is still not eval.c's machine. The next attempt should collapse dispatch, stack
entries, and strict markers together rather than adding active/inactive segment
bookkeeping to the old spine API.

### Rejected S1 Hybrid App/Marker Stack Splice

After the 493.0s GC-compression gate, I tried a more direct F2 splice:
`PersistentSpine` became one stack of app entries plus strict int markers, with
head-order indexing derived from the stack tail. The intent was to leave outer
apps on the stack across reductions, pop only consumed apps, and avoid
`StrictRedex` snapshots for integer strict primitives.

The splice was rejected before any full gate:

| variant | 1M self-host result | reading |
|---|---:|---|
| app/marker stack, no outer rethread | stopped after 35,659 steps | semantic bug; `reduce_main` returned success early |
| app stack, markers disabled, old rethread restored | 286.8 ms | semantically correct but much slower |
| app stack, markers disabled, no outer rethread | 241.6 ms | semantically correct but still much slower |
| post-revert sanity | 149.1 ms | back in accepted band |

Reading: this confirms the earlier warning from the segmented-stack probe. A
hybrid stack bolted into the old driver keeps too many old boundaries and makes
the hot path worse; removing the rethread bridge without replacing the whole
return path is unsound. The next S1 attempt should be a replacement evaluator
loop with app entries, markers, dispatch, and WHNF return handled in one place,
not another `PersistentSpine` storage rewrite.

### Rejected P2 Direct Scalar Redex Overwrite Probe

`eval.c` writes arithmetic and conversion scalar results directly into the
consumed redex cell with `SETINT`/`SETINT64`/`SETDBL`/`SETFLT`, instead of
allocating a fresh scalar and installing an indirection. I tested the analogous
Rust change for strict `Int`, `Int64`, `Float64`, `Float32`, and conversion
frames: scalar results overwrote the consumed app cell; boolean/order results
kept the existing indirection-to-permanent-combinator path.

The probe was correct on tests and a small 1M slice, but regressed the 100M
self-host slice and was reverted:

| run | baseline | direct scalar overwrite | result |
|---|---:|---:|---|
| 1M self-host slice | 149.1 ms post-revert sanity | 146.8 ms | local win |
| 100M self-host slice | 11.67s / 8.64M steps/s | 12.32s / 8.18M steps/s | rejected |
| forced-GC tests | passed | passed | semantics ok |

Reading: this is another eval.c-shaped move that does not pay while bolted onto
the current frame/redex/spine machine. In C the redex write is part of the same
single-stack return path; in Rust it adds another branch/shape to a driver that
still has to carry `StrictRedex` and rethread outer apps. Fold scalar redex
writes into the replacement evaluator loop, not as a standalone patch.

The pre-GC 1200s gate crashed the VM and lost `/tmp/mhs-selfhost.comb`; after
regenerating the input with `./bin/mhs -i -imhs -isrc -ilib MicroHs.Main
-o/tmp/mhs-selfhost.comb`, the GC-enabled gate showed RSS sawtoothing around
5.4 GB by observation and completed successfully:

```text
timeout 1200s env MHS_GC_NODE_INTERVAL=4194304 target/release/mhs-rust-bench \
  --input /tmp/mhs-selfhost.comb --mode main --warmup-iters 0 --iters 1 -- \
  ./bin/mhs -i -imhs -isrc -ilib MicroHs.Main -o/tmp/mhs-selfhost-rust-out.comb
```

```text
parse_reduce_render_total_ms: 716828.052
whnf_steps_per_iter: 3659074769.0
whnf_steps_per_s: 5104536.2
step_limited_iters: 0
serialize_sink: 661902
```

The generated output matched the C-generated input exactly:

```text
sha256(/tmp/mhs-selfhost.comb) = 29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a
sha256(/tmp/mhs-selfhost-rust-out.comb) = 29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a
cmp_exit=0
```

The latest prefix-materialization 16M gate was:

```text
timeout 900s env MHS_GC_NODE_INTERVAL=16777216 target/release/mhs-rust-bench \
  --input /tmp/mhs-selfhost.comb --mode main --warmup-iters 0 --iters 1 -- \
  ./bin/mhs -i -imhs -isrc -ilib MicroHs.Main -o/tmp/mhs-selfhost-rust-out-prefix.comb
```

```text
parse_reduce_render_total_ms: 673813.006
whnf_steps_per_iter: 3659074902.0
whnf_steps_per_s: 5430401.1
step_limited_iters: 0
serialize_sink: 661902
gc_collections: 22
gc_freed_nodes_total: 4176223883
gc_last_live_nodes: 2614351
gc_last_free_nodes: 366749583
gc_high_water_nodes: 369363934
gc_current_nodes: 369363934
gc_current_free_nodes: 336870095
```

The output again matched byte-for-byte:

```text
sha256(/tmp/mhs-selfhost.comb) = 29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a
sha256(/tmp/mhs-selfhost-rust-out-prefix.comb) = 29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a
cmp_exit=0
```

## ByteString/Pointer Diagnostics

The old temporary `MHS_TRACE_INVALID_BYTES=1` diagnostics in the ByteString and
pointer paths were removed by the later invalid-byte trace cleanup; this section
is retained as historical context for the bugs they diagnosed.

Two C-parity gaps have concrete reproducers and are fixed in the current
worktree:

| reproducer | result |
|---|---|
| `bsnew 0 4` followed by `bswrite` at index 1 and `bsread` at index 1 | passes; mutable bytes now track logical size separately from backing capacity |
| `bs2fp`/`fp+`/`fp2p` followed by `^poke_uint8`, then `bsread` | passes; node-backed ByteString pointers can now be written through `write_pointer_bytes` |

The failing diagnostic self-host gate was:

```text
timeout 600s env MHS_TRACE_INVALID_BYTES=1 target/release/mhs-rust-bench \
  --input /tmp/mhs-selfhost.comb --mode main --warmup-iters 0 --iters 1 -- \
  ./bin/mhs -i -imhs -isrc -ilib MicroHs.Main -o/tmp/mhs-selfhost-rust-out.comb
```

It failed with the older direct trace:

```text
invalid bytes: reductions=1858478134
decode_pointer ptr=-9097150794930585600
run benchmark main: invalid ByteString operation
```

The next diagnostic run added caller context:

```text
suspicious pointer value: reductions=1858477741 ptr=-9097153187227369472 root=NodeId(2176871316):Ptr(-9097153187227369472)
...
suspicious pointer value: reductions=1858478132 ptr=-9097150794930585600 root=NodeId(2176871873):Ptr(-9097150794930585600)
invalid bytes: reductions=1858478134
decode_pointer ptr=-9097150794930585600
invalid bytes context: domain=ffi name=mpz_mul reductions=1858478134
invalid bytes context: domain=foreign_ptr_unop name=fp2p reductions=1858478134
invalid bytes context: domain=ffi name=mpz_add reductions=1858478134
...
invalid bytes context: domain=ffi name=mpz_cmp reductions=1858478134
thread 'main' (3) panicked ... run benchmark main: invalid ByteString operation
```

This was not a direct `bsread`/`bswrite` bounds problem. The bogus pointer was
coming from the Integer/GMP emulation path: `new_mpz` returns a `ForeignPtr`,
`primUnsafeCoerce` turns it into `ForeignPtr MPZ`, `fp2p` unwraps the raw
pointer, and then `mpz_*` decodes it. The specific failing pointer is also a
width clue: `NodeId(2176871873) << 32` crosses the signed `i64` high bit and
becomes `-9097150794930585600`. That late failure was a synthetic node-pointer
encoding bug, exposed once the grow-only arena reached node ids above `2^31`.

The current worktree fixes that by making positive node-backed pointers use a
small side-table slot instead of the raw arena node id, while preserving pointer
identity for repeated `bs2fp`/`new_mpz` roots. After the fix, `cargo test
--workspace`, `cargo build --release --bin mhs-rust-bench`, `git diff
--check`, and both small pointer reproducers passed. A traced 600s full gate
then timed out with no invalid-byte trace and no output comb. A later
GC-enabled 1200s gate completed, so the old failure sequence was: pointer
encoding bug first, then no-GC OOM/time.

## Active Lessons

| lesson | implemented | current reading |
|---|---|---|
| one evaluator loop should own strict forcing | yes for hot path | S1 stack path owns app entries, strict markers, dispatch, WHNF return, CHKARG-style argument binding/taking, unchecked eval.c-style arg loads, compact stack entries, one-word AP fun descent, and redex update for hot combinator/strict primitives; strict `Int` and WHNF frames now own the popped redex directly; ready strict-frame returns, strict force markers, K/A-family `GOIND`, and identity-alias `I`/`Ord`/`Chr` rewrites now continue inside `stack_eval_step` without a `StackStep::Force` handoff; 1M profile has 0 strict-redex snapshots and 0 remaining-app scans |
| compact node representation matters | yes | 56 -> 32 -> 24 -> 16-byte authoritative cells helped self-host; bench now logs `node_size_bytes=16`, `node_id_size_bytes=4`, `prim_size_bytes=4`; primitive tags classify strict actions directly, the structural cell arena moved the full gate to 105.859s, the cell-era `CHKARG` pop/take/direct-write slice moved it to 99.022s, strict `Int` redex-owning frames moved it to 95.345s, WHNF redex-owning frames moved it to 93.202s, raw tag/trusted free-list accessors moved it to 91.503s, F14 ForeignPtr finalizer records moved it to 90.237s, S1 unified stack `ret`/`top` continuation moved it to 89.141s, S2 parse-label non-root treatment moved it to 88.623s, S1 allocator free-head invariant tightening moved it to 86.619s, S1 identity-alias `GOIND` continuation moved it to 86.500s, S1 one-word app-fun spine descent moved it to 85.189s, S2 mark-time app-edge canonicalization moved it to 83.752s, S1 trusted stack redex/update access moved it to 82.154s, eval.c permanent compound caches moved it to 80.927s, app-allocation profiling-bookkeeping cold split moved it to 80.584s / 81.336s, inline primitive cell decoding moved it to 80.198s with an 80.309s first gate, release `panic=abort` with checked LZMA decode moved it to 77.387s, and the trusted-WHNF/app-fun/hot-cell series now puts the best full gate at 74.668s with 49.00M steps/s and 23.445s GC pause |
| avoid eager spine argument forcing | yes | matching C's lazier descent cut resolve calls roughly in half |
| parser allocation matters, but is no longer primary | mostly | borrowed tokens, large-input prealloc, primitive interning, and post-cell parse-time small-int sharing fixed Rust-only parser/initial-graph tax; the old small-int rejection is superseded, but parser-only work should still wait unless it reduces the hot live set or matches C representation directly |
| app update/rebuild traffic is central | partial | app-site profiling shows normal combinator rewrites dominate app allocation; S1 stack removes eager rethreading on the hot path, unchecked arg loads remove another eval.c-missing check from the hot rewrite arms, app-continuation descent avoids returning to the outer driver after app-producing rewrites, direct `ap`/`ap2` root pushes delete generic descent work on known app results, the dedicated app-allocation path removes generic app dispatch from the dominant node kind, compact profile-head frames remove a string-option payload from default strict-frame traffic, authoritative cells remove the old hot `Node` tag/payload layer, fixed-arity `CHKARG` pop/take plus direct app/free writes remove another generic stack/update layer, strict `Int` redex-owning frames remove the `app_end/used` replay layer for the hottest strict frame kind, WHNF redex-owning frames remove the same replay/allocation layer for `seq`/`isint`/`IO.strict`, raw tag/trusted free-list accessors remove another representation/invariant-check layer from App/Prim/scalar/resolve/free-list traffic, unified `ret`/`top` continuation removes another driver handoff around strict-force and `GOIND` returns, allocator free-head invariant removes the remaining release `Option` branch from reused app allocation, identity-alias `GOIND` removes another reduced-step handoff, one-word app-fun descent removes the arg-word load from the AP walk, trusted stack redex/update access removes the impossible-error `Result` layer from hot app rewrites/updates, permanent compound caches remove C-avoided helper app construction for `fst`/`snd`/`Just`/`Pair Unit`, app-allocation profiling-bookkeeping cold split and inline primitive decoding recover source-level PGO/codegen debt, release `panic=abort` removes unwind scaffolding, and the trusted-WHNF/app-fun/hot-cell series removes more Rust-only checked/control layers from the hot reducer. Latest accepted full gate is 74.668s, but profiling still puts the next mutator buckets at app allocation, inner descent, arg reads, and app updates |
| persistent spine needs one owner | replaced on hot path | the accepted S1 stack is a replacement loop with explicit app-base tracking; the stale `PersistentSpine` dispatcher is unreachable and tempting cleanup, but deleting it alone regressed the byte-identical full gate to 88.171s, so leave it until a perf-neutral cleanup/outline proves out |
| primitive heads want better representation | yes for current shape | parse-time primitive interning, compact `RuntimePrim(u16)`, in-cell primitive tags, and direct strict-action dispatch are implemented; fallback helper dispatch still uses names, but only on the cold/non-strict helper path |
| GC is a performance feature | partial | S2 closes arena growth, mark-time IND compression, reuses the GC mark-work stack, can profile mark vs sweep behind `gc-phase-profile`, no longer keeps parse labels as permanent roots, now runs C-style ForeignPtr finalizer records during sweep, rewrites App child slots during mark to resolved/cached targets like eval.c's pointer-to-slot mark path, and implements eval.c-style weak key/value clearing through a weak side list; S1 stack plus authoritative cells/compact stack entries/trusted args/app-only eval stack, strict Int immediate args/redex-owning frames, WHNF redex-owning frames, raw cell tag/free-list accessors, unchecked stack arg loads, C-style app-continuation descent, direct `ap`/`ap2` app-result descent, dedicated app allocation, compact profile-head frames, stack scalar `SET*` writes, fixed-arity `CHKARG` pop/take/direct app/free writes, parse-time small-int sharing, cfg-gated profiling cleanup, unified `ret`/`top` continuation, parse-label non-root treatment, allocator free-head invariant tightening, identity-alias `GOIND` continuation, one-word app-fun descent, mark-time app-edge canonicalization, trusted stack redex/update access, permanent compound roots, app-allocation profiling-bookkeeping cold split, inline primitive cell decoding, release `panic=abort`, and trusted reducer cell reads put the best default full gate at 74.668s; GC pause is still about 23.445s on that gate, so GC remains material; the weak-pointer slice also proved the negative form of Lennart's design: a whole-arena weak scan regressed the full gate to 87.293s, while the side-list version came back to 82.185s; `gc-phase-profile` GCRED opportunity counters found zero eval.c mark-reduction opportunities on the 100M self-host slice, so broad GCRED should wait for a workload/counter that justifies it; S4 young-region counters showed old-to-young remembered-set pressure is tiny, but the actual allocation-set minor collector prototype lost badly: best 100M was 3.189s versus 2.436s default, and full self-host regressed to 113.490s despite lower high-water and 22.471s GC pause. Do not retry that per-allocation/per-write-barrier/free-slot design; the later dense free-slot stack probe also regressed 10M/100M because side-stack fill/pop cost outweighed any sweep/write simplification. Future GC work needs a lower-tax region/bump or otherwise different shape; scheduler-backed weak finalizer spawning and full GCRED semantics are still open |

## Current Theory

The remaining self-host blocker is now mechanical throughput, not correctness
or lost sharing. `NOTES.md` measured C at 48.2s for 3,651,953,898 reductions
and Rust at 707.8s for 3,659,075,092 WHNF steps before the prefix/S2 work:
the work count matches, while cost per step is far higher. App-site profiling
also says allocation is mostly normal combinator rewrite payloads, so narrow
local helpers are usually the wrong level unless they remove real eval.c-missing
driver work. S2 closes the heap growth loop and mark-time indirection
compression proves GC is still a useful performance feature. The S1
single-stack hot path is the first replacement-loop slice that changes the
slope: compact cells, compact stack entries, trusted stack args, static runtime
fallback names, the app-only eval stack, strict Int immediate args, unchecked
eval.c-style stack arg loads, C-style app-continuation descent, direct
`ap`/`ap2` app-result descent, dedicated app allocation, compact profile-head
frame payloads plus reusable GC mark-work and feature-gated GC
phase/profile-overhead profiling, stack scalar `SET*` writes, the authoritative
cell arena, fixed-arity `CHKARG` pop/take/direct app/free writes, strict
`Int` redex-owning frames, WHNF redex-owning frames, raw cell tag/trusted
free-list accessors, and post-cell parse-time small-int sharing now self-host
with byte-identical output, F14 ForeignPtr finalizer records brought the best
observed gate to 90.237s, the S1 unified stack `ret`/`top` continuation brought
it to 89.141s, the S2 parse-label non-root slice brought it to 88.623s, and
the S1 allocator free-head invariant tightening brought it to 86.619s, the
S1 identity-alias `GOIND` continuation brought it to 86.500s, the S1
one-word app-fun spine descent brought it to 85.189s, S2 mark-time
app-edge canonicalization brought it to 83.752s, S1 trusted stack
redex/update access brought it to 82.154s, eval.c permanent compound
caches brought it to 80.927s, the app-allocation profiling-bookkeeping
cold split brought it to 80.584s with an 81.336s repeat, inline
primitive cell decoding brought it to 80.198s with an 80.309s first gate,
release `panic=abort` with checked LZMA decode brought it to 77.387s,
trusted WHNF/app-fun/hot-cell reads now bring it to 74.668s.
The parser-small-int retest reduces the parsed
graph from 160,961 to 153,185 cells and full high-water from 40,014,305 to
40,006,682 cells without the old 100M regression.
The latest accepted full gate
has about 23.445s GC pause, so the next stretch is still a mix of mutator mechanics and
material GC pause. S0
layer profiling previously showed strict-redex snapshots
copying 12.0M app ids and remaining-spine scans inspecting 30.1M app ids per
1M steps; the accepted stack slice drives both to zero. The fallback
primitive-helper move cuts old `EvalSpine` bridge hits from 3,281/1M to 90/1M.
The compact-entry split confirms the main remaining cost is still mechanical,
but the newest stack counters correct one stale reading: 263.1M is arity
histogram mass, while real stack descent and arg work is about 3.0M app pushes
and 3.0M arg reads per 1M steps. Stack app-result updates already reuse redex
cells and allocate zero apps, and fallback rethreading moves only 11 app cells
per 1M steps. The latest `eval-phase-profile` refresh before the free-head slice
keeps the same shape on the new machine. A 10M feature-build control now takes
`331.007ms`, while the fully profiled run with per-head/per-site sub-buckets
takes `10279.438ms`; the useful sub-buckets inside `stack_eval_step` are app
allocation `394.847ms`, inner descent `325.762ms`, arg reads `199.594ms`, app
updates `164.257ms`, rewrites `36.200ms`, and force-frame pushes `6.988ms`.
After the one-word app-fun slice, the fresh 10M feature-build control is
`324.987ms`, while the fully profiled run takes `9256.176ms`; the useful
sub-buckets are app allocation `346.997ms`, inner descent `301.984ms`, arg reads
`183.331ms`, app updates `147.863ms`, rewrites `33.338ms`, and force-frame pushes
`6.713ms`. A current-tree default profile control before trusted stack
redex/update access took `311.419ms` for 10M normal and `2007.345ms` profiled,
with 11,215,900 app allocations, 8,292,661 stack app updates, 30,119,444 stack
arg reads, and the same 90 fallback eval-loop steps; the trusted update slice
then moved the same 10M gate to `295.019ms`, the 100M gate to `2.371s`, and the
byte-identical full gate to `82.154s`.
The eval.c permanent compound-cache slice then reuses C-style `fst`, `snd`,
`Just`, and `Pair Unit` compound prefixes as GC roots and moves the
byte-identical full gate to `80.927s`.
The app-allocation profiling-bookkeeping cold split then moved the current
default-release gate to `80.584s` with an `81.336s` repeat, the inline
primitive cell decoder moved it again to `80.198s` with an `80.309s` first
gate, release `panic=abort` with checked LZMA decode moved it below 80s
to `77.387s`, and trusted WHNF/app-fun/hot-cell reads move it to `74.668s`;
these are source-level/default-build codegen-debt and eval.c trust-rule
recoveries rather than new graph-shape wins.
The latest stack-head arity profile sharpens the F2 target: hot B/C/S-family
heads are overwhelmingly over-applied under long outer spines (`B@extra`
1.94M, `C@extra` 1.56M, `S'@extra` 0.81M per 10M profiled steps), while
exact-arity hits are small (`B@exact` only 3,030). The rejected direct
`B (B a b) y z` superinstruction confirms the warning: even a semantically
clean two-reduction shortcut can look plausible on 100M and still lose the
full self-host gate when it only matches direct nested heads. The next useful
shape probably has to consume the outer app context more like eval.c's loop or
remove broader app allocation/descent/update work, not add another narrow hot
branch. The follow-up continuation-transition profile keeps that conclusion:
long-spine B/S' self-transitions exist but are only tens of thousands per 10M
steps, while strict primitive returns to `Int@0` dominate the continuation
table. That points first at eval.c's strict-argument detail of following
already-resolved `T_IND` chains for immediate int arguments before installing
another strict frame. That standalone probe then lost the full gate, and the
app-allocation site-shape profile adds the next boundary: B/C/S-family app
sites dominate, but their operand shapes are spread across generic `App`,
`Indir`, and scalar forms rather than one repeatable special shape. The next
F2 candidate should therefore remove a general layer from app allocation,
redex update, or descent, not add another direct operand-pattern branch.
The post-cell `gc-phase-profile` 100M
slice shows 268.773ms total GC pause split into 86.890ms mark and 153.969ms
sweep; the accepted `CHKARG` 100M slice has 268.580ms total GC pause, so GC
remains material but not newly worsened. The eval.c-style free bitmap follow-up proved
sweep reduction is not a standalone win in this Rust layout. The app-only eval-stack slice, unchecked
arg-load slice, app-continuation slice, direct `ap`/`ap2` root-push slice, and
dedicated app-allocation slice validate the eval.c direction again: app stack
entries should be app ids, reads from them should trust the stack invariant,
known app-result shapes should avoid generic descent, app allocation should not
pay generic node-allocation dispatch, fixed-arity hot rewrites should pop
the stack segment before committing the graph rewrite, and strict `Int`/WHNF
frames should own the popped redex instead of replaying `app_end/used` on
return. The raw-tag/free-list slice validates the same direction at the cell
boundary: trust the low tag word and internal free-list invariant in the hot
path, keeping corruption checks in debug rather than default release. The
free-head invariant slice extends that same rule to the remaining release
`Option` branch in `pop_free_node`; the 100M repeats improved to 2.450s/2.463s,
and the full gate improved to 86.619s. The identity-alias `GOIND` slice
extends the accepted `ret`/`top` continuation to `I`/`Ord`/`Chr`: the 10M
phase profile cuts stack loop iterations from 613k to 281k and reduced exits
from 484k to 211k, but the full gate only nudges to 86.500s because app
allocation/update/descent and GC pause still dominate. The one-word app-fun
slice proves the descent bucket is still real: changing the hot AP walk to load
only the fun/tag word, like eval.c, moved the byte-identical full gate to
85.189s even though the 100M bounded slice was not a clean win. The app-edge
canonicalization slice proves the GCRED/mark-slot family is still real too:
rewriting App child slots during mark to resolved indirection targets and cached
small-int nodes cuts high-water to 38.8M cells and moves the full gate to
83.752s. Trusted stack redex/update access then removes the impossible-error
`Result` layer from hot app rewrite/update helpers and moves the full gate to
82.154s. Permanent compound caches then copy another eval.c representation
choice for `fst`/`snd`/`Just`/`Pair Unit` helper prefixes and move the full gate
to 80.927s. The direct `GOPAIR` runtime-return probe then lost the full gate
at 82.600s and raised high-water to 39.5M cells even though the 100M
cold/noinline slice looked competitive, so the remaining helper return path is
not a standalone lever. A fresh GC-window check says the same for just widening
the allocation interval: 64M reduced the bounded 100M run to one GC, but the
larger arena made the slice slower at 2.978s, so 32M stays the current gate
default. A direct `Prim::Known` stack-dispatch probe removed the common
`EvalHead` enum shape for known combinators and won a noisy 100M slice, but the
byte-identical full gate regressed to 85.951s with 25.135s GC pause, so that
dispatch enum is not the remaining F2/S1 lever either. Extending the same idea
indiscriminately to root/stable slots and cold payload children added mark
overhead without another heap-shape win and regressed the full gate to 88.717s,
so further GCRED work needs counters or a more selective parent-slot model. Several
post-raw-tag micro-probes sharpened the boundary: numeric known-head dispatch,
single-match known-head dispatch, direct in-cell hot known-code dispatch, raw
GC mark/sweep tag loops, and eval-time first-indirection compression all
regressed and were reverted; the compression retest was byte-identical but lost
the full gate at 98.036s. The post-app-fun raw free-head sentinel stayed
noise-band on 100M and was reverted before full. A broader keep-redex app-stack
continuation probe matched eval.c's stack-pointer idea by leaving the rewritten
`GOAP` redex on the app stack instead of popping then pushing it; it improved
the no-GC 10M slice and stayed competitive at 100M, but the byte-identical full
gate regressed to 86.813s with higher GC pause, so the accepted pop/push shape
stays accepted. Moving the remaining `StackStep::Whnf`
strict-frame finish/rethread path into `stack_eval_step` likewise matched
eval.c's `RET` shape and won a noisy 100M slice, but the byte-identical full
gate regressed to 85.593s with 25.806s GC pause, so the outer WHNF handoff also
stays for now. The F14 result is not a new mutator strategy, but it
does validate moving C's non-graph runtime state out of ordinary node roots:
ForeignPtr finalizers now live in a sweep-owned side table, not as Haskell nodes
marked from every live ForeignPtr.
The rejected direct-inner-`GOAP2`
probe sharpens that boundary: pushing the known inner app helped 1M/100M slices
but lost the full gate, so the next change needs to remove a broader
rewrite/update/allocation layer rather than specialize one more app-result hop.
The accepted `CHKARG`, strict-`Int` redex, and WHNF redex slices are exactly
such layer removals. The remaining gap to C is still inside the
combinator/strict rewrite
arms and their update/allocation shape, plus still-material GC pause, not in
lost sharing, raw spine descent, or free-list representation alone. A direct
independent-app-pair batching probe preserved allocation order
but regressed 1M to 46.637ms and 100M to 3.433s, so batching allocator calls is
another local code-shape probe unless the cell/update representation changes.
The
post-app-allocation probes keep the same boundary: trusting only the hot
`apply_stack_app` redex lookup won 1M and lost 100M, and a smaller 16M GC window
won 100M but lost the full gate badly because 250 collections pushed GC pause
to 51.8s. App-stack `Vec::set_len` pops and fixed node-arena pre-reservation
also won only short/noisy slices and lost the accepted 100M band. Forcing the
Rust enum discriminant to `repr(u8)` did not change `Node=16` and lost the
100M band, so a real cell/tag rewrite is needed if we go after representation
again. A compact marker-word app stack was the first post-acceptance structural
F2 probe, but keeping frame payloads side-stored while adding markers to the
app stream regressed the full gate to 122.8s and raised GC pause to 30.2s. That
says the remaining single-stack work has to remove a real return/frame layer,
not merely encode the existing split stack with marker words. The first staged
cell/tag attempt made the same mistake at the heap level: a synchronized
12-byte App/Indir/Free side-cell arena moved hot link reads but kept the
16-byte `Node` shell, so 100M regressed to 4.43s and GC pause more than
doubled. A real cell tier must replace the hot arena or wait; mirroring it is
just extra cache pressure. A direct `Node::{Known,Runtime}` primitive-tag split
also lost the 100M slice, so top-level enum reshuffling is not the missing C
tag-word win. The current tree has 71 direct `Node::App` /
`Node::Indir` / `Node::Free` sites across parse, GC, render, IO helpers, and the
stack evaluator. That is small enough for a deliberate authoritative-cell
conversion, but too broad for a safe helper-only cleanup. The next credible
representation slice is therefore a real two-word `RawCell`/tag-word arena
where app, indir, free, primitive, scalar, and cold side-object nodes are decoded
from one authoritative cell. Do not repeat a parallel side table, and do not
expect a `Node` facade/helper pass to move the benchmark. A broad
default-build profile-gating probe won 100M slices but regressed the full gate
to 152.7s, so code-shape/profile-branch deletion is not enough by itself. The
current stack scalar `SET*` retest is accepted because the full gate stayed
byte-identical and nudged the default best to 118.739s, but its bounded and
full results are noise-band; keep it as eval.c-aligned update discipline, not
as evidence that more standalone scalar rewrites are the path below 90s.

## Rejected Narrow Probes

Only keep these as "do not retry alone" markers:

| probe | result |
|---|---|
| dead persistent-dispatcher deletion | removed unreachable `PersistentSpine`/`persistent_eval_step`/old `EvalFrameStack`/`whnf_frames=true` code after verifying no current caller reaches it; 10M/100M stayed plausible at 328.314ms/2.415s and the full gate was byte-identical, but wall time regressed to 88.171s with 25.161s GC pause versus the 80.927s best, likely release codegen/layout fallout from deleting ~1.7k lines; reverted, post-revert bounded sanity was 311.608ms/2.398s |
| runtime strict-action noinline split | moved `RuntimePrim::strict_action` out of the stack evaluator with `#[inline(never)]` to test the PGO/code-layout thesis without touching B/C/S rewrites; bounded slices looked good at 10M 280.325ms and 100M 2.200s / 222.790ms GC, but the byte-identical full gate regressed to 86.214s with 26.095s GC pause versus the 80.927s best and 82.x current band; reverted, release bench rebuilt, post-revert 100M was 2.295s / 229.418ms GC |
| lazy runtime fallback-name lookup | made fallback-name lookup lazy for runtime primitives, removing eager `runtime.name()` calls on strict actions. Checks passed and isolated 8x 100M A/B looked favorable at base avg 2055.140ms vs candidate avg 2015.378ms (`-1.93%`) with identical steps/heap/sink, but the byte-identical full gate was slightly slower at 74.911s versus the 74.668s `d5efc37f` best; reverted |
| trusted descent Result split | split `descend_stack_from` and the outer stack reducer descent so the non-profile main path called `resolve_whnf_trusted` directly instead of the `Result`-returning `resolve_for_whnf`; fmt/check/tests passed, but 100M regressed clearly to 2.521s / 39.99M steps/s with 246.868ms GC pause versus the 2.300s same-session control, with identical steps/heap/sink. Reverted and rebuilt; restored 100M returned to 2.300s / 43.84M steps/s / 227.270ms GC |
| direct cached primitive accessors | replaced hot literal `self.prim("K")`/`"B"`/`"C"`/`"I"`/`"IO.>>="`/boolean calls with direct cached accessors for eval.c-style permanent combinators. Isolated 8x 100M A/B looked mildly favorable at base avg 2211.785ms vs candidate avg 2183.147ms (`-1.29%`) with identical steps/heap/sink, but byte-identical full gates were 74.760s and 74.676s versus the 74.668s committed best and slightly perturbed full step/high-water counts; reverted |
| constructor-time immediate I/K/A redex compression | broad app-construction compression of `I x`, `(K x) y`, and `(A x) y` is not byte-shape safe: the existing partial-arity test caught `((B (I I)) 9)` changing to `((B I) 9)`. Narrow K/A-only compression preserved tests but regressed bounded self-host to 370.005ms at 10M and 2.585s at 100M while saving too few steps, so no full gate was run; reverted, release bench rebuilt |
| app-taken immediate function-redex shortcut | restricted I/K/A compression to `app_taken` results that are immediately evaluated in function position, with budget and self-indirection guards; tests passed, but 10M and 100M step counts were unchanged while wall time regressed to 381.639ms and 3.011s, so the direct-pattern shortcut missed the resolved opportunities and only added hot-path branch cost; reverted, release bench rebuilt |
| direct B/B superinstruction | fused direct `B (B a b) y z` to `a (b (y z))` in the hot `B` arm; 10M/100M looked plausible at 290.630ms and 2.265s / 222.118ms GC, but the byte-identical full gate regressed to 79.157s with 23.604s GC pause versus the 77.387s/77.918s current band, so reverted. The arity profile says most useful B/C/S work sits under long outer spines, not direct exact-shape nested heads |
| raw app-stack push | replaced `Vec::push` in `EvalStack::push_app` with an inline raw pointer write plus cold grow path to mimic eval.c's stack pointer; correct on fmt/check/tests, but bounded self-host regressed to 317.355ms at 10M and 2.566s / 244.617ms GC at 100M, so reverted before full. The app-stack cost is not isolated `Vec::push` capacity handling |
| batched app-allocation accounting | charged known multi-app rewrites once before using a pre-accounted app allocator, mirroring eval.c `GCCHECK(k)` at the counter level; 100M looked slightly plausible at 2.295s / 219.257ms GC, but the byte-identical full gate regressed to 78.758s versus the 77.387s/77.918s current band, so reverted. Counter batching alone is not the missing eval.c allocation protocol |
| B-I stack rewrite specialization | added `B x I z -> x z` in the hot stack `B` arm, matching eval.c's GCRED identity in mutator form; tests passed and 100M removed 1,665 steps, but wall time regressed to 2.639s versus the accepted S-I band, so the extra hot branch/code shape cost beat the tiny graph-work saving; reverted, release bench rebuilt |
| IO.return K continuation specialization | added `IO.return x world K -> x` in the hot stack `IO.return` arm after the post-S-I site profile showed 23,241 `red_k@IO.return.kx` opportunities per 10M steps. It saved the expected 10M heap traffic and bounded 10M ran in 277.055ms, but 100M was mixed and the byte-identical full gate regressed to 88.467s / 41.36M steps/s / 25.405s GC pause; reverted |
| C' direct identity specialization | added direct `C' x I z w -> x w z` after the post-S-I site profile showed 34,519 `red_i@C'.yw` opportunities per 10M steps. Direct matching only saved 18 cells at 10M and 37 steps at 100M, then 100M regressed to 2.456s; reverted without full gate. The useful opportunities are resolved/indirected, not direct primitive arguments |
| direct inner `GOAP2` descent | pushed both the rewritten root app and a freshly allocated inner app for clean `GOAP2` shapes; the old retest won 1M/100M but missed full at 131.8s/133.2s versus the then-best 131.6s, and the current compact-cell retest again won bounded 100M (`2.369s`) but regressed the byte-identical full gate to `86.784s` versus the 80.927s best, so broad direct `ap2` pushing stays rejected alone |
| guarded stack `GOIND` continuation | let K/A/KK/KA/K2/K3/K4 indirection rewrites continue inside `stack_eval_step` unless directly on a strict-frame boundary, matching part of eval.c's `GOIND(x); goto top`; first unguarded 10M hit `expected Int` because Rust's strict-frame `ret:` handling still lived in the outer loop, guarded 10M was 301.375ms, 100M was 2.577s, but the byte-identical full gate regressed to 97.466s with 28.303s GC pause versus the 90.237s best, so reverted. Superseded by the accepted unified `ret`/`top` slice, which moved ready strict-frame return handling into the stack loop and completed in 89.141s |
| numeric known-head stack dispatch | carried raw `u16` known-prim codes through `stack_eval_step` to avoid `KnownPrim` enum decode and match closer to eval.c's integer tag switch; 1M regressed to 39.156ms and 100M to 2.667s versus the fresh 37.802ms / 2.619s same-session baseline, so reverted; post-revert 1M sanity was 36.604ms |
| single known-head match dispatch | merged `IO.strict`/`seq`/`isint` into the main `KnownPrim` match to remove one known-head match on hot B/C/S reductions; same-session 100M regressed to 2.561s versus the 2.533s control, so reverted before a full gate |
| direct in-cell hot known-code dispatch | added a direct code fast path for the top B/C/S/S'/C'/C'B app-producing heads before `EvalHead` construction; same-session 100M regressed to 2.690s versus the 2.533s control, so reverted before a full gate |
| fun-only app descent helper | changed the inner and outer stack descent loops to use a helper returning only the app fun pointer instead of `app_fields`; same-session 100M regressed to 2.567s versus the 2.525s control, so reverted before a full gate |
| raw GC mark/sweep tag loop | changed mark/indirection compression to raw tag tests and inlined sweep free-list rebuild; `gc-phase-profile` 100M regressed from 230.779ms pause (73.508ms mark / 128.855ms sweep) to 236.916ms pause (73.721ms mark / 135.083ms sweep), so reverted |
| eval-time first-indirection compression after compact cells | made `resolve_for_whnf` short-circuit the entry indirection like eval.c; 1M regressed to 38.671ms, 100M was mixed at 2.556s then 2.618s, and the byte-identical full gate regressed to 98.036s with 27.551s GC pause, so reverted |
| FFI/JS arity-capped stack materialization | capped FFI/JS argument copying to declared arity (`ffi_arity + world`, JS tags, JS wrapper pair), dropping 10M `profile_arg_materialized_nodes` from 10,522,021 to 150,988; the current retest completed byte-identically in 83.510s with 23.937s GC pause but still missed the 80.927s compound-cache best, and an older full gate had regressed to 100.762s with 28.113s GC pause, so reverted |
| stack `Bytes` redex-owning frame extension | extended the accepted strict `Int`/WHNF redex-frame pattern to `Bytes` binops; 1M regressed to 39.538ms, 100M was competitive at 2.580s, but the byte-identical full gate was 92.679s versus the parser-small-int 91.598s repeat / 91.503s best, so reverted |
| broad shared-buffer `Node::Bytes` for F17 | changed every immutable `Bytes` payload to an `Arc`/`Rc` shared-buffer view with COW writes; correct on tests but 100M self-host regressed to 2.67-3.13s and the byte-specific rows did not justify perturbing every bytes node, so narrowed to a separate cold `BytesView` node |
| `target-cpu=native` release build | separate-target release build with `RUSTFLAGS='-C target-cpu=native'` gave 1M 47.427ms and 100M 3.504s / 28.77M with 310.2ms GC, worse than the default rebuild band, so codegen flags are not the current lever |
| trusted app-result update redex lookup | made only `apply_stack_app` trust the app stack like `ARG(TOP(i))`, leaving frame rewrites and rethreading checked; 1M improved to 46.220ms, but 100M regressed to 3.501s versus the accepted 3.305s app-allocation slice, so reverted |
| trusted hot cell accessors | changed private hot `cell`/`cell_at` and app/free setters to debug-checked unchecked release indexing; 100M was mixed at 2.375s / 2.462s / 2.485s and full was byte-identical but regressed to 86.518s with 26.072s GC pause versus the 86.500s identity-`GOIND` best, so reverted |
| redundant high-water field removal | removed the separate `gc_high_water_nodes` field/update because arena length is monotonic and high-water equals `nodes.len()`; 100M regressed to 2.739s then 2.481s versus the 85.189s app-fun baseline band, and restore returned to 2.460s, so reverted before a full gate |
| raw free-head sentinel | replaced `free_head: Option<NodeId>` with a raw `u32::MAX` sentinel to avoid option packing/unpacking in the intrusive free list; 100M was noise-band at 2.484s / 2.458s versus a restored same-session 2.473s control, so reverted before a full gate |
| keep-redex app stack continuation | kept the consumed redex app slot on `EvalStack.apps` for all `GOAP`-style app-producing rewrites instead of truncating then pushing it again; 10M improved to 294.551ms and 100M was competitive at 2.451s, but the byte-identical full gate regressed to 86.813s with 26.285s GC pause versus the 85.189s best; restored 100M was 2.427s, so reverted |
| WHNF-frame return inside `stack_eval_step` | moved the remaining strict-frame WHNF return/rethread path into the stack loop instead of returning `StackStep::Whnf` to the outer driver; 100M improved to 2.375s versus a 2.413s control, but the byte-identical full gate regressed to 85.593s with 25.806s GC pause versus the 85.189s best; post-revert 100M was 2.432s, so reverted |
| broader GC slot canonicalization | extended the accepted App-edge mark-time canonicalization to program root/stable slots and cold `Weak`/`MVar`/`BytesView`/`Array` child slots; output stayed byte-identical, but 100M regressed to 2.498s with no extra heap-shape win, full regressed to 88.717s with 25.603s GC pause versus the 83.752s App-edge-only best, and post-revert 100M returned to 2.400s, so reverted |
| 16M GC interval after app allocation | 100M solo improved to 3.265s / 30.87M steps/s, but the full gate regressed to 151.8s with 250 GCs and 51.8s GC pause versus the 120.708s 32M best, despite byte-identical output |
| app-stack `POP` via `Vec::set_len` | replaced hot `Vec::truncate` calls with a C-like stack-pointer decrement helper; 100M only moved to 3.433s/3.441s versus a noisy 3.488s same-session control and stayed slower than the 3.305s accepted app-allocation slice, so reverted without a full gate |
| fixed node-arena pre-reserve | added and tested an opt-in `MHS_NODE_RESERVE=41943040` to avoid Vec growth/copying; 1M improved to 43.641ms but 100M regressed to 3.517s, so removed the config hook |
| `Node #[repr(u8)]` discriminant hint | forced the Rust enum discriminant width; `Node` stayed 16 bytes, 1M was 46.321ms, and 100M regressed to 3.482s / 28.95M steps/s, so reverted |
| compact marker word in app stack | encoded frame markers in the same 4-byte stack stream as app ids while keeping frame payloads side-stored; 100M was only a same-session noise win and the full gate regressed to 122.8s with 30.2s GC pause, so reverted |
| parallel App/Indir/Free cell shell | synchronized a 12-byte side-cell arena with the existing 16-byte `Node` arena and moved app/indir/free hot reads to it; 1M regressed to 61.2ms and 100M to 4.43s with 640ms GC pause, so reverted before a full gate |
| raw app allocation when profiling is disabled | attempted to bypass `app_with_site` in normal app-site rewrites; 1M self-host regressed from ~65.1ms to 68.1ms, likely from extra branch/codegen shape, so reverted before 100M/full gate |
| stack `GOAP` continuation return | 100M slice improved to 5.33s, but full self-host regressed from 192.4s to 196.7s despite byte-identical output; this was the return-to-driver variant, not the accepted in-`stack_eval_step` app-continuation descent |
| strict-force continuation inside `stack_eval_step` | after app-continuation, made strict marker pushes descend/evaluate inside `stack_eval_step` with an internal ready-frame check; 100M was neutral at 3.759s, but full self-host regressed from 137.6s to 138.1s despite lower GC pause and byte-identical output, so reverted |
| stack-only scalar redex overwrite retry | after S1/compact cells/arg binding, direct stack-frame scalar writes regressed the 1M self-host slice to 66.2ms; post-revert sanity was 61.2ms, so reverted before 100M/full |
| strict `Int` first-immediate carry frame | stored an already-WHNF first operand as an `i64` while forcing the second operand; 10M profile cut `Int` frame pushes from 504,396 to 328,117 and 500M was neutral, but the full self-host gate regressed from 143.9s to 145.6s with byte-identical output, so reverted |
| always-on GC phase timing | mark/sweep timing is useful, but keeping the timing fields and `Instant` calls in the default build regressed the byte-identical full gate to 154.8-154.9s; keep it behind `gc-phase-profile`, where the 100M slice shows 72.7ms mark and 190.9ms sweep |
| eval.c-style free bitmap allocator | reduced 100M GC pause to 266-268ms and full GC pause to 28.1s, but full self-host regressed from the 143.7s best to 150.6s; a fast-pop retry still lost 100M at 4.27s, so restored the intrusive `Free(next)` list with the `free_nodes == 0` allocation fast path |
| outer GC safe-point call guard | skipped `maybe_collect_garbage_between_steps` until allocation pressure reached the GC interval; 1M moved only 52.15ms -> 51.81ms and 100M regressed/noised to 4.14s versus the restored 4.10s reading, so reverted |
| trusted stack update entries | changed `stack_entry_app`, stack app rewrites, and rethreading to trust app entries like stack arg loads; 1M was flat, but 100M regressed to 5.02s and 4.86s versus the accepted 4.46s/4.62s band, so reverted |
| forced inline stack step | `#[inline(always)]` on the large stack evaluator left 1M flat but regressed 100M to 4.91s, so the `StackStep` cost needs a real loop refactor rather than an inlining hint |
| direct stack head dispatch | replaced stack `EvalHead` classification with direct node-tag matching; 1M regressed to 69.1ms, so the local enum round-trip is not the isolated problem |
| single known-head match plus preserved free list | merging `seq`/`IO.strict`/`isint` into the main `KnownPrim` match and preserving the existing free-list across GC looked neutral on 1M/100M, but the full gate regressed to 179.3s with byte-identical output, so reverted |
| const-specialized profile vs normal stack step | making `stack_eval_step` const-generic over profile mode looked neutral on old 1M/100M slices but regressed the old full gate to 175.0s; the current compact-cell retest won 10M at `283.594ms` but was not a clean 100M win and regressed the byte-identical full gate to `84.112s` versus the 80.927s best, so reverted again |
| broad eval-profile feature gate | compiling general profile checks out of default builds improved 100M slices to 3.77s/3.78s, but the full gate regressed to 152.7s with 33.7s GC pause and byte-identical output, so reverted |
| eval.c `GOPAIR` runtime-return shape enum | changed runtime/FFI helpers to return `Node`/`Pair`/`UnitPair` shapes so the stack path could overwrite the consumed redex as the outer pair app; the unoutlined 100M slice regressed to 2.772s, cold/noinline recovered 100M to 2.367s, but the byte-identical full gate regressed to 82.600s with 39.5M high-water versus the 80.927s compound-cache best, so reverted |
| focused stack app-site profile split | bypassed `app_with_site` from the stack hot path when profiling was disabled; 10M regressed to 340.971ms, 100M was only noisy at 2.422s, and full self-host regressed to 85.721s versus the 82.154s trusted-stack best, so reverted |
| unused stack arg loads in discard-heavy combinators | changed `K`/`A`/`Z`/`J`/`L`/`KK`/`KA`/`O`/`K2`/`K3`/`K4` arms to read only used args instead of batched args; 1M regressed to 53.4ms and 100M was only noise-band at 4.06s, so reverted without a full gate |
| app-stack argument cache | stored App args beside App ids on `EvalStack` to avoid later `ARG(TOP(i))` loads; 100M regressed to 2.908s / 34.67M steps/s with 294.369ms GC versus the slow restored 2.632s / 38.30M sanity, so reverted without a full gate |
| eval.c `GOAP2` allocation order | reordered independent `S'`/`B'`/`C'B` inner app allocations to match eval.c's textual `GOAP2` order more closely; 10M regressed to 385.102ms, 100M was only noise-band at 2.428s, and the byte-identical full gate regressed to 82.827s versus the 80.927s compound-cache best, so reverted |
| low-bit app tag encoding | changed `Cell.word0` to encode App as low bit `0` and all non-app tags as low bit `1`, reducing app-fun extraction from byte-tag/`>>8` to one-bit/`>>1`; after fixing the primitive tag decoder, tests passed and output was byte-identical, but 10M regressed to 396.531ms, 100M to 2.498s, and full self-host to 92.637s versus the 80.927s compound-cache best, so reverted |
| manual `Vec<u64>` GC mark bitmap | replaced the packed `Vec<bool>` mark map with direct word/mask operations; 100M improved slightly to 2.375s with 222.774ms GC pause, but the byte-identical full gate regressed to 84.578s with 25.491s GC pause versus the 80.927s compound-cache best, so reverted |
| low-to-high free-list reuse order | changed sweep to prepend dead cells in high-to-low order, making reused free cells come back low-to-high like eval.c's `free_map` scanner; the original 10M improved to 292.599ms and full GC pause fell to 23.120s on the repeat, but byte-identical full gates were 81.622s and 81.277s versus the 80.927s compound-cache best, so reverted. Current weak-pointer/tree retest again lowered GC pause but missed on wall time: 100M 2.523s with 217.310ms GC and full 84.969s with 24.580s GC, so reverted again |
| dense free-slot stack allocator | replaced the intrusive `Free(next)` list with a side `Vec<NodeId>` free stack while keeping high-to-low reuse order; tests/build passed and `.text` shrank slightly, but no-GC 10M regressed to 311.507ms, 100M regressed to 2.558s with 383.043ms GC pause and a 209.251ms first collection, while restored intrusive-list 100M was 2.318s with 222.631ms GC pause; reverted before full |
| direct fresh-App allocation helper | bypassed generic `push_cell` for fresh App nodes and assigned the monotonic high-water directly; bounded gates were plausible at 336.697ms/2.477s, but the byte-identical full gate regressed to 88.846s with 26.996s GC pause, so reverted |
| parsed primitive-cache seed | seeded the fixed runtime `PrimCache` from parser-interned primitive singleton nodes, saving three live/high-water cells and completing a byte-identical full gate in 82.228s with 24.050s GC pause, but still missing the 80.927s compound-cache best, so reverted |
| allocation-set minor GC | implemented a feature-gated non-moving minor collector with allocation-set tracking, remembered old sources, and lazy minor-free slots; the best 100M variant was still 3.189s versus the 2.436s S4 default sanity, and the byte-identical full gate regressed to 113.490s despite lower high-water and 22.471s GC pause, so reverted |
| standalone thin LTO plus single release codegen unit probe | earlier tried workspace `profile.release` `lto = "thin"` and `codegen-units = 1` as an isolated performance knob; 1M regressed to 54.0ms and 100M to 4.29s, so that standalone probe was reverted without a full gate. The setting is now accepted as part of the real-module source-organization checkpoint to erase codegen-unit/module-layout variance; full gates at `76ff9d3e` and `8933a6d1` stayed byte-identical and in the current 74-77s band |
| boxed profile head in strict frames | shrinking `profile_head` from `Option<String>` to `Option<Box<str>>` made normal 1M worse at 55.5ms and profile total worse at 216ms; 100M was only neutral at 4.26s, so reverted without a full gate |
| free-list pop as `NodeId` plus non-saturating counters | tried to reduce allocation-path overhead after app-only stack; 1M regressed to 54.3/54.0ms and 100M stayed noise-band at 4.36/4.33s, so reverted without a full gate |
| hot `StackStep::Continue` driver refactor | moved hot `Reduced`/`Force` step handling into `stack_eval_step`; normal 1M regressed to 56.5ms and 100M stayed noise-band at 4.31s, so reverted without a full gate |
| inline stack app update in hot macro | inlined `apply_stack_app` into `app_step_reductions`; 1M profile stayed noise-band at 53.5ms, but 100M ran 4.50s/4.39s versus the accepted app-stack band, so reverted without a full gate |
| preserve existing GC free list across sweep | isolated the free-list-preservation half of the older mixed probe; 1M no-GC slices regressed to 65.5/56.1ms and 100M regressed to 4.57s with 335ms GC pause, so reverted without a full gate |
| independent app-pair batch allocation | batched independent inner app allocations in the hot stack arms (`S`, `S'`, `B'`, `C'B`, and `IO.>>`) while preserving allocation order; 1M regressed to 46.637ms and 100M to 3.433s versus the accepted compact-profile-head 44.841ms / 3.370s band, so reverted without a full gate |
| compact `Vec<NodeId>` free stack | replaced the intrusive `Node::Free(next)` chain with a compact free-id vector to remove the reused-allocation tag load; 1M was flat/slightly worse at 54.4ms and 100M regressed to 4.35s with 404ms GC pause, so reverted without a full gate |
| batch two-app allocation helper | used the new per-head timing profile to batch independent two-app allocations in hot `S`/`S'`/`B'`/`C'B`/`IO.>>` stack arms; 1M was 57.5ms then 54.9ms and 100M was neutral at 4.18s/4.22s, so reverted without a full gate |
| larger GC windows | 100M self-host at 64M/128M allocation windows slowed to 5.51s/6.28s versus the 32M band; the larger arena/cache footprint beats the lower collection count |
| skipping outer-app rethreading / C-style continuation reuse | self-host jumped from milliseconds to seconds; resolve chains grew to 5,556 |
| first-extra-app compression only | fixed blow-up but did not improve self-host and mixed common rows |
| app-result-only tail reuse / guarded `App` writes | removed some writes, but branch/read cost or lost compression erased the gain |
| static `Prim` names for current runtime prims | scalar microbench win, self-host regression |
| evaluator first-indirection compression | fewer profiled indirections, slower wall time and self-host proxy |
| unconditional parser preallocation | helped self-host, regressed short inputs; keep the 64 KiB threshold |
| two-tier EvalSpine spill buffer | fixed heap-spine count, but 50k self-host slice slowed from 24.8 ms to 25.3 ms and proxy slowed from 40.4 ms to 41.9 ms |
| enum-variant arithmetic `Prim` tags | removed the string cascade but regressed 1M slice to 248.2 ms and full self-host to 770.1s |
| unfinished `StrictPrim` enum tag | abandoned after `NOTES.md` landed; it was another narrow tag-on-fat-node probe and should only return as part of compact cells/in-cell tags |
| sparse `NodeId -> PrimDispatch` side table | removed the same cascade but regressed 1M slice to 220.5 ms and full self-host to 731.3s |
| parse-time small-int interning before compact cells | old pre-cell probe shrank parsed nodes from 160,961 to 153,185 but regressed repeated 100M slices to 12.22s/12.60s; superseded by the post-cell retest, which keeps parsed small-int sharing and repeats the full gate at 91.598s |
| direct scalar redex overwrite | matched eval.c's `SETINT`/`SETDBL` idea locally, but regressed 100M self-host from 11.67s to 12.32s; keep it for the replacement loop |
| moved persistent-spine strict redex | avoided the app snapshot by moving the whole spine into the frame, but regressed 1M to 168.3 ms and 100M to 13.35s by losing the stable spine owner |
| Vec-backed persistent spine with head at the back | targeted long-spine locality but regressed 10M slice from 2,729.2 ms to 2,860.6 ms |
| strict-redex tail ownership | byte-identical, but regressed 100M slice to 15.31s and full self-host to 773.3s by losing persistent-spine buffer locality |
| segmented persistent strict stack | byte-identical, but active/inactive segment bookkeeping regressed 100M slice to 13.64s and full self-host to 619.3s |
| hybrid app/marker stack splice | no-rethread marker variant returned success after only 35,659 steps; marker-disabled variants were correct but regressed 1M to 286.8 ms / 241.6 ms; accepted replacement stack differs by owning dispatch, WHNF return, and explicit app-base tracking |
| direct stack WHNF/std-handle/control helper move | treating `IO.stdout`/`stdin`/`stderr` as primitive WHNF and copying `IO.getArgRef`/`catch` into the stack match made self-host stop after 3,973 steps; std handles need C-like runtime objects, not a shortcut |
| fat `Node::StdHandle` runtime-object variant | semantically removed std-handle bridge hits, but regressed 100M slices to 6.94s / 6.27s versus the 6.01s accepted helper-dispatch slice; wait for compact cells |

## 2026-07-05 PGO Refresh On Current Runtime

| item | result |
|---|---|
| setup | ran `rust/microhs-runtime/tools/native/build-selfhost-pgo.sh` with `MHS_PGO_DIR=/tmp/mhs-rust-pgo-8933-refresh`, full self-host training, `MHS_GC_NODE_INTERVAL=33554432`, `MHS_PGO_TRAIN_TIMEOUT=900s`; rustc `1.91.0`, LLVM `21.1.2`; current tree is `8933a6d1` plus uncommitted feature-gated stack-app-origin profiling diagnostics |
| default release sample 1 | byte-identical full self-host in `72.786s`, `3,659,074,983` steps, `50.27M` steps/s, `124` GCs, `23.977s` GC pause, high-water `39,192,428`, SHA `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`, `cmp_exit=0` |
| default release sample 2 | byte-identical full self-host in `77.637s`, `3,659,075,116` steps, `47.13M` steps/s, `124` GCs, `25.304s` GC pause, high-water `39,192,442`, same SHA, `cmp_exit=0` |
| PGO training run | instrumented full self-host completed in `94.023s`, `38.92M` steps/s, `124` GCs, `28.929s` GC pause; produced `/tmp/mhs-rust-pgo-8933-refresh/merged.profdata` |
| PGO-use sample 1 | byte-identical full self-host in `68.292s`, `3,659,074,926` steps, `53.58M` steps/s, `124` GCs, `25.782s` GC pause, high-water `39,192,422`, same SHA, `cmp_exit=0` |
| PGO-use sample 2 | byte-identical full self-host in `70.260s`, `3,659,075,059` steps, `52.08M` steps/s, `124` GCs, `26.231s` GC pause, high-water `39,192,436`, same SHA, `cmp_exit=0` |
| PGO delta | two-run default avg `75.212s`; two-run PGO avg `69.276s`; PGO is about `-7.9%` wall time on this noisy full gate. This is now the best measured execution mode, but still not the headline target because the target remains default `cargo build --release` parity with C `-O3` |
| profile signal | top internal block counts in the merged profile are still the evaluator core: `descend_stack_from` `8.013B`, `reduce_whnf_from_inner` `4.276B`, `stack_eval_step` `3.651B`, plus `write_args_head_order` `705M`, `mark_canonical_child` `500M`, `mark_reachable` `255.984M`, `finish_ready_stack_frame` `159.085M`, and `set_app_node_at` `107.667M` |
| codegen signal | PGO grows/rearranges the hot evaluator instead of merely shrinking cold code: `.text` grows from `1,112,534` to `1,122,622` bytes; `stack_eval_step` grows from `0x9c37` to `0x1397f`, `finish_ready_stack_frame` from `0x15bf` to `0x1bde`, `eval_loop_step` from `0x4328` to `0x6cca`, and `fallback_runtime_prim_rewrite` from `0x133c` to `0xa576`. Reading: the remaining PGO win is still mostly profile-guided inlining/layout around evaluator hot paths, not a simple cold-split lever |
| source probe from PGO info | tried a narrow no-op profiling-hook split: inline only the `self.profile.is_some()`/zero-count checks for `profile_shortcut` and fallback primitive-dispatch probes, moving HashMap/string work to cold noinline helpers. Verification passed `cargo fmt --check`, `cargo check`, `cargo check --features eval-phase-profile`, release bench build, 100M bounded sanity, and a full byte-identical self-host gate |
| hook-split probe result | 100M bounded sanity was `2.260s`, `44.60M` steps/s, `3` GCs, `233.602ms` GC pause; full self-host was byte-identical in `72.620s`, `3,659,074,926` steps, `50.39M` steps/s, `124` GCs, `24.355s` GC pause, high-water `39,192,422`, same SHA, `cmp_exit=0`. This is plausible but not accepted as a measurable source win without an alternated A/B, because same-session default full samples ranged from `72.786s` to `77.637s` |
| reading | PGO remains valuable as a compass. It still says the wall is evaluator core shape plus app/descent/update/arg traffic, and it exposes a small always-compiled profiling-probe cost. Do not ship PGO as the answer; use it to prioritize default-build source changes that remove hot calls/work or change the evaluator algorithm, then gate with full self-host and, where possible, alternated 100M A/B |

## 2026-07-05 GC Interval Probe From Revised NOTES.md

| item | result |
|---|---|
| setup | zero-code full self-host probe from `NOTES.md` rev 3, using the restored normal release binary after reverting the inconclusive profiling-hook source probe; only `MHS_GC_NODE_INTERVAL` changes |
| 32M same-session reference | default-release samples from the PGO refresh were byte-identical at `72.786s` / `23.977s` GC pause / high-water `39.19M` cells and `77.637s` / `25.304s` GC pause / high-water `39.19M` cells |
| 64M interval | byte-identical full self-host in `72.030s`, `3,659,074,812` steps, `50.80M` steps/s, `62` GCs, `16.445s` GC pause, high-water `72.71M` cells, SHA `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`, `cmp_exit=0` |
| 128M interval | byte-identical full self-host in `61.702s`, `3,659,074,831` steps, `59.30M` steps/s, `31` GCs, `10.274s` GC pause, high-water `138.07M` cells, same SHA, `cmp_exit=0`; this is the best measured default binary execution so far, but with much larger peak heap |
| 256M interval | byte-identical full self-host in `62.155s`, `3,659,074,831` steps, `58.87M` steps/s, `15` GCs, `7.824s` GC pause, high-water `272.03M` cells, same SHA, `cmp_exit=0` |
| accepted native default | changed the non-WASI default `GC_NODE_INTERVAL` from `32M` to `128M`, leaving `MHS_GC_NODE_INTERVAL` as an override and keeping the WASI default at `500k`. New protocol for the 60s band: run full self-host 3x and use the median. No-env release self-host samples were byte-identical at `61.255s`, `61.963s`, and `62.207s`; median `61.963s`, median steps `3,659,075,021`, median speed `59.05M` steps/s, `31` GCs, median GC pause `10.320s`, high-water `138.07M` cells, SHA `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`, `cmp_exit=0` for all samples |
| fair-heap companion | per revised `NOTES.md`, the `128M` default is a memory-for-time ceiling gauge, not fair against eval.c's fixed 50M-cell heap. Ran `MHS_GC_NODE_INTERVAL=41943040` (40Mi alloc window) for a Rust high-water of `46.41M` cells (~743MB, slightly under C's 800MB). Three full self-host samples were byte-identical at `73.583s`, `74.876s`, and `75.671s`; median `74.876s`, `3,659,074,888` steps, `48.87M` steps/s, `99` GCs, median GC pause `22.484s`, high-water `46.41M` cells, SHA `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`, `cmp_exit=0` for all samples. Reading: at fair memory Rust is still in the old `74-75s` band; the 128M default buys ~13s by spending ~3x the fair heap |
| reading | widening the window proves GC frequency/sweep is a double-digit-second lever: 128M cuts GC pause by roughly `13.7-15.0s` versus same-session 32M and wall time by `11.1-15.9s`. 256M cuts GC pause further but loses wall time versus 128M because the huge arena hurts locality/cache. This supports the revised NOTES direction: do not rely on a bigger window as the real fix; build a reclaim/layout design such as R2 that removes sweep work without growing the working set |
| R5 sweep-trim probe | tried replacing the general sweep tombstone path with a sweep-only `push_free_node_sweep` that clears cold payload slots only for old `Cold` cells and writes the `Free` cell directly. Verification passed `cargo fmt --check`, `cargo check`, `cargo check --features gc-phase-profile`, `cargo test --lib` 41/41, release bench build, 100M sanity, default full, and 128M full; outputs were byte-identical. Results: 100M `2.206s` / `232.406ms` GC, 32M full `73.693s` / `24.255s` GC, and 128M full `62.773s` / `10.435s` GC. Rejected/reverted because it was not a measurable source win and regressed the 128M interval run versus the plain `61.702s` result |
| R2 line-reclaim probe | implemented a feature-gated non-moving line allocator with 8-cell lines, full-line reclaim, stable `NodeId`s, and no per-cell `Free` tombstone writes. Default/feature `cargo check` and release feature builds passed. First variant cleaned stale cold payloads on reused allocation via `set_cell_at`: 100M was byte-shaped at `2.419s`, `41.67M` steps/s, `161.484ms` GC, high-water `34.93M`; full self-host was byte-identical but rejected at `84.388s`, `43.36M` steps/s, `21.832s` GC, high-water `58.34M`, SHA matched, `cmp_exit=0`. Second variant moved dead-cold cleanup back to sweep and used direct reclaimed-cell writes: 100M was `2.382s`, `42.32M` steps/s, `233.872ms` GC, high-water `34.93M`; full self-host was byte-identical but worse at `87.701s`, `41.72M` steps/s, `25.660s` GC, high-water `58.34M`, SHA matched, `cmp_exit=0`. Rejected/reverted: this full-line-only R2 shape reduces tombstone writes but fragments/grows the arena badly and does not cut GC pause enough. Future R2 needs a different partial-line strategy or a moving nursery, not this line-only free stack |
| R2 per-line free-mask probe | tried the next feature-gated R2 shape: 8-cell line masks so partial lines can be reused without writing per-cell `Free` tombstones; allocation consumed mask bits and wrote reclaimed cells directly, with dead cold payload cleanup kept in sweep. Default and feature checks passed and the release feature build completed. Rejected at the 100M gate without a full run: `2.486s`, `40.55M` steps/s, `3` GCs, `278.142ms` GC pause, high-water `33.94M`, sink `661902`. This is slower than both default and the earlier full-line R2 bounded slices, so mask bookkeeping is already too expensive in this shape |

## 2026-07-05 Raw Known-Dispatch Probe From Revised NOTES.md

| item | result |
|---|---|
| setup | tested the revised `NOTES.md` decode-collapse item by adding raw wire-code constants and switching only the hot stack reducer's known-head path from `decode_known_prim`/`EvalHead::Known` to a `u16` code match; fallback reducer and public `Prim` decoding stayed decoded. Verification passed `cargo fmt --check`, default `cargo check`, `cargo check --features eval-phase-profile`, `cargo check --features gc-phase-profile`, release bench build, 100M bounded gate, full self-host, SHA/cmp, and `git diff --check` |
| 100M bounded | `2.290s`, `44.02M` steps/s, `3` GCs, `232.078ms` GC pause, high-water `33,939,310`, sink `661902`; same steps as baseline (`100,811,779`) and plausible but inside the current 100M noise band |
| full self-host | byte-identical in `76.920s`, `3,659,074,869` steps, `47.57M` steps/s, `124` GCs, `25.850s` GC pause, high-water `39,192,416`, SHA `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`, `cmp_exit=0` |
| reading | not accepted as a measurable source win: the full gate lands in the 74-78s current band and misses the `74.668s` best / same-day `72.786s` sample. The note's expectation was right about low ceiling; do not keep iterating on known-dispatch source shape unless paired with a broader evaluator representation change |

## 2026-07-05 NOTES.md Rev 3 Closeout

| item | status |
|---|---|
| zero-code GC interval probe | done; `128M` interval became the native default and was pushed in `f02e9631`. Three-sample no-env median is `61.963s`, but the fair-heap companion is `74.876s` at `46.41M` cells, so the 128M result is recorded as a memory-for-time ceiling gauge rather than a fair C comparison |
| R5 sweep trim | done and rejected; direct sweep tombstone helper did not improve full self-host or 128M full gate |
| R2 line reclaim | tried full-line reclaim and per-line free-mask variants. Both kept stable `NodeId`s and were feature-gated. Full-line reclaim fragmented/grew the arena and regressed full self-host; per-line masks failed the 100M sanity at `2.486s`. Rejected for these shapes |
| decode collapse | done and rejected; raw known-dispatch path was byte-identical but missed the full gate and was reverted |
| R1 copying nursery | deferred pending rev 4 direction. Rev 3 says this is the next high-ceiling option after R2 under-delivers, but it breaks stable `NodeId` and is a larger design step than the current simplification directive |
| simplification directive | active after rev 4 closeout. R2 partial-line was rejected and 8B/u30 packed cells were accepted; next cleanup should be performance-neutral or better, with sizeable runtime cleanup gated by paired/3x self-host runs |

## 2026-07-05 NOTES.md Rev 4 Intake

| item | status |
|---|---|
| rev 4 landed | `NOTES.md` now starts as rev 4 and adds section 0: the 128M result is a memory-for-time presentation improvement, not a fair structural speedup. Fair headline remains the 40M/fair-heap median `74.876s` at `46.41M` cells, while the 128M default median is `61.963s` at `138.07M` cells |
| rev 4 live sequence | before performance-neutral simplification, measure partial-line R2 first, then consider 8B u30 cells and only later R1 copying nursery |
| in-tree R2 claim | rev 4 says partial-line R2 is already committed behind `--features immix-line-reclaim` at `c62cef08`, but this checkout maps `c62cef08` to `Use aborting release panics`; `git fetch origin` did not reveal any ref or history containing `immix-line-reclaim`, `line_reclaim`, `free_mask`, or a matching feature. Treat rev 4's benchmark directive as valid, but implement the missing feature locally before measuring it |
| local partial-line R2 implementation | added the missing `immix-line-reclaim` feature locally: 8-cell line masks, no per-cell `Free` tombstone writes, dead cold payload cleanup during sweep, direct reclaimed-cell writes, stable `NodeId`s. `cargo fmt --check`, default `cargo check`, feature `cargo check`, default `cargo test --lib` 41/41, and feature `cargo test --lib` 41/41 passed |
| local partial-line R2 100M A/B | initial 8-run alternating A/B (`MHS_REPEAT=4`, 100M step limit, candidate `--features immix-line-reclaim`) failed alloc-neutrality: base avg `2009.101ms`, candidate avg `2228.004ms` (`+10.90%`), same `100,811,779` steps, `3` GCs, high-water `33,939,361`, sink `661902`; candidate GC pause was slightly lower (`214.320ms` vs `217.414ms`) but allocation/codegen cost dominated |
| local partial-line R2 hoist | after hoisting current line/base/mask out of the mask vector, checks still passed and the 8-run A/B improved but still failed: base avg `2019.253ms`, candidate avg `2128.981ms` (`+5.43%`), same steps/GCs/high-water/sink; candidate GC pause `214.806ms` vs base `219.217ms`. Rejected without full self-host because it missed rev 4's ~1% gate; this matches the older per-line free-mask rejection rather than rev4's expected fair-61s path |
| 8B u30 packed-cell implementation | accepted and made default (`default = ["packed-cell"]`), with the previous 16-byte cell retained behind `--no-default-features`. Packed `Cell` is one `u64`: 4-bit tag; u30/u30 `App` fun/arg; u30 Indir/Free/Cold payloads; 60-bit inline `Int`/`ThreadId`; 60-bit inline `Float32`; wide `Int64`/`Float64`/pointer/raw-fun-pointer and out-of-range scalars cold-box through the existing `cold_nodes` table. Program-level scalar helpers provide the cold fallback so evaluator call sites do not grow ad hoc cold checks. Bench output now includes `cell_size_bytes` |
| 8B u30 verification | passed `cargo fmt --check`, default `cargo check`, `cargo check --no-default-features`, `cargo check --features eval-phase-profile`, `cargo check --features gc-phase-profile`, default `cargo test --lib` 41/41, `cargo test --lib --no-default-features` 41/41, release `mhs-rust-bench` build, release `wasm32-unknown-unknown` build, release `wasm32-wasip1` `mhs-rust` build, and `git diff --check` |
| 8B u30 100M A/B | feature candidate before flipping default: 8-run alternating 100M/32M self-host A/B (`MHS_REPEAT=4`, candidate `--features packed-cell`) was base avg `2121.597ms` versus packed avg `1886.066ms` (`-11.10%`), same `100,811,779` steps, `3` GCs, high-water `33,939,359`, sink `661902`; step-limited runs produce no output combs |
| 8B u30 full self-host A/B | paired 3x full self-host at the current 128M window (`MHS_GC_NODE_INTERVAL=134217728`, `MHS_STEP_LIMIT=none`, 900s timeout) was byte-identical (`cmp_result=0`): 16-byte fallback samples `63.974s`, `63.580s`, `63.569s` (median `63.580s`) versus packed samples `62.750s`, `63.183s`, `62.687s` (median `62.750s`, avg `-1.31%`). Both used `3,659,075,040` steps, `31` GCs, high-water `138,068,222` cells, sink `661902`; GC pause avg fell `10.205s -> 9.998s`, and `cell_size_bytes` fell `16 -> 8` |
| rev 4 closeout | R2 partial-line was rejected for alloc-path cost; 8B/u30 packed cells were accepted and made the default after bounded and full gates. R1 copying nursery remains the next structural heap/locality lever, but the user's directive now switches the near-term work to performance-neutral simplification |

## 2026-07-05 Performance-Neutral Simplification

| item | result |
|---|---|
| stale stack-app-origin profiling scaffold | removed the uncommitted `eval-phase-profile` stack-app-origin diagnostic scaffold from the worktree after rev 4 made it irrelevant. Net runtime source diff returned to zero; `cargo fmt --check`, default `cargo check`, `cargo check --features eval-phase-profile`, and default `cargo test --lib` 41/41 passed |
| dead persistent/old frame reducer deletion retest | tried a source cleanup deleting the unreachable `PersistentSpine` reducer, old `EvalFrameStack`/`StrictRedex` frame path, stale strict-redex profiling hooks, and associated scratch-app GC roots (~1.7k LOC removed). Verification passed `cargo fmt`, default `cargo check`, `cargo check --features eval-phase-profile`, and `cargo test --lib` 41/41. Source A/B at 100M/32M looked neutral-positive: baseline avg `2030.068ms`, candidate avg `2005.791ms` (`-1.20%`), same `100,811,779` steps, `3` GCs, high-water `33,939,363`, sink `661902`; step-limited runs do not produce output combs. Full default/no-env gate rejected it: same-session baseline median `64.026s` (`64.026s`, `64.856s`, `63.232s`) versus candidate median `64.959s` (`65.965s`, `64.922s`, `64.959s`), byte-identical outputs, `31` GCs, high-water `138.07M`. Also measured an explicit 32M candidate median `77.812s`, byte-identical, but that is the old/fairer GC window rather than the current default. Reverted; this repeats the earlier lesson that deleting this dead path perturbs release codegen/layout more than it helps runtime |
| benchmark binary module cleanup | converted `src/bin/mhs-rust-bench.rs` from hand `include!` fragments to real bin-local modules with `pub(super)` boundaries. Runtime library code is untouched, so this is a tooling/readability cleanup rather than a self-host performance probe. Verification passed `cargo fmt`, `cargo check --all-targets`, `cargo test --lib` 41/41, release `mhs-rust-bench` build, and an `identity-chain:1000` smoke through the release bench binary |
| 8B-cell accessor prep | tried a narrow source-prep slice for the u30/8B-cell direction by moving direct `Cell.word0/word1` payload reads out of resolver/GC/profile/core call sites into `Cell` accessors (`app_fun_trusted`, `indir_target_trusted`, `pointer_payload_trusted`, `prim_name`) and using `Cell::free` for free-list tombstones. Verification passed `cargo fmt`, default `cargo check`, `cargo check --features eval-phase-profile`, `cargo check --features gc-phase-profile`, and `cargo test --lib` 41/41. The 8-run 100M/32M source A/B looked good: baseline avg `2059.558ms`, candidate avg `1989.360ms` (`-3.41%`), same `100,811,779` steps, `3` GCs, high-water `33,939,373`, sink `661902`; step-limited runs produce no output combs. Full default/no-env gate rejected it for now: same-session baseline median `65.481s` (`64.610s`, `65.481s`, `66.518s`) versus candidate median `66.303s` (`66.303s`, `64.916s`, `66.368s`), byte-identical outputs, `31` GCs, high-water `138.07M`. Reverted; prep work for packed cells needs to be bundled with an actual layout win or re-tested under a stronger full A/B |
| unconditional packed-cell cleanup | accepted as performance-neutral simplification after removing the obsolete `packed-cell` feature, the 16-byte `Cell { word0, word1 }` fallback, and the associated scalar/pointer tag cfg branches. Verification passed `cargo fmt --check`, default `cargo check`, `cargo test --lib` 41/41, `cargo check --features eval-phase-profile`, `cargo check --features gc-phase-profile`, release `mhs-rust-bench` build, release `wasm32-unknown-unknown` build, release `wasm32-wasip1 --bin mhs-rust` build, and `git diff --check`. Source A/B used separately built binaries from committed baseline `11334945` and the cleanup candidate. 100M/32M alternating A/B: baseline avg `1897.871ms`, candidate avg `1878.876ms` (`-1.00%`), same `100,811,779` steps, `3` GCs, high-water `33,939,381`, sink `661902`, `cell_size_bytes=8`; step-limited outputs are absent by design. Paired 3x full self-host at the 128M window: baseline samples `63.229s`, `62.162s`, `62.260s` (median `62.260s`, avg `62.550s`) versus candidate `62.625s`, `62.480s`, `62.135s` (median `62.480s`, avg `62.413s`); byte-identical outputs (`cmp_result=0`), same `3,659,075,230` steps, `31` GCs, high-water `138,068,242`, sink `661902`, `cell_size_bytes=8`. Reading: not a speed claim; the full median is `+0.35%` while the full average is `-0.22%`, so this is neutral cleanup that removes ~400 LOC and one stale representation axis |
| whole-heap debug clone API removal | accepted as small cold-path simplification: removed public `Program::nodes()`, which cloned the whole arena and was used only by tests and `IO.deserialize`; tests now use `node_for_debug`, and `append_parsed_program` walks parsed nodes by index instead of materializing a `Vec<Node>`. Verification passed `cargo fmt --check`, default `cargo check`, `cargo check --features eval-phase-profile`, `cargo check --features gc-phase-profile`, `cargo test --lib` 41/41 including the `IO.deserialize` smokes, release `mhs-rust-bench` build, release `wasm32-unknown-unknown` build, release `wasm32-wasip1 --bin mhs-rust` build. Two 8-run 100M/32M A/B passes against committed `7019222a` were noisy but acceptable: pass 1 had one slow candidate outlier and averaged baseline `1859.818ms` versus candidate `1913.508ms` (`+2.89%`), while pass 2 averaged baseline `1839.095ms` versus candidate `1817.679ms` (`-1.16%`); all runs had identical `100,811,779` steps, `3` GCs, high-water `33,939,387`, sink `661902`, and `cell_size_bytes=8`. No full gate: the change is cold/API cleanup, not a sizeable evaluator chunk |
| runtime API visibility tightening | accepted as API-surface cleanup: removed unused public `Program::label`, made `Program::new` crate-visible for the parser instead of public, and made `push_node` runtime-internal because all callers are runtime modules/tests. This does not change the CLI/bench/wasm path or public reduction behavior. Verification passed `cargo fmt --check`, default `cargo check`, `cargo check --all-targets`, `cargo check --features eval-phase-profile`, `cargo check --features gc-phase-profile`, `cargo test --lib` 41/41, release `mhs-rust-bench` build, release `wasm32-unknown-unknown` build, and release `wasm32-wasip1 --bin mhs-rust` build. No benchmark gate: visibility-only cleanup |
| debug-node API visibility tightening | accepted as cold API-surface cleanup: made `Program::node_for_debug` runtime-internal after confirming every remaining caller is inside the runtime module tree or tests (`IO.deserialize`, serialization/rendering helpers, profiling labels, and unit tests). Verification passed `cargo fmt --check`, default `cargo check`, `cargo check --features eval-phase-profile`, `cargo check --features gc-phase-profile`, `cargo test --lib` 41/41, `cargo check --all-targets`, release `mhs-rust-bench` build, release `wasm32-unknown-unknown` build, and release `wasm32-wasip1 --bin mhs-rust` build. No benchmark gate: visibility-only cleanup with no CLI/bench/wasm behavior change |
| browser JS wrapper API tightening | accepted as wasm/API-surface cleanup: removed the unused public `Program::apply_stable_ptr_pointer`, stopped exporting `JsValue` from the public Rust facade, made `set_js_program_handle`, `apply_js_wrapper_index`, and `js_wrapper_tags` crate-visible, and cfg-gated the browser-only `JsValue` conversion helpers to `wasm32-unknown-unknown`. The wasm C ABI exports remain unchanged. Verification passed `cargo fmt --check`, default `cargo check`, `cargo check --features eval-phase-profile`, `cargo check --features gc-phase-profile`, `cargo test --lib` 41/41, `cargo check --all-targets`, release `mhs-rust-bench` build, release `wasm32-unknown-unknown` build, and release `wasm32-wasip1 --bin mhs-rust` build. No full benchmark gate: browser-wrapper visibility/cfg cleanup, not evaluator behavior. Current 100M/32M sanity after the cleanup was normal: `1868.347ms`, `53.96M` steps/s, `100,811,779` steps, `3` GCs, `218.307ms` GC pause, high-water `33,939,308`, sink `661902` |
| Node constructor visibility tightening | accepted as API-surface cleanup: made the `Node` convenience constructors (`prim`, `bigint`, `bytes`, `array`, `ffi`, `js_wrap`, `fun_ptr`, `tick`) crate-visible now that `Program::new` is not public and external callers cannot build runtime programs from raw nodes through the public API. The public `Node` enum variants are unchanged. Verification passed `cargo fmt --check`, default `cargo check`, `cargo check --features eval-phase-profile`, `cargo check --features gc-phase-profile`, `cargo test --lib` 41/41, `cargo check --all-targets`, release `mhs-rust-bench` build, release `wasm32-unknown-unknown` build, and release `wasm32-wasip1 --bin mhs-rust` build. No benchmark gate: visibility-only cleanup |
| R5 sweep free-node fast path | accepted as source-speed cleanup: `push_free_node` now checks `CellTag::Cold` before dropping cold payloads and writes the `Cell::free` tombstone directly for the common non-cold reclaimed cell, avoiding the generic `set_cell_at` cold-payload path on every sweep free-list write. Verification passed `cargo fmt --check`, default `cargo check`, `cargo check --features gc-phase-profile`, `cargo check --features eval-phase-profile`, `cargo test --lib` 41/41, release `mhs-rust-bench` build, release `wasm32-unknown-unknown` build, release `wasm32-wasip1 --bin mhs-rust` build, and `git diff --check`. 100M/32M alternating A/B: baseline avg `1799.147ms` versus candidate avg `1779.657ms` (`-1.08%`), same `100,811,779` steps, `3` GCs, high-water `33,939,338`, sink `661902`; candidate GC pause avg fell from `214.190ms` to `195.473ms`. Paired 3x full self-host at the 128M window: baseline median `59.319s`/avg `59.392s` versus candidate median `58.503s`/avg `58.474s` (`-1.38%` median, `-1.55%` avg), byte-identical outputs (`cmp_all=1`, SHA `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`), same `3,659,074,812` steps, `31` GCs, high-water `138,068,198`, sink `661902`. Reading: this is the good R5 shape, distinct from the earlier rejected direct tombstone-helper probe, because it keeps cold-payload safety but removes the generic setter on the sweep hot path |
| free-slot allocation write cleanup | accepted as performance-neutral simplification: removed the single-use `cell_at`, `set_free_cell_at`, and `push_node_fresh` helpers, made `pop_free_node` read the arena directly, and made free-list reuse write cells directly. The non-app `push_node` free-slot path now avoids the generic `set_node_at`/`drop_cold_payload` path because a popped free-list cell is already `Free`; the app path writes `Cell::app` directly under the same debug assertion. Verification passed `cargo fmt --check`, default `cargo check`, `cargo check --features eval-phase-profile`, `cargo check --features gc-phase-profile`, `cargo test --lib` 41/41, release `mhs-rust-bench` build, release `wasm32-unknown-unknown` build, release `wasm32-wasip1 --bin mhs-rust` build, and `git diff --check`. 100M/32M alternating A/B: baseline avg `1785.616ms` versus candidate avg `1767.975ms` (`-0.99%`), same `100,811,779` steps, `3` GCs, high-water `33,939,355`, sink `661902`; step-limited runs produce no output combs. Paired 3x full self-host at the 128M window: baseline median `57.939s`/avg `58.084s` versus candidate median `58.044s`/avg `58.040s` (`+0.18%` median, `-0.08%` avg), byte-identical outputs (`cmp_all=1`, SHA `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`), same `3,659,074,945` steps, `31` GCs, high-water `138,068,212`, sink `661902`. Reading: accept as neutral code-shape cleanup, not as a speed claim |
| stale eval rewrite-opportunity profiling trim | accepted as profile-surface cleanup: removed the eval-phase-only stack rewrite arg-pattern and rewrite-opportunity diagnostics (`red_i`, `red_k`, `red_a`, `red_bi`, `red_bxi`, `red_cc*`, `red_flip`) plus their profile output accessors and stack call sites. App-allocation shape, resolved shape, arity, transition, and timing profiling remain. Verification passed `cargo fmt --check`, default `cargo check`, `cargo check --features eval-phase-profile`, `cargo check --features gc-phase-profile`, `cargo test --lib` 41/41, `cargo check --all-targets`, `cargo check --all-targets --features eval-phase-profile`, release `mhs-rust-bench` build, release `wasm32-unknown-unknown` build, release `wasm32-wasip1 --bin mhs-rust` build, release `eval-phase-profile` bench build, a 1M-step eval-profile smoke, and `git diff --check`. Default 100M/32M sanity after the trim was normal at `1812.952ms`, `55.61M` steps/s, `100,811,779` steps, `3` GCs, `203.406ms` GC pause, high-water `33,939,296`, sink `661902`. No full self-host gate: default generated code is unaffected because the removed stack instrumentation was cfg-gated behind `eval-phase-profile` |
| stale strict-redex profile counter trim | accepted as profile-surface cleanup: removed always-compiled counters and hooks for strict-redex snapshots and remaining-app scans, which were old diagnostics for the pre-stack spine/frame path and were zero in the current eval-profile smoke. Persistent force/fallback counts, eval frame push counts, stack rewrite/app update counts, app allocation shapes, arity, transitions, and timings remain. Verification passed `cargo fmt --check`, default `cargo check`, `cargo check --features eval-phase-profile`, `cargo check --features gc-phase-profile`, `cargo test --lib` 41/41, `cargo check --all-targets --features eval-phase-profile`, release `mhs-rust-bench` build, release `eval-phase-profile` bench build, release `wasm32-unknown-unknown` build, release `wasm32-wasip1 --bin mhs-rust` build, a 1M-step eval-profile smoke (`profile_total_ms=1474.251`, `profile_persistent_forces=35793`, `profile_eval_frame_pushes=51377`, sink `661902`), and `git diff --check`. No full self-host gate: this removes profiling counters and guarded calls only, not default reduction behavior |
| R1 moving-GC remapper scaffold | accepted as feature-gated structural prep, not a default-runtime change: added `moving-gc = []` and `program/remap.rs`, compiled only under `cfg(any(test, feature = "moving-gc"))`. The remapper rewrites packed heap `App`/`Indir` cells, cold payload holders (`BytesView`, `MVar`, `Array`, `Weak`), `Program` side roots, stable pointers, weak/finalizer queues, BFILE read-only memory view bases, primitive/compound caches, `gc_mark_work`, `gc_young_profile_allocated_slots` under `gc-phase-profile`, evaluator frame stacks, machine stacks, eval/persistent spines, scratch vectors, and rebuilds `node_pointer_slots` after remapping `node_pointers`; free-list links are intentionally not remapped because moving GC must rebuild free state. Verification passed `cargo fmt --check`, default `cargo check`, `cargo check --features moving-gc`, `cargo check --features eval-phase-profile`, `cargo check --features gc-phase-profile`, `cargo test --lib` 42/42 including `moving_gc_remapper_rewrites_node_id_holders`, release `mhs-rust-bench` build, release `wasm32-unknown-unknown` build, release `wasm32-wasip1 --bin mhs-rust` build, and `git diff --check`. Final default 100M/32M sanity after gating was `1830.189ms`, `55.08M` steps/s, `100,811,779` steps, `3` GCs, `196.132ms` GC pause, high-water `33,939,296`, sink `661902`, `cell_size_bytes=8`. No full 3x self-host: default release does not compile the remapper; the gate for future R1 behavior is the `moving-gc` feature path |
| stale GCRED opportunity profiling trim | accepted as feature-surface cleanup: removed the `gc-phase-profile` RED opportunity counters (`gc_red_i/k/a/bi/bxi/cc*` and flip), the GCRED-specific heap-shape classifier, the mark-loop hook, and the corresponding bench/profile output keys. Kept `gc-phase-profile` mark/sweep timings and young-region viability counters. Verification passed `cargo fmt --check`, default `cargo check`, `cargo check --features gc-phase-profile`, `cargo check --features eval-phase-profile`, `cargo test --lib` 42/42, release `mhs-rust-bench` build, release `mhs-rust-bench --features gc-phase-profile` build, release `wasm32-unknown-unknown` build, release `wasm32-wasip1 --bin mhs-rust` build, a 10M `gc-phase-profile` smoke with phase/young counters and no RED keys (`240.200ms`, `10,028,866` steps, no GC), and `git diff --check`. Final default 100M/32M sanity after rebuilding without `gc-phase-profile`: `1801.046ms`, `55.97M` steps/s, `100,811,779` steps, `3` GCs, `191.860ms` GC pause, high-water `33,939,302`, sink `661902`, `cell_size_bytes=8`. No full self-host gate: this removes feature-only diagnostics and does not change default reduction behavior |
| raw app-allocation shape profiling trim | accepted as eval-profile surface cleanup: removed the superseded raw `profile_app_allocation_site_shapes` map/output and its `profile_node_shape_key` helper, leaving the resolved app-allocation site-shape profile as the single shape diagnostic. Verification passed `cargo fmt --check`, default `cargo check`, `cargo check --features eval-phase-profile`, `cargo check --features gc-phase-profile`, `cargo test --lib` 42/42, release `mhs-rust-bench --features eval-phase-profile` build, default release `mhs-rust-bench` build, release `wasm32-unknown-unknown` build, release `wasm32-wasip1 --bin mhs-rust` build, and an isolated 1M eval-profile smoke (`profile_total_ms=1412.183`, `1,001,388` steps) that still prints `profile_app_allocation_resolved_site_shapes` and no raw shape header. Final default 100M/32M sanity was `1866.776ms`, `54.00M` steps/s, `100,811,779` steps, `3` GCs, `206.909ms` GC pause, high-water `33,939,308`, sink `661902`, `cell_size_bytes=8`. No full self-host gate: this removes `eval-phase-profile`-only diagnostics and does not change default reduction behavior |
| stale F2 force-surface profiling trim | accepted as profile-surface cleanup: removed the stale `eval_whnf_value` call/slow counters and recursive `reduce_node_whnf` entry-shape profile, along with their profile output keys and the now-unused string label parameter on `eval_whnf_value`. The previous F2 diagnostic showed this native-recursive force surface is tiny and non-actionable versus the hot stack loop; persistent force/fallback counts, frame counts, stack timings, arity/transition profiles, and resolved app-allocation shapes remain. Verification passed `cargo fmt --check`, default `cargo check`, `cargo check --features eval-phase-profile`, `cargo check --features gc-phase-profile`, `cargo test --lib` 42/42, release `mhs-rust-bench` build, release `mhs-rust-bench --features eval-phase-profile` build, release `wasm32-unknown-unknown` build, release `wasm32-wasip1 --bin mhs-rust` build, and a 1M eval-profile smoke (`profile_total_ms=1366.987`, `1,001,388` steps, sink `661902`) that still prints `profile_app_allocation_resolved_site_shapes` and no force-surface headers. Default 128M-window 100M sanity was `2376.105ms`, `42.43M` steps/s, `100,811,779` steps, `0` GCs, high-water `117,096,561`, sink `661902`; explicit 32M-window sanity was `1788.391ms`, `56.37M` steps/s, `3` GCs, `193.901ms` GC pause, high-water `33,939,296`, sink `661902`. No full self-host gate: this deletes stale profiling surface and an unused diagnostic label argument; prior F2 data showed this helper is not the self-host hot wall |
| stale unbounded spine-arity profiling trim | accepted as profile-output cleanup: removed the always-profiled `spine_arity` histogram, derived `profile_spine_arity_entries`, `profile_heap_spines`, the `heap_spine` plumbing in `profile_step`, and the now-unused `EvalSpine::is_heap` helper. Kept `profile_max_spine_arity` and the newer `eval-phase-profile` arity classes/transitions, which are the useful long-spine diagnostic. Verification passed `cargo fmt --check`, default `cargo check`, `cargo check --features eval-phase-profile`, `cargo check --features gc-phase-profile`, `cargo check --all-targets`, `cargo check --all-targets --features eval-phase-profile`, `cargo test --lib` 42/42, release `mhs-rust-bench` build, release `mhs-rust-bench --features eval-phase-profile` build, release `wasm32-unknown-unknown` build, release `wasm32-wasip1 --bin mhs-rust` build, and a 1M eval-profile smoke (`profile_total_ms=1359.408`, `1,001,388` steps, sink `661902`) that no longer prints the full arity table. Final 100M/32M sanity was `1753.440ms`, `57.49M` steps/s, `100,811,779` steps, `3` GCs, `184.606ms` GC pause, high-water `33,939,294`, sink `661902`, `cell_size_bytes=8`. No full self-host gate: this trims profile-only counters/output and call-site plumbing guarded by `self.profile.is_some()` |
| stale primitive-dispatch profiling trim | accepted as profile-surface cleanup after rev 4 closed the dispatch-probe loop: removed the `primitive_dispatch_probes`/`primitive_dispatch_hits` maps, profile output sections, strict-dispatch profiler hook, and fallback-dispatch probe/hit calls. The fallback runtime-primitive dispatch order remains unchanged; the old probe macro is now a direct eager `.or(...)` chain with the same error behavior. Verification passed `cargo fmt --check`, default `cargo check`, `cargo check --features eval-phase-profile`, `cargo check --features gc-phase-profile`, `cargo check --all-targets`, `cargo check --all-targets --features eval-phase-profile`, `cargo test --lib` 42/42, release `mhs-rust-bench` build, release `mhs-rust-bench --features eval-phase-profile` build, release `wasm32-unknown-unknown` build, release `wasm32-wasip1 --bin mhs-rust` build, and a 1M eval-profile smoke (`profile_total_ms=1435.875`, `1,001,388` steps, sink `661902`) with no primitive-dispatch probe/hit sections. Final 100M/32M sanity was `1853.797ms`, `54.38M` steps/s, `100,811,779` steps, `3` GCs, `192.357ms` GC pause, high-water `33,939,300`, sink `661902`, `cell_size_bytes=8`. No full self-host gate: this deletes profiling counters and no-op fallback profile calls, not evaluator dispatch semantics |
| stale small-int cache profiling trim | accepted as profile-surface cleanup: removed the `small_int_cache_hits`, `small_int_cache_misses`, and `non_small_int_allocations` counters, hooks, profile output keys, and the guarded profile branches inside `Program::int`. The small-int cache itself and non-small-int allocation behavior are unchanged. Verification passed `cargo fmt --check`, default `cargo check`, `cargo check --features eval-phase-profile`, `cargo check --features gc-phase-profile`, `cargo check --all-targets`, `cargo check --all-targets --features eval-phase-profile`, `cargo test --lib` 42/42, release `mhs-rust-bench` build, release `mhs-rust-bench --features eval-phase-profile` build, release `wasm32-unknown-unknown` build, release `wasm32-wasip1 --bin mhs-rust` build, and a 1M eval-profile smoke (`profile_total_ms=1420.907`, `1,001,388` steps, sink `661902`) with no small-int cache sections. Final 100M/32M sanity was `1785.604ms`, `56.46M` steps/s, `100,811,779` steps, `3` GCs, `193.684ms` GC pause, high-water `33,939,300`, sink `661902`, `cell_size_bytes=8`. No full self-host gate: this is a small profile-counter trim; the next sizeable default-code chunk should refresh the 3x full gate |
| stale fallback rewrite profiling trim | accepted as profile-surface cleanup: removed the always-compiled fallback reducer `spine_rewrites`/`app_rewrites` counters, hooks, output keys, and profile call sites. The current stack rewrite and stack app-update counters remain, which are the useful diagnostics for the live hot path. Verification passed `cargo fmt --check`, default `cargo check`, `cargo check --features eval-phase-profile`, `cargo check --features gc-phase-profile`, `cargo check --all-targets`, `cargo check --all-targets --features eval-phase-profile`, `cargo test --lib` 42/42, release `mhs-rust-bench` build, release `mhs-rust-bench --features eval-phase-profile` build, release `wasm32-unknown-unknown` build, release `wasm32-wasip1 --bin mhs-rust` build, and a 1M eval-profile smoke (`profile_total_ms=1352.034`, `1,001,388` steps, sink `661902`) with no fallback rewrite sections and still only `90` fallback eval-loop steps. Final 100M/32M sanity was `1723.868ms`, `58.48M` steps/s, `100,811,779` steps, `3` GCs, `182.872ms` GC pause, high-water `33,939,298`, sink `661902`, `cell_size_bytes=8`. No full self-host gate: this trims stale fallback/profile accounting only, not default reduction semantics |
| stale stack counter trim | accepted as profile-surface cleanup: removed the dead `stack_step_force` counter/output after confirming `StackStep::Force` no longer exists, and removed the redundant `stack_app_update_allocations` subcounter/output while keeping `stack_app_updates` and `stack_app_update_apps`. Verification passed `cargo fmt --check`, default `cargo check`, `cargo check --features eval-phase-profile`, `cargo check --features gc-phase-profile`, `cargo check --all-targets`, `cargo check --all-targets --features eval-phase-profile`, `cargo test --lib` 42/42, release `mhs-rust-bench` build, release `mhs-rust-bench --features eval-phase-profile` build, release `wasm32-unknown-unknown` build, release `wasm32-wasip1 --bin mhs-rust` build, and a 1M eval-profile smoke (`profile_total_ms=1466.008`, `1,001,388` steps, sink `661902`) with no `profile_stack_step_force` or `profile_stack_app_update_allocations` keys. Final 100M/32M sanity was `1807.711ms`, `55.77M` steps/s, `100,811,779` steps, `3` GCs, `201.576ms` GC pause, high-water `33,939,300`, sink `661902`, `cell_size_bytes=8`. No full self-host gate: this trims profile counters/output only |
| eval-phase profile field cfg cleanup | accepted as feature-boundary cleanup: cfg-gated the stack phase counters/timers, the stack eval-step head timing map, and the sorted-time helper behind `eval-phase-profile`, matching their existing writers/readers and removing default-build dead storage. Verification passed `cargo fmt --check`, default `cargo check` without warnings, `cargo check --features eval-phase-profile`, `cargo check --features gc-phase-profile`, `cargo check --all-targets`, `cargo check --all-targets --features eval-phase-profile`, `cargo test --lib` 42/42, release `mhs-rust-bench` build, release `mhs-rust-bench --features eval-phase-profile` build, release `wasm32-unknown-unknown` build, release `wasm32-wasip1 --bin mhs-rust` build, and a 1M eval-profile smoke (`profile_total_ms=1349.603`, `1,001,388` steps, sink `661902`) that still prints the eval-phase timing sections. Final 100M/32M sanity was `1750.533ms`, `57.59M` steps/s, `100,811,779` steps, `3` GCs, `189.688ms` GC pause, high-water `33,939,312`, sink `661902`, `cell_size_bytes=8`. No full self-host gate: default reduction behavior is unchanged and this only moves feature-only profile fields behind their feature |
| stale node-allocation profiling trim | accepted as profile-surface cleanup: removed the old `node_allocations` map/output, the generic `profile_node_allocation` hook in `push_node`, and the duplicate App node-allocation accounting inside app-allocation bookkeeping. The useful allocation diagnostics remain: total app allocations, app allocation sites, and `eval-phase-profile` resolved app-allocation site shapes. Verification passed `cargo fmt --check`, default `cargo check`, `cargo check --features eval-phase-profile`, `cargo check --features gc-phase-profile`, `cargo check --all-targets`, `cargo check --all-targets --features eval-phase-profile`, `cargo test --lib` 43/43, release `mhs-rust-bench` build, release `mhs-rust-bench --features eval-phase-profile` build, release `wasm32-unknown-unknown` build, release `wasm32-wasip1 --bin mhs-rust` build, a 1M eval-profile smoke (`profile_total_ms=1338.445`, `1,001,388` steps, sink `661902`) that still prints `profile_app_allocation_sites` and `profile_app_allocation_resolved_site_shapes` but no `profile_node_allocations`, and `git diff --check`. Final 100M/32M sanity was `1757.354ms`, `57.37M` steps/s, `100,811,779` steps, `3` GCs, `192.195ms` GC pause, high-water `33,939,310`, sink `661902`, `cell_size_bytes=8`. No full self-host gate: this removes stale profiling output and a profile-only generic allocation hook, not active reduction semantics |
| expected-bytes trace-field trim | accepted as disabled-diagnostic cleanup: removed the stale `MHS_TRACE_EXPECTED_BYTES` env read, the `Program::trace_expected_bytes` field, and verbose `ExpectedBytes` error logging. `expected_bytes_error` still returns the same `EvalError::ExpectedBytes(id)`. Verification passed `cargo fmt --check`, default `cargo check`, `cargo check --features eval-phase-profile`, `cargo check --features gc-phase-profile`, `cargo check --all-targets`, `cargo check --all-targets --features eval-phase-profile`, `cargo test --lib` 43/43, release `mhs-rust-bench` build, release `wasm32-unknown-unknown` build, release `wasm32-wasip1 --bin mhs-rust` build, and `git diff --check`. No benchmark gate: this deletes disabled error-path logging and one `Program` field, not reduction behavior |
| invalid-byte trace cleanup | accepted as stale diagnostic cleanup: removed the temporary `MHS_TRACE_INVALID_BYTES` macro/env checks, invalid-byte operation context logging, suspicious pointer logging, and the pass-through error wrappers around bytes, FFI, and foreign-pointer dispatch. All affected paths still return the same `EvalError::InvalidByteString` or existing non-byte errors; `node_trace_summary` remains for real error payloads. Verification passed `cargo fmt --check`, default `cargo check`, `cargo check --features eval-phase-profile`, `cargo check --features gc-phase-profile`, `cargo check --all-targets`, `cargo check --all-targets --features eval-phase-profile`, `cargo test --lib` 43/43, release `mhs-rust-bench` build, release `wasm32-unknown-unknown` build, release `wasm32-wasip1 --bin mhs-rust` build, and `git diff --check`. Final 100M/32M sanity was `1731.370ms`, `58.23M` steps/s, `100,811,779` steps, `3` GCs, `198.065ms` GC pause, high-water `33,939,312`, sink `661902`, `cell_size_bytes=8`. No full self-host gate: this removes stale error-path diagnostics and one default pointer-eval env-var check, not reduction semantics |
| post-profile-trim full self-host refresh | after `53b34412`, `2745d4ca`, and `0a92c682`, current default release at the 128M speed window completed 3x byte-identically: `58.772s`, `58.657s`, `59.297s` (median `58.772s`, avg `58.909s`), `3,659,074,736` steps, `31` GCs, GC pauses `9.006s`, `8.999s`, `9.124s` (avg `9.043s`), high-water `138,068,132`, sink `661902`, `cell_size_bytes=8`, SHA `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`, `cmp=0` for all three. Reading: current source remains in the ~59s band; not a new source-speed claim because the changes are profiling/simplification cleanup |

## 2026-07-05 R1 Moving Nursery Root/Fixup Audit

This is a no-code audit for the next structural heap step. Current GC can mark reachability, but R1 needs mutable remapping: every live `NodeId` holder must be visited and rewritten after evacuation. The audit is from `core_types.rs` `Program` fields, `gc.rs` mark roots/mark reachability, `eval_state.rs` stack/frame state, and host handle modules.

| holder | current mark coverage | moving-GC fixup requirement |
|---|---|---|
| heap cells | `mark_reachable` follows `App(fun,arg)` and `Indir(target)`, and canonicalizes some App children during mark | rewrite packed `Cell::App` fun/arg and `Cell::Indir` target to forwarded ids. `Free` cells are not live roots; any nursery collection must discard/rebuild young free state rather than preserve the old free-list links |
| cold payloads | `mark_reachable` follows `BytesView.base`, `MVar(Some)`, `Array(items)`, pointer targets from positive `Ptr`/`RawFunPtr`/`ForeignPtr.ptr`, and handles `Weak` through the weak pass | add a mutable visitor for `Node::BytesView.base`, `Node::MVar`, `Node::Array`, `Node::Weak.{key,value,finalizer}`. For positive guest pointers, update the `node_pointers` slot table target, not the guest pointer integer. Cold payload indices can stay stable if copied `Cold` cells keep the same `cold_nodes` index and old from-space cells do not drop payloads during evacuation |
| `Program` side roots | `mark_program_roots` covers `root`, `stable_ptrs`, `pending_weak_finalizers`, BFILE read-only memory view bases, `arg_ref_array`, `world`, `small_ints`, `prim_cache`, and `compound_cache`; `labels` are retained after mark | rewrite every listed field in-place. `labels` must either be remapped before retain or rebuilt from live forwarded targets. `node_pointer_slots: HashMap<NodeId, usize>` must be rebuilt after remapping `node_pointers`, because its keys are old ids |
| evaluator transient state | `mark_program_roots` delegates to `mark_eval_stack`, `mark_machine_stack`, `mark_eval_spine`, `mark_persistent_spine`, and scans `scratch_args`/`scratch_apps` | moving collection needs mutable walkers for old `EvalFrameStack`, live `EvalStack`, `EvalSpine` inline `MaybeUninit` slots, heap vectors, `PersistentSpine` `VecDeque`, and scratch vectors. This is the highest-risk correctness surface because these are active redex/context owners, not passive roots |
| weak/finalizer queues | current weak pass canonicalizes live weak keys, clears dead key/value pairs, marks finalizers, and later drains `pending_weak_finalizers` | forwarding must happen before weak liveness decisions clear keys. Remap surviving weak ids in `weak_nodes`, remap queued `pending_weak_finalizers`, and keep foreign finalizer side-table indices unchanged because they are not `NodeId`s |
| host/BFILE tables | `BFileKind::ReadOnlyMemoryView { base }` is marked as a root; allocations, native handles, dirs, JS wrapper tags, and foreign finalizer records are not `NodeId` holders | remap read-only memory BFILE bases. Other host tables are stable handles/bytes, but their payloads may contain guest pointer integers whose slot targets live in `node_pointers`; update slots, not the bytes |
| profiling/diagnostics | default `EvalProfile` stores string/count maps; `ProfileHead` is transient `Option<NodeId>` in frames; `gc_young_profile_allocated_slots` stores ids only under `gc-phase-profile` | remap transient `ProfileHead` through frame walkers. For `gc_young_profile_allocated_slots`, either remap the list in profile builds or retire it for R1 nursery metrics; do not let profile-only ids become a disabled-feature tax |

| implementation consequence | reading |
|---|---|
| first artifact | done behind `cfg(any(test, feature = "moving-gc"))`: a shared mutable `NodeId` remapper now covers the holders above, has a non-identity coverage test, and can be enabled for the first moving collector without touching default release code |
| full-heap evacuation scaffold | done behind `cfg(any(test, feature = "moving-gc"))`: `evacuate_marked_heap_for_moving_gc` compacts an already-marked heap into a fresh dense cell/cold-payload arena, drops the free list, retains only live labels/weak ids, clears stale mark scratch, and then uses the shared remapper to fix heap edges, cold payloads, program roots, BFILE read-only memory roots, node-pointer slots, eval spines, persistent spines, scratch roots, and machine stack roots. Coverage test builds a heap with dead holes, cold `BytesView`/`Array` payloads, BFILE and node-pointer roots, labels, and transient stack/spine roots, then verifies all ids are rewritten and the compact heap has no free-list state. Verification passed `cargo fmt --check`, default `cargo check`, `cargo check --features moving-gc`, `cargo check --features eval-phase-profile`, `cargo check --features gc-phase-profile`, `cargo check --all-targets`, `cargo check --all-targets --features eval-phase-profile`, `cargo check --all-targets --features moving-gc`, `cargo test --lib` 43/43, release `mhs-rust-bench` build, release `mhs-rust-bench --features moving-gc` build, release `wasm32-unknown-unknown` build, release `wasm32-wasip1 --bin mhs-rust` build, and default 100M/32M sanity (`1773.044ms`, `56.86M` steps/s, `100,811,779` steps, `3` GCs, `197.456ms` GC pause, high-water `33,939,304`, sink `661902`, `cell_size_bytes=8`). No full self-host gate: default release does not compile the evacuation scaffold |
| next moving prototype | wire a feature-gated collector path that runs mark/weak/finalizer handling, extends the live set for any `node_pointers` that must preserve guest pointer integers, then calls the full-heap evacuation scaffold and finally resets allocation cadence. After that passes, narrow it toward nursery-only evacuation |
| avoid | do not start with another per-allocation/per-write barrier check. The S4 data already showed barrier-check scaffolding can dominate even when remembered-set hits are tiny |

## 2026-07-05 Current 8B Fair-Memory Interval Refresh

| item | result |
|---|---|
| setup | zero-code full self-host refresh after packed cells became unconditional and the simplification pass landed. Built current release `mhs-rust-bench` and ran `MHS_GC_NODE_INTERVAL=83886080` (80Mi allocation window), `MHS_STEP_LIMIT=none`, `timeout 900s`, three samples. This is the current fairer-memory point: high-water is `88,151,784` 8-byte cells (~705MB of cell arena, plus mark/cold/vector overhead), much closer to eval.c's 800MB heap than the 128M speed setting |
| 80Mi interval 3x | byte-identical outputs (`cmp=0`, SHA `29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a`) at `70.101s`, `68.975s`, `63.538s`; median `68.975s`, average `67.538s`. All samples used `3,659,074,812` steps, `49` GCs, high-water `88,151,784` cells, sink `661902`, and `cell_size_bytes=8`. GC pause samples were `13.369s`, `13.167s`, `12.244s`; median `13.167s` |
| reading | 8-byte cells change the fairness picture: the old 40M/16-byte fair companion (`74.876s` at `46.41M` 16-byte cells) is now too conservative for current layout. At roughly C-sized memory, current Rust is in the high-60s band rather than the mid-70s band, while the 128M speed setting remains faster at ~62s by using a larger arena (`138.07M` 8-byte cells) and fewer GCs (`31` vs `49`). The structural gap is still GC/locality: 80Mi spends ~13.2s in GC versus ~10.0s at 128M and C's 47.85s total oracle |

## Active Tradeoffs

| item | reading |
|---|---|
| persistent-spine pure/IO alias path | superseded by the S1 stack hot path; the old dispatcher is unreachable, but deleting it as a standalone cleanup regressed the full gate to 88.171s, so keep it until a perf-neutral cleanup/outline lands |
| WHNF frames for `seq`/`IO.strict`/`isint` | required; without them a `seq` force near 78k ran millions of nested reductions |
| FFI/BFILE long continuations | current targeted refresh shows most small rows now beat C. The committed read-only memory BFILE view is isolated to host-buffer/BFILE paths, improves absolute Rust time for `bfile-read-chain:200` and `utf8-bfile-read-chain:200`, and passes the 77.533s full self-host gate, but same-run ratios are still above C (`bfile-read-chain:200` 1.44x, `utf8-bfile-read-chain:200` 1.49x, `ffi-mem-chain:200` last refresh 1.17x). Keep these as canaries, but return the main effort to F2/S1 app allocation/descent/update work unless a BFILE change is similarly isolated |

## Next

| priority | work |
|---|---|
| 1 | Continue performance-neutral simplification, but only where it removes real surface area or stale profiling/runtime structure. Do not spend more time shaving public API unless it deletes meaningful coupling |
| 2 | When returning to R1, use the `moving-gc` feature-gated remapper as the fixup substrate for the first full-heap or nursery evacuation prototype. Preserve guest pointer integers by remapping `node_pointers` slots and rebuilding `node_pointer_slots` |
| 3 | Gate any sizeable runtime simplification or enabled R1 behavior with the current protocol: 8-run alternating 100M/32M A/B for source neutrality, and paired 3x full self-host at the 128M speed window when the change touches evaluator, allocator, active GC roots, dispatch layout, or large runtime code shape. Test/feature-gated scaffolds still need `cargo fmt --check`, default/feature checks, `cargo test --lib`, release bench build, wasm/browser build, and WASI build |
| 4 | Keep S2's outermost-only GC guard until nested evaluator roots disappear; do not collect inside recursive `reduce_node_whnf` calls. For non-self-host parity/perf, keep `ffi-mem-chain`, `bfile-read-chain`, and `utf8-bfile-read-chain` as canaries, but do not let them displace the self-host gate |
