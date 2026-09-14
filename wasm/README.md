# Handwritten WebAssembly runtime

`standalone/` implements the MicroHs runtime in WebAssembly text format (WAT).
It assembles and links with Binaryen. It does not compile C or Rust to WASM.
The same command module runs in Wasmtime and in a browser with a WASI adapter.
The compiler uses the existing pure Haskell `lib/no-gmp` Integer implementation.
Foreign functions use typed JS or WASM imports.

The standalone runtime has compiled the compiler and reproduced its output on
another self-host pass. Chromium produced the same bytes as Wasmtime. See
`../LOG.md` for the recorded inputs, output hashes, experiments, and limits.
These results establish compiler execution. Component checks cover additional
runtime services that the compiler does not call.

## Build and use the standalone runtime

Required tools are Binaryen (`wasm-as`, `wasm-opt`, and `wasm-merge`) and a WASI
host. Binaryen 124 and Wasmtime 46.0.1 were tested. Set `BINARYEN_BIN` when these
Binaryen tools are not on `PATH`.

```sh
export BINARYEN_BIN=/path/to/binaryen/bin
wasm/standalone/build.sh
wasmtime run -W max-wasm-stack=8388608 --dir .::. \
  target/mhs-wasm/mhs-standalone.wasm compiler.comb \
  -- ./bin/mhs -i -imhs -isrc -ilib/no-gmp -ilib MicroHs.Main \
  -otarget/mhs-wasm/compiler-next.comb
```

The input must use the pure Haskell Integer backend. Put `-ilib/no-gmp` before
`-ilib` when compiling it. A compiler image that imports GMP services is not a
standalone input. GHC can produce the initial compiler image. Subsequent compiler
passes run in WAT. The program name after `--` becomes the Haskell program name.
See [BOOTSTRAP.md](standalone/BOOTSTRAP.md) for the complete seed and fixed-point procedure.
Add another `--dir` preopen if the program needs files outside the project.
Node and JavaScript are not required for terminal execution.

`--output FILE.wasm` selects an output path. `--debug` skips optimization of the
support module. Reducer helper inlining remains enabled. The build enables
specific WebAssembly features. It avoids `--all-features`, which can enable
experimental encodings unsupported by browsers.

## Direct JS and WASM imports

```haskell
foreign import wasm "math add" add :: Int -> Int -> Int
foreign import javascript "return $0 + $1" addJS :: Int -> Int -> Int
foreign import wasm "state set" set :: Int -> IO ()
```

Compile with `-oPREFIX.wat`. The compiler emits WAT call bindings, typed imports,
a companion `.comb` program, and JS metadata. Pass `--ffi PREFIX` to the
standalone build. A `MODULE=FILE.wat` or `MODULE=FILE.wasm` argument links another
WASM module directly. The host can also supply its instance exports at runtime.

The scalar types are `Int`, `Word`, `Int64`, `Word64`, `Float`, and `Double`.
Pure and `IO` results are supported, including `IO ()`. JS receives exact i64
values as `bigint`. The target supports at most six scalar arguments. Pointers,
heap objects, callbacks, and foreign exports are outside this interface.
Arbitrary C imports are rejected. Historical standard-library names identify
fixed WAT runtime services and do not perform C symbol lookup.

See [the FFI contract and examples](standalone/FFI.md) for complete commands,
signedness rules, generated file formats, and browser integration.

## Browser execution

The adapter is [browser-wasi.ts](../rust/microhs-runtime/tools/wasm/wasi/browser-wasi.ts).
It supplies files, arguments, environment, standard streams, and WASI polling.
See [its API documentation](../rust/microhs-runtime/tools/wasm/wasi/README.md).
Pass additional WASM instance exports as imports, or load the generated JS
metadata. The adapter preserves unsigned scalar values and exact i64 values.

Run long compiler work in a worker. Execution is synchronous. Clock polling can
use `Atomics.wait` in a worker with shared-memory support. Hosts that cannot
block that way use a timed polling fallback. The module has no browser-specific
imports beyond the host operations supplied through WASI and explicit foreign
imports.

## Implementation map

The design reuses checked behavior from both existing runtimes. C supplies the
serialized graph contract and exact formatting reference. Rust supplies useful
iterative traversal, MD5, and state-management patterns. WAT uses explicit
continuations and a nonmoving heap. Neither runtime is compiled into the
standalone module.

| Source | Responsibility |
| --- | --- |
| [ABI.md](standalone/ABI.md) | Memory layout, roots, module interfaces, and continuations |
| [reducer.wat](reducer.wat) | Graph traversal, indirection compression, core combinators, tuples, and tags |
| `standalone/heap.wat`, [HEAP.md](standalone/HEAP.md) | Bitmap node allocation, managed payloads, marking, and reclamation |
| `standalone/evaluate.wat`, `plan.wat` | Strictness, evaluation order, continuation frames, and exceptions |
| `standalone/threads.wat`, [THREADS.md](standalone/THREADS.md) | Cooperative scheduling, blocked operations, and thread roots |
| `standalone/rnf.wat`, [RNF.md](standalone/RNF.md) | Iterative full graph evaluation and cyclic graphs |
| `standalone/loader.wat`, `names.wat`, `serialize.wat` | Graph input, primitive names, sharing, and output |
| [SERIALIZE.md](standalone/SERIALIZE.md) | Graph format, streaming input, and floating-point text compatibility |
| `standalone/values.wat`, `numeric.wat`, `float-parse.wat` | Boxed scalars, checked arithmetic, and exact decimal parsing |
| `standalone/primitives-*.wat` | Byte strings, arrays, I/O sequencing, and state operations |
| `standalone/services-*.wat` | Memory access, numerical library functions, MD5, and file operations |
| [MATH.md](standalone/MATH.md) | Numerical algorithms, source notices, accuracy, and special cases |
| `standalone/wasi*.wat`, `command.wat` | WASI calls, command arguments, process entry, and diagnostics |
| [COMMAND.md](standalone/COMMAND.md) | Entry-point contract, exception display, and exit status |
| [FFI.md](standalone/FFI.md) | Generated scalar JS/WASM bindings and host requirements |

Source comments describe each helper and its invariants. Allocation never
collects or evaluates. The evaluator publishes graph references before each
collection. Strict arguments use continuation frames instead of recursive
calls through the engine stack. Haskell exceptions use those frames; the
standalone runtime does not require WebAssembly exceptions.

The standalone heap uses eight-byte application nodes. It holds 75 million
nodes in the same 600 MB arena that held 50 million twelve-byte nodes in the
initial port. Rare Int64 and Double values use separate payload blocks. The
collector skips pointer traversal for raw payload blocks. This layout reduced
full compiler time in the diagnostic experiments recorded in `../LOG.md`.
Wide scalars share the payload allocator's 1,048,576-entry index limit.
The selected collector also shortens indirection chains and loads four object
headers before scanning them. Cycle checks and preserved root addresses keep
these changes compatible with suspended evaluation and weak pointers.

Two other WASM experiments were rejected. A `return_call` evaluator passed the
compiler check but ran slower than the loop. A typed WasmGC graph probe worked
in Wasmtime and Chromium, including cyclic graphs, but gave no speed advantage
over a temporary arena with known graph lifetimes. That arena does not perform
general collection, so the probe does not establish complete-runtime costs.
Core WasmGC also lacks the weak-reference and finalizer operations this runtime
uses. See the [GC proposal's remaining requirements](https://github.com/WebAssembly/gc/blob/main/proposals/gc/Post-MVP.md#weak-references).

## Matched self-host benchmarks

The final standalone runtime took 48.82 seconds at the median of three runs
on an AMD Ryzen 9 7950X under WSL2. Every run compiled the current compiler from
the same 696,922-byte input and frozen sources. All outputs matched their target
references. These are process wall times; Wasmtime AOT compilation is excluded.

| Runtime | Median seconds | Range seconds | Median peak RSS, MiB |
| --- | ---: | ---: | ---: |
| Native C | 50.22 | 48.46–50.40 | 788.6 |
| Native Rust | 52.54 | 50.65–52.63 | 747.9 |
| C compiled to WASM | 75.05 | 74.45–75.86 | 607.9 |
| Rust compiled to WASM | 71.53 | 71.33–72.04 | 1560.2 |
| WAT reducer with C support | 50.56 | 50.54–51.03 | 607.9 |
| Standalone WAT | 48.82 | 48.32–50.14 | 613.1 |

Standalone WAT used 35.0% less time than C-WASM and 31.7% less than Rust-WASM.
Its median was also 2.8% below native C and 7.1% below native Rust. The native C
and standalone ranges overlap, so the C result is a modest median advantage.
These measurements describe this compiler workload and these build settings.

The native and final standalone samples come from the final confirmation in
`target/mhs-wasm/selfhost.080E62/`. The compiled-WASM and hybrid samples come
from `target/mhs-wasm/selfhost.DdSU6T/`, using the identical source/input workload.
Each row has three samples. In the final confirmation, the previous standalone
collector measured 53.59 seconds; the selected collector reduced that by 8.9%.
The complete candidate comparison and hashes are in `../LOG.md`.

The Rust controls used allocation interval 78,643,200. C-WASM used Clang `-O3`
and whole-module Binaryen `-O3`. The hybrid used reducer helper inlining without
whole-module optimization, which had slowed that variant. Wasmtime 46.0.1 used
`-O opt-level=2` and default engine inlining. GCC 13.3.0 built native C; Rust
1.97.1 used the release profile. The standalone build uses Binaryen 124.

[standalone/bench.sh](standalone/bench.sh) compares native C, native Rust,
C-WASM, Rust-WASM, standalone WAT, and the optional hybrid control. It requires
Linux and GNU `time`. Supply a source directory, one compiler input, and a
five-column TSV manifest without a header:

```text
native-c	native-c	/absolute/path/mhsbench	/absolute/path/native-expected.comb	-
native-rust	native-rust	/absolute/path/mhs-rust-bench	/absolute/path/native-expected.comb	78643200
c-wasm	wasi-c	/absolute/path/c.wasm	/absolute/path/wasm-expected.comb	-
rust-wasm	wasi-rust	/absolute/path/rust.wasm	/absolute/path/native-expected.comb	78643200
wat	wasi-wat	/absolute/path/standalone.wasm	/absolute/path/wasm-expected.comb	-
```

```sh
MHS_REPEAT=3 wasm/standalone/bench.sh source-tree compiler.comb runtimes.tsv
```

The runner copies sources, input, executables, and expected outputs before
measurement. Wasmtime AOT compilation occurs outside the timed process. All
runs compile `MicroHs.Main` with the same `lib/no-gmp` search order. Alternate
rounds reverse runtime order. Every output must match its reference byte for
byte. The result directory contains hashes, complete logs, output files, and
`results.tsv`. Process wall time includes module startup and parsing.

Record the Rust allocation interval explicitly. It controls collection after a
number of allocations, not a memory limit. C nodes occupy 16 bytes natively and
12 bytes on wasm32. Rust cells occupy 8 bytes. Standalone WAT reserves 75 million
8-byte nodes plus separate payload memory. Equal node counts are not equal
memory budgets. Compare maximum process RSS as well as time.

WAT uses 32-bit `Int` and `Word`. Rust retains 64-bit machine integers on WASM.
The verified compiler outputs differ only in two signed encodings of the
32-bit bitmask `0x80000000`. Keep separate exact native and wasm32 references.
Do not normalize away output differences.

## Hybrid and C controls

[build.sh](build.sh) retains the initial experiment: WAT handles core graph
reduction and compiled C supplies the remaining runtime. This path requires a
WASI SDK with `setjmp`/`longjmp` support, plus Binaryen. Clang 21.1.8 was tested.

```sh
export WASI_SDK=/path/to/wasi-sdk
wasm/build.sh --evaluator c --output target/mhs-wasm/c.wasm
wasm/build.sh --evaluator wat --output target/mhs-wasm/hybrid.wasm
```

These commands use the C benchmark argument interface. Run with
`--mode main --warmup-iters 0 --iters 1 INPUT.comb -- PROGRAM ARGS...`.
Wasmtime needs `-W exceptions=y,max-wasm-stack=8388608`. `--program FILE.c` builds
compiler-generated C instead. `--optimize` applies Binaryen `-O3` to the linked
module; compare that option on both controls. `--profile` adds reducer exit
counters for diagnostic builds. Use builds without counters for timing.

The older [bench.sh](bench.sh) compares only C-compatible benchmark modules.
It freezes a Git source commit selected by `MHS_SOURCE_REF` and uses the default
Integer library. It is not the standalone no-gmp comparison runner.

## Target limits

WASI Preview 1 limits files to preopened directories. This implementation has
no sockets, process creation, host shell, signals, or terminal raw mode.
Unavailable operations report failure. Process CPU time returns zero because
Preview 1 has no process CPU clock. The compiler can emit `.comb`, `.wat`, and
source files, but it cannot start an external compiler through WASI.

The loader accepts uncompressed v8.4 graphs. Standalone code must use
`lib/no-gmp`; GMP nodes and services are not part of this runtime. Raw pointers
refer to linear memory. The heap does not move, and managed interior pointers
retain their allocation. Host imports cannot retain Haskell heap references or
invoke collection. Resource exhaustion is a runtime failure.

The branch also corrects a C `imath.c` unsigned-remainder defect found during
32-bit oracle checks. A separate pre-existing `lib/no-gmp` conversion defect
can affect native `Int` or `Word` values wider than 32 bits. The compiler
workload does not trigger it, and the standalone target has 32-bit machine
integers. This experiment does not repair that unrelated library behavior.
