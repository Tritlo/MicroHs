# Standalone command and IO

The WASI command accepts `INPUT.comb -- PROGRAM [ARG ...]`. It loads one complete
bytecode buffer, constructs the argument list, and evaluates `main World`.
Normal completion returns status 0.

The exported `initialize` function sets up the heap and WASI state. The exported
`evaluate(root)` function returns a graph node on success. It returns zero for
an uncaught Haskell exception and retains that exception at `0x3004`. It restores
the caller's application stack and releases evaluator continuations before it
returns. The exported `parse(address, length)` function requires a complete
bytecode buffer. Its malformed-input failures use the fatal command diagnostic.

`IO.print` and `IO.serialize` force the value to weak head normal form, then call
`$serialize`. They write the completed byte buffer through the selected BFILE.
The buffer already contains its final LF. `IO.pp` prints to stderr and preserves
the argument's current graph without evaluation. Unsupported graph values raise
the catchable Serialize exception, runtime code 8.

`IO.deserialize` calls `$bfile_read_record` to read exactly one record. The record
reader accounts for names, numeric tokens, quoted escapes, and raw byte counts.
A brace inside a byte string does not end the record. The original BFILE retains
its unread trailer, including decoded Unicode values and CRLF state.
`$parse_prefix` validates the record. It returns zero and raises Deserialize,
runtime code 9, for malformed input. Its detailed first failure code remains in
`$parse_status`. The caller releases the temporary input buffer after parsing.

The command displays an uncaught typed exception through the existing
`SomeException` dictionary representation. The display function is the graph
`U (U (K2 A))`, as in the C and Rust runtimes. `$collect_string` evaluates one list
cell and one character at a time through separate calls to `$execute`. It then
encodes each character as modified UTF-8. It does not call the evaluator from a
primitive or from an active evaluator frame.

The command uses these temporary roots during exception display:

| Address | Value |
| --- | --- |
| `0x3060` | Original exception |
| `0x3064` | Current String list |
| `0x3068` | Mutable byte-string builder |
| `0x3070` | Complete exception message |

Runtime exception integers 0 through 9 map to stack overflow, heap overflow,
thread killed, user interrupt, DivideByZero, blocked MVar, blocked STM,
arithmetic overflow, Serialize, and Deserialize. Other integers display as an
unknown runtime exception. A failure inside the display method produces the
message `exception display failed`. The original exception remains available.

The command retains the existing display-text convention for `ExitSuccess` and
returns status 0. It also recognizes `ExitFailure N` and returns status N without
an uncaught-exception message. Status zero in an explicit failure becomes 1.
A custom exception with the same display text follows the same exit handling.
Other exceptions print the program name and message to stderr, then return 1.
