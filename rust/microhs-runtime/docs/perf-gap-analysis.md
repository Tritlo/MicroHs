# Performance gap analysis: Rust non-PGO vs C non-PGO

Status: final analysis for the 2026-07 perf effort. No runtime change is implied
by this document.

## Bottom line

The remaining gap is real but narrow: Rust non-PGO is about 46.8s vs C non-PGO
about 44.3s at the 128M-cell self-host heap, or 1.06x C. The same Rust source
with PGO reaches C non-PGO parity, so the gap is not an algorithmic GC problem
and not a missing high-level reducer transformation. It is the last codegen/layout
margin inside the already-hot reducer.

The best current verdict is:

- Most of the residual 5-6% is source-hostile block layout and caller/callee
  integration that PGO recovers in `reduce_whnf_from_stack` plus the inlined
  `stack_eval_step` body.
- The remaining source-addressable representation lever, packed `App` fun/arg
  unpacking, was measured and rejected by the wide-cell prototype. Removing the
  shifts made the program slower, not faster.
- GC is not the remaining gap. Full self-host GC behavior is almost identical
  between default and PGO Rust binaries, and PGO closes the wall gap without any
  GC algorithm change.

This is not a proof that no future source transform can win 1-2%, but it is enough
to call the rewrite done: the known simple source levers have been exhausted, and
the validated shippable representation is the compact 8-byte packed cell.

## Evidence base

Commands used for this capstone:

```sh
env CARGO_TARGET_DIR=/tmp/cap-default \
  cargo build --release --manifest-path rust/microhs-runtime/Cargo.toml \
  --bin mhs-rust-bench

MHS_GC_NODE_INTERVAL=134217728 MHS_PGO_DIR=/tmp/cap-pgo \
  rust/microhs-runtime/tools/native/build-selfhost-pgo.sh

objdump -d --start-address=0x7ba40 --stop-address=0x86399 \
  /tmp/cap-default/release/mhs-rust-bench > /tmp/cap-default-stack.dis

objdump -d --start-address=0x86470 --stop-address=0x8adf5 \
  /tmp/cap-default/release/mhs-rust-bench > /tmp/cap-default-reduce.dis

objdump -d --start-address=0x366a0 --stop-address=0x471bc \
  /tmp/cap-pgo/use/release/mhs-rust-bench > /tmp/cap-pgo-reduce-stack-inline.dis
```

Both full self-host outputs from the default and PGO binaries produced:

```text
29b8c5a55e0952bd25a9a06d5723033e0367ebaf53cfd598fa00f8e65a80d98a
```

The bounded 80M-step no-GC callgrind comparison, using the same binaries and
`--cache-sim=yes --branch-sim=yes`, was:

| build | Ir | I1mr | Bim | note |
|---|---:|---:|---:|---|
| default release | 14.109B | 29.33M | 59.70M | standalone `stack_eval_step` |
| PGO release | 13.707B | 2.64M | 59.05M | `stack_eval_step` inlined into `reduce_whnf_from_stack` |

This is the strongest single result in the analysis. PGO's deterministic frontend
win is overwhelmingly I-cache/block-layout: I1 misses drop by about 91%, while
indirect branch mispredicts barely move. The indirect dispatch is still expensive,
but PGO did not close the wall gap by making the known-prim jump table predictable.

## PGO disassembly diff

The default release has a standalone hot reducer:

```text
000000000007ba40 000000000000a959 stack_eval_step
0000000000086470 0000000000004985 reduce_whnf_from_stack
000000000008b060 0000000000000de9 finish_ready_stack_frame
0000000000095c00 0000000000004ca6 fallback_runtime_prim_rewrite
```

The PGO release has no standalone `stack_eval_step` symbol. Its body is merged
into `reduce_whnf_from_stack`:

```text
00000000000366a0 0000000000010b1c reduce_whnf_from_stack
0000000000034640 0000000000001608 finish_ready_stack_frame
000000000004a620 0000000000008ebf fallback_runtime_prim_rewrite
```

The combined default text for `reduce_whnf_from_stack + stack_eval_step` is about
0xf2de bytes. The PGO integrated reducer is about 0x10b1c bytes. PGO did not make
the reducer smaller. It made the executed layout denser.

### Basic-block reordering and hot/cold placement

Default starts the hot `stack_eval_step` at `0x7ba40`, then reaches the known-prim
dispatch through:

```text
0x7baf8  mov    (%rdx,%rax,8),%rax     ; load packed Cell by NodeId
0x7bafc  mov    %eax,%ecx
0x7bafe  and    $0xf,%ecx              ; low tag
0x7bb01  cmp    $0x3,%rcx              ; Prim tag
0x7bb0b  mov    %rax,%r15
0x7bb0e  shr    $0x4,%r15              ; prim id payload
0x7bb2f  jmp    *%rcx                  ; known-prim jump table
```

PGO moves the equivalent dispatch into the caller body:

```text
0x3697c  mov    0x0(%rbp),%rax         ; nodes pointer
0x36983  mov    (%rax,%rbx,8),%rax     ; load packed Cell by NodeId
0x36987  mov    %eax,%ecx
0x36989  and    $0xf,%ecx
0x3698c  shr    $0x4,%rax
0x36990  cmp    $0x3,%rcx
0x369a7  movslq (%r9,%rcx,4),%rdx
0x369b1  jmp    *%rdx
```

The operation is the same, but the placement is not. In default, callgrind
attributes 12.13M I1 misses to `stack_eval_step` alone. In PGO, the whole inlined
`reduce_whnf_from_stack` body has only 0.92M I1 misses. That is too large to
explain by one cold arm or one macro; it is machine-block placement over the
integrated reducer.

Estimated share of PGO's 5-6% wall win: 2.5-4.0 percentage points.

Source-addressability: low. Round 6 tried source `#[cold]`/`#[inline(never)]`
splits of rare paths around the standalone function boundary. The PGO diff shows
that the winning transform is not "split those same rare arms out of
`stack_eval_step`"; it is caller/callee integration plus profile-guided block
placement. Source cold-splitting can move blocks, but it cannot reproduce the same
fallthrough graph or global machine-block placement without the profile.

### Register allocation and spill/reload shape

Default `stack_eval_step` has a 0x128-byte stack frame:

```text
0x7ba40  push %rbp; push %r15; ...; push %rbx
0x7ba4a  sub $0x128,%rsp
```

PGO's integrated `reduce_whnf_from_stack` has a much larger 0x538-byte frame:

```text
0x366a0  push %rbp; push %r15; ...; push %rbx
0x366aa  sub $0x538,%rsp
```

Static stack-reference counts are higher in the integrated PGO disassembly
(3471 `%rsp` references vs 1473 in default `stack_eval_step`), so PGO is not
winning by simply deleting all spills. The plausible register-allocation win is
narrower: it removes the hot call boundary between `reduce_whnf_from_stack` and
`stack_eval_step`, so the caller's reducer state, stack state, and budget state can
be allocated through the fused loop instead of being materialized for each call.

Estimated share of PGO's wall win: 0.5-1.5 percentage points, overlapping with
block layout.

Source-addressability: medium in theory, low in practice. A source
`#[inline(always)]` on `stack_eval_step` would force the same boundary removal, but
it would do so without PGO's block placement and with all rare paths dragged into
one default-layout body. That is exactly the risk exposed by round 6: source layout
nudges worsened I1/Bim when they were not coupled to profile-guided placement.

### Inlining decisions across the function boundary

This is the most concrete PGO-only difference. Default has both symbols; PGO has
no `stack_eval_step` symbol. Callgrind confirms the hot work moved:

| default self Ir | function |
|---:|---|
| 12.514B, 88.70% | `stack_eval_step` |
| 0.223B, 1.58% | `reduce_whnf_from_stack` |

| PGO self Ir | function |
|---:|---|
| 12.805B, 93.42% | `reduce_whnf_from_stack` including inlined stack step |

The PGO integrated function is larger and has more static calls and indirect jumps
than default `stack_eval_step` alone, but it executes fewer instructions overall
(13.707B vs 14.109B program Ir). This points to profile-guided inlining plus
layout, not static code-size reduction.

Estimated share of PGO's wall win: 1.5-2.5 percentage points.

Source-addressability: partially, but not safely isolated. Manual inlining is
source-addressable; the profile-guided choice of what to inline around it and how
to place the resulting blocks is not.

### Branch layout and fallthrough

Default `stack_eval_step` has one static indirect jump in the standalone disasm:
the known-prim jump table at `0x7bb2f`. PGO's integrated reducer has 13 static
indirect jumps, because it includes the caller and other dispatch sites. Despite
that, bounded Bim is slightly lower under PGO:

```text
default Bim: 59.70M
PGO     Bim: 59.05M
```

The important branch result is negative: PGO did not materially solve indirect
branch prediction. Conditional branch mispredicts are actually higher in the PGO
callgrind model (100.6M -> 132.7M), while I1 misses collapse. Therefore the
source-level target is not another known-prim arm reorder; round 1 already took
the easy profile-ordered dispatch win, and round 6's layout attempts did not find
more.

Estimated share of PGO's wall win: mostly via I-cache/fallthrough locality,
included in the basic-block estimate above; direct Bim improvement explains little.

Source-addressability: low. Hottest-first source ordering is already present in
the combinator match (`B`, `C`, `S'`, `C'`, `P`, ...). Further source arm moves are
likely noise unless guided by a fresh profile and verified by wall A/B.

## Per-combinator instruction accounting

The top source-ordered hot combinator arms in `stack.rs` are `B`, `C`, `S'`, `C'`,
and `P`. The C oracle handles the same cases in `evali` with `CHKARGn` and
`GOAP/GOAP2` macros:

```c
#define CHKARG3 do { CHECK(3); POP(3); n = TOP(-1); \
  z = ARG(n); y = ARG(TOP(-2)); x = ARG(TOP(-3)); } while(0)
#define CHKARG4 do { CHECK(4); POP(4); n = TOP(-1); \
  w = ARG(n); z = ARG(TOP(-2)); y = ARG(TOP(-3)); x = ARG(TOP(-4)); } while(0)
#define GOAP(f,a)  do { FUN(n) = (f); ARG(n) = (a); goto ap; } while(0)
#define GOAP2(f,a,b) do { FUN(n) = new_ap((f), (a)); ARG(n) = (b); goto ap2; } while(0)
```

The C hot path pays direct pointer field loads and stores:

- `TOP(i)` is pointer-stack indexing.
- `ARG(node)` is one direct load.
- `FUN(node)`/`ARG(node)` rewrite direct struct fields.
- `new_ap` allocates a node and writes tag/fun/arg directly.

Rust's default path, after the round-4 unchecked-index work, is still different:

- Load `Cell` by indexed `NodeId`: `mov (%nodes,%id,8),%cell`.
- Decode tag: `and $0xf`.
- For `App` arguments, unpack payload with `shr $0x22` for right/arg and
  `shr $0x4` plus mask for left/fun.
- Allocate through `push_node`/free-list state and update allocation accounting.
- Repack rewritten cells into the 8-byte `Cell`.
- Re-enter the stack loop through Rust enum/result control flow rather than C's
  local `goto ap/ap2`.

Approximate per-reduction instruction counts from the default disassembly shape
and callgrind hot-path totals:

| combinator | rewrite | C evali rough path | Rust default rough path | Rust extra cause |
|---|---|---:|---:|---|
| `B` | `B x y z = x (y z)` | 25-35 insn | 55-75 insn | three NodeId arg loads, one `App` allocation, packed unpack/repack, allocation counters |
| `C` | `C x y z = x z y` | 30-40 insn | 65-90 insn | `GOAP2` shape: one new app plus redex rewrite; same unpack/counter tax |
| `S'` | `S' x y z w = x (y w) (z w)` | 45-65 insn | 95-130 insn | two new apps plus final rewrite; more packed app creation and stores |
| `C'` | `C' x y z w = x (y w) z` | 40-60 insn | 90-120 insn | one inner app, one outer app/rewrite; same stack arg path |
| `P` | `P x y z = z x y` | 30-40 insn | 65-90 insn | `GOAP2` equivalent; one new app plus redex rewrite |

These are deliberately ranges, not cycle claims. The disassembly is optimized and
interleaved with stack-frame and capacity checks, so exact dynamic attribution by
combinator would require hardware sampling or a purpose-built reducer trace. The
stable finding is the attribution:

- NodeId indexed load is not the main Rust-vs-C delta. It is a single indexed
  load, comparable to C's pointer-field load.
- Packed fun/arg unpack is visible but limited. Default `stack_eval_step` contains
  62 static `shr $0x22`, 300 static `shr $0x4`, and 628 references to the
  30-bit mask. The wide-cell prototype removed the target shifts, but wall got
  worse by the manager-measured 14.46%.
- Allocation/accounting and rewrite structure are larger than C's `NEWAP`/`GOAP`
  macros, but the simple source cuts have already been mined. Round 4's trusted
  indexing and allocation-path cuts were the last large source-addressable win.
- Dispatch remains expensive, but PGO's Bim result shows the final gap is not
  primarily the indirect jump table.

## GC accounting

The full default self-host run used for this capstone:

```text
parse_reduce_render_total_ms: 43633.102
whnf_steps_per_s: 83868767.8
gc_collections: 31
gc_high_water_nodes: 138023837
gc_total_pause_ms: 8484.841
```

The full PGO run:

```text
parse_reduce_render_total_ms: 41676.541
whnf_steps_per_s: 87806099.4
gc_collections: 31
gc_high_water_nodes: 138023829
gc_total_pause_ms: 9065.365
```

Both runs free exactly 4,167,749,112 nodes over 31 collections. Typical
collections have only about 0.5M-3.8M live nodes and about 133M-136M freed nodes
from a 134M-138M-node arena, so the collector is sweeping a mostly-dead arena.
That is a large absolute cost, but it is not the residual C gap:

- PGO reaches parity without changing GC semantics or representation.
- PGO's GC pause total is slightly higher in this run, while total time is lower.
- The wide-cell experiment roughly doubled memory/RSS and was rejected, which
  confirms that preserving GC bandwidth is more important than deleting a few
  unpack instructions.

I did not remeasure the C collector in this capstone run. The conclusion that GC
is at or near parity relies on the manager-provided standing measurement and on
the PGO fact above: if Rust GC were the remaining 6%, PGO of the mutator source
would not close the gap while leaving GC work effectively unchanged.

## Reconciliation of the perf rounds

Round 4 translated to wall because it removed actual per-step mutator work:
trusted stack/arena indexing and allocation-path tightening cut instructions on
the path that runs for nearly every reduction. That was a real representational
overhead cut.

Round 5 cut Ir but regressed wall because the remaining loop is throughput and
frontend sensitive. At this point, fewer modeled instructions are not necessarily
faster if the instruction mix, alignment, or block layout loses throughput.

Round 6 source layout recovery failed because it attacked layout at the wrong
abstraction boundary. The PGO disassembly shows no standalone `stack_eval_step`;
the win is integrated `reduce_whnf_from_stack` layout. Splitting rare source arms
out of the standalone hot function cannot reproduce PGO's fused fallthrough graph
and register allocation. In fact, the no-GC PGO/default counters show that the
main frontend difference is I1 misses, not indirect branch prediction.

Rounds 7-8 killed the last plausible representation rewrite. The 16-byte wide
cell removed the visible packed-app shifts (`shr $0x22` and `shr $0x4`) but was
manager-measured at +14.46% wall and 2x memory. That validates the packed 8-byte
cell: the shifts are cheaper than the memory bandwidth, GC sweep bandwidth, and
larger-cell instruction overhead needed to remove them.

## Source-addressable residue

The only source-addressable transforms still worth considering are very small and
high-risk:

1. A profile-specific manual inline of `stack_eval_step` into
   `reduce_whnf_from_stack`, accepted only by load-controlled wall A/B. This is
   the closest source analogue to PGO's biggest structural change, but it risks
   reproducing round 6's failure without PGO's block placement.
2. A narrow manual split of a block proven by PGO disassembly to be cold inside
   the integrated reducer, not merely cold in source. This requires comparing a
   fresh PGO block map against default and should be treated as a one-off
   experiment, not a new optimization campaign.

Everything else has been falsified or bounded:

- More dispatch arm reordering: bounded by unchanged Bim under PGO.
- Wider cells: falsified by wide-cell wall/RSS.
- More unchecked indexing: largely done in round 4.
- GC bitmap/sweep tweaks: useful only if independently measured, but not the
  remaining C gap.

## Final verdict

The residual 6% is not mathematically "PGO-only" in the sense that no source code
could ever reproduce it. It is PGO-only for practical purposes in this codebase:
PGO's observed win is fused reducer layout and block placement, while every
simple shippable source analogue has either already landed, lost, or been
validated as the wrong tradeoff. The packed 8-byte `Cell` should remain the
default, and the non-PGO Rust reducer should be considered at its empirical floor
for the current algorithm and representation.
