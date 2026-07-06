# Decision record: keep the packed 8-byte cell

**Status:** decided (2026-07-06). The packed 8-byte `Cell` is retained. A 16-byte
"wide" cell was prototyped, measured, and **rejected**. Do not retry without a
materially different idea.

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
