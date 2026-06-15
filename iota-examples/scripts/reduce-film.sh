#!/usr/bin/env bash
# Render the reduction of a definition as a film: each combinator step's term,
# drawn as an iota tree (GitHub-dark), one second per step, with explanations on
# the recursive-call and result steps.
#
#   reduce-film.sh MODULE ROOT OUT.mp4      (defaults: Lt Lt.test .../lt23.mp4)
#
# Needs: bin/gmhs, ghc (for iota/reduce + iota/treedraw deps), python3,
# ImageMagick, ffmpeg.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
MOD="${1:-Lt}"; DEF="${2:-Lt.test}"; OUT="${3:-iota-examples/films/lt23.mp4}"
PROG=iota-examples/programs
W="$(mktemp -d)"; mkdir -p "$(dirname "$OUT")"

[ -x iota/reduce ] || ghc -O0 -outputdir "$W/b" -o iota/reduce iota/Reduce.hs
# gmhs prints the dump then exits non-zero (no `main`); that is expected.
"$ROOT_DIR/bin/gmhs" -i"$PROG" -ilib -ddump-combinator "$MOD" > "$W/raw" 2>/dev/null || true
grep "^$MOD\." "$W/raw" > "$W/dump"

# GitHub-dark palette
BG='#0d1117'; FG='#e6edf3'; AC='#7ee787'; MUT='#6e7681'; F2=DejaVu-Sans-Mono
export IOTA_BG="$BG" IOTA_LEAF='#ffe08a' IOTA_EDGE_L=0.62

iota/reduce --iota "$W/dump" "$DEF"        > "$W/iota.txt"   # one 0/1 string per step
iota/reduce --full "$W/dump" "$DEF"        > "$W/trace.txt"  # the combinator term per step
N=$(wc -l < "$W/iota.txt")

# trees first, so we can size all frames to the largest
for i in $(seq 0 $((N-1))); do
  sed -n "$((i+1))p" "$W/iota.txt" | python3 iota/treedraw.py radial iota "$W/t$i.svg" "" >/dev/null
  convert -density 58 -background "$BG" "$W/t$i.svg" "$W/t$i.png"
done
maxW=0; maxH=0
for i in $(seq 0 $((N-1))); do
  read w h < <(identify -format '%w %h\n' "$W/t$i.png")
  if (( w > maxW )); then maxW=$w; fi
  if (( h > maxH )); then maxH=$h; fi
done
CW=$(( (maxW+160)/2*2 )); TH=$(( maxH/2*2 )); CAPH=150

# explanation for a step's term: lt N M -> "is N < M ?";  A -> "A = T = True"
explain () {
  case "$1" in
    "lt "*) set -- $1; echo "is ${2} < ${3} ?" ;;
    "A")    echo "A = T = True" ;;
    *)      echo "" ;;
  esac
}

for i in $(seq 0 $((N-1))); do
  term=$(sed -n "$((i+1))p" "$W/trace.txt" | sed -E 's/^ *[0-9]+  //')
  short=$(printf '%s' "$term" | cut -c1-74)
  expl=$(explain "$term")
  convert -size ${CW}x${CAPH} xc:"$BG" -font "$F2" \
    -fill "$MUT" -pointsize 24 -gravity northwest -annotate +24+16 "step $i / $((N-1))" \
    -fill "$FG"  -pointsize 30 -gravity north     -annotate +0+16 "$short" "$W/cap$i.png"
  if [ -n "$expl" ]; then
    convert "$W/cap$i.png" -font "$F2" -fill "$AC" -pointsize 34 \
      -gravity north -annotate +0+82 "$expl" "$W/cap$i.png"
  fi
  convert -background "$BG" "$W/t$i.png" -gravity center -background "$BG" -extent ${CW}x${TH} "$W/tr$i.png"
  convert "$W/cap$i.png" "$W/tr$i.png" -background "$BG" -append \
    -gravity center -background "$BG" -extent ${CW}x$((CAPH+TH)) "$W/frame$(printf '%03d' $i).png"
done

ffmpeg -y -framerate 1 -i "$W/frame%03d.png" \
  -vf "fps=30,scale=trunc(iw/2)*2:trunc(ih/2)*2" -c:v libx264 -pix_fmt yuv420p "$OUT"
rm -rf "$W"
echo "wrote $OUT"
