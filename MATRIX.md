# MicroHs Rust Matrix

Updated: 2026-07-04

Working dashboard for the Rust rewrite of the MicroHs C runtime/evaluator. The
compiler stays in Haskell; Rust consumes and executes compiler-produced `.comb`.

`EVALLESSONS.md` and `NOTES.md` are local scratch material. This file tracks
only the baseline, the current bottleneck, and decisions that should steer the
next rewrite work.

## Current State

| item | state |
|---|---|
| branch | `microhs-rust`, ahead of `origin/microhs-rust`, not pushed |
| committed checkpoint | `self-hosting-binary-match` |
| active working tree | S2 GC heap-loop/instrumentation plus mark-time indirection compression, eval.c-style app-edge parent-slot canonicalization for indirections/small ints, reusable mark-work stack, feature-gated GC phase profiling, eval.c-style parse-label non-root treatment, and F14 shared `ForeignPtr` finalizer records run during sweep; S1 eval.c-style single-stack hot path, S1 app-only strict-redex/persistent-spine slices, S1 stack argument binding, S1 compact stack-entry split, S1 trusted stack argument loads, S1 unchecked stack arg loads, S1 trusted stack redex/update access, S1 raw cell tag accessors plus trusted free-list pop, S1 allocator free-head invariant tightening, S1 identity-alias `GOIND` continuation for `I`/`Ord`/`Chr`, S1 one-word app-fun spine descent, S1 app-only eval stack, S1 C-style app-continuation descent after `GOAP`/`GOAP2` rewrites, S1 direct `ap`/`ap2` app-result descent, S1 dedicated app allocation path, S1 fixed-arity `CHKARG`-style pop/take rewrites for hot stack combinators/IO graph rewrites, S1 strict `Int` redex-owning frames, S1 WHNF redex-owning frames for `seq`/`IO.strict`/`isint`, S1 unified stack `ret`/`top` continuation for strict-frame returns and force/`GOIND` continuations, direct known-app/free-cell writes, stack scalar `SET*`-style redex overwrites, runtime-primitive static fallback names, persistent/eval head classification, S3 parse-time primitive and small-int interning, S3 authoritative 16-byte cell arena with cold payload side table, eval.c-style permanent compound caches for `fst`/`snd`/`Just`/`Pair Unit`, direct `RuntimePrim` strict-action dispatch, F12 catch masking-state restoration graph, F17 cold `BytesView` nodes for immutable substring/tail views, S0 evaluator-layer/stack-update/app-allocation profiling, feature-gated per-head/per-site stack phase/profile-overhead profiling, and bench representation-size output; local scratch notes present |
| compiler scope | unchanged; keep the compiler in Haskell |
| runtime scope | Rust replacement for the C runtime/evaluator and host support |
| node layout | bench now prints representation sizes; current release output is `node_size_bytes=16`, `node_id_size_bytes=4`, `prim_size_bytes=4`; the authoritative arena is `Cell { word0, word1 }` with App/Indir/Free/Prim/scalar tags in-cell, while `Node` is now a parse/debug facade for cold boxed payloads |
| parity state | benchmark and smoke sinks match; Rust self-host compile produces byte-identical output; best current-tree full gate is now the eval.c permanent compound cache run at 80.927s, byte-identical, 45.21M steps/s, and 23.793s GC pause; fresh C oracle full self-host is 47.85s |

## Verification Gates

| gate | status |
|---|---|
| Rust fmt/test/release bench build | passed after the S3 authoritative cell arena, S1 single-stack hot path, S2 GC + S1 app-only spine/head-classification worktree, rejected strict-tail/segmented-stack reverts, S3 parse-time primitive interning, S3 compact representation work, direct runtime-primitive strict dispatch, S1 stack argument binding, S1 compact stack-entry split, S1 trusted stack argument loads/static fallback names, S1 app-only eval stack, S0 layer/update profiling, feature-gated phase/profile-overhead profiling, strict Int immediate args, unchecked stack arg loads, reusable GC mark-work stack, feature-gated GC phase profiling, rejected bitmap allocator restore, S1 app-continuation descent, S1 direct app-result descent, S1 dedicated app allocation, stack scalar `SET*` redex overwrites, S1 fixed-arity `CHKARG` pop/take rewrites, direct known-app/free-cell writes, S1 strict `Int` redex-owning frames, S1 WHNF redex-owning frames, raw cell tag/trusted free-list accessors, parse-time small-int interning retest, F10 iterative serializer, F17 cold bytes views, F14 shared ForeignPtr finalizer records, F12 catch masking-state restoration, per-head/per-site eval profiling, bench representation-size output, S1 unified stack `ret`/`top` continuation, S2 parse-label non-root GC treatment, S1 allocator free-head invariant tightening, S1 identity-alias `GOIND` continuation, S1 one-word app-fun spine descent, S2 mark-time app-edge canonicalization, S1 trusted stack redex/update access, and eval.c permanent compound caches |
| wasm library | release `cargo check --target wasm32-unknown-unknown` passed after eval.c permanent compound caches; earlier debug/release wasm builds passed after S1 dedicated app allocation, and Node `host.mjs` import smoke rendered `42` from the release wasm |
| EVALLESSONS correctness batch | current `cargo test --manifest-path rust/microhs-runtime/Cargo.toml --lib` passes 37/37; covers F1 string escapes, F3 catchable arithmetic RTS exceptions, F4 uncaught RTS/showExn formatting and `ExitSuccess` handling, F6 shared/cyclic serializer labels, F7 unknown prim failures, F10 iterative deep serializer smoke, F11 mpz wire format, F12 catch handler masking/restoration smoke, F15 `putchar`/`lz77c`, F16 faithful modified UTF-8 decoding, F17 immutable bytestring views for `bssubstr`/`tailUTF8`, F18 ignored-IO shortcut depth cap, F19 compression fixture/round-trip coverage, and F21 `mpz_get_d` decimal rounding; F14 also has focused null-finalizer GC and direct `^&closeb` address-FFI smokes |
| common benchmark sinks | matching |
| rare/smoke benchmark sinks | matching; no longer expanded here |
| self-host `--help` proxy | sink-comparable with C using `--c-mhsbench-mode main` |
| full self-host compile | best current-tree eval.c permanent compound cache gate completes byte-identically in 80.927s with 3,659,075,078 steps, 45.21M steps/s, 125 GCs, 23.793s GC pause, high-water 38,820,730 cells, and the same output SHA; fresh C oracle full self-host completes in 47.85s with the same output SHA; tagged checkpoint remains `self-hosting-binary-match` at 673.8s |

## Open Work

| area | state |
|---|---|
| evaluator/S1/F2 | normal WHNF reduction now uses a plain app-id stack plus separate strict frames for the hot combinator, strict-primitive, and fallback primitive-helper paths; hot fixed-arity combinator and IO graph-rewrite arms pop/take stack args in eval.c `CHKARGn` style, read stack args with unchecked eval.c-style `ARG(TOP(i))` loads, trust the checked stack segment when fetching redex apps for hot rewrites/updates, test hot `Cell` tags through raw low bits, descend app spines by reading only the app fun/tag word before later `CHKARG` arg loads, overwrite the consumed app redex cell directly, continue through `GOAP`/`GOAP2`-style app results using direct `ap`/`ap2` root pushes, continue through strict force markers and K/A/I/Ord/Chr-family `GOIND` rewrites inside `stack_eval_step`, and allocate `Node::App` through a dedicated hot helper/direct trusted free-cell reuse path instead of generic `push_node`; reused app allocation now also trusts the `free_nodes > 0 => free_head is Some` invariant in release; strict `Int` binops and WHNF frames for `seq`/`IO.strict`/`isint` now pop the consumed redex before forcing and carry that redex as a frame root; `IO.strict` reuses the redex as the returned `action value` app instead of allocating a new app plus indirection; scalar strict-frame results overwrite the consumed redex cell instead of allocating a scalar node plus indirection; the hot stack path no longer emits a `StackStep::Force` handoff; strict-redex snapshots and `remaining_apps_contain` scans are zero in the 1M profile; remaining bridge hits are std-handle/control helpers |
| correctness/F batch | F1/F3/F4/F6-sharing/F7/F10-deep-serializer/F11/F12-catch-masking/F14-ForeignPtr-finalizers/F15/F16/F17/F18/F19/F21 are implemented or verified in the current tree; F20's std-handle flush-per-write issue no longer exists because writes and explicit flushes are separate; keep F10's broader `IO.print`/any-BFILE behavior as later work |
| parser/S3 | primitive occurrences and parsed small `Int` literals are interned at parse time, matching eval.c's singleton primitive and permanent small-int table model more closely |
| GC/F5 | safe-point mark/sweep plus S2 allocation-pressure trigger, intrusive free list, mark-time indirection compression, reusable mark-work stack, parse-label non-root treatment, optional `gc-phase-profile` mark/sweep timings, and C-style sweep-time ForeignPtr finalizers are implemented; full gates pass, but weak finalizer clearing/spawning and GCRED parity are still pending |
| representation/S3 | payload-out plus `NodeId` narrowing and compact runtime-primitive tags is now an authoritative 16-byte cell arena, not a mirrored side table; strict primitive classification dispatches directly from in-cell tags, cold payloads sit in a side table, and `Node` survives only as parser/debug/serialization interchange |
| JS FFI | wasm shim smokes pass; high-level browser glue still pending |
| `IO.serialize`/sharing/cycles | shared app graphs and cyclic app graphs now emit C-style `:label`/`_label` output using `NodeId` labels and round-trip through the existing parser; the serializer itself is now iterative and passed a 12k-deep WHNF app-chain smoke |
| self-hosting | full compile succeeds and matches the C-generated comb byte-for-byte; latest current-tree eval.c permanent compound cache gate is 80.927s versus fresh C oracle 47.85s, a new best and about 1.69x C |

Smoke-only subsystems now stay out of the live table unless they regress:
MVar, StablePtr, Weak, ForeignPtr, MD5, compression, process/env/filesystem,
BFILE codecs, and broad FFI coverage.

## Common Performance Snapshot

Older same-machine measurements after 32-byte nodes, lazy spine arguments,
borrowed parser tokens, and large-input parser preallocation. Ratios and sink
agreement are more useful than raw timings; rerun this table on the
authoritative-cell machine if the small scenarios matter again.

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
SHA; the current Rust full gate is about 1.69x slower. The current parser-small-int
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
| remaining serializer work | broader F10 `IO.print` behavior, especially any-BFILE/prefix output |
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

The current worktree has temporary `MHS_TRACE_INVALID_BYTES=1` diagnostics in
the ByteString and pointer paths. Remove or gate-clean these before a
committable checkpoint unless the next result proves they should become a real
debug flag.

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
| compact node representation matters | yes | 56 -> 32 -> 24 -> 16-byte authoritative cells helped self-host; bench now logs `node_size_bytes=16`, `node_id_size_bytes=4`, `prim_size_bytes=4`; primitive tags classify strict actions directly, the structural cell arena moved the full gate to 105.859s, the cell-era `CHKARG` pop/take/direct-write slice moved it to 99.022s, strict `Int` redex-owning frames moved it to 95.345s, WHNF redex-owning frames moved it to 93.202s, raw tag/trusted free-list accessors moved it to 91.503s, F14 ForeignPtr finalizer records moved it to 90.237s, S1 unified stack `ret`/`top` continuation moved it to 89.141s, S2 parse-label non-root treatment moved it to 88.623s, S1 allocator free-head invariant tightening moved it to 86.619s, S1 identity-alias `GOIND` continuation moved it to 86.500s, S1 one-word app-fun spine descent moved it to 85.189s, S2 mark-time app-edge canonicalization moved it to 83.752s, S1 trusted stack redex/update access moved it to 82.154s, and eval.c permanent compound caches now put the best full gate at 80.927s |
| avoid eager spine argument forcing | yes | matching C's lazier descent cut resolve calls roughly in half |
| parser allocation matters, but is no longer primary | mostly | borrowed tokens, large-input prealloc, primitive interning, and post-cell parse-time small-int sharing fixed Rust-only parser/initial-graph tax; the old small-int rejection is superseded, but parser-only work should still wait unless it reduces the hot live set or matches C representation directly |
| app update/rebuild traffic is central | partial | app-site profiling shows normal combinator rewrites dominate app allocation; S1 stack removes eager rethreading on the hot path, unchecked arg loads remove another eval.c-missing check from the hot rewrite arms, app-continuation descent avoids returning to the outer driver after app-producing rewrites, direct `ap`/`ap2` root pushes delete generic descent work on known app results, the dedicated app-allocation path removes generic app dispatch from the dominant node kind, compact profile-head frames remove a string-option payload from default strict-frame traffic, authoritative cells remove the old hot `Node` tag/payload layer, fixed-arity `CHKARG` pop/take plus direct app/free writes remove another generic stack/update layer, strict `Int` redex-owning frames remove the `app_end/used` replay layer for the hottest strict frame kind, WHNF redex-owning frames remove the same replay/allocation layer for `seq`/`isint`/`IO.strict`, raw tag/trusted free-list accessors remove another representation/invariant-check layer from App/Prim/scalar/resolve/free-list traffic, unified `ret`/`top` continuation removes another driver handoff around strict-force and `GOIND` returns, allocator free-head invariant removes the remaining release `Option` branch from reused app allocation, identity-alias `GOIND` removes another reduced-step handoff, one-word app-fun descent removes the arg-word load from the AP walk, trusted stack redex/update access removes the impossible-error `Result` layer from hot app rewrites/updates, and permanent compound caches remove C-avoided helper app construction for `fst`/`snd`/`Just`/`Pair Unit`; latest accepted full gate is 80.927s, but profiling still puts the next mutator buckets at app allocation, inner descent, arg reads, and app updates |
| persistent spine needs one owner | replaced on hot path | the accepted S1 stack is a replacement loop with explicit app-base tracking; do not return to `PersistentSpine` storage variants except as fallback cleanup |
| primitive heads want better representation | yes for current shape | parse-time primitive interning, compact `RuntimePrim(u16)`, in-cell primitive tags, and direct strict-action dispatch are implemented; fallback helper dispatch still uses names, but only on the cold/non-strict helper path |
| GC is a performance feature | partial | S2 closes arena growth, mark-time IND compression, reuses the GC mark-work stack, can profile mark vs sweep behind `gc-phase-profile`, no longer keeps parse labels as permanent roots, now runs C-style ForeignPtr finalizer records during sweep, and now rewrites App child slots during mark to resolved/cached targets like eval.c's pointer-to-slot mark path; S1 stack plus authoritative cells/compact stack entries/trusted args/app-only eval stack, strict Int immediate args/redex-owning frames, WHNF redex-owning frames, raw cell tag/free-list accessors, unchecked stack arg loads, C-style app-continuation descent, direct `ap`/`ap2` app-result descent, dedicated app allocation, compact profile-head frames, stack scalar `SET*` writes, fixed-arity `CHKARG` pop/take/direct app/free writes, parse-time small-int sharing, cfg-gated profiling cleanup, unified `ret`/`top` continuation, parse-label non-root treatment, allocator free-head invariant tightening, identity-alias `GOIND` continuation, one-word app-fun descent, mark-time app-edge canonicalization, trusted stack redex/update access, and permanent compound roots put the best default full gate at 80.927s; GC pause is still 23.793s on that gate, so GC remains material; weak finalizer/full GCRED semantics are still open |

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
redex/update access brought it to 82.154s, and eval.c permanent compound
caches bring it to 80.927s.
The parser-small-int retest reduces the parsed
graph from 160,961 to 153,185 cells and full high-water from 40,014,305 to
40,006,682 cells without the old 100M regression.
The latest accepted full gate
has 23.793s GC pause, so the next stretch is still a mix of mutator mechanics and
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
| direct inner `GOAP2` descent | pushed both the rewritten root app and a freshly allocated inner app for clean `GOAP2` shapes; 1M improved to 46.958ms and 100M to 3.467s, but full gates were 131.8s and 133.2s versus the then-best 131.6s direct-app gate despite byte-identical output, so reverted |
| guarded stack `GOIND` continuation | let K/A/KK/KA/K2/K3/K4 indirection rewrites continue inside `stack_eval_step` unless directly on a strict-frame boundary, matching part of eval.c's `GOIND(x); goto top`; first unguarded 10M hit `expected Int` because Rust's strict-frame `ret:` handling still lived in the outer loop, guarded 10M was 301.375ms, 100M was 2.577s, but the byte-identical full gate regressed to 97.466s with 28.303s GC pause versus the 90.237s best, so reverted. Superseded by the accepted unified `ret`/`top` slice, which moved ready strict-frame return handling into the stack loop and completed in 89.141s |
| numeric known-head stack dispatch | carried raw `u16` known-prim codes through `stack_eval_step` to avoid `KnownPrim` enum decode and match closer to eval.c's integer tag switch; 1M regressed to 39.156ms and 100M to 2.667s versus the fresh 37.802ms / 2.619s same-session baseline, so reverted; post-revert 1M sanity was 36.604ms |
| single known-head match dispatch | merged `IO.strict`/`seq`/`isint` into the main `KnownPrim` match to remove one known-head match on hot B/C/S reductions; same-session 100M regressed to 2.561s versus the 2.533s control, so reverted before a full gate |
| direct in-cell hot known-code dispatch | added a direct code fast path for the top B/C/S/S'/C'/C'B app-producing heads before `EvalHead` construction; same-session 100M regressed to 2.690s versus the 2.533s control, so reverted before a full gate |
| fun-only app descent helper | changed the inner and outer stack descent loops to use a helper returning only the app fun pointer instead of `app_fields`; same-session 100M regressed to 2.567s versus the 2.525s control, so reverted before a full gate |
| raw GC mark/sweep tag loop | changed mark/indirection compression to raw tag tests and inlined sweep free-list rebuild; `gc-phase-profile` 100M regressed from 230.779ms pause (73.508ms mark / 128.855ms sweep) to 236.916ms pause (73.721ms mark / 135.083ms sweep), so reverted |
| eval-time first-indirection compression after compact cells | made `resolve_for_whnf` short-circuit the entry indirection like eval.c; 1M regressed to 38.671ms, 100M was mixed at 2.556s then 2.618s, and the byte-identical full gate regressed to 98.036s with 27.551s GC pause, so reverted |
| FFI/JS arity-capped stack materialization | capped FFI/JS argument copying to declared arity (`ffi_arity + world`, JS tags, JS wrapper pair), dropping 10M `profile_arg_materialized_nodes` from 10,522,021 to 150,988, but the byte-identical full self-host gate regressed to 100.762s with 28.113s GC pause; the arity-reuse follow-up also left 100M worse/noisy at 2.792s, so reverted |
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
| const-specialized profile vs normal stack step | making `stack_eval_step` const-generic over profile mode looked neutral on 1M/100M, but the full gate regressed to 175.0s with byte-identical output, so reverted |
| broad eval-profile feature gate | compiling general profile checks out of default builds improved 100M slices to 3.77s/3.78s, but the full gate regressed to 152.7s with 33.7s GC pause and byte-identical output, so reverted |
| eval.c `GOPAIR` runtime-return shape enum | changed runtime/FFI helpers to return `Node`/`Pair`/`UnitPair` shapes so the stack path could overwrite the consumed redex as the outer pair app; the unoutlined 100M slice regressed to 2.772s, cold/noinline recovered 100M to 2.367s, but the byte-identical full gate regressed to 82.600s with 39.5M high-water versus the 80.927s compound-cache best, so reverted |
| focused stack app-site profile split | bypassed `app_with_site` from the stack hot path when profiling was disabled; 10M regressed to 340.971ms, 100M was only noisy at 2.422s, and full self-host regressed to 85.721s versus the 82.154s trusted-stack best, so reverted |
| unused stack arg loads in discard-heavy combinators | changed `K`/`A`/`Z`/`J`/`L`/`KK`/`KA`/`O`/`K2`/`K3`/`K4` arms to read only used args instead of batched args; 1M regressed to 53.4ms and 100M was only noise-band at 4.06s, so reverted without a full gate |
| eval.c `GOAP2` allocation order | reordered independent `S'`/`B'`/`C'B` inner app allocations to match eval.c's textual `GOAP2` order more closely; 10M regressed to 385.102ms, 100M was only noise-band at 2.428s, and the byte-identical full gate regressed to 82.827s versus the 80.927s compound-cache best, so reverted |
| low-bit app tag encoding | changed `Cell.word0` to encode App as low bit `0` and all non-app tags as low bit `1`, reducing app-fun extraction from byte-tag/`>>8` to one-bit/`>>1`; after fixing the primitive tag decoder, tests passed and output was byte-identical, but 10M regressed to 396.531ms, 100M to 2.498s, and full self-host to 92.637s versus the 80.927s compound-cache best, so reverted |
| manual `Vec<u64>` GC mark bitmap | replaced the packed `Vec<bool>` mark map with direct word/mask operations; 100M improved slightly to 2.375s with 222.774ms GC pause, but the byte-identical full gate regressed to 84.578s with 25.491s GC pause versus the 80.927s compound-cache best, so reverted |
| thin LTO plus single release codegen unit | added workspace `profile.release` `lto = "thin"` and `codegen-units = 1`; build stayed cheap but 1M regressed to 54.0ms and 100M to 4.29s, so reverted without a full gate |
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

## Active Tradeoffs

| item | reading |
|---|---|
| persistent-spine pure/IO alias path | superseded by the S1 stack hot path; keep the old path only as a fallback reference while cold helpers move over |
| WHNF frames for `seq`/`IO.strict`/`isint` | required; without them a `seq` force near 78k ran millions of nested reductions |
| FFI/BFILE long continuations | currently regressed; do not optimize narrow rows ahead of self-host unless the same long-spine ownership issue is fixed generally |

## Next

| priority | work |
|---|---|
| 1 | Attack the measured post-raw-tag/free-list S1 buckets in order: app allocation, inner descent, arg reads, and app updates. Prefer a vertical eval.c-shaped change that removes work from those buckets over local codegen or scalar/tag probes |
| 2 | Push from the 80.927s eval.c permanent compound-cache best below 80s and then closer to C; use full self-host as the arbiter because the accepted strict-`Int` slice won full despite a slightly slower 100M bounded slice, the WHNF slice won full despite a noisier feature-build 10M control, the raw-tag/free-list slice won the full gate while the 10M feature-build control stayed noisy, F14's 100M slice was slower before the final cold fix while the full gate became the new best, the guarded-`GOIND` probe only became useful after the broader ready-return layer moved into the stack loop, the parse-label root fix won full despite a noisy/slower bounded mutator slice, the free-head invariant slice won both repeated 100M and full gates, the identity-alias `GOIND` slice won full only slightly despite large exit-counter reductions, one-word app-fun descent won the full gate despite a slower/noisy 100M bounded slice, app-edge canonicalization won full by changing heap shape despite a locally higher 100M GC pause, trusted stack redex/update access won full by removing a real Rust-only hot-path error layer, and permanent compound caches won full by copying eval.c's permanent helper-prefix shape |
| 3 | Keep S2's outermost-only GC guard until nested evaluator roots disappear; do not collect inside recursive `reduce_node_whnf` calls |
| 4 | Re-run common/rare rows after the next profile pass; long FFI/BFILE rows and ForeignPtr finalizer rows are the canaries for whether cold continuations and sweep-time host cleanup are still shaped correctly |
