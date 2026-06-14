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

radial () {   # radial <Module> <root> <out> <title>
  "$IOTA" iota "$WORK/$1.dump" "$2" | python3 iota/treedraw.py radial iota "$WORK/$3.svg" "$4"
  convert -density 70 -depth 8 -background black "$WORK/$3.svg" "$PIC/$3.png"
  echo "  $PIC/$3.png"
}

echo "rendering:"
dump Demo
topdown Demo Demo.tw  tw_comb  "tw = f (f x)  ->  S B I"
radial  Demo Demo.tw  tw_iota  "tw -> iota (581 symbols)"

dump Church6
radial  Church6 Church6.six six_iota "Church 6 -> iota (2806 symbols)"

dump ChurchLt
topdown ChurchLt ChurchLt.lt lt_comb  "Church-numeral (<) -- combinator tree"
radial  ChurchLt ChurchLt.lt lt_iota  "Church-numeral (<) -> iota (15403 symbols)"

dump ChurchList
topdown ChurchList ChurchList.cons cons_comb "cons = \\h t c n -> c h (t c n)"
radial  ChurchList ChurchList.l3   l3_iota   "Church list [a,b,c] -> iota (19323 symbols)"

dump Prim
topdown Prim Prim.mix mix_comb "mix -- primitive (+) boxed amongst combinators"

dump Quicksort
radial Quicksort Quicksort.qs3   qs3_iota   "quicksort [a,b,c] -> iota (83007 symbols)"
radial Quicksort Quicksort.ex312 qs312_iota "quicksort [3,1,2] -> iota (75927 symbols)"

rm -rf "$WORK"
echo "done."
