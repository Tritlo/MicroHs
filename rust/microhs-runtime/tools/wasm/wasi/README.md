# Browser WASI commands

`browser-wasi.ts` runs a compiled WASI command with the existing
`@bjorn3/browser_wasi_shim` dependency. It supports the C evaluator, the
handwritten WASM reducer, and the standalone WAT runtime. Use a worker for long compiler runs. The command runs
synchronously after module instantiation.

Pass WebAssembly exports directly to `imports`:

```typescript
import { runWasi } from "./browser-wasi";

const module = await WebAssembly.compileStreaming(fetch("/direct.wasm"));
const kernel = await WebAssembly.instantiateStreaming(fetch("/kernel.wasm"));
const result = await runWasi(module, {
  imports: { mhs_example: kernel.instance.exports },
  onStdout: (text) => console.log(text),
});
console.log(result.exitCode, result.stdout);
```

The imported functions stay in WebAssembly. The host does not wrap each call in
a JavaScript function. `wasm/build.sh` can also merge the modules. A merged module
does not need the `mhs_example` import object.

For standalone programs, `args` is `["runtime", "program.comb", "--", "program"]`
and `files` must contain the matching `program.comb` bytes. The standalone build
uses `wasm/standalone/build.sh`. See [direct foreign calls](../../../../../wasm/standalone/FFI.md)
for compiler commands and the `.wat` companion files.

Pass generated `.imports.json` metadata as the `javascript` option to run scalar
JavaScript imports. `javascript-imports.ts` also exports `createJavascriptImports`
for other hosts. It compiles synchronous `$0`-style bodies and preserves unsigned
`Word` and `Word64` values. JavaScript receives `Int64` and `Word64` as `bigint`.
The helper rejects Promise results and incorrect scalar result types.

`args` includes the program name. `env` maps names to values. `stdin` accepts text
or bytes. `files` maps guest paths to text or bytes. Parent directories are
created for input files. `directories` creates empty directories; its default
is `["tmp"]`. Absolute and relative paths use the same guest root. Paths must
not contain `..`.

The result contains the exit code, execution time in milliseconds, stdout,
stderr, and all files after execution. The returned file paths are relative to
the guest root. File data is copied. `onStdout` and `onStderr` receive text as
the command writes it. The captured output also includes text without a final
newline.

The adapter corrects the UTF-8 argument size calculation in shim version 0.4.2.
It also supplies Preview 1 `poll_oneoff` with multiple clock and file events,
48-byte subscriptions, 32-byte events, and the returned event count. Relative
and absolute realtime and monotonic timers are supported. Memory files and
console writes are immediately ready. Unknown descriptors produce `BADF`
events. This adapter does not supply asynchronous network descriptors.

An isolated worker uses `Atomics.wait` for timers when `SharedArrayBuffer` is
available. A main thread or a worker without shared memory waits synchronously
by polling the clock. Such a wait occupies that thread for the full delay and
prevents its JavaScript callbacks from running. Use a worker with cross-origin
isolation for programs that wait on timers. The host behavior follows the
[WASI Preview 1 specification](https://raw.githubusercontent.com/WebAssembly/WASI/a2b96e81c0586125cc4dc79a5be0b78d9a059925/legacy/preview1/docs.md).

It sends environment values as UTF-8. MicroHs `System.Environment.lookupEnv`
currently reads environment bytes as characters. Use ASCII environment values
with that MicroHs function.

Check the TypeScript file from the repository root:

```sh
tsc --strict --target ES2022 --module ES2022 --moduleResolution bundler \
  --lib ES2022,DOM --noEmit \
  rust/microhs-runtime/tools/wasm/wasi/browser-wasi.ts
```

On 2026-09-14, Chromium 151 passed these smoke checks with the adapter:

- The complete compiler ran on the standalone compact WAT runtime and produced
  the same 696922-byte fixed-point output as Wasmtime.
- Standalone WASM and JavaScript imports preserved scalar and IO results.
  Foreign functions, 64-bit values, floats, and binary bytes survived serialization.
- Main-thread and isolated-worker polling passed multiple timers, file events,
  descriptor errors, and event-count checks. The worker used `Atomics.wait`.
- C/WASI and the handwritten WASM reducer ran `DirectWasm.hs` with identical
  output. Direct imports and merged modules both passed.
- All eight scalar and IO imports passed. The i64 result `9007199254741003`
  remained exact.
- A MicroHs program read a Unicode file path and returned an output file.
- A WASI C command received Unicode arguments, environment values, and stdin.
  It returned exit code 7, stderr, split UTF-8 stdout, and binary file bytes.

The existing `wasi-selfhost.mjs` runs the Rust evaluator with the same shim in
Node. It is a separate benchmark harness.
