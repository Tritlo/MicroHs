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
ZBG='#0b0f17'
ZFONT="DejaVu Sans Mono"   # fallback; prefer Berkeley Mono if installed
if fc-match 'BerkeleyMono Nerd Font' 2>/dev/null | grep -qi berkeley; then
  ZFONT="BerkeleyMono Nerd Font"
fi
# definition under each: iota form for S/K/I, combinator algebra for the rest
# (all verified by reduction).  The P in Y=SPP is the SK form of \x.f(x x) (an
# open term), NOT the pairing combinator P listed in the zoo.
declare -A DEF=(
  [S]='(ι(ι(ι(ιι))))' [K]='(ι(ι(ιι)))' [I]='(ιι)'
  [B]='S(KS)K' [C]='S(BBS)(KK)' [A]='KI' [U]='CI' [Z]='BK'
  [P]='BC(CI)' [R]='CC' [O]='B(BK)(BC(CI))' [J]='BK(CI)'
  ["S'"]='B(BS)B' ["B'"]='BB' ["C'"]='B(BC)B' ["C'B"]='C'"'"'B'
  [K2]='BKK' [K3]='BK2K' [K4]='BK3K' [Y]='SPP'
)
# Pango renders labels so Berkeley Mono can carry the combinator algebra while
# the iota/lambda glyphs fall back to DejaVu automatically.
zoo_tiles=()
for c in $ZOO; do
  s="$("$IOTA" iota "$WORK/empty.dump" "$c")"
  safe="$(printf '%s' "$c" | tr "'" p)"
  printf '%s' "$s" | python3 iota/treedraw.py radial iota "$WORK/zr_$safe.svg" "" >/dev/null
  convert -density 55 -background "$ZBG" "$WORK/zr_$safe.svg" \
    -resize 330x290 -gravity center -extent 360x300 "$WORK/zi_$safe.png"
  convert -background "$ZBG" pango:"<span font='$ZFONT 25' foreground='#e3e9f2'>$c  (${#s})</span>" "$WORK/zn_$safe.png"
  convert -background "$ZBG" pango:"<span font='$ZFONT 25' foreground='#9fb0c8'>${DEF[$c]}</span>" "$WORK/zd_$safe.png"
  convert "$WORK/zn_$safe.png" "$WORK/zd_$safe.png" -background "$ZBG" \
    -gravity center -append -extent 360x96 "$WORK/zlbl_$safe.png"
  convert "$WORK/zi_$safe.png" "$WORK/zlbl_$safe.png" -background "$ZBG" -append "$WORK/zfull_$safe.png"
  zoo_tiles+=( "$WORK/zfull_$safe.png" )
done
montage "${zoo_tiles[@]}" -tile 5x4 -geometry +6+6 -background "$ZBG" "$WORK/zgrid.png"
ZW="$(identify -format '%w' "$WORK/zgrid.png")"
convert -background "$ZBG" -size "${ZW}x" -gravity center \
  pango:"<span font='$ZFONT 42' foreground='#eef2f8'>MicroHs combinators as iota trees (symbol counts)</span>" "$WORK/zh1.png"
convert -background "$ZBG" -size "${ZW}x" -gravity center \
  pango:"<span font='$ZFONT 30' foreground='#9fb0c8'>ι = λf.((fλa.λb.λc.((ac)(bc)))λd.λe.d)</span>" "$WORK/zh2.png"
convert -size "${ZW}x16" xc:"$ZBG" "$WORK/zpad.png"
convert "$WORK/zpad.png" "$WORK/zh1.png" "$WORK/zh2.png" "$WORK/zpad.png" "$WORK/zgrid.png" \
  -background "$ZBG" -append -depth 8 "$PIC/zoo_iota.png"
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
