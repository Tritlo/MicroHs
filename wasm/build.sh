#!/usr/bin/env bash
set -euo pipefail

repo=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
evaluator=wat
output="$repo/target/mhs-wasm/mhs.wasm"
program=
optimize=false
profile=false
modules=()
features=(--enable-exception-handling --enable-bulk-memory --enable-mutable-globals
  --enable-sign-ext --enable-nontrapping-float-to-int --enable-reference-types
  --enable-multivalue --enable-multimemory)
usage() {
  echo "usage: $0 [--evaluator c|wat] [--output FILE.wasm] [--program FILE.c] [--optimize] [--profile] [MODULE=FILE.wasm|FILE.wat ...]" >&2
  echo 'Set WASI_SDK to a WASI SDK with setjmp support. Set BINARYEN_BIN if its tools are not on PATH.' >&2
}
while (($#)); do
  case $1 in
    --evaluator|--output|--program)
      (($# >= 2)) || { usage; exit 2; }
      case $1 in
        --evaluator) evaluator=$2;;
        --output) output=$2;;
        --program) program=$2;;
      esac
      shift 2;;
    --optimize) optimize=true; shift;;
    --profile) profile=true; shift;;
    --help|-h) usage; exit 0;;
    *=*) modules+=("$1"); shift;;
    *) usage; exit 2;;
  esac
done
[[ $evaluator == c || $evaluator == wat ]] || { usage; exit 2; }
if $profile && [[ $evaluator != wat || -n $program ]]; then
  echo '--profile requires the WAT benchmark evaluator.' >&2
  exit 2
fi
if [[ -z ${WASI_SDK:-} || ! -x $WASI_SDK/bin/clang ]]; then
  echo 'Set WASI_SDK to the WASI SDK directory.' >&2
  exit 2
fi
if [[ -n ${BINARYEN_BIN:-} ]]; then
  export PATH="$BINARYEN_BIN:$PATH"
fi
mkdir -p -- "$(dirname -- "$output")"
work=$(mktemp -d "$(dirname -- "$output")/.mhs-wasm-build.XXXXXX")
trap 'rm -rf -- "$work"' EXIT
sources=("$repo/src/runtime/mhsbench.c")
if [[ -n $program ]]; then
  sources=("$program" "$repo/src/runtime/eval.c" "$repo/src/runtime/main.c")
fi
flags=(-O3 -Wall -Wno-address-of-packed-member
  -I"$repo/src/runtime/wasi" -I"$repo/src/runtime"
  -mllvm -wasm-enable-sjlj -mllvm -wasm-use-legacy-eh=false
  -Wl,-z,stack-size=5242880)
if [[ $evaluator == wat ]]; then
  flags+=(-DMHS_WAT_REDUCER)
  if $profile; then flags+=(-DMHS_WAT_PROFILE); fi
  for symbol in stack stack_ptr stack_size glob_slice cells free_map next_scan_index num_free num_alloc \
    combB combC combK combK2 combK3 intTable \
    red_bb red_z red_r red_k2 red_k3 red_k4 red_ccb; do
    flags+=("-Wl,--export=$symbol")
  done
fi
"$WASI_SDK/bin/clang" "${flags[@]}" "${sources[@]}" -lm -lsetjmp -o "$work/support.wasm"
merge=("$work/support.wasm" support)
if [[ $evaluator == wat ]]; then
  wasm-as "$repo/wasm/reducer.wat" -o "$work/reducer-raw.wasm"
  wasm-opt -O3 --inline-functions-with-loops --always-inline-max-function-size=256 \
    "$work/reducer-raw.wasm" -o "$work/reducer.wasm"
  merge+=("$work/reducer.wasm" reducer)
fi
for module in "${modules[@]}"; do
  name=${module%%=*}
  file=${module#*=}
  [[ -n $name && -f $file ]] || { echo "invalid module: $module" >&2; exit 2; }
  merge+=("$file" "$name")
done
if ((${#merge[@]} > 2)); then
  wasm-merge "${features[@]}" "${merge[@]}" -o "$work/linked.wasm"
else
  cp -- "$work/support.wasm" "$work/linked.wasm"
fi
if $optimize; then
  wasm-opt "${features[@]}" -O3 "$work/linked.wasm" -o "$output"
else
  cp -- "$work/linked.wasm" "$output"
fi
echo "$output"
