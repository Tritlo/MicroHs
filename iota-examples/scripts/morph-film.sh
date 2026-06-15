#!/usr/bin/env bash
# Render a reduction as a *smooth, semantic* morphing film with a generated
# soundtrack.  Each combinator animates its meaning (discarded branches grey out and
# drift, copies flash then emerge, the redex twinkles); captions track the ongoing
# reduction and a 2.5 s hold lets the result + chord ring.
#
#   morph-film.sh MODULE ROOT OUT.mp4 [seconds] [fps]
#     defaults: Lt  Lt.test  iota-examples/films/morph_lt23.mp4  120  30
#
# Fast streaming pipeline (no per-frame SVG->PNG): iota/morph (id-tracked reducer +
# provenance) -> morph_render.py --raw streams rgb24 frames into ffmpeg (libx264);
# captions are burnt in with ffmpeg drawtext from the --dims manifest, and
# morph_music.py's soundtrack is muxed in.
# Needs: bin/gmhs, ghc, python3 (numpy), ffmpeg.  (No ImageMagick.)
set -uo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
MOD="${1:-Lt}"; DEF="${2:-Lt.test}"; OUT="${3:-iota-examples/films/morph_lt23.mp4}"
SECS="${4:-120}"; FPS="${5:-30}"
MUSIC_MODE="${MUSIC_MODE:-comb}"   # how the soundtrack picks pitch: comb | size
FONT="${MORPH_FONT:-/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf}"
PROG=iota-examples/programs
W="$(mktemp -d)"; mkdir -p "$(dirname "$OUT")"

[ -x iota/morph ] || ghc -O0 -outputdir "$W/b" -o iota/morph iota/Morph.hs
# gmhs prints the dump then exits non-zero (no `main`); that is expected.
"$ROOT_DIR/bin/gmhs" -i"$PROG" -ilib -ddump-combinator "$MOD" > "$W/raw" 2>/dev/null || true
grep "^$MOD\." "$W/raw" > "$W/dump"
iota/morph --iota "$W/dump" "$DEF" > "$W/trace.txt"

# output canvas size, frame count, geometry scale + caption-timing manifest
read VW VH NF GS < <(python3 iota/morph_render.py --dims "$W/caps.tsv" "$SECS" "$FPS" < "$W/trace.txt")
echo "frames: $NF (~$(python3 -c "print(f'{$NF/$FPS:.1f}')")s @ ${FPS}fps), ${VW}x${VH}"
TOTAL="$(python3 -c "print(f'{$NF/$FPS:.3f}')")"
FILT="$(python3 iota/caps_filter.py "$W/caps.tsv" "$W/td" "$FONT" "$TOTAL" "$GS")"

# soundtrack on the same schedule, then stream frames straight into ffmpeg
python3 iota/morph_music.py "$W/music.wav" "$SECS" "$FPS" "$MUSIC_MODE" < "$W/trace.txt"
python3 iota/morph_render.py --raw "$SECS" "$FPS" < "$W/trace.txt" \
  | ffmpeg -y -f rawvideo -pix_fmt rgb24 -s "${VW}x${VH}" -framerate "$FPS" -i - -i "$W/music.wav" \
      -filter_complex "[0:v]tpad=stop_mode=clone:stop_duration=2.5,${FILT}[v]" \
      -map "[v]" -map 1:a -c:v libx264 -crf 20 -pix_fmt yuv420p -c:a aac -b:a 192k -shortest "$OUT" 2>/dev/null
rm -rf "$W"
echo "wrote $OUT"
