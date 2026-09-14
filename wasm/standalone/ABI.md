# Standalone WAT runtime contract

This directory implements a runtime with no C or Rust compilation step.
The runtime has passed compiler self-hosting in Wasmtime and Chromium.
The separate hybrid control remains available through `../build.sh`.

The build assembles WAT fragments and links the WAT reducer.
The output imports host operations from WASI Preview 1. A browser
provides those operations through the TypeScript WASI adapter. General foreign
calls use JS or WASM imports. The runtime does not resolve C symbols or
call C function pointers.

## Graph memory

Each node occupies eight bytes. Offset 0 contains the function pointer or tag.
Offset 4 contains the argument pointer, a 32-bit scalar, or a payload pointer.
Int64 and Double point to separate eight-byte managed payloads. Nodes have
eight-byte alignment. This keeps the common application node compact.

- An application stores its function pointer directly at offset 0.
- An indirection stores its target pointer with bit 1 set.
- A tagged value stores `(tag << 2) | 1` at offset 0.
- Tags retain the existing serialized primitive meanings.
- A byte string uses tag 9. Offset 4 contains its descriptor pointer with bit 0
  set. A foreign pointer uses the same tag with that bit clear. `$bval` removes
  the bit before descriptor access.
- A byte string descriptor contains byte length at offset 0 and data pointer
  at offset 4. Substrings can share a data allocation.
- An array uses tag 11. Its descriptor contains length at offset 0, followed
  by node pointers at offset 4.
- A raw pointer uses tag 7. Offset 4 is a linear-memory address.

The node arena and payload arena do not move. The collector uses the node
bitmap and a separate payload mark bit. Payload allocations have size classes
and free lists. Each payload header records its size and pointer-field mask.
An array marks all node elements. A byte buffer contains no graph references.
An interior pointer keeps its containing allocation alive.

Collection shortens IND chains in known graph links. Root addresses and
conservative payload words remain unchanged. The mark loop loads four object
headers at a time and reloads an IND header if another trace can have changed
it. Weak-key lookup uses the same cycle-safe resolver. See `HEAP.md` for these
invariants and `THREADS.md` for weak reachability.

The following addresses are reserved. Changes must update the runtime and this
table together.

| Start | Use |
| --- | --- |
| `0x00000100` | State words exported to the reducer |
| `0x00001000` | Permanent integer table and runtime roots |
| `0x00008000` | Static names and messages |
| `0x00010000` | Application stack, up to 1,000,000 entries |
| `0x00500000` | Continuation frames, 64 bytes per frame |
| `0x00900000` | Collector work stack |
| `0x00d00000` | Ordered payload-block index |
| `0x01800000` | Generated foreign names, at most 8 MiB |
| `0x02000000` | Node arena, 75,000,000 eight-byte nodes |
| `0x26000000` | Free-node bitmap, 9,375,000 bytes |
| `0x28000000` | Payload arena, grows with linear memory |

## Allocation and collection

Allocation functions never invoke the collector or evaluator. They either
allocate, grow payload memory, or report resource exhaustion. The evaluator
collects only at its top loop, before a primitive starts. A primitive must
finish its graph update before control returns to that loop.

There is no recursive evaluator call. A strict primitive pushes a continuation
frame and evaluates each required argument in the main loop. Its arguments and
partial results remain in the graph stack or continuation frame. The collector
therefore does not need to inspect the engine's native call stack.

The root set contains the current node, permanent runtime nodes, the application
stack, continuation frames, argument array, and live thread state. The parser
uses the application stack for temporary roots. Parsing cannot collect.

## Shared helper functions

WAT fragments form one support module. They can call these functions by name.
All addresses and node references use i32.

| Function | Result and contract |
| --- | --- |
| `$node(tag)` | Allocate a zeroed tagged node. Does not collect. |
| `$ap(function, argument)` | Allocate one application. Does not collect. |
| `$int(value)` | Return a boxed i32. Use permanent small integers when possible. |
| `$int64(value)` | Box an i64. |
| `$double(value)` | Box an f64. |
| `$float(value)` | Box an f32. |
| `$ptr(address)` | Box a linear-memory pointer. |
| `$blob_alloc(size, mask)` | Allocate payload bytes. Mask bit N identifies pointer word N. Mask -1 scans every word. |
| `$blob_free(address)` | Release an owned payload allocation. |
| `$bytes(address, length)` | Copy bytes and return a byte-string node. |
| `$bytes_view(address, length)` | Return a byte-string descriptor over existing payload memory. |
| `$string(address, length)` | Build a lazy Haskell string from UTF-8 bytes. |
| `$prim(tag)` | Return the permanent primitive node for a tag. |
| `$heap_gc_target(node)` | Follow and shorten IND links with cycle detection. Does not evaluate, allocate, or mark. Use during collection. |
| `$prim_lookup(address, length)` | Resolve a serialized primitive name. Return its node or zero. |
| `$service_lookup(address, length)` | Resolve a fixed runtime service or generated foreign import. Return its index or -1. |
| `$service_arity(index)` | Return the number of scalar arguments, excluding World. |
| `$fail(code)` | Report a runtime failure and stop. Does not return. |
| `$parse(address, length)` | Parse an uncompressed v8.4 program and return its root. |
| `$parse_prefix(address, length)` | Parse a graph. Return zero and publish a Haskell exception on invalid input. |
| `$parse_f64(address, length)` | Parse a serialized decimal floating-point literal. |

Service names in existing `.comb` files can retain their historical spelling.
They identify a fixed runtime operation, not a C ABI entry. Unknown service
names must fail explicitly. They must not silently return a default value.

## Continuations and primitive calls

A primitive receives evaluated arguments. It must not invoke evaluation.
The evaluator controls strictness and argument order. Integer and byte-string
binary primitives evaluate the right argument before the left argument, as the
existing evaluator does. Lazy arguments remain graph nodes.

A continuation frame has sixteen i32 words. The first eight hold frame kind,
opcode, saved application-stack index, parent base index, update node, arity,
strictness mask, and next argument index. The last eight hold graph references.
Exception handlers use an explicit frame kind. Raising an exception unwinds
frames to the nearest handler. No host exception is needed for Haskell control
flow.

RNF uses frame kind 3 and a managed pending-node vector. Its bitmap and vector
remain in the frame's reference words. State `0x3054` holds the active `noerr`
flag. State `0x3058` holds the active bitmap address. See `RNF.md` for the frame
layout and the guarded primitive semantics. Unwinding releases these payloads.
An uncaught exception restores the entry stack depth and clears all frames,
so the exported evaluator can be called again.

The scheduler preserves explicit evaluator state in managed snapshots.
Blocked primitives return zero with switch request 2 at `0x309c`. They retry
their primitive head after wakeup. Atomic evaluation uses frame kind 4.
See `THREADS.md` for thread records, MVars, asynchronous exception delivery,
WASI polling, stable pointers, and weak-pointer collection.

## Acceptance checks

1. Assemble and instantiate the runtime without a C or Rust compiler.
2. Check graph reductions, primitive results, exceptions, and collection against
   an existing evaluator.
3. Compile the compiler to a new `.comb` file and reach a target fixed point.
4. Run the same module in Wasmtime and Chromium through WASI.
5. Exercise direct JS and WASM foreign calls without a C wrapper.
6. Compare self-host performance with identical compiler input and source files.

The existing pure Haskell `lib/no-gmp` Integer implementation removes the
bignum C dependency. All matched performance controls use that same compiler
program and library selection. See `BOOTSTRAP.md` for the initial image and
target fixed-point commands.
