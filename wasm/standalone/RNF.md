# Graph normal-form traversal

The serialized `rnf` primitive takes a machine integer flag and a graph.
It returns the unit constructor after it traverses the graph.
`rnf.wat` implements the traversal. `evaluate.wat` supplies each required WHNF
result through the normal evaluator loop. There is no recursive evaluator call.

## Traversal semantics

The traversal marks each node address before it evaluates that node to WHNF.
If the result is an application, it visits the function and then the argument.
It does not enter array elements, byte buffers, or other payload descriptors.
This follows the runtime's application-graph traversal. It is not a traversal
of every reference that the collector can trace.

A visited bitmap prevents repeated work for shared nodes and constructor cycles.
A cycle through a demanded function spine can still diverge during evaluation.
The traversal does not convert a divergent computation into a value.

The integer flag selects `noerr` behavior. Zero uses normal evaluation.
A nonzero value leaves these primitive applications unreduced:

- `RAISE`
- `IO_PERFORMIO`
- `BSFROMUTF8`
- `BSUNPACK`
- `RNF`

The traversal still visits those applications' function and argument links.
This preserves the existing guarded-primitive behavior. It is not a general
exception handler. Arithmetic overflow and division by zero can still raise.
It does not suppress every possible fully applied IO primitive.
The `IO_PERFORMIO` guard prevents ordinary embedded IO actions from being run.

An RNF call with zero can start another RNF call while it evaluates a graph.
The inner call saves and restores the outer traversal state.
An RNF application encountered while `noerr` is set remains unreduced.

## Continuation and pending nodes

Each RNF call uses one kind-3 continuation frame. It does not use one frame for
each graph edge. A managed vector contains the pending graph nodes.
It uses a last-in, first-out order. The traversal pushes the argument before
the function, so it visits the function first.

The vector starts with 256 entries. It doubles when full.
Its payload uses pointer mask `-1`, so every pending node remains a GC root.
Pop clears the vacated word. Growth copies the live entries and installs the
new vector in the frame before it releases the old allocation.

The kind-3 frame uses these words:

| Byte offset | Meaning |
| --- | --- |
| 0 | Frame kind, 3 |
| 4 | Serialized RNF tag |
| 8 | Saved application-stack top |
| 12 | Saved parent evaluation base |
| 16 | RNF application to update when traversal completes |
| 20 | Primitive arity, 2 |
| 24 | Previous `noerr` flag |
| 28 | Unused after strict flag evaluation |
| 32 | Evaluated integer flag node |
| 36 | Original graph argument |
| 40 | Visited-bitmap payload address |
| 44 | Pending-vector payload address |
| 48 | Number of pending nodes |
| 52 | Pending-vector capacity |
| 56 | Previous active-bitmap address |
| 60 | Primitive-head node |

The standard frame scanner retains all pointer fields at offsets 32 through 60.
The current node remains in the evaluator root while WHNF evaluation proceeds.
State `0x3054` contains the active `noerr` flag.
State `0x3058` contains the active visited-bitmap address.
These words belong to the evaluator. Callers must not use them as custom roots.

## Collection and address reuse

The visited bitmap has one bit for each of the 75,000,000 node addresses.
It requests 9,375,000 bytes from the payload allocator, whose size class is
16 MiB. Its pointer mask is zero. Bitmap bits are not payload pointer words.
The continuation frame retains the bitmap allocation.

A graph update can make a visited node unreachable. Collection can reclaim
that node and later reuse its address. After each safe-point collection,
`$rnf_after_gc` clears visited bits for free node addresses in every active RNF
frame. It preserves bits for live nodes. Thus address reuse does not skip a new
node, and collection does not lose cycle detection for a live graph.

The traversal allocates and updates its vector without collecting.
Payload pressure can request collection. The main evaluator consumes that
request at its next safe point, after all references are in the frame or roots.

## Completion and exceptions

On normal completion, the traversal restores the previous active state and
releases its bitmap and vector. The evaluator then updates the RNF application
to the unit constructor and removes the frame.

Exception unwinding performs the same cleanup for each discarded RNF frame.
It stops at the nearest exception-handler frame. An RNF frame outside that
handler remains active and retains its own traversal state.

An uncaught exception remains in the exception root at `0x3004`.
The exported `evaluate` function returns zero. Before returning, it restores
the caller's entry stack depth and clears the continuation count.
A later `evaluate` call can therefore run without host-side stack repair.
Successful evaluation also restores the entry stack depth.

Invalid pending node addresses fail with code 80.
Pending-vector arithmetic that would exceed its supported capacity fails with
code 79. Normal allocation limits use the heap failure codes in `HEAP.md`.
