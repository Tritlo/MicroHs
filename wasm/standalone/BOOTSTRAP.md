# Bootstrap the compiler with the WAT runtime

Run these commands from the repository root. They use an existing `bin/gmhs`,
Binaryen, and Wasmtime. Use `make bin/gmhs` to build the seed compiler with GHC
if needed. No C-to-WASM or Rust-to-WASM compiler is used in this sequence.

First assemble the runtime and create a compiler seed. The explicit source
search path selects the pure Haskell Integer implementation in `lib/no-gmp`.

```sh
mkdir -p target/mhs-wasm/bootstrap
wasm/standalone/build.sh --output target/mhs-wasm/mhs-standalone.wasm
bin/gmhs -i -imhs -isrc -ilib/no-gmp -ilib MicroHs.Main \
  -otarget/mhs-wasm/bootstrap/stage1.comb
```

Keep the source files unchanged while running the next two generations. Each
generation executes the preceding compiler on the same source and search path.
The directory preopen gives the command access to the source and output files.

```sh
for stage in 2 3; do
  previous=$((stage - 1))
  wasmtime run --dir .::. target/mhs-wasm/mhs-standalone.wasm \
    "target/mhs-wasm/bootstrap/stage$previous.comb" \
    -- ./bin/mhs -i -imhs -isrc -ilib/no-gmp -ilib MicroHs.Main \
    "-otarget/mhs-wasm/bootstrap/stage$stage.comb"
done
cmp target/mhs-wasm/bootstrap/stage2.comb target/mhs-wasm/bootstrap/stage3.comb
```

A successful `cmp` establishes the target fixed point. The first generation
comes from the native compiler. Its machine-word constants can differ from the
WASM32 result, so compare stage 2 with stage 3. Preserve the source snapshot,
search path, and seed when comparing evaluator performance.

Use the resulting compiler to generate a standalone foreign-call program:

```sh
wasmtime run --dir .::. target/mhs-wasm/mhs-standalone.wasm \
  target/mhs-wasm/bootstrap/stage3.comb \
  -- ./bin/mhs -i -imhs -ilib/no-gmp -ilib -iwasm/examples DirectWasm \
  -otarget/mhs-wasm/direct.wat
wasm/standalone/build.sh --ffi target/mhs-wasm/direct \
  --output target/mhs-wasm/direct.wasm mhs_example=wasm/examples/direct.wat
wasmtime run --dir target/mhs-wasm::. target/mhs-wasm/direct.wasm \
  direct.comb -- direct
```

The compiler writes `.wat`, `.imports.wat`, `.imports.json`, and `.comb` files.
The build assembles WAT and links the scalar implementation module. See
[direct foreign calls](FFI.md) for the type contract and JavaScript bindings.
