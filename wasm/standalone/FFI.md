# Direct JavaScript and WebAssembly calls

The `.wat` compiler output connects scalar Haskell foreign imports to typed
WebAssembly imports. The standalone build assembles these functions with the
WAT runtime. This path does not generate or compile C wrappers.

See [the compiler bootstrap](BOOTSTRAP.md) to build a compiler fixed point with
the standalone runtime and use that compiler to generate these files.

## Build and run

Build the compiler with GHC, then compile the example with the pure Haskell
Integer library. Put `lib/no-gmp` before `lib` in the source search path.

```sh
make bin/gmhs
mkdir -p target/mhs-wasm
bin/gmhs -i -imhs -ilib/no-gmp -ilib -iwasm/examples DirectWasm \
  -otarget/mhs-wasm/direct.wat
wasm/standalone/build.sh --ffi target/mhs-wasm/direct \
  --output target/mhs-wasm/direct.wasm mhs_example=wasm/examples/direct.wat
wasmtime run --dir target/mhs-wasm::. target/mhs-wasm/direct.wasm \
  direct.comb -- direct
```

`-oPREFIX.wat` writes four files:

| File | Contents |
| --- | --- |
| `PREFIX.wat` | Scalar argument access, result boxing, and foreign-name dispatch. This is a WAT module fragment. |
| `PREFIX.imports.wat` | Typed imports. Each import occupies one line. |
| `PREFIX.comb` | The linked, uncompressed v8.4 program. Raw byte strings retain their bytes. |
| `PREFIX.imports.json` | JavaScript bodies and scalar signatures. The binding list is empty for a WASM-only program. |

Pass the prefix without an extension to `--ffi`. Use the matching `.comb` file
when running the command. The runtime command syntax is
`RUNTIME.wasm INPUT.comb -- PROGRAM [ARG ...]`. `PROGRAM` becomes the Haskell
program name. The remaining arguments become its command-line arguments.

`MODULE=FILE.wat` or `MODULE=FILE.wasm` merges an implementation module into the
output. Omit that argument to retain imports for a host to supply. In a browser,
pass another instance's `exports` directly as the named import object. A direct
WASM-to-WASM call does not pass through a JavaScript wrapper.

## Scalar contract

```haskell
foreign import wasm "math add" add :: Int -> Int -> Int
foreign import javascript "return $0 + $1" addJS :: Int -> Int -> Int
foreign import wasm "state set" set :: Int -> IO ()
```

WASM declarations name one import module and one export. Names use ASCII
letters, digits, `_`, `.`, `-`, `/`, or `:`. Pure calls and `IO` calls support
the following types after newtype erasure:

| Haskell type | WebAssembly type | JavaScript value |
| --- | --- | --- |
| `Int` | `i32` | Signed `number` |
| `Word` | `i32` | Unsigned `number` |
| `Int64` | `i64` | Signed `bigint` |
| `Word64` | `i64` | Unsigned `bigint` |
| `Float` | `f32` | `number`, rounded at the WASM boundary |
| `Double` | `f64` | `number` |
| `IO ()` result | No result | The result is ignored |

The JavaScript column describes generated JavaScript bindings. A host that
supplies ordinary JavaScript functions for raw WASM imports must convert
unsigned arguments itself. The JavaScript WASM boundary supplies signed `i32`
and `i64` values.

Calls support at most six scalar arguments. `World` is internal and does not
appear in the imported signature. Pointers, strings, heap objects, callbacks,
`Bool`, pure `()`, and foreign exports are not supported by this output target.
The compiler rejects unsupported declarations that remain in the linked
program. Unused declarations do not require host imports.

A pure foreign function must have no observable effects. JavaScript bodies use
the existing `$0`, `$1`, and subsequent parameter names. A scalar body returns
its result with `return`. An `IO ()` body can consist of statements. Bodies run
synchronously. They cannot suspend the Haskell evaluator or return a Promise.
An exception thrown by JavaScript rejects `runWasi` at the host boundary.
Use `bigint` arithmetic for `Int64` and `Word64`. Conversion through `number`
can lose integer bits.

Historical library service names such as `getb`, `malloc`, and Storable accessors
select fixed operations in the WAT runtime. They do not resolve C symbols.
Arbitrary C imports are rejected on this output target. Existing C output
continues to support its existing native and Emscripten paths. Use `lib/no-gmp`
to avoid bignum service dependencies in standalone programs. Unsupported fixed
runtime operations report an explicit runtime error.

## Browser JavaScript bindings

Compile `DirectJavascript.hs` in the same way and pass its JSON metadata to the
browser adapter:

```typescript
import { runWasi } from "../../rust/microhs-runtime/tools/wasm/wasi/browser-wasi";

const module = await WebAssembly.compileStreaming(fetch("/javascript.wasm"));
const program = new Uint8Array(await (await fetch("/javascript.comb")).arrayBuffer());
const javascript = await (await fetch("/javascript.imports.json")).json();
const result = await runWasi(module, {
  args: ["runtime", "program.comb", "--", "javascript"],
  files: { "program.comb": program },
  javascript,
});
console.log(result.stdout);
```

`javascript-imports.ts` also exports `createJavascriptImports` for other hosts.
It returns the `javascript` import namespace. It preserves unsigned arguments,
uses `bigint` for 64-bit integers, and checks scalar result types. It compiles
the supplied bodies with the JavaScript `Function` constructor. These bodies
have the same host access as application JavaScript. Load metadata from trusted
programs. A Content Security Policy must permit this constructor, or the host
must provide the generated import names with its own precompiled functions.

## Runtime interface

The generated fragment defines four functions. They receive relative import
indices. The runtime adds 65536 to distinguish them from fixed services.

| Function | Contract |
| --- | --- |
| `$foreign_lookup(name, length)` | Return the relative index, or `-1` for an unknown name. |
| `$foreign_arity(index)` | Return the number of scalar arguments. |
| `$foreign_name(index)` | Return a NUL-terminated name for serialization. |
| `$foreign_invoke(index, args)` | Read evaluated boxed arguments and return one boxed result. |

The evaluator controls argument evaluation, I/O sequencing, and collection.
Generated calls do not invoke the evaluator or collector. Name bytes occupy
`0x01800000` through `0x01ffffff`, before the graph heap. The compiler checks the
limit of 8 MiB. WASM and JavaScript internal names include the source module,
so declarations in different modules remain distinct.
