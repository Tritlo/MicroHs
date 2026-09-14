# Graph serialization

`$serialize(root, header)` returns a byte-string node. The output includes a final
LF. A nonzero header selects the uncompressed v8.4 postfix format. A zero header
selects prefix printing. The function returns zero and raises runtime exception
8 when a graph contains an unsupported value. The caller writes the completed
byte string to its output stream.

The serializer does not evaluate or collect. Its first walk checks the graph and
marks shared applications, arrays, and byte strings. Its second walk writes the
graph. Both walks use an explicit stack. Each array frame retains one child
index, so a large array does not add all its elements to the stack at once.
Labels use node indices in the arena. The printer marks a shared node before it
visits that node's children. This permits cyclic graphs and forward references.
The first walk detects pure indirection cycles and raises exception 8.
Such cycles represent divergent fields. Cyclic applications remain supported.

Two temporary bitmaps cover the node arena. The output and walk stack grow in
managed payload allocations. The serializer releases all temporary allocations.
An unsupported value does not produce a partial byte string. Resource exhaustion
uses the runtime allocation failure path.

The accepted values are primitives, applications, indirections, machine
integers, Int64, Float, Double, byte strings, arrays, tick labels, fixed services,
and the null and `closeb` finalizer handles. Null pointers and standard streams
have portable graph representations. Other process-local pointers, foreign
pointers, MVars, weak references, and thread identifiers raise exception 8.

The Float and Double formatter retains the C runtime's 16 significant decimal
digits and its `.0` suffix rule. It converts the exact binary value to decimal
integer digits, then rounds ties to even. It does not call a host formatter.
The Rust runtime uses a shortest round-trip format instead. The retained C
format can change a Double after a serialize/parse round trip.
This is a text compatibility choice. It is not a lossless floating-point format.
