# MicroHs Rust Runtime Architecture

This runtime executes compiler-produced `.comb` files. The Haskell compiler is
still the source of truth for code generation; the Rust side is a runtime,
serializer, host environment, and benchmark harness.

## Core Model

The heap is a single arena indexed by `NodeId` (`u32`). Hot nodes live in one
8-byte `Cell`: a 4-bit tag plus either two 30-bit node ids or one compact scalar.
Wide or uncommon payloads (`Int64`, `Float64`, byte arrays, weak nodes, FFI
records, and similar values) are stored in a cold side table and referenced from
the cell.

This layout is the main performance contract:

- application nodes are one packed cell, so spine descent reads one word per
  application;
- node ids are stable across GC, so host handles and graph references remain
  valid;
- cold payloads stay out of the hot combinator path.

`Node` is the interchange/debug shape. It is used by the parser, serializer,
tests, and cold payload table, but the evaluator normally sees `Cell` directly.
That split is deliberate: do not route hot app descent or primitive dispatch
through `Node` unless the value is already known to be cold.

## Evaluator

Reduction is a single explicit-stack WHNF reducer. `EvalStack` has an application
spine (`apps`) and strict frames (`frames`). The hot path descends through packed
`App` cells, dispatches known combinators and runtime primitives, rewrites the
redex in place, then continues from the new head.

Strict primitives do not use recursion to force arguments. They push typed stack
frames (`Int`, `Float64`, `Bytes`, conversions, and WHNF frames), evaluate the
needed child to WHNF, then finish the frame. The fallback reducer is only for
cases the stack reducer deliberately delegates: dynamic host calls, JavaScript
FFI, and general runtime primitive paths that need the broader machinery.

The normal WHNF flow is:

1. Resolve indirections for the requested root.
2. Descend packed `App` cells, pushing app nodes onto `EvalStack::apps`.
3. Decode the non-app head as a known primitive, runtime primitive, FFI/JS call,
   or already-WHNF value.
4. For known combinators and simple runtime primitives, consume the checked app
   segment in eval.c `CHKARGn` style and rewrite the redex cell directly.
5. For strict primitives, push a typed frame and force only the child needed by
   that frame.
6. For delegated cases, rebuild the app arguments in head order and use the
   fallback reducer/runtime-dispatch path.

The code follows this split physically:

| module | role |
|---|---|
| `runtime/eval_state/` | transient reducer state: app stack, strict frames, eval spines, profile-frame payloads, and runtime-primitive head classification |
| `runtime/program/eval/stack.rs` | hot WHNF stack loop and known-combinator dispatch |
| `runtime/program/eval/frames.rs` | strict-frame completion for ints, floats, bytes, conversions, `seq`, `IO.strict`, and related forced values |
| `runtime/program/eval/reduce.rs` | fallback reducer, app-spine filling, and general delegated reduction |
| `runtime/program/eval/helpers.rs` | app construction, WHNF resolution helpers, profiling cold paths, and small evaluator utilities |
| `runtime/program/runtime_dispatch/` | runtime primitives that leave the pure reducer path: FFI, JS, arrays, refs, bytes, and pointer operations |
| `runtime/program/values/` | value forcing/projection plus byte, allocation, errno, syscall, and bigint helper methods used by runtime primitives |

The stack reducer is intentionally not abstracted behind traits or virtual
interfaces. It mutates the arena, eval stack, GC accounting, and profiler in one
tight loop; adding an abstraction there is a performance decision, not merely a
source-organization decision.

## GC

GC is non-moving mark-sweep. It runs between top-level reduction steps once the
allocation interval is reached, marks all graph roots, marks through active eval
state and host handles, then sweeps the arena into an intrusive free list threaded
through dead cells. Weak pointers and foreign finalizers are resolved after the
mark phase.

The collector preserves node ids. That is simpler for host references and graph
sharing, and it is why the runtime can keep raw `NodeId` handles in BFILE views,
stable pointers, FFI records, and temporary eval stacks.

The GC root set must include:

- the current reduction root and temporary eval stack contents;
- strict-frame payloads, including profile-head payloads when profiling is
  enabled;
- parser and serializer temporary labels while they are live graph references;
- BFILE handles, stable pointers, weak pointer keys/values/finalizers according
  to eval.c weak semantics, FFI allocations that point back into the arena, and
  host callbacks that store graph nodes.

If a new subsystem stores `NodeId` outside the arena itself, it needs an explicit
mark path or it must be proven temporary across GC safe points.

## Host Boundary

Host operations are grouped under `runtime/host` and `runtime/program`:

- `host/` contains platform services, filesystem access, bigint helpers, render
  callbacks, and browser JavaScript FFI shims;
- `program/bfile/` owns MicroHs BFILE handles and byte-oriented IO;
- `program/runtime_dispatch/` handles runtime primitives that leave the pure
  reducer path.

Native, WASI, and browser wasm share the evaluator and heap. Target-specific
files should stay at the host boundary so the reducer remains target-independent.

Current target split:

| target | boundary |
|---|---|
| native | std/libc filesystem, process, sockets, and native BFILE handles |
| `wasm32-wasip1` | CLI-shaped std/WASI path and native-style BFILE where WASI supports it |
| `wasm32-unknown-unknown` | browser host shims for filesystem/env/JS FFI, with no direct libc/syscall access |

The C/eval.c wasm32 path has a known integer-wrap/float divergence because its
`value_t` follows the target pointer width. The Rust wasm path uses the same
evaluator as native and should not special-case this in reducer code; target
quirks belong in host glue or comparison harnesses.

## Measurement Contract

The main end-to-end gate is self-hosting: Rust runs the MicroHs compiler `.comb`
to compile MicroHs itself, and the output must compare byte-identically with the
C runtime. Small refactors can use `cargo check`, `cargo test --lib`, and profile
smokes; changes that touch evaluation, allocation, GC roots, dispatch shape, or
large runtime layout should use the self-host neutrality harness
(`tools/native/bench-selfhost-neutrality.sh`) and a full self-host run.

For source-organization-only changes, the expected proof is:

- format/check/test across default, `profile`, and `gc-phase-profile`;
- wasm and WASI target checks when the touched modules are shared with those
  targets;
- bounded self-host sanity and a 3-run full self-host median when the change
  touches large runtime code shape or hot evaluator/allocator/GC modules.

### Reading performance changes

The self-host is instruction-bound, not memory-bound: callgrind's cache model
puts the last-level miss rate near 0.2%, and ~88% of all instructions run inside
`stack_eval_step`. Two consequences for how changes are judged:

- A deterministic bounded callgrind instruction count (e.g. an 80 M-step
  self-host prefix, `--tool=callgrind --cache-sim=yes --branch-sim=yes`) is the
  low-noise per-change arbiter, and predicts wall well for changes that remove
  work — bounds checks, indirection, redundant dispatch.
- It stops predicting wall at the layout-sensitive margin: inlining and
  branch-structure changes can cut instructions yet regress wall by shifting
  i-cache or branch-prediction behaviour. Confirm a whole optimization round with
  a load-controlled interleaved A/B (candidate vs base binary, alternating in one
  session); absolute cross-session wall drifts with machine load and is not a
  reliable signal on its own.

The current standing and gap analysis (Rust ~1.06x non-PGO C at a 128 M-cell heap;
the residual gap is representational — `NodeId`-indexed arena and cold side-table
versus C's raw `NODEPTR`/`FUN`/`ARG` access — not GC and not memory locality) live
in `rust/README.md`.
