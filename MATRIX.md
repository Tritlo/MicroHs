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
| committed checkpoint | `self-hosting-binary-match` |
| active working tree | local scratch notes only |
| compiler scope | unchanged; keep the compiler in Haskell |
| runtime scope | Rust replacement for the C runtime/evaluator and host support |
| node layout | `Node` is 32 bytes; cold `ForeignPtr`, `JsCall`, `Weak`, and `MutableBytes` payloads are boxed |
| parity state | benchmark and smoke sinks match; Rust self-host compile now produces byte-identical output, but still misses the original 600s target |

## Verification Gates

| gate | status |
|---|---|
| Rust fmt/test/release bench build | passed on the F2 persistent-spine + pointer + GC worktree |
| wasm library | not rerun after the F2 persistent-spine worktree |
| common benchmark sinks | matching |
| rare/smoke benchmark sinks | matching; no longer expanded here |
| self-host `--help` proxy | sink-comparable with C using `--c-mhsbench-mode main` |
| full self-host compile | completes with GC enabled in 673.8s and produces byte-identical output; still above the 600s gate and far above the long-run 120s target |

## Open Work

| area | state |
|---|---|
| evaluator/F2 | persistent-spine path is fast enough for bounded self-host slices; remaining perf work is mostly long FFI/BFILE continuation spines |
| GC/F5 | first safe-point mark/sweep scaffold is implemented; root completeness is only slice-smoked so far |
| JS FFI | wasm shim smokes pass; high-level browser glue still pending |
| `IO.serialize`/sharing/cycles | incomplete; defer until evaluator/GC shape is clearer |
| self-hosting | full compile succeeds and matches the C-generated comb byte-for-byte; remaining target is runtime below 600s, then below 120s |

Smoke-only subsystems now stay out of the live table unless they regress:
MVar, StablePtr, Weak, ForeignPtr, MD5, compression, process/env/filesystem,
BFILE codecs, and broad FFI coverage.

## Common Performance Snapshot

Same-machine measurements after 32-byte nodes, lazy spine arguments, borrowed
parser tokens, and large-input parser preallocation. Ratios and sink agreement
are more useful than raw timings.

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

Small scalar and mixed rows still mostly beat C. The current F2 worktree
regresses long FFI/BFILE continuation rows because the persistent path keeps
large spines live across those calls. The full self-host path now has semantic
parity against the C-generated compiler comb: Rust produces byte-identical
output, but takes 673.8s versus the original 600s gate.

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
- dead nodes are tombstoned as `Indir(None)` and reused through a free-list;
- live roots include the current root, original root, labels, stable pointers,
  cached prims, small ints, world, argument array, strict-frame stack,
  persistent/eval spines, scratch spine vectors, and node-backed pointer
  targets reachable through live `ForeignPtr`/`Ptr`/`FunPtr` values;
- weak values/finalizers are currently marked conservatively; this keeps the
  first pass safe but does not implement C's weak/finalizer semantics yet;
- default trigger is every 16M arena slots; `MHS_GC_NODE_INTERVAL=N` can force
  smaller intervals for smokes, and `0` disables the collector.

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

GC interval sweep on the 100M self-host main slice:

| `MHS_GC_NODE_INTERVAL` | time | steps/s | collections | freed nodes | high-water nodes |
|---:|---:|---:|---:|---:|---:|
| `0` | 19.44s | 5.19M | 0 | 0 | 117,763,780 |
| `4,194,304` | 16.15s | 6.24M | 7 | 114,433,273 | 29,625,311 |
| `16,777,216` | 16.13s | 6.25M | 3 | 99,289,532 | 50,596,826 |
| `67,108,864` | 16.66s | 6.05M | 1 | 66,690,767 | 67,374,042 |

The 16M default is the best reading so far: same bounded speed as 4M with fewer
collections, substantially faster than grow-only, and the full gate now
completes in 673.8s with fallback primitive prefix materialization capped.

This is intentionally not full C GC parity yet: no finalizer execution, no weak
clearing, no GCRED, and no collection inside helper allocations. The useful
property is narrower: bounded self-host graph shapes survive safe-point
collection and can reuse arena slots without moving live `NodeId`s.

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
| one evaluator loop should own strict forcing | mostly | generic strict helper folding is done; remaining F2 work should keep moving strict eval paths into the loop |
| compact node representation matters | yes | 56 -> 32 bytes helped self-host; preserve cache locality while changing evaluator structure |
| avoid eager spine argument forcing | yes | matching C's lazier descent cut resolve calls roughly in half |
| parser allocation matters, but is no longer primary | mostly | borrowed tokens and large-input prealloc fixed Rust-only parser tax; avoid more parser work unless measurements point back there |
| app update/rebuild traffic is central | partial | app-site profiling shows normal combinator rewrites dominate app allocation; remaining target is spine carrying/rethreading and arena/cache pressure |
| persistent spine needs one owner | mostly | self-host slice resolve/rewrite volume collapsed; Vec-backed persistent spine regressed, so keep the current owner shape for now |
| primitive heads want better representation | profiled | F8/P3 probe counts are measurable, but enum-variant and sparse side-table dispatch both regressed full self-host; only revisit with a compact in-node id |
| GC is a performance feature | partial | safe-point mark/sweep exists; long-run root completeness and C finalizer/weak semantics are still open |

## Current Theory

The remaining self-host blocker is now throughput, not correctness of the full
gate. C has a compact tag-dispatch reducer that overwrites redex cells; Rust
now has side-table synthetic pointers and a first non-moving safe-point
collector, but still allocates almost one `App` per WHNF step in the early
self-host slice. App-site profiling says that allocation is mostly normal
combinator rewrite payloads, so the near-term target is the overhead around
large persistent spines, rewrite rethreading, and 32-byte arena locality rather
than another standalone primitive-dispatch cache. The full compile completes in
707.8s at ~5.17M WHNF steps/s. The immediate gate is under 600s; the long-run
target is under 120s.

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
| enum-variant arithmetic `Prim` tags | removed the string cascade but regressed 1M slice to 248.2 ms and full self-host to 770.1s |
| sparse `NodeId -> PrimDispatch` side table | removed the same cascade but regressed 1M slice to 220.5 ms and full self-host to 731.3s |
| Vec-backed persistent spine with head at the back | targeted long-spine locality but regressed 10M slice from 2,729.2 ms to 2,860.6 ms |

## Active Tradeoffs

| item | reading |
|---|---|
| persistent-spine pure/IO alias path | worth keeping: 50k slice resolve calls fell to 3.1k and 1M app rewrites are gone |
| WHNF frames for `seq`/`IO.strict`/`isint` | required; without them a `seq` force near 78k ran millions of nested reductions |
| FFI/BFILE long continuations | currently regressed; do not optimize narrow rows ahead of self-host unless the same long-spine ownership issue is fixed generally |

## Next

| priority | work |
|---|---|
| 1 | reduce full self-host from 707.8s to below the 600s gate; keep using `timeout 900s` until lower caps are justified |
| 2 | P2/P1: reduce large-spine carrying/rethreading and arena/cache pressure; app-site profiling says the app allocation sites are mostly normal combinator rewrites |
| 3 | F8/P3: revisit primop tag dispatch only with an in-node compact id or other shape that avoids the rejected enum/side-table overhead |
| 4 | reduce long FFI/BFILE continuation-spine overhead without regressing the self-host path |
