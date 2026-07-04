#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
repo_root=$(cd -- "$script_dir/../.." && pwd)
manifest="$repo_root/rust/microhs-runtime/Cargo.toml"

input=${MHS_SELFHOST_COMB:-/tmp/mhs-selfhost.comb}
profile_dir=${MHS_PGO_DIR:-/tmp/mhs-rust-pgo-selfhost}
raw_dir="$profile_dir/raw"
merged_profile="$profile_dir/merged.profdata"
gc_interval=${MHS_GC_NODE_INTERVAL:-33554432}
train_timeout=${MHS_PGO_TRAIN_TIMEOUT:-900s}

host=$(rustc -vV | awk '/^host:/ { print $2 }')
sysroot=$(rustc --print sysroot)
llvm_profdata=${LLVM_PROFDATA:-"$sysroot/lib/rustlib/$host/bin/llvm-profdata"}

if [[ ! -x "$llvm_profdata" ]]; then
  echo "llvm-profdata not found: $llvm_profdata" >&2
  exit 1
fi

if [[ ! -f "$input" ]]; then
  echo "self-host input not found: $input" >&2
  exit 1
fi

mkdir -p "$raw_dir"
find "$raw_dir" -type f -name '*.profraw' -delete

env CARGO_TARGET_DIR="$profile_dir/gen" \
  RUSTFLAGS="-Cprofile-generate=$raw_dir" \
  cargo build --release --manifest-path "$manifest" --bin mhs-rust-bench

env LLVM_PROFILE_FILE="$raw_dir/mhs-%p-%m.profraw" \
  MHS_GC_NODE_INTERVAL="$gc_interval" \
  timeout "$train_timeout" \
  "$profile_dir/gen/release/mhs-rust-bench" \
  --input "$input" --mode main --warmup-iters 0 --iters 1 -- \
  "$repo_root/bin/mhs" -i -imhs -isrc -ilib MicroHs.Main -o"$profile_dir/train.comb"

"$llvm_profdata" merge -o "$merged_profile" "$raw_dir"/*.profraw

env CARGO_TARGET_DIR="$profile_dir/use" \
  RUSTFLAGS="-Cprofile-use=$merged_profile" \
  cargo build --release --manifest-path "$manifest" --bin mhs-rust-bench

echo "$profile_dir/use/release/mhs-rust-bench"
