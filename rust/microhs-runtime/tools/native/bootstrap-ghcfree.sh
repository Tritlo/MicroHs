#!/usr/bin/env bash
# Bootstrap the MicroHs compiler from the committed combinator seed using only
# the native Rust runtime.
#
# This needs neither GHC nor a C compiler. In contrast, `make newmhs` needs GHC,
# and `make bin/mhs` needs cc plus a fresh generated/mhs.c for this branch.
#
# When the compiler source changes, regenerate the seed by rebuilding bin/mhs via
# `make newmhs`, self-compiling with:
#
#   ./bin/mhs -i -imhs -isrc -ilib MicroHs.Main -o /tmp/mhs-selfhost.comb
#
# Then verify the new fixed-point hash and copy it over generated/mhs.comb.

set -euo pipefail

readonly EXPECTED_SHA="94dcf3ba6d8d5f39e98daced1f799c6d4bbbbaffe039d1c272c71752b0e01e88"

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/../../../.." && pwd)"
tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

cd "$repo_root"

seed="generated/mhs.comb"
fresh="$tmpdir/compiler.comb"
log="$tmpdir/bootstrap.log"
runtime="target/release/mhs-rust-bench"

seed_sha="$(sha256sum "$seed" | awk '{print $1}')"
if [[ "$seed_sha" != "$EXPECTED_SHA" ]]; then
  echo "FAIL seed sha mismatch: $seed_sha" >&2
  exit 1
fi

cargo build --release --manifest-path rust/microhs-runtime/Cargo.toml --bin mhs-rust-bench

if ! MHS_GC_NODE_INTERVAL=78643200 "$runtime" \
  --input "$seed" \
  --mode main \
  --warmup-iters 0 \
  --iters 1 \
  -- \
  mhs \
  -i \
  -imhs \
  -isrc \
  -ilib \
  MicroHs.Main \
  -o"$fresh" >"$log" 2>&1; then
  cat "$log" >&2
  exit 1
fi

fresh_sha="$(sha256sum "$fresh" | awk '{print $1}')"
if [[ "$fresh_sha" != "$EXPECTED_SHA" ]]; then
  echo "FAIL fresh sha mismatch: $fresh_sha" >&2
  exit 1
fi

if ! cmp -s "$seed" "$fresh"; then
  echo "FAIL fresh compiler comb differs from $seed" >&2
  exit 1
fi

echo "PASS $fresh_sha"
