#!/usr/bin/env bash
# Assemble the standalone WAT runtime. No C or Rust compiler is used.
set -euo pipefail
here=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
repo=$(cd -- "$here/../.." && pwd)
output="$repo/target/mhs-wasm/mhs-standalone.wasm"
debug=false
ffi=
modules=()
while (($#)); do
  case $1 in
    --output|--ffi)
      (($# >= 2)) || { echo "missing $1 value" >&2; exit 2; }
      if [[ $1 == --output ]]; then output=$2; else ffi=$2; fi
      shift 2;;
    --debug) debug=true; shift;;
    --help|-h)
      echo "usage: $0 [--output FILE.wasm] [--debug] [--ffi PREFIX] [MODULE=FILE.wat|FILE.wasm ...]"
      echo 'Set BINARYEN_BIN if wasm-as, wasm-opt, and wasm-merge are not on PATH.'
      exit 0;;
    *=*) modules+=("$1"); shift;;
    *) echo "unknown option: $1" >&2; exit 2;;
  esac
done
if [[ -n ${BINARYEN_BIN:-} ]]; then export PATH="$BINARYEN_BIN:$PATH"; fi
features=(--enable-bulk-memory --enable-mutable-globals --enable-sign-ext
  --enable-nontrapping-float-to-int --enable-multivalue --enable-multimemory)
mkdir -p -- "$(dirname -- "$output")"
work=$(mktemp -d "$(dirname -- "$output")/.standalone-build.XXXXXX")
trap 'rm -rf -- "$work"' EXIT
fragments=(names foreign heap values float-parse loader numeric plan rnf threads evaluate
  primitives-bytes primitives-arrays primitives-state primitives-control
  services-memory services-math services-digest wasi services-io serialize command)
for fragment in "${fragments[@]}"; do
  [[ -f "$here/$fragment.wat" ]] || { echo "missing fragment: $fragment.wat" >&2; exit 1; }
done
foreign="$here/foreign-none.wat"
if [[ -n $ffi ]]; then
  [[ -f $ffi.wat && -f $ffi.imports.wat ]] || { echo "missing FFI fragments: $ffi" >&2; exit 1; }
  foreign="$ffi.wat"
fi
{
  printf '(module\n'
  cat "$here/wasi-imports.wat"
  if [[ -n $ffi ]]; then cat "$ffi.imports.wat"; fi
  # WAT imports must occur before memory, globals, and function definitions.
  sed -n '/^(import "reducer" /p' "$here/evaluate.wat"
  printf '(memory (export "memory") 10240 32768)\n'
  for fragment in "${fragments[@]}"; do
    sed '/^(import "reducer" /d' "$here/$fragment.wat"
    printf '\n'
  done
  cat "$foreign"
  printf ')\n'
} > "$work/support.wat"
wasm-as "${features[@]}" "$work/support.wat" -o "$work/support-raw.wasm"
# Select the compact stride before optimization. No runtime branch is added.
[[ $(sed -n '/^  (global $node_bytes i32 (i32.const 12))$/p' "$here/../reducer.wat" | wc -l) == 1 ]] || {
  echo 'missing reducer node-stride declaration' >&2; exit 1;
}
sed 's/(global $node_bytes i32 (i32.const 12))/(global $node_bytes i32 (i32.const 8))/' \
  "$here/../reducer.wat" > "$work/reducer.wat"
wasm-as "$work/reducer.wat" -o "$work/reducer-raw.wasm"
wasm-opt -O3 --inline-functions-with-loops --always-inline-max-function-size=256 \
  "$work/reducer-raw.wasm" -o "$work/reducer.wasm"
if $debug; then
  cp -- "$work/support-raw.wasm" "$work/support.wasm"
else
  wasm-opt "${features[@]}" -O3 "$work/support-raw.wasm" -o "$work/support.wasm"
fi
merge=("$work/support.wasm" support "$work/reducer.wasm" reducer)
for module in "${modules[@]}"; do
  name=${module%%=*}
  file=${module#*=}
  [[ -n $name && $name != support && $name != reducer && -f $file ]] || {
    echo "invalid module: $module" >&2; exit 2;
  }
  merge+=("$file" "$name")
done
wasm-merge "${features[@]}" "${merge[@]}" -o "$output"
echo "$output"
