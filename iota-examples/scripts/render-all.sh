#!/usr/bin/env bash
# Regenerate every picture in iota-examples/pictures/ from the programs.
#
# Requirements: bin/gmhs (run `make bin/gmhs`), a Haskell ghc to build the
# renderer, python3, and ImageMagick `convert`.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

GMHS=./bin/gmhs
IOTA=./iota/iota
PROG=iota-examples/programs
PIC=iota-examples/pictures
WORK="$(mktemp -d)"
mkdir -p "$PIC"

[ -x "$GMHS" ] || { echo "build bin/gmhs first: make bin/gmhs" >&2; exit 1; }
[ -x "$IOTA" ] || ghc -O0 -outputdir "$WORK/build" -o "$IOTA" iota/Iota.hs

dump () {  # dump <Module> ; keeps only that module's combinator defs.
  # gmhs prints the dump then exits non-zero (no `main`); that is expected.
  "$GMHS" -i"$PROG" -ilib -ddump-combinator "$1" > "$WORK/$1.raw" 2>/dev/null || true
  grep "^$1\." "$WORK/$1.raw" > "$WORK/$1.dump"
}

topdown () {  # topdown <Module> <root> <out> <title>
  "$IOTA" sexp "$WORK/$1.dump" "$2" | python3 iota/treedraw.py topdown sexp "$WORK/$3.svg" "$4"
  convert -density 140 -background white "$WORK/$3.svg" "$PIC/$3.png"
  echo "  $PIC/$3.png"
}

radial () {   # radial <Module> <root> <out> <title> ; symbol count is appended
  local s; s="$("$IOTA" iota "$WORK/$1.dump" "$2")"
  printf '%s' "$s" | python3 iota/treedraw.py radial iota "$WORK/$3.svg" "$4 (${#s} symbols)"
  convert -density 70 -depth 8 -background black "$WORK/$3.svg" "$PIC/$3.png"
  echo "  $PIC/$3.png  (${#s} symbols)"
}

echo "rendering:"
dump Demo
topdown Demo Demo.tw  tw_comb  "tw = f (f x)  ->  S B I"
radial  Demo Demo.tw  tw_iota  "tw -> iota"

dump Church6
radial  Church6 Church6.six six_iota "Church 6 -> iota"

dump ChurchLt
topdown ChurchLt ChurchLt.lt lt_comb  "Church-numeral (<) -- combinator tree"
radial  ChurchLt ChurchLt.lt lt_iota  "Church-numeral (<) -> iota"

dump ChurchList
topdown ChurchList ChurchList.cons cons_comb "cons = \\h t c n -> c h (t c n)"
radial  ChurchList ChurchList.l3   l3_iota   "Church list [a,b,c] -> iota"

dump Quicksort
radial Quicksort Quicksort.qs3   qs3_iota   "quicksort [a,b,c] -> iota"
radial Quicksort Quicksort.ex312 qs312_iota "quicksort [3,1,2] -> iota"

# --- the combinator zoo: every MicroHs combinator as its own iota tree ---
# Each combinator is rendered straight from an empty dump (the tool expands the
# combinator's reduction rule to S/K/I, then to iota), and tiled into one montage.
: > "$WORK/empty.dump"
ZOO="S K I B C A U Z P R O J S' B' C' C'B K2 K3 K4 Y"
# definition shown under each: iota form for S/K/I, combinator algebra for the
# rest (all verified by reduction).  P in Y = S P P is the SK form of \x.f(x x).
declare -A DEF=(
  [S]='ι(ι(ι(ιι)))' [K]='ι(ι(ιι))' [I]='ιι'
  [B]='S(KS)K' [C]='S(BBS)(KK)' [A]='KI' [U]='CI' [Z]='BK'
  [P]='BC(CI)' [R]='CC' [O]='B(BK)(BC(CI))' [J]='BK(CI)'
  ["S'"]='B(BS)B' ["B'"]='BB' ["C'"]='B(BC)B' ["C'B"]="C' B"
  [K2]='BKK' [K3]='B K2 K' [K4]='B K3 K' [Y]='S P P'
)
radial_tiles=()
for c in $ZOO; do
  s="$("$IOTA" iota "$WORK/empty.dump" "$c")"
  safe="$(printf '%s' "$c" | tr "'" p)"
  printf '%s' "$s" | python3 iota/treedraw.py radial iota "$WORK/zr_$safe.svg" "" >/dev/null
  convert -density 55 -depth 8 -background '#0b0f17' "$WORK/zr_$safe.svg" "$WORK/zr_$safe.png"
  radial_tiles+=( -label "$c  (${#s})
${DEF[$c]}" "$WORK/zr_$safe.png" )
done
montage "${radial_tiles[@]}" -tile 5x4 -geometry 360x396+8+10 -background '#0b0f17' \
  -fill '#dbe2ee' -pointsize 30 -title "MicroHs combinators in iota  —  defn under each   (P = \\x. f (x x))" \
  "$PIC/zoo_iota.png"
convert "$PIC/zoo_iota.png" -depth 8 "$PIC/zoo_iota.png"
echo "  $PIC/zoo_iota.png"

small_tiles=()       # the small ones are legible top-down
for c in S K I A; do
  s="$("$IOTA" iota "$WORK/empty.dump" "$c")"
  printf '%s' "$s" | python3 iota/treedraw.py topdown iota "$WORK/zt_$c.svg" "" >/dev/null
  convert -density 90 -background white "$WORK/zt_$c.svg" "$WORK/zt_$c.png"
  small_tiles+=( -label "$c  (${#s})" "$WORK/zt_$c.png" )
done
montage "${small_tiles[@]}" -tile 2x2 -geometry 640x470+14+16 -background white \
  -fill '#222' -pointsize 26 -title "Small combinators as iota trees (top-down)" \
  "$PIC/zoo_topdown.png"
convert "$PIC/zoo_topdown.png" -depth 8 "$PIC/zoo_topdown.png"
echo "  $PIC/zoo_topdown.png"

rm -rf "$WORK"
echo "done."
