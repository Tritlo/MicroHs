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

## GC

GC is non-moving mark-sweep. It runs between top-level reduction steps once the
allocation interval is reached, marks all graph roots, marks through active eval
state and host handles, then sweeps the arena into an intrusive free list threaded
through dead cells. Weak pointers and foreign finalizers are resolved after the
mark phase.

The collector preserves node ids. That is simpler for host references and graph
sharing, and it is why the runtime can keep raw `NodeId` handles in BFILE views,
stable pointers, FFI records, and temporary eval stacks.

## Host Boundary

Host operations are grouped under `runtime/host` and `runtime/program`:

- `host/` contains platform services, filesystem access, bigint helpers, render
  callbacks, and browser JavaScript FFI shims;
- `program/bfile/` owns MicroHs BFILE handles and byte-oriented IO;
- `program/runtime_dispatch/` handles runtime primitives that leave the pure
  reducer path.

Native, WASI, and browser wasm share the evaluator and heap. Target-specific
files should stay at the host boundary so the reducer remains target-independent.

## Measurement Contract

`MATRIX.md` is the live ledger for performance and correctness gates. The main
end-to-end gate is self-hosting: Rust runs the MicroHs compiler `.comb` to compile
MicroHs itself, and the output must compare byte-identically with the C runtime.
Small refactors can use `cargo check`, `cargo test --lib`, and profile smokes;
changes that touch evaluation, allocation, GC roots, dispatch shape, or large
runtime layout should use the self-host neutrality harness and full self-host
runs recorded in `MATRIX.md`.
