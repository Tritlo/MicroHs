# Decision record: keep the packed 8-byte cell (with a gated wide fallback)

**Status:** decided (2026-07-06). The packed 8-byte `Cell` is the **default** and
was retained as such after a 16-byte "wide" cell was measured **slower** (below).
The wide cell is **not** the default, but it now also exists as an **opt-in
`wide-cell` cargo feature** — not for speed, but as a **correctness escape hatch**
that raises the arena index cap (see "Wide cell as a cap escape hatch" below). Do
not make the wide cell the default without a materially different idea.

## Context

The Rust self-host reducer sits at ~1.06x the non-PGO C runtime (byte-identical,
128M-cell heap). The workload is instruction/throughput-bound in `stack_eval_step`
(~88% of instructions; LL-cache miss ~0.2%, so *not* memory-latency-bound). After
micro-optimisation (round 5) and PGO-layout-recovery (round 6, fat-LTO +
cold-splitting) were exhausted, the remaining representational tax was the packed
`App` fun/arg **unpack** (shift/mask on the 8-byte word), ~5-8% of the hot
function, which C avoids via direct `FUN`/`ARG` struct fields.

## What was tried

Prototyped a **16-byte tagged-union `Cell`** (`tag + union{ids:{left:u32,right:u32},
u64, i64, f64}`) behind a `wide-cell` cargo feature — direct App fields, no
shift/mask. Scalar inlining (`Int64`/`Float64`) was deliberately deferred so
cold-table order (and thus byte-identity) was preserved. The default build stayed
8-byte `PackedCell` throughout.

The prototype **self-hosted byte-identically** at `cell_size_bytes: 16`, and the
target unpack instructions genuinely disappeared (`stack_eval_step`: `shr $0x22`
62→0, `shr $0x4` 300→0).

## Measured result — rejected

Load-controlled interleaved wall A/B, 128M cells, median of 5, byte-identical:

| build | wall (median) | RSS | Ir (80M no-GC) |
|---|---:|---:|---:|
| default 8-byte | **44.54s** | 1257 MB | 14.109B |
| wide 16-byte | **50.98s** | 2310 MB | 14.850B (+5.3%) |

**+14.46% slower, 0/5 pairs faster, 2x memory.** Removing the (cheap, well-predicted)
unpack shifts *added* more than it saved: the 16-byte cell costs extra tag handling
and wider data movement (+5.3% Ir), and the doubled arena roughly doubles GC-sweep
bandwidth and working-set cache pressure — which is the bulk of the ~9% wall gap
beyond the Ir delta. Branch mispredicts barely moved (−0.3%).

## Consequences

- The packed 8-byte cell is **validated**: its compactness is load-bearing (it is
  what keeps GC near C's), and the unpack tax it costs is far cheaper than the
  memory it would take to remove.
- The **representation lever is exhausted.** Together with rounds 5-6, this makes
  1.06x the empirical floor for this algorithm + representation. The residual ~6%
  to non-PGO parity is diffuse intra-function block layout recoverable *only* by
  PGO (which reaches C non-PGO parity on the same source but is not shipped, for
  reproducible-build reasons). It is not reachable by any shippable source change
  found so far.
- Structure-of-arrays (tags in a parallel array) and App-only side tables were
  analysed and not prototyped: both add a second hot load to remove cheap shifts,
  a poor trade on an instruction-bound-but-not-cache-bound loop.

## Wide cell as a cap escape hatch (`wide-cell` feature)

The packed 8-byte cell encodes `App`/`Indir` references as **30-bit** indices, so
the node arena is capped at `CELL_NONE_ID = 2^30 - 1 ≈ 1.07 B` cells; `push_cell`
asserts this and a program needing more **aborts** ("node arena exceeded packed
ids"). C's native pointers have no such ceiling. The self-host peaks at ~1-4 M
live cells (~250x under the cap), so this is not a practical limit today — but it
is a hard one.

For heaps that would otherwise abort, build with `--features wide-cell`:

- `Cell` becomes **16 bytes** with direct `u32` `left`/`right` App fields (no
  shift/mask), scalars still via `cold_nodes` (so the self-host stays
  byte-identical: SHA `29b8c5a5` under both features).
- `CELL_NONE_ID` rises to `u32::MAX`, so the cap becomes **~2^32 ≈ 4.29 B** cells
  (~4x today, ~68 GB of arena). A `#[cfg]` unit test round-trips a `NodeId` above
  `2^30` and asserts `size_of::<Cell>()` is 8 (default) / 16 (feature).
- **Cost, measured** (median-of-5 interleaved, Rust 1.96). The wide cell is 16
  bytes (2x the arena), so at the **standard equal-RSS ~787 MB budget** it holds
  only ~half the cells (~36 M vs ~75 M) and GCs ~twice as often: **+38% wall**
  (u8 47.2 s vs u16 65.2 s at ~796 MB matched). At equal *cell count* (128 M,
  which is not equal memory) it is +23% wall / 1.84x RSS. Either way it is a
  correctness fallback, **not** a performance option — enable it only when you
  would actually hit the packed cap. (If 4.29 B is ever insufficient, a 32-byte /
  u64-index tier is possible at ~4x memory; deliberately not built — 4.29 B is
  ~1000x the self-host's live-cell peak.)
