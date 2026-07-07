# JavaScript FFI

This documents dynamic `foreign import javascript` and
`foreign export javascript` for the Rust MicroHs runtime.

The Rust runtime is a generic combinator interpreter.  It does not use the old
C/emscripten model of static `EMSCRIPTEN_KEEPALIVE` stubs baked into the
runtime.  The compiler serializes self-describing JS FFI tokens into `.comb`;
the Rust wasm runtime parses those tokens, calls the browser host, and exposes
exports back to JS.

Scope:

- Dynamic JS imports.
- Haskell closures wrapped as JS callbacks.
- JS-visible Haskell exports.
- `--no-main` export-only libraries.
- Separate embedder ABI used by browser/compiler hosts.
- Not `src/runtime/eval.c`.

## Tokens And Tags

The compiler writes three JS FFI encodings.

Import:

```text
~<tags> "<body>"
```

Wrapper:

```text
`<tags>
```

Export trailer:

```text
\n#####\n
JS_EXPORTS 1\n
"<js-name>" _<label> <arg-tags-or-> <ret-tag> <IO|PURE>\n
.\n
```

The first trailer bytes are exactly newline, five hashes, newline.  Version is
`JS_EXPORTS 1`.  Each export record names the external JS property, the
renumbered closure root, argument tags, return tag, and whether the exported
Haskell value returns `IO`.

The runtime tag string is always result first, then arguments.  For an export
record the parser forms:

```text
runtime-tags = <ret-tag><arg-tags>
```

Tag table:

| Tag | Haskell side | JS side |
| --- | --- | --- |
| `I` | `Int` | signed 32-bit number |
| `U` | `Word`, `Char` | unsigned 32-bit number |
| `D` | `Double` | number |
| `F` | `Float` | number, rounded through `f32` |
| `P` | `Ptr a` | pointer value; small pointers as numbers, other i64 values as synthetic handles |
| `B` | `Bool` | boolean |
| `J` | `JSVal` representation, `ForeignPtr Mhs.JavaScript.JSValRep` after newtype erasure | host object handle |
| `V` | `()` return only | `undefined` |
| `S` | `ByteString` | JS string bytes |

Compiler locations:

- `src/MicroHs/ExpPrint.hs`: `toStringP` emits `~` and wrapper backtick tokens
  for `LForImp _ (ImpJS ...)`; `toJsExportTrailer` emits `JS_EXPORTS`;
  `jsRetTag`, `jsArgTag`, `jsScalarTag`, and `jsWrapperTags` define the tag
  mapping.
- `src/MicroHs/FFI.hs`: `makeFFI` filters `ImpJS` out of static FFI import
  generation, and carries the `IsJavascript` bit for exports.
- `src/MicroHs/Exp.hs`: `mkForExp` and `getForExp` encode C exports as `FE`
  and JS exports as `FEJS`.
- `src/MicroHs/Desugar.hs`: `ForExp` with `Cjavascript` becomes `FEJS`.
- `src/MicroHs/Main.hs`: `.comb` and `.combffi` output append
  `toJsExportTrailer`; C stub generation is kept on the C/emcc path.

Unsupported JS FFI types fail in the compiler through
`src/MicroHs/ExpPrint.hs:jsTagErr`.

## Import Path

Comb input:

```text
~<tags> "<body>"
```

Main code path:

1. `rust/microhs-runtime/src/parse.rs` sees `~`, reads the tag token and quoted
   body, and creates `Node::JsCall(Box<JsCallNode>)`.
2. `rust/microhs-runtime/src/runtime/core_types.rs` defines `Node::JsCall`,
   `Node::JsWrap`, and `JsCallNode`.
3. The evaluator sees the cold node at the head of an application spine:
   `runtime/program/eval/stack.rs` and `runtime/program/eval/reduce.rs` turn it
   into `EvalHead::JsCall`.
4. The evaluator calls `Program::js_call` in
   `runtime/program/runtime_dispatch/js.rs`.
5. `js_call` validates tags with `validate_js_tags`, evaluates strict Haskell
   arguments by tag, and builds `JsArg` values.
6. `runtime/host/js_ffi/browser.rs` sends the body and args to JS:
   `host_js_prepare_call`, then one of `host_js_call_int`,
   `host_js_call_uint`, `host_js_call_double`, `host_js_call_ptr`,
   `host_js_call_object`, `host_js_call_bool`, `host_js_call_string`, or
   `host_js_call_void`.
7. `rust/microhs-runtime/tools/wasm/browser/host.mjs` implements
   `mhs_js_register`, `mhs_js_argreset`, `mhs_js_push_*`, and `mhs_js_call_*`.
   It creates a JS function with `Function.apply(null, ["$0", ..., body])`,
   pushes args into `state.argbuf`, calls the function, and converts the result.
8. `Program::js_call` builds the result node and returns it paired with the IO
   token argument.

Native/non-browser builds use `runtime/host/js_ffi/unsupported.rs`.  These
functions return `EvalError::UnsupportedJsFfi`, except `host_js_obj_free`, which
is a no-op.  That is why native builds fail cleanly on JS calls.

### Import Marshalling

`runtime/host/js_ffi.rs` defines the host-side marshalled value types:

- `JsArg`: Rust-to-JS import args.
- `JsValue`: JS-to-Haskell wrapper/export args and results on browser wasm.
- `validate_js_tags`: accepts `I U D F P B J S`, plus `V` only at index 0.

`runtime/program/runtime_dispatch/js.rs:js_call` maps tags:

- `I`: `eval_int` to `JsArg::Int`; result `Node::Int`.
- `U`: `eval_int` to `JsArg::UInt`; result `Node::Int`.
- `D`: `eval_float64`; result `Node::Float64`.
- `F`: `eval_float32`, sent as double; result `Node::Float32`.
- `P`: `eval_pointer_value`; result `Node::Ptr`.
- `B`: `eval_bool`; result primitive `A` or `K`.
- `J`: `eval_js_object_handle`; result from `js_object_node`.
- `S`: `eval_bytes`; result `Node::bytes`.
- `V`: result `Node::prim("I")`.

Strings:

- `S` args use `eval_bytes`, then `mhs_js_push_str`.
- `host.mjs:mhs_js_push_str` decodes the bytes as UTF-8.
- `S` results use `mhs_js_call_str`; Rust copies the returned host bytes in
  `copy_host_bytes` and frees the host allocation with `mhs_rust_dealloc`.
- `host.mjs` publishes `UTF8ToString`, `lengthBytesUTF8`, and
  `stringToNewUTF8` in `installStringHelpers`.
- The wasm helpers are `mhs_rust_active_read`,
  `mhs_rust_active_cstring_len`, and `mhs_rust_active_alloc` in `wasm.rs`.

Pointers:

- Rust uses i64 for `P`.
- `host.mjs:ptrToJs` returns a JS number for non-negative 32-bit values.
- Other i64 values get stable synthetic JS handles via `state.ptr` and
  `state.ptrReverse`.
- `host.mjs:ptrFromJs` reverses that mapping.

Bools:

- `B` args use `JsArg::Bool`.
- `host.mjs:mhs_js_push_bool` stores a JS boolean.
- `host.mjs:mhs_js_call_bool` returns `1` or `0`.
- Callback/export args use `mhs_js_arg_bool`.
- Callback/export results use `mhs_js_set_res_bool`.

### JSVal Lifecycle

`J` values are JS host objects represented in Haskell as the erased
`ForeignPtr Mhs.JavaScript.JSValRep`.

Lifecycle:

1. The browser host interns an object with `state.intern`, returning a u32
   handle.
2. Rust stores the handle as a `Node::ForeignPtr` using
   `Program::js_object_node` in `runtime/program/handles.rs`.
3. `js_object_node` attaches `ForeignFinalizer::JsObjFree`.
4. GC marks live foreign finalizer groups and stable pointers in
   `runtime/program/gc.rs`.
5. When the foreign pointer dies, `run_foreign_finalizer` sees
   `ForeignFinalizer::JsObjFree` and calls `host_js_obj_free`.
6. Browser `host.mjs:mhs_js_obj_free` deletes `state.obj[handle]` and reuses
   the slot through `state.objfree`.
7. Unsupported/native `host_js_obj_free` is a no-op.

The important ownership rule is: host JS objects are released by MicroHs GC
through the foreign pointer finalizer, not by explicit Haskell user code.

## Wrapper Callbacks

Comb input:

```text
`<tags>
```

This is produced by:

```haskell
foreign import javascript "wrapper" mkCallback
  :: (a1 -> ... -> IO r) -> IO JSVal
```

Compiler:

- `src/MicroHs/ExpPrint.hs:jsWrapperTags` checks that the import has exactly
  one callback argument and an `IO JSVal` result.
- The serialized tags describe the callback itself: callback result tag,
  followed by callback argument tags.

Runtime:

1. `parse.rs` sees backtick and creates `Node::JsWrap { tags }`.
2. `eval/stack.rs` or `eval/reduce.rs` sees `EvalHead::JsWrap`.
3. `runtime_dispatch/js.rs:Program::js_wrap` validates tags.
4. `js_wrap` requires two Haskell args: the closure and the IO token.
5. `js_wrap` registers tags with `register_js_wrapper_tags`.
6. `js_wrap` pins the Haskell closure with `new_stable_ptr_handle`.
7. `js_wrap` calls `host_js_make_wrapper(program_handle, stable_ptr,
   wrapper_index)`.
8. Browser `host.mjs:mhs_js_make_wrapper` interns a JS function.
9. When JS calls that function, it pushes a frame into `state.wargs` and calls
   `mhs_rust_wrapper_invoke`.
10. `wasm.rs:mhs_rust_wrapper_invoke` reads args with `read_wrapper_args`,
    calls `Program::apply_js_wrapper_index`, and writes the result with
    `set_wrapper_result`.
11. `Program::apply_js_wrapper_index` calls `apply_js_closure` with `is_io =
    true`, so it applies the Haskell closure, wraps it in `IO.performIO`,
    reduces to WHNF, and marshals the JS result.

`state.argbuf` is for imports.  `state.wargs` is a separate stack of callback
argument frames, so nested JS calls do not clobber import arguments.

The callback reduction limit is `usize::MAX` in `wasm.rs`.

If `host_js_make_wrapper` fails, `Program::js_wrap` frees the stable pointer it
just allocated.

## Export Path

Exports are not fake graph nodes.  They are a trailer after the graph.

Format:

```text
\n#####\n
JS_EXPORTS 1\n
"<js-name>" _<label> <arg-tags-or-> <ret-tag> <IO|PURE>\n
.\n
```

Example:

```text
#####
JS_EXPORTS 1
"addOne" _405 I I IO
"isPos" _407 I B PURE
"scale" _428 DD D PURE
.
```

`-` means no args.  Runtime tags are `I I` -> `II`, `I B` -> `BI`,
`DD D` -> `DDD` after moving return first.

Compiler:

- `src/MicroHs/ExpPrint.hs:toJsExportTrailer` filters the `ForExpTable` for
  JS exports, writes quoted JS name, renumbered closure label, argument tags,
  return tag, and `IO` or `PURE`.
- `src/MicroHs/Main.hs:mainCompile` writes this trailer after `outData` for
  `.comb` and `.combffi`.
- The old C header/stub generation is skipped for `.comb` and `.combffi`.

Parse and register:

1. `parse.rs:parse_program_file` (the slice entry of the unified
   `Parser<ByteSource>`) parses the normal graph through `}`.
2. It then calls `parse_js_export_trailer`.
3. If no `#####` or no `JS_EXPORTS` marker follows, the parser restores the old
   position and returns no exports.
4. For `JS_EXPORTS`, version must be `1`.
5. Each record resolves `_label` through the parse label table.
6. The parser creates `JsExportDecl { name, closure, tags, is_io }`.
7. `Program::new` is called.
8. `Program::register_js_exports` validates tags, pins each closure with
   `new_stable_ptr_handle`, registers tags with `register_js_wrapper_tags`,
   and stores `JsExport { name, stable_ptr, wrapper_index, is_io }`.

The stable pointer is a GC root because `runtime/program/gc.rs` marks
`Program::stable_ptrs`.

Wasm ABI:

```text
mhs_rust_js_export_count(handle) -> u32
mhs_rust_js_export_name(handle, export_index) -> *const u8
mhs_rust_js_export_invoke(handle, export_index) -> i32
```

All three are in `rust/microhs-runtime/src/wasm.rs`.

- `count` returns 0 on invalid/free handle.
- `name` writes the name into `RESULT_BYTES` and returns a pointer, or null on
  invalid/free handle.
- `invoke` reads JS args with the same `read_wrapper_args` helper used by
  callbacks, calls `Program::apply_js_export_index`, writes the result with
  `set_wrapper_result`, and returns 0 on success or 1 on failure.

Pure vs IO:

- `Program::apply_js_export_index` reads `is_io` from the export table.
- `apply_js_closure` applies marshalled JS args to the stable closure.
- If `is_io`, it applies `IO.performIO` before reducing.
- If `PURE`, it reduces the applied closure directly.
- Both cases reduce to WHNF and marshal with `js_value_from_node`.

Browser host:

- `host.mjs:newProgram` calls `mhs_rust_program_new`, then
  `makeJsExports(state, handle)`.
- `makeJsExports` enumerates count/name and creates a plain JS object.
- The returned runtime object exposes it through `get exports()`.
- `exportObject(handle)` rebuilds an export object for a specific handle.

Use:

```javascript
const m = await instantiateMicroHsRuntime(wasm, options);
const handle = m.newProgram(combBytes);
console.log(m.exports.addOne(41));
```

For programs with a `main`, call `m.reduceMain(handle, limit)` if desired.
For export-only programs, do not call `reduceMain`.

After `m.freeProgram(handle)`, the wasm program table slot is set to `None`.
The export object still exists in JS, but calling it returns a nonzero status
from `mhs_rust_js_export_invoke`; `host.mjs` throws
`MicroHs JavaScript export failed`.

## Embedder ABI

This is a separate surface from `foreign import javascript` /
`foreign export javascript`.  JS FFI is the language bridge inside compiled
MicroHs programs.  The embedder ABI is the host/control surface used by
browser workers and compiler wrappers around a wasm runtime instance.

### Feature Gate

All embedder-ABI additions are behind the `embedded` cargo feature:

```sh
cargo build --manifest-path rust/microhs-runtime/Cargo.toml --features embedded
```

The feature is off by default (`default = []`).  The non-embedded hot path is
kept untouched: the only embedder hook in `Program::reduce_main` is
`mhs_host_poll`, and it is fully `cfg`-elided when `embedded` is off.  The
branch uses `std::cfg_select!` for the two real `cfg` pairs:

- `runtime/program/public.rs`: `host_poll` is a browser-wasm import and a
  native no-op.
- `wasm.rs`: `mhs_rust_program_new` stores parse/setup error detail only for
  embedded builds.

The browser wasm build already enables the feature through
`tools/wasm/browser/build-browser-bench.sh`.

### Runtime ABI

Host-provided import:

```text
mhs_host_poll(steps_so_far: u64) -> i32
```

`Program::reduce_main` calls this every `EMBEDDED_POLL_INTERVAL` reductions
(`4 * 1024 * 1024`, about 4M).  A nonzero return is a cooperative cancel.  The
runtime reports main status `4` and writes `cancelled by host poll` to the
result buffer.  This is deliberately cheaper than making `reduce_main`
resumable: the host gets progress and a cancel point without killing a warm
worker.

Embedded-only exports:

```text
mhs_rust_program_reduce_main_status() -> i32
mhs_rust_program_stats(handle) -> *const u8
mhs_rust_last_error_ptr() -> *const u8
mhs_rust_last_error_len() -> usize
```

Main status codes:

| Code | Meaning |
| --- | --- |
| 0 | ok |
| 1 | runtime error |
| 2 | step limit |
| 3 | exception raised |
| 4 | cancelled |

`mhs_rust_program_stats(handle)` writes six little-endian `u64` values into
the shared result buffer and returns its pointer.  Read the byte length through
`mhs_rust_result_len()`; for this record it must be 48.  Field order:

```text
reductions
liveNodes
currentNodes
gcCollections
lastLiveNodes
highWaterNodes
```

`mhs_rust_last_error_ptr()` / `mhs_rust_last_error_len()` expose parse/setup
error detail.  `mhs_rust_program_new` sets this buffer on embedded parse
failure or handle allocation failure, and clears it after a successful program
creation.

### Compiler Dump Channel

`-ddump-combinator-out=FILE` is the deterministic named-artifact channel for
the embedder compiler flow.  `compileCacheTop` writes `showLDefs` to the named
file.  It works the same in native and wasm builds.

This is not `-o`.  `-o` writes the linked postfix wire format: names are erased
and it requires a successful `main` link.  Combinate's source modules are
deliberately main-less, so the browser compiler needs the named combinator
dump file instead.  The browser runtime also discards stdout as a dump channel,
so a file artifact is required.

### Browser Host Glue

`tools/wasm/browser/host.mjs` provides the import:

```javascript
mhs_host_poll(stepsSoFar)
```

It calls `options.onPoll(stepsSoFar)` when present.  The hook is non-throwing:
exceptions from `onPoll` are caught and treated as "do not cancel".

The same host wrapper exposes these runtime helpers:

```javascript
runtime.reduceMainStatus()
runtime.lastError()
runtime.stats(handle)
```

They are guarded for non-embedded builds.  If the wasm export is absent,
`reduceMainStatus()` returns `-1`, `lastError()` returns `""`, and
`stats(handle)` returns `null`.

### Stable Compiler API

`tools/wasm/browser/compiler.mjs` is the stable embedder compile boundary:

```javascript
const compiler = await createCompiler({ wasm, comb, files });
const out = compiler.compile(source, { module, flags });
compiler.close();
```

`createCompiler({ wasm, comb, files })` warms one runtime instance, preloads
caller-owned absolute VFS files, and reuses the instance across compiles.
`compile(source, { module, flags })` writes `/work/<Module>.hs`, runs `mhs`,
and returns:

```javascript
{ status, dump, stderr, error, stats }
```

`/work` is scratch.  The wrapper removes files from previous compiles before
each run and again on `close()`.

The argv order is intentional:

```text
mhs -i -i/work -imhs -isrc -ilib ...flags -ddump-combinator-out=/work/<Module>.dump <Module>
```

The first `-i` clears inherited include paths.  A later bare `-i` would clear
the paths added before it, so callers should not append one casually in
`flags`.

`compile()` reads and returns the dump even when `status != "ok"`.  That is the
expected Combinate case: a main-less module can write its combinator dump and
then fail on the missing `main`.  This replaces the older "ignore
`No definition found for: Ex.main`" host hack.

### Web Distribution

`tools/wasm/browser/build-web-dist.sh` builds the plain deployable tree used by
Combinate CI:

```text
microhs_runtime.wasm
compiler.mjs
host.mjs
mhs.comb
include/lib/...
manifest.json
prewarm.mhscache
```

The wasm is the embedded build.  `generated/mhs.comb` is copied as `mhs.comb`.
Only `lib/` is copied into `include/lib/`; that is sufficient for user
compiles because the compiler implementation sources are already baked into the
comb.  `manifest.json` maps dist include files to VFS paths and records the
prewarm cache location.

The `dist/` directory is gitignored.  `prewarm.mhscache` is generated at dist
time from the current compiler comb and library sources, then copied into the
dist tree.  It is version-tied output, not a committed source artifact.

### GHC-Free Bootstrap And Cache Checks

`generated/mhs.comb` is the committed bootstrap seed for the Rust runtime path.
`tools/native/bootstrap-ghcfree.sh` verifies that seed and self-compiles a
fresh compiler comb through `mhs-rust-bench` using only cargo and the Rust
runtime.  It needs neither GHC nor `cc`.

When compiler source changes, regenerate the seed by rebuilding `bin/mhs` with
`make newmhs`, then self-compiling:

```sh
./bin/mhs -i -imhs -isrc -ilib MicroHs.Main -o /tmp/mhs-selfhost.comb
```

Verify the fixed-point hash, then copy the result over `generated/mhs.comb`.

`tools/native/test-mhscache.sh` checks the `.mhscache` path.  It proves that a
cache written by the Rust runtime can be read back by the Rust runtime, and, if
`bin/mhs` is available, that a cache written by the C runtime is readable by the
Rust runtime.

### Prewarmed Cache Status

Browser workers load `prewarm.mhscache` and use `-CR` to avoid cold
compiler/cache work for common library modules.

This pays off.  Reading the cache via `-CR` was originally pathologically slow
(a 471KB cache took 14+ minutes natively, and browser runs timed out around
240s, slower than a ~90s cold compile).  The cause was not GC: `deserialize_bfile`
read the serialized graph one byte at a time and re-parsed the whole accumulated
buffer after every byte -- O(N^2).  `IO.deserialize` now parses the graph in a
single streaming pass, straight from the BFILE through the unified
`Parser<ByteSource>` (`parse_graph_only` over a `StreamSource`), so `-CR` on that
cache drops to ~0.5s -- far faster than a cold compile.

### Byte Identity

For this branch, the merged compiler with JS FFI and
`-ddump-combinator-out=FILE` self-hosts to fixed point:

```text
3489b7bf0c58ca0f004be73cea1a14fcafc99e2cd8bef81847f6832676b9ddfb
```

Runtime-ABI changes do not move that hash.  Compiler-source growth does.
Rust == C was verified for this fixed point.

## `--no-main` / Export-Only

Flag:

```text
--no-main
```

Compiler locations:

- `src/MicroHs/Flags.hs`: `Flags.noMain`.
- `src/MicroHs/Main.hs`: usage text, argument decoding, and `mainCompile`.

Default mode still roots the program at `<module>.main`:

```text
root = Var <module>.main
```

`--no-main` mode uses the existing no-link-style dummy root:

```text
root = #0
```

`src/MicroHs/ExpPrint.hs:renumberCMdl` already roots all foreign export
closures:

```text
roots = freeVars emain ++ exported-closure-ids
```

So with `--no-main`, the dummy entry is only the required graph entry.  The JS
export closures are the real library roots.  This avoids synthesizing a fake
`main`.

`src/MicroHs/Main.hs:mainCompile` errors if `--no-main` is set and there is no
`foreign export javascript`:

```text
--no-main requires at least one foreign export javascript
```

Runtime/host load path is unchanged.  `newProgram` parses the graph, registers
the trailer exports, and returns.  The host can call `m.exports.<name>` without
running a main.

## Cold Paths

JS FFI is not on the common reduction path.

Cold marking in current code:

- `runtime_dispatch/js.rs`: `Program::js_call`, `Program::js_wrap`,
  `Program::register_js_exports` have `#[cold]`.
- `eval/stack.rs` and `eval/reduce.rs`: branches for `EvalHead::JsCall` and
  `EvalHead::JsWrap` call `std::hint::cold_path()`.
- `parse.rs:parse_js_export_trailer` has `#[cold]` and `cold_path`.
- `program/public.rs`: `apply_js_wrapper_index`, `apply_js_export_index`,
  `js_export_name`, `js_export_tags`, and `js_export_count` are cold.
- `wasm.rs`: `mhs_rust_js_export_count`, `mhs_rust_js_export_name`, and
  `mhs_rust_js_export_invoke` are cold.
- `gc.rs`: `ForeignFinalizer::JsObjFree` branch calls `cold_path`.

Reason: most cells are ordinary combinator graph cells.  `JsCall`, `JsWrap`,
export metadata, and host JS object finalizers live in cold side structures.
The hot reducer should pay only a branch when the head is a rare cold node.

## Byte Identity

The important invariant is fixed-point byte identity for self-hosting:

1. Compile `MicroHs.Main` to a comb with the current compiler.
2. Run that comb through the Rust runtime in `--mode main`.
3. The output comb must byte-match the input comb.

The SHA is not a permanent protocol value.  It moves when compiler source grows.
Observed slice gates moved:

```text
29b8c5a5...  older base
79af9660...  after import/runtime polish
4cc4e3e1...  after JS export compiler/runtime
b037942e...  after --no-main compiler flag
```

That movement is expected when `src/MicroHs` changes and `bin/mhs` is rebuilt.
For ordinary programs with no JS imports/exports, serialization is additive:
no JS import token and no JS export trailer are emitted, so comb output stays
the ordinary format.

Current Rust self-host recipe:

```sh
cargo build --release --manifest-path rust/microhs-runtime/Cargo.toml --bin mhs-rust-bench

./bin/mhs -i -imhs -isrc -ilib MicroHs.Main -o/tmp/rjf-selfhost.comb
sha256sum /tmp/rjf-selfhost.comb

MHS_GC_NODE_INTERVAL=78643200 target/release/mhs-rust-bench \
  --input /tmp/rjf-selfhost.comb \
  --mode main \
  --warmup-iters 0 \
  --iters 1 \
  -- \
  ./bin/mhs -i -imhs -isrc -ilib MicroHs.Main -o/tmp/o.comb

sha256sum /tmp/o.comb
cmp /tmp/rjf-selfhost.comb /tmp/o.comb
```

For C comparisons, current `rust/README.md` says to use the in-process
`mhs-rust-bench --c-mhsbench ./bin/mhsbench` path, not the old
process-spawning `mhseval` path.  The old Makefile still has
`bootcombtest`, which uses `bin/mhseval` and `cmp`, but the Rust benchmark
harness is the main gate for this branch.

## Build And Run

Native Rust runtime:

```sh
cargo build --release --manifest-path rust/microhs-runtime/Cargo.toml
cargo fmt --check --manifest-path rust/microhs-runtime/Cargo.toml
```

Browser wasm runtime:

```sh
. ~/emsdk/emsdk_env.sh
bash rust/microhs-runtime/tools/wasm/browser/build-browser-bench.sh
```

Rebuild compiler after `src/MicroHs` changes:

```sh
make newmhs
git checkout -- generated/mhs.c
```

### Import Smoke

Temporary `Mhs.JavaScript` shim, until a public module exists:

```haskell
module Mhs.JavaScript(JSVal, JSValRep) where
import Foreign.ForeignPtr(ForeignPtr)
data JSValRep
newtype JSVal = JSVal (ForeignPtr JSValRep)
```

Program:

```haskell
module Main where
import Data.String(fromString)
import qualified Data.ByteString as B
import Mhs.JavaScript(JSVal)

foreign import javascript "return $0 + $1" jsAdd :: Int -> Int -> IO Int
foreign import javascript "return $0 + '-js'" jsString :: B.ByteString -> IO B.ByteString
foreign import javascript "return !$0" jsNot :: Bool -> IO Bool
foreign import javascript "wrapper" mkCallback :: (Int -> IO Int) -> IO JSVal
foreign import javascript "return $0($1) + 1" callCallback :: JSVal -> Int -> IO Int

cb :: Int -> IO Int
cb x = return (x + 10)

main :: IO ()
main = do
  jsAdd 20 22 >>= print
  s <- jsString (fromString "str")
  B.putStr (B.append s (fromString "\n"))
  jsNot False >>= print
  f <- mkCallback cb
  callCallback f 5 >>= print
```

Compile:

```sh
./bin/mhs -i -imhs -isrc -ilib -i/tmp/jsffi-slice2 \
  /tmp/jsffi-slice2/Main.hs -o/tmp/jsffi-slice2/smoke.comb
```

Observed tokens:

```text
~IJI "return $0($1) + 1"
`II
~BB "return !$0"
~SS "return $0 + '-js'"
~III "return $0 + $1"
```

Browser host driver shape:

```javascript
import { readFile } from "node:fs/promises";
import { instantiateMicroHsRuntime } from "/home/tritlo/Code/microhs-rust/rust/microhs-runtime/tools/wasm/browser/host.mjs";

const wasm = "/home/tritlo/Code/microhs-rust/target/wasm32-unknown-unknown/release/microhs_runtime.wasm";
const runtime = await instantiateMicroHsRuntime(wasm, {
  stdout(bytes) { process.stdout.write(Buffer.from(bytes)); },
  stderr(bytes) { process.stderr.write(Buffer.from(bytes)); },
});
const handle = runtime.newProgram(await readFile("/tmp/jsffi-slice2/smoke.comb"));
const status = runtime.reduceMain(handle, Number.MAX_SAFE_INTEGER);
console.log(`status=${status}`);
runtime.freeProgram(handle);
```

Expected output:

```text
status=0
stdout=42
str-js
True
16

stderr=
```

### Export With Main Smoke

Program:

```haskell
module Main where

foreign export javascript "addOne" addOne :: Int -> IO Int
foreign export javascript "isPos" isPos :: Int -> Bool
foreign export javascript "scale" scale :: Double -> Double -> Double

addOne :: Int -> IO Int
addOne x = do
  putStrLn "addOne called"
  return (x + 1)

isPos :: Int -> Bool
isPos x = x > 0

scale :: Double -> Double -> Double
scale x y = x * y

main :: IO ()
main = putStrLn "exports ready"
```

Compile:

```sh
./bin/mhs -i -imhs -isrc -ilib /tmp/jsffi-slice3/Main.hs -o/tmp/jsffi-slice3/exports.comb
```

Observed trailer:

```text
#####
JS_EXPORTS 1
"addOne" _405 I I IO
"isPos" _407 I B PURE
"scale" _428 DD D PURE
.
```

Driver:

```javascript
import { readFile } from "node:fs/promises";
import { instantiateMicroHsRuntime } from "/home/tritlo/Code/microhs-rust/rust/microhs-runtime/tools/wasm/browser/host.mjs";

const wasm = "/home/tritlo/Code/microhs-rust/target/wasm32-unknown-unknown/release/microhs_runtime.wasm";
const stdout = [];
const stderr = [];
const m = await instantiateMicroHsRuntime(wasm, {
  stdout(bytes) { stdout.push(new TextDecoder().decode(bytes)); },
  stderr(bytes) { stderr.push(new TextDecoder().decode(bytes)); },
});

const handle = m.newProgram(await readFile("/tmp/jsffi-slice3/exports.comb"));
const status = m.reduceMain(handle, Number.MAX_SAFE_INTEGER);
console.log(`mainStatus=${status}`);
console.log(`names=${Object.keys(m.exports).join(",")}`);
console.log(`addOne=${m.exports.addOne(41)}`);
console.log(`isPos=${m.exports.isPos(7)},${m.exports.isPos(-1)}`);
console.log(`scale=${m.exports.scale(2.5, 4)}`);
m.freeProgram(handle);
try {
  m.exports.addOne(1);
  console.log("afterFree=<unexpected success>");
} catch (error) {
  console.log(`afterFree=${error.message}`);
}
console.log(`stdout=${stdout.join("").replaceAll("\n", "\\n")}`);
console.log(`stderr=${stderr.join("").replaceAll("\n", "\\n")}`);
```

Expected output:

```text
mainStatus=0
names=addOne,isPos,scale
addOne=42
isPos=true,false
scale=10
afterFree=MicroHs JavaScript export failed
stdout=exports ready\naddOne called\n
stderr=
```

### Export-Only Smoke

Program:

```haskell
module ExportOnly where

foreign export javascript "twiceIO" twiceIO :: Int -> IO Int
foreign export javascript "isEven" isEven :: Int -> Bool

twiceIO :: Int -> IO Int
twiceIO x = do
  putStrLn "twiceIO called"
  return (x * 2)

isEven :: Int -> Bool
isEven x = x `mod` 2 == 0
```

Compile:

```sh
./bin/mhs --no-main -i -imhs -isrc -ilib \
  /tmp/jsffi-slice3b/ExportOnly.hs \
  -o/tmp/jsffi-slice3b/export-only.comb
```

Observed trailer:

```text
#####
JS_EXPORTS 1
"twiceIO" _404 I I IO
"isEven" _407 I B PURE
.
```

Driver:

```javascript
const handle = m.newProgram(await readFile("/tmp/jsffi-slice3b/export-only.comb"));
console.log(`exportOnly.names=${Object.keys(m.exports).join(",")}`);
console.log(`exportOnly.twiceIO=${m.exports.twiceIO(21)}`);
console.log(`exportOnly.isEven=${m.exports.isEven(8)},${m.exports.isEven(7)}`);
m.freeProgram(handle);
```

Do not call `reduceMain` for this program.

Expected output:

```text
exportOnly.names=twiceIO,isEven
exportOnly.twiceIO=42
exportOnly.isEven=true,false
exportOnly.stdout=twiceIO called\n
exportOnly.stderr=
```

Negative checks:

```sh
./bin/mhs -i -imhs -isrc -ilib /tmp/jsffi-slice3b/ExportOnly.hs \
  -o/tmp/jsffi-slice3b/export-only-default.comb
```

Expected:

```text
No definition found for: ExportOnly.main
```

```sh
./bin/mhs --no-main -i -imhs -isrc -ilib /tmp/jsffi-slice3b/NoExports.hs \
  -o/tmp/jsffi-slice3b/noexports.comb
```

Expected:

```text
--no-main requires at least one foreign export javascript
```

## File Map

Compiler:

- `src/MicroHs/ExpPrint.hs`: emits JS import/wrapper tokens, emits export
  trailer, defines tag mapping, links export roots in `renumberCMdl`.
- `src/MicroHs/FFI.hs`: filters `ImpJS` out of static imports; keeps C/emcc
  export stub support separate from `.comb` trailers.
- `src/MicroHs/Exp.hs`: `FE`/`FEJS` encoding for foreign exports.
- `src/MicroHs/Desugar.hs`: lowers `foreign export javascript` to `FEJS`.
- `src/MicroHs/Names.hs`: names needed by JS tag recognition.
- `src/MicroHs/Translate.hs`: rejects direct interpreter execution of `ImpJS`.
- `src/MicroHs/Flags.hs`: `noMain` flag.
- `src/MicroHs/Main.hs`: argument decoding, `--no-main` root choice,
  `.comb` trailer write, C-stub skip for `.comb`.

Runtime:

- `rust/microhs-runtime/src/parse.rs`: parses `~`, backtick, and
  `JS_EXPORTS` trailer.
- `rust/microhs-runtime/src/runtime/core_types.rs`: `Node::JsCall`,
  `Node::JsWrap`, `JsCallNode`, `JsExportDecl`, `JsExport`,
  `ForeignFinalizer::JsObjFree`, and `Program` JS fields.
- `rust/microhs-runtime/src/runtime/program/eval/stack.rs`: hot stack reducer
  detects `JsCall`/`JsWrap` heads and enters cold JS dispatch.
- `rust/microhs-runtime/src/runtime/program/eval/reduce.rs`: alternate reducer
  path with the same JS head dispatch.
- `rust/microhs-runtime/src/runtime/program/runtime_dispatch/js.rs`:
  `js_call`, `js_wrap`, `register_js_wrapper_tags`, `register_js_exports`,
  and JS value marshalling.
- `rust/microhs-runtime/src/runtime/program/public.rs`:
  `apply_js_wrapper_index`, `apply_js_export_index`, export metadata access,
  and wasm memory helpers.
- `rust/microhs-runtime/src/runtime/program/handles.rs`: stable pointer
  handles, JS object `ForeignPtr` construction, and JS finalizer allocation.
- `rust/microhs-runtime/src/runtime/program/gc.rs`: stable-root scan and
  `JsObjFree` finalizer execution.
- `rust/microhs-runtime/src/runtime/host/js_ffi.rs`: wasm-vs-unsupported
  module selection, tag validation, `JsArg`, `JsValue`.
- `rust/microhs-runtime/src/runtime/host/js_ffi/browser.rs`: browser host ABI
  calls for JS imports, wrappers, object release, and string copy/free.
- `rust/microhs-runtime/src/runtime/host/js_ffi/unsupported.rs`: native stubs
  that fail cleanly on JS FFI.
- `rust/microhs-runtime/src/wasm.rs`: program table, active-program helpers,
  export ABI, wrapper invoke ABI, arg/result marshalling.

Host:

- `rust/microhs-runtime/tools/wasm/browser/host.mjs`: wasm instantiation,
  host filesystem/stdout/stderr, JS import execution, wrapper callbacks,
  export object construction, pointer and string helpers.
- `rust/microhs-runtime/tools/wasm/browser/build-browser-bench.sh`: browser
  wasm build entry point.
- `rust/microhs-runtime/tools/wasm/browser/browser-selfhost.mjs`: browser wasm
  self-host helper.
- `rust/microhs-runtime/tools/wasm/browser/browser-compare.mjs`: browser wasm
  benchmark comparison helper.

## Limitations And Follow-Ups

- `JSVal` is shipped as the public library type `lib/Mhs/JavaScript.hs`
  (`import Mhs.JavaScript(JSVal)`): a `newtype JSVal = JSVal (ForeignPtr JSValRep)`
  whose `JSValRep` phantom is the name `jsScalarTag` in ExpPrint keys the `J` tag
  on.  Pure Haskell, no C dependency.  (Earlier smoke tests predating the module
  used a local shim of the same shape.)  The module currently exports only the
  opaque type; higher-level JS helper functions are not provided yet.

- There is no async JS FFI in this Rust path.  Existing `wasm-js-ffi-async`
  work is for the C/emscripten side and should not be assumed to apply here.

- The branch `microhs-rust-wasm-js-ffi` was a draft/reference source for some
  runtime polish.  Do not treat it as authoritative for Rust exports.

- Export errors currently report a generic `MicroHs JavaScript export failed`
  in `host.mjs` unless the wasm result buffer contains text.

- `--no-main` is export-only.  It does not introduce a general library compile
  mode for non-JS uses.

- The C runtime `src/runtime/eval.c` is intentionally not part of this design.
