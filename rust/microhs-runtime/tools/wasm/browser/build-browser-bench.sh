#!/usr/bin/env bash
set -euo pipefail

repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"
emcc="${EMCC:-/home/tritlo/emsdk/upstream/emscripten/emcc}"
em_cache="${EM_CACHE:-/tmp/mhs-emcc-cache}"
out="$repo/rust/microhs-runtime/tools/wasm/browser/browser-bench-c.mjs"

if [[ ! -x "$emcc" ]]; then
  echo "emcc not found at $emcc; set EMCC=/path/to/emcc" >&2
  exit 1
fi

mkdir -p "$em_cache"

cargo build \
  --release \
  --manifest-path "$repo/rust/microhs-runtime/Cargo.toml" \
  --target wasm32-unknown-unknown \
  --lib

EM_CACHE="$em_cache" "$emcc" \
  -O3 \
  -sMODULARIZE=1 \
  -sEXPORT_ES6=1 \
  -sENVIRONMENT=web,node \
  -sEXPORTED_FUNCTIONS=_main \
  -sEXPORTED_RUNTIME_METHODS=FS,callMain \
  -sALLOW_MEMORY_GROWTH \
  -sTOTAL_STACK=5MB \
  -sSINGLE_FILE \
  -DUSE_SYSTEM_RAW \
  -Wno-address-of-packed-member \
  -I"$repo/src/runtime" \
  -I"$repo/src/runtime/unix" \
  "$repo/src/runtime/mhsbench.c" \
  -lm \
  -o "$out"

echo "built $out"
echo "built $repo/target/wasm32-unknown-unknown/release/microhs_runtime.wasm"
