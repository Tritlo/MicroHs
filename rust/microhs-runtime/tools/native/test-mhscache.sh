#!/usr/bin/env bash
# Check that .mhscache round-trips through the native Rust runtime, and that a
# cache written by the C runtime can be read by the Rust runtime.

set -euo pipefail

readonly EXPECTED_COMPILER_SHA="94dcf3ba6d8d5f39e98daced1f799c6d4bbbbaffe039d1c272c71752b0e01e88"
readonly GC_INTERVAL="78643200"

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/../../../.." && pwd)"
tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

runtime="$repo_root/target/release/mhs-rust-bench"
# Drive off the committed GHC-free seed comb, not a bin/mhs rebuild: `make bin/mhs`
# would compile the UPSTREAM generated/mhs.c (no dump flag), not our compiler.
compiler="$repo_root/generated/mhs.comb"
srcdir="$tmpdir/src"
workdir="$tmpdir/work"

sha_file() {
  sha256sum "$1" | awk '{print $1}'
}

file_size() {
  wc -c <"$1" | tr -d ' '
}

fail_case() {
  local case_name="$1"
  shift
  echo "$case_name: FAIL $*" >&2
  exit 1
}

run_rust_mhs() {
  local cwd="$1"
  local log="$2"
  shift 2

  (
    cd "$cwd"
    MHS_GC_NODE_INTERVAL="$GC_INTERVAL" "$runtime" \
      --input "$compiler" \
      --mode main \
      --warmup-iters 0 \
      --iters 1 \
      -- \
      mhs \
      "$@"
  ) >"$log" 2>&1
}

run_c_mhs() {
  local cwd="$1"
  local log="$2"
  shift 2

  (
    cd "$cwd"
    "$repo_root/bin/mhs" "$@"
  ) >"$log" 2>&1
}

print_log() {
  local log="$1"
  if [[ -s "$log" ]]; then
    sed 's/^/  /' "$log" >&2
  fi
}

first_diff() {
  local left="$1"
  local right="$2"
  local diff
  diff="$(cmp -l "$left" "$right" | head -n 1 || true)"
  if [[ -n "$diff" ]]; then
    awk '{print "byte="$1 " expected="$2 " actual="$3}' <<<"$diff"
  else
    echo "first_diff=unknown"
  fi
}

mkdir -p "$srcdir" "$workdir"

cat >"$srcdir/Main.hs" <<'EOF'
module Main where

import qualified Prelude()

main = \x -> x
EOF

cd "$repo_root"

if [[ ! -f "$compiler" ]]; then
  fail_case "COMPILER" "seed comb not found: $compiler (run bootstrap-ghcfree.sh / make newmhs)"
fi
compiler_sha="$(sha_file "$compiler")"
if [[ "$compiler_sha" != "$EXPECTED_COMPILER_SHA" ]]; then
  fail_case "COMPILER" "seed sha mismatch expected=$EXPECTED_COMPILER_SHA actual=$compiler_sha"
fi

cargo build --release --manifest-path rust/microhs-runtime/Cargo.toml --bin mhs-rust-bench

include_args=(-i"$srcdir")

rust_write_dir="$workdir/rust-write"
rust_no_cache_dir="$workdir/rust-no-cache"
c_write_dir="$workdir/c-write"
mkdir -p "$rust_write_dir" "$rust_no_cache_dir" "$c_write_dir"

rust_write_comb="$rust_write_dir/tiny-rust-write.comb"
rust_read_comb="$rust_write_dir/tiny-rust-read.comb"
no_cache_comb="$rust_no_cache_dir/tiny-no-cache.comb"
c_write_comb="$c_write_dir/tiny-c-write.comb"
c_read_by_rust_comb="$c_write_dir/tiny-c-cache-rust-read.comb"

if ! run_rust_mhs "$rust_write_dir" "$rust_write_dir/write.log" \
  -CW "${include_args[@]}" Main -o"$rust_write_comb"; then
  print_log "$rust_write_dir/write.log"
  fail_case "RUST-WRITE" "compile failed"
fi

rust_cache="$rust_write_dir/.mhscache"
if [[ ! -s "$rust_cache" ]]; then
  fail_case "RUST-WRITE" "missing or empty cache=$rust_cache"
fi
echo "RUST-WRITE: PASS cache_sha=$(sha_file "$rust_cache") cache_bytes=$(file_size "$rust_cache") comb_sha=$(sha_file "$rust_write_comb")"

if ! run_rust_mhs "$rust_no_cache_dir" "$rust_no_cache_dir/no-cache.log" \
  "${include_args[@]}" Main -o"$no_cache_comb"; then
  print_log "$rust_no_cache_dir/no-cache.log"
  fail_case "RUST-READ" "no-cache compile failed"
fi

if ! run_rust_mhs "$rust_write_dir" "$rust_write_dir/read.log" \
  -CR "${include_args[@]}" Main -o"$rust_read_comb"; then
  print_log "$rust_write_dir/read.log"
  fail_case "RUST-READ" "cache-read compile failed"
fi

if ! cmp -s "$no_cache_comb" "$rust_read_comb"; then
  fail_case "RUST-READ" "output diverged expected_sha=$(sha_file "$no_cache_comb") actual_sha=$(sha_file "$rust_read_comb") $(first_diff "$no_cache_comb" "$rust_read_comb")"
fi
echo "RUST-READ: PASS nocache_sha=$(sha_file "$no_cache_comb") read_sha=$(sha_file "$rust_read_comb")"

if [[ ! -x "$repo_root/bin/mhs" ]]; then
  echo "CROSS-RUNTIME: SKIP no bin/mhs (build with: make bin/mhs; must be the 47ea3c40 compiler)"
  exit 0
fi

if ! run_c_mhs "$c_write_dir" "$c_write_dir/write.log" \
  -CW "${include_args[@]}" Main -o"$c_write_comb"; then
  echo "CROSS-RUNTIME: FAIL c-write-failed" >&2
  print_log "$c_write_dir/write.log"
  exit 0
fi

c_cache="$c_write_dir/.mhscache"
if [[ ! -s "$c_cache" ]]; then
  echo "CROSS-RUNTIME: FAIL c-cache-missing-or-empty cache=$c_cache" >&2
  exit 0
fi

if ! run_rust_mhs "$c_write_dir" "$c_write_dir/read-by-rust.log" \
  -CR "${include_args[@]}" Main -o"$c_read_by_rust_comb"; then
  echo "CROSS-RUNTIME: FAIL rust-read-failed c_cache_sha=$(sha_file "$c_cache") c_cache_bytes=$(file_size "$c_cache")" >&2
  print_log "$c_write_dir/read-by-rust.log"
  exit 0
fi

if ! cmp -s "$no_cache_comb" "$c_read_by_rust_comb"; then
  echo "CROSS-RUNTIME: FAIL output-diverged c_cache_sha=$(sha_file "$c_cache") expected_sha=$(sha_file "$no_cache_comb") actual_sha=$(sha_file "$c_read_by_rust_comb") expected_bytes=$(file_size "$no_cache_comb") actual_bytes=$(file_size "$c_read_by_rust_comb") $(first_diff "$no_cache_comb" "$c_read_by_rust_comb")" >&2
  exit 0
fi

echo "CROSS-RUNTIME: PASS c_cache_sha=$(sha_file "$c_cache") c_cache_bytes=$(file_size "$c_cache") read_sha=$(sha_file "$c_read_by_rust_comb")"
