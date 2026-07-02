# The dynamic JavaScript FFI

MicroHs has a dynamic `foreign import javascript` for programs run by the
emscripten (WASM) runtime.  The JavaScript body is serialized into the
combinator file and turned into a JS function when the program is loaded, so
the same `.comb` that runs natively (minus the JS imports) runs in the browser
or under Node with full JS access, with no per-program C compilation.  This is
what lets the MicroHs compiler itself run in a web page (`-temscripten_compile`)
and compile programs that immediately script the page.

The feature is opt-in: the runtime code is compiled only with
`-DWANT_JSFFI=1` (which requires emscripten).  A default build contains none
of it, and in particular the JS FFI adds no `new Function`, so no new CSP
`unsafe-eval` requirement (the pre-existing `js_eval_*` helpers still `eval`
if a program calls them).  A runtime without the feature reports a clean
error if given a `.comb` that uses JS imports.

Building with the feature:

```sh
make jsffi                        # generated/mhseval-jsffi.js (Node, NODERAWFS)
mhs -temscripten_jsffi Prog -op.js  # compile a program against the flavored runtime
```

For the browser, build the runtime as a module, without `-sEXIT_RUNTIME` so
the instance stays alive and `"wrapper"` callbacks keep working after `main`
returns:

```sh
emcc -O3 -DWANT_JSFFI=1 -sEXPORTED_FUNCTIONS=_main \
     -sEXPORTED_RUNTIME_METHODS=FS,callMain,stringToNewUTF8,UTF8ToString \
     -sFORCE_FILESYSTEM=1 -sALLOW_MEMORY_GROWTH -sTOTAL_STACK=5MB -sSINGLE_FILE \
     -sMODULARIZE=1 -sEXPORT_NAME=MhsEval -Wno-address-of-packed-member \
     -Isrc/runtime -Isrc/runtime/unix \
     src/runtime/main.c src/runtime/eval.c src/runtime/comb.c -lm -o mhseval-web.js
```

and on the page:

```js
const m = await MhsEval({ noInitialRun: true });
m.FS.writeFile('/prog.comb', bytes);              // the .comb as raw bytes
m.callMain(['+RTS', '-r/prog.comb', '-RTS']);
```

## Imports

```haskell
foreign import javascript "return $0 + $1" add :: Int -> Int -> Int
foreign import javascript "console.log(UTF8ToString($0))" clog :: CString -> IO ()
```

The string is a JavaScript function body; the arguments are `$0`, `$1`, etc.
(Emscripten's numbering; GHC's wasm JSFFI counts from `$1`), and a result is
`return`ed.  The body runs in global scope (the runtime publishes
`UTF8ToString` and `stringToNewUTF8` on `globalThis` for string work).  Both
pure and `IO` types are allowed.

Marshallable types, and the tag each gets in the serialized token
(`~<tags> "<body>"`, return tag first):

| Haskell | tag | JS |
|---|---|---|
| `Int` | `I` | number (signed 32-bit on wasm32) |
| `Word`, `Char` | `U` | number (unsigned; `Char` as its codepoint) |
| `Double` | `D` | number |
| `Float` | `F` | number |
| `Ptr a` | `P` | number (unsigned address) |
| `Bool` | `B` | boolean (to JS; from JS any truthy value) |
| `JSVal` | `J` | any JS value |
| `ByteString` | `S` | string (UTF-8 transcoded) |
| `()` | `V` | undefined (return type only) |

`Int64`/`Word64` are not supported (they are boxed on wasm32; a `bigint`
bridge is future work).  `ByteString` is a text bridge: the bytes are
transcoded between UTF-8 and a JS string (embedded NULs survive; non-UTF-8
bytes do not).

## JSVal

`Mhs.JavaScript` exports the opaque type `JSVal`: a GC-managed reference to an
arbitrary JavaScript value.  A body can return one (`IO JSVal`) and take them
as arguments; the runtime interns the value in a registry and frees the slot
when the `JSVal` is garbage collected, so JS objects are released rather than
leaked.  There is no eager `freeJSVal` yet (that needs a run-a-finalizer-now
primitive; `finalizeForeignPtr` is not implemented either).

## Callbacks: `foreign import javascript "wrapper"`

```haskell
foreign import javascript "wrapper" mkCB :: (Int -> IO ()) -> IO JSVal
```

turns a Haskell closure into a real JS function value, e.g. to install as an
event listener.  The wrapper type must be `(a -> .. -> IO r) -> IO JSVal` with
marshallable argument and result types (the callback result must be `IO`).
Calls are synchronous; a call after `main` has returned runs on a fresh
evaluator thread (the instance must be built without `-sEXIT_RUNTIME` to stay
alive).  The closure's `StablePtr` is never freed: JS may retain the function
indefinitely, so its lifetime is the instance's.

The JS side re-enters the runtime through the `mhs_wrapper_invoke` export,
which is `EMSCRIPTEN_KEEPALIVE` whenever the feature is compiled in (custom
builds using `-sEXPORT_KEEPALIVE=0` would have to export it explicitly).

## Errors and re-entry

A JS exception thrown by an import is fatal: it is logged and the runtime
exits (this matches GHC's synchronous JSFFI path).  A pathological callback
argument (e.g. a throwing `valueOf`) degrades to `0`/`""` instead of
unwinding through the runtime.  The function registry is append-only for the
life of the instance, so `IO.serialize`/`deserialize` round-trip JS imports,
and each `"wrapper"` invocation keeps its arguments on a private frame, so
nested and re-entrant calls do not interfere.

## Compared to GHC's wasm JSFFI

Same surface for synchronous work: dynamic imports, `JSVal` with GC lifetime,
`"wrapper"` callbacks, and the `Bool`/`Char`/`Word` semantics match GHC's
documented behavior.  Not done: `await`ing a `Promise` (all calls are
synchronous), static named `foreign export javascript` for interpreted
programs (the runtime's export list is fixed at link time; `"wrapper"` covers
it, and compiled programs have the real thing), `Int64`/`Word64` as `bigint`,
and catchable JS exceptions.

## Deployment notes

- Registering an import uses `new Function`, so a page whose
  Content-Security-Policy forbids `unsafe-eval` cannot use the feature
  (a build without it registers nothing at load time and is unaffected).
- A `.comb` is a program: one that uses JS imports runs arbitrary JS in the
  page when loaded, just as any `.comb` can run arbitrary IO.  Do not load
  untrusted ones.
- Load a `.comb` into the WASM filesystem as binary (`arrayBuffer`, not
  `text()`; it contains raw bytes).
- `combVersion` was deliberately not bumped for the new tokens: the version
  check is exact-match, so a bump would invalidate every existing `.comb`
  (including the checked-in bootstrap) for an additive, optional token.
