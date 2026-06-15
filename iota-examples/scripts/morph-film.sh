#!/usr/bin/env bash
# Render a reduction as a *smooth* morphing film: the iota tree of each graph
# reduction step is tweened into the next (persisting nodes glide, copied subtrees
# grow in, dropped subtrees fade out), with the rule applied shown as a caption
# ("K x y -> x") and an explanation on the key steps ("is 2 < 3 ?", "A = T = True").
#
#   morph-film.sh MODULE ROOT OUT.mp4 [seconds] [fps]
#     defaults: Lt  Lt.test  iota-examples/films/morph_lt23.mp4  85  30
#
# Pipeline: iota/morph (id-tracked reducer) -> morph_render.py (SVG tween frames)
#           -> ImageMagick (rasterise) -> ffmpeg (mp4).
# Needs: bin/gmhs, ghc, python3, ImageMagick, ffmpeg.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
MOD="${1:-Lt}"; DEF="${2:-Lt.test}"; OUT="${3:-iota-examples/films/morph_lt23.mp4}"
SECS="${4:-88}"; FPS="${5:-30}"
PROG=iota-examples/programs
W="$(mktemp -d)"; mkdir -p "$(dirname "$OUT")"

[ -x iota/morph ] || ghc -O0 -outputdir "$W/b" -o iota/morph iota/Morph.hs

# gmhs prints the dump then exits non-zero (no `main`); that is expected.
"$ROOT_DIR/bin/gmhs" -i"$PROG" -ilib -ddump-combinator "$MOD" > "$W/raw" 2>/dev/null || true
grep "^$MOD\." "$W/raw" > "$W/dump"

# id-tracked reduction trace, iota-expanded:  RULE \t term \t (id-tagged sexp)
iota/morph --iota "$W/dump" "$DEF" > "$W/trace.txt"
echo "steps: $(wc -l < "$W/trace.txt")"

# tween frames (SVG) -> the renderer prints how many it wrote
NF="$(python3 iota/morph_render.py "$W/frames" "$SECS" "$FPS" < "$W/trace.txt")"
echo "frames: $NF  (~$(python3 -c "print(f'{$NF/$FPS:.1f}')")s @ ${FPS}fps)"

# rasterise all SVGs in one ImageMagick process (mogrify reuses it -> much faster
# than a convert per frame), then encode.
mogrify -density 96 -background '#0d1117' -format png "$W/frames"/f*.svg
ffmpeg -y -framerate "$FPS" -i "$W/frames/f%05d.png" \
  -vf "scale=trunc(iw/2)*2:trunc(ih/2)*2" -c:v libx264 -pix_fmt yuv420p -crf 20 "$OUT" 2>/dev/null
rm -rf "$W"
echo "wrote $OUT"
