#!/bin/bash
# Equal-MEMORY self-host matrix: C vs Rust, non-PGO + PGO, at a common RSS budget.
#
# A Rust cell is 8 bytes (packed index pair / scalar); a C node is 16 bytes (two
# native words). So equal cell-count is NOT equal memory -- it hands C ~2x the RAM.
# This benchmark matches RSS instead: C at its default heap (~787MB) vs Rust at a
# GC interval tuned to the same footprint.
#
# Rerun at another budget by overriding the C heap flag and Rust interval, e.g.:
#   CHEAP="-H134217728" RINT=222298112 MLABEL="~2GB" bash matrix.sh
#
# Requires: bin/mhseval (make bin/mhseval), a self-host comb at /tmp/mhs-selfhost.comb,
# a C compiler, and the Rust toolchain. Reports median-of-3 wall + RSS + Rust/C ratios.
set -u
repo_root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../../../.." && pwd)
cd "$repo_root"
SELF="${MHS_SELFHOST_COMB:-/tmp/mhs-selfhost.comb}"
NORTH=47ea3c4062aa0f86d4bf77fe9dc864374b9b29073ed220e8d8f3fb41bb5ae862
CHEAP="${CHEAP:-}"            # C RTS heap flag; empty = C default (~787MB)
RINT="${RINT:-78643200}"     # Rust GC interval; 75M cells ~= 796MB (matches C default ~787MB)
MLABEL="${MLABEL:-~790MB (C default heap)}"
INC="-Isrc/runtime -Isrc/runtime/unix"; CSRC="src/runtime/main.c src/runtime/eval.c src/runtime/comb.c"
to_s() { awk -F: '{ if (NF==2) printf "%.2f",$1*60+$2; else printf "%.2f",$1*3600+$2*60+$3 }'; }
med()  { printf '%s\n' "$@" | sort -n | awk '{a[NR]=$0} END{print a[int((NR+1)/2)]}'; }
rss()  { awk '/Maximum resident/{printf "%.0f",$NF/1024}' "$1"; }
wl()   { grep -oP 'Elapsed.*: \K[0-9:.]+' "$1" | to_s; }
ok()   { [ "$(sha256sum "$1" 2>/dev/null|cut -d' ' -f1)" = "$NORTH" ] && echo OK || echo BAD; }

[ -f "$SELF" ]     || { echo "self-host comb not found: $SELF" >&2; exit 1; }
[ -x bin/mhseval ] || { echo "bin/mhseval missing (run: make bin/mhseval)" >&2; exit 1; }

echo "### building binaries ###"
cargo build --release --manifest-path rust/microhs-runtime/Cargo.toml --bin mhs-rust-bench 2>&1 | tail -1
RNP=target/release/mhs-rust-bench
# C-PGO: -O3 -flto, trained at the SAME heap budget
rm -rf /tmp/cpgo && mkdir -p /tmp/cpgo
cc -Wall -O3 -flto -fprofile-generate=/tmp/cpgo $INC $CSRC -lm -o /tmp/mhseval.gen 2>/dev/null
/tmp/mhseval.gen +RTS $CHEAP -r"$SELF" -RTS -i -imhs -isrc -ilib MicroHs.Main -o/tmp/ctrain.comb >/dev/null 2>&1
cc -Wall -O3 -flto -fprofile-use=/tmp/cpgo -fprofile-correction $INC $CSRC -lm -o /tmp/mhseval.pgo 2>/dev/null && echo "C-PGO built (train $(ok /tmp/ctrain.comb))"
# Rust-PGO: trained at the SAME interval
MHS_GC_NODE_INTERVAL=$RINT bash rust/microhs-runtime/tools/native/build-selfhost-pgo.sh >/tmp/rpgo.log 2>&1 && echo "Rust-PGO built" || echo "Rust-PGO FAILED (see /tmp/rpgo.log)"
RPG=/tmp/mhs-rust-pgo-selfhost/use/release/mhs-rust-bench

# NB: >/dev/null discards the binary's stdout (a /tmp PGO build prints a harmless
# "cannot find mhs.conf" line there); wall/RSS come from time -v on stderr.
runC() { /usr/bin/time -v "$1" +RTS $CHEAP -r"$SELF" -RTS -i -imhs -isrc -ilib MicroHs.Main -o/tmp/mx-$2$3.comb >/dev/null 2>/tmp/mx-$2$3.t
         echo "$(wl /tmp/mx-$2$3.t)"; echo "$2$3 $(wl /tmp/mx-$2$3.t)s $(rss /tmp/mx-$2$3.t)MB $(ok /tmp/mx-$2$3.comb)" >&2; }
runR() { MHS_GC_NODE_INTERVAL=$RINT /usr/bin/time -v "$1" --input "$SELF" --mode main --warmup-iters 0 --iters 1 -- \
           ./bin/mhs -i -imhs -isrc -ilib MicroHs.Main -o/tmp/mx-$2$3.comb >/dev/null 2>/tmp/mx-$2$3.t
         echo "$(wl /tmp/mx-$2$3.t)"; echo "$2$3 $(wl /tmp/mx-$2$3.t)s $(rss /tmp/mx-$2$3.t)MB $(ok /tmp/mx-$2$3.comb)" >&2; }

echo "### interleaved 4-way, median-of-3, budget=$MLABEL (CHEAP='$CHEAP' RINT=$RINT) ###" >&2
declare -a cnp cpg rnp rpg
for i in 1 2 3; do
  cnp+=("$(runC ./bin/mhseval CNP $i)")
  cpg+=("$(runC /tmp/mhseval.pgo CPG $i)")
  rnp+=("$(runR $RNP RNP $i)")
  rpg+=("$(runR $RPG RPG $i)")
done
CNP=$(med "${cnp[@]}"); CPG=$(med "${cpg[@]}"); RNP=$(med "${rnp[@]}"); RPG=$(med "${rpg[@]}")
Rc=$(rss /tmp/mx-CNP1.t); Rr=$(rss /tmp/mx-RNP1.t)
echo ""
echo "=== EQUAL-MEMORY MATRIX ($MLABEL) ==="
printf '%-14s %8s   %s\n' "runtime" "wall" "RSS"
printf '%-14s %7ss   %sMB\n' "C non-PGO"   "$CNP" "$Rc"
printf '%-14s %7ss   %sMB\n' "Rust non-PGO" "$RNP" "$Rr"
printf '%-14s %7ss   %sMB\n' "C PGO"       "$CPG" "$Rc"
printf '%-14s %7ss   %sMB\n' "Rust PGO"    "$RPG" "$Rr"
awk -v a="$CNP" -v b="$RNP" 'BEGIN{printf "\nnon-PGO  Rust/C: %.4f (%+.1f%%)\n",b/a,(b-a)*100/a}'
awk -v a="$CPG" -v b="$RPG" 'BEGIN{printf "PGO      Rust/C: %.4f (%+.1f%%)\n",b/a,(b-a)*100/a}'
