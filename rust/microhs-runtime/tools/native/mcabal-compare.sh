#!/usr/bin/env bash
# C-vs-Rust comparison for the `mcabal build` workload: build the MicroHs base
# library with MicroCabal, driving the SAME mcabal.comb and mhs.comb on the C
# evaluator (bin/mhseval) and the Rust runtime (mhs-rust), with the sub-compiler
# `mhs` bound to the C evaluator or bin/mhsr respectively. This is a whole-build
# analogue of matrix.sh: its wall time is dominated by the sequential sub-mhs
# compiles of the base modules, so it measures aggregate compile throughput.
#
# Reports median-of-N wall + max-process RSS (time -v sees the largest single
# sub-compile, since MicroCabal builds sequentially), and gates on base.pkg
# CONTENT parity. The .pkg is LZMA-compressed and the C lzma-sdk and Rust
# lzma-sdk-rs produce different compressed bytes for identical content, so raw
# byte-compare is not a valid gate; instead compile a probe module through each
# package and compare the resulting combs.
#
# Requires: a sibling ../MicroCabal checkout (github.com/augustss/MicroCabal;
# override with MICROCABAL=/path), plus cc and the Rust toolchain. Builds
# bin/mhseval, bin/mhsr, mhs-rust, and mcabal.comb as needed.
set -euo pipefail

repo_root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../../../.." && pwd)
cd "$repo_root"

microcabal="${MICROCABAL:-$(cd "$repo_root/.." && pwd)/MicroCabal}"
[ -d "$microcabal/src" ] || {
  echo "MicroCabal not found at $microcabal (git clone https://github.com/augustss/MicroCabal; or set MICROCABAL=/path)" >&2
  exit 1
}
comb="$repo_root/generated/mhs.comb"
iters="${ITERS:-3}"
case "$iters" in ''|*[!0-9]*|0) echo "ITERS must be a positive integer" >&2; exit 1 ;; esac
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

to_s() { awk -F: '{ if (NF==2) printf "%.2f",$1*60+$2; else printf "%.2f",$1*3600+$2*60+$3 }'; }
med()  { printf '%s\n' "$@" | sort -n | awk '{a[NR]=$0} END{print a[int((NR+1)/2)]}'; }
rss()  { awk '/Maximum resident/{printf "%.0f",$NF/1024}' "$1"; }
wl()   { grep -oP 'Elapsed.*: \K[0-9:.]+' "$1" | to_s; }

echo "### building binaries ###" >&2
make -s bin/mhseval bin/mhsr >/dev/null
cargo build --release --manifest-path rust/microhs-runtime/Cargo.toml --bin mhs-rust 2>&1 | tail -1 >&2
mhsrust="$repo_root/target/release/mhs-rust"

# mcabal itself, compiled from the MicroCabal checkout with our own bin/mhsr.
mcabal="$work/mcabal.comb"
bin/mhsr -i"$microcabal/src" -ilib MicroCabal.Main -o"$mcabal"

# `mhs` shims on PATH: C = bin/mhseval running mhs.comb; Rust = bin/mhsr. Both
# binaries live in <root>/bin, so each resolves the inplace lib/ the same way.
cshim="$work/cshim"; rshim="$work/rshim"; mkdir -p "$cshim" "$rshim"
printf '#!/bin/sh\nexec "%s" +RTS -r"%s" -RTS "$@"\n' "$repo_root/bin/mhseval" "$comb" > "$cshim/mhs"
chmod +x "$cshim/mhs"
ln -sf "$repo_root/bin/mhsr" "$rshim/mhs"

pkgver="$(sed -n 's/^version:[[:space:]]*//p' lib/base.cabal | head -1)"
pkg="lib/dist-mcabal/base-$pkgver.pkg"

# One timed `mcabal build` in lib/, from a clean dist-mcabal (cleaned OUTSIDE the
# timed region). Saves the produced package to $4. Args after $4 are the runner
# argv; `build` is appended. Timing is read from $work/<label><iter>.t afterwards.
# MHS=mhs forces MicroCabal to resolve the sub-compiler via the PATH shim (it
# honours $MHS if set), and MHSDIR is cleared so mhs uses the inplace lib/. Called
# at top level, never in $(...) — where `set -e` would be disabled — so a failed
# build aborts; the explicit `|| return` also guards the cp.
runb() {
  local label="$1" iter="$2" shim="$3" out="$4"; shift 4
  rm -rf lib/dist-mcabal
  ( cd lib && env -u MHSDIR MHS=mhs PATH="$shim:$PATH" /usr/bin/time -v "$@" build ) >/dev/null 2>"$work/$label$iter.t" || return
  cp "$pkg" "$out" || return
  echo "$label$iter $(wl "$work/$label$iter.t")s $(rss "$work/$label$iter.t")MB" >&2
}

echo "### interleaved mcabal build, median-of-$iters ###" >&2
for i in $(seq 1 "$iters"); do
  runb C "$i" "$cshim" "$work/base-C.pkg" "$repo_root/bin/mhseval" +RTS -r"$mcabal" -RTS
  runb R "$i" "$rshim" "$work/base-R.pkg" "$mhsrust" --main "$mcabal" mcabal
done
declare -a cw rw
for i in $(seq 1 "$iters"); do cw+=("$(wl "$work/C$i.t")"); rw+=("$(wl "$work/R$i.t")"); done
CW="$(med "${cw[@]}")"; RW="$(med "${rw[@]}")"
CR="$(rss "$work/C1.t")"; RR="$(rss "$work/R1.t")"

# Content parity: compile a probe module through each produced package; equal
# output combs => the exercised slice of the two packages is equivalent, i.e. the
# raw .pkg bytes differ only in LZMA encoding, not in compiled content. Package
# modules take precedence over the inplace lib/, so the probe genuinely reads
# base-C.pkg / base-R.pkg; it is a representative smoke, not a full-package diff.
probe="$work/probe"; mkdir -p "$probe"
cat > "$probe/Probe.hs" <<'HS'
module Probe where
import Prelude
import Data.List
main :: IO ()
main = print (length (take 3 (map (+ 1) [1 .. 10 :: Int])))
HS
bin/mhsr -i"$probe" -p"$work/base-C.pkg" Probe -o"$work/viaC.comb" 2>/dev/null
bin/mhsr -i"$probe" -p"$work/base-R.pkg" Probe -o"$work/viaR.comb" 2>/dev/null
if cmp -s "$work/viaC.comb" "$work/viaR.comb"; then parity="EQUIVALENT"; else parity="DIVERGED"; fi

echo ""
echo "=== mcabal build: base library, C-eval vs Rust (median-of-$iters) ==="
printf '%-14s %9s   %s\n' "runtime" "wall" "max-proc RSS"
printf '%-14s %8ss   %sMB\n' "C (mhseval)" "$CW" "$CR"
printf '%-14s %8ss   %sMB\n' "Rust"        "$RW" "$RR"
awk -v a="$CW" -v b="$RW" 'BEGIN{printf "\nRust/C wall: %.4f (%+.1f%%)\n",b/a,(b-a)*100/a}'
echo "base.pkg content parity (compile-through-package): $parity"
[ "$parity" = EQUIVALENT ] || exit 1
