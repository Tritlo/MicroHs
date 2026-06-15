#!/usr/bin/env bash
# Render a "long reduction" from the Turing Tarpit (Hemann & Holk, "Visualizing the
# Turing Tarpit") as a morph film.  Iota/Jot programs are pure S/K terms, so a
# divergent program -- the paper's non-terminating cluster -- is just an S/K term
# that reduces forever; we reduce it (capped at N steps), render every step as an
# iota tree, and morph between them.  It never resolves: the mandala just grows.
#
#   tarpit-film.sh DUMP ROOT OUT.mp4 [steps] [seconds] [fps] [musicmode]
#     e.g. tarpit-film.sh iota-examples/tarpit/programs.dump Tarpit.grow out.mp4 40 55 30 comb
#
# Reuses iota/morph_render.py + iota/morph_music.py.  Needs ghc, python3,
# ImageMagick, ffmpeg (no gmhs -- the program is given directly as a combinator dump).
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
DUMP="$1"; ROOT="$2"; OUT="${3:-iota-examples/films/tarpit.mp4}"
STEPS="${4:-40}"; SECS="${5:-55}"; FPS="${6:-30}"; MUSIC="${7:-comb}"
W="$(mktemp -d)"; mkdir -p "$(dirname "$OUT")"

[ -x iota/morph ] || ghc -O0 -outputdir "$W/b" -o iota/morph iota/Morph.hs

# id-tracked reduction, capped at STEPS (the program diverges):  rule \t term \t iota-sexp
iota/morph --iota "$DUMP" "$ROOT" "$STEPS" > "$W/trace.txt"
echo "steps: $(wc -l < "$W/trace.txt")  (cap $STEPS)"

NF="$(python3 iota/morph_render.py "$W/frames" "$SECS" "$FPS" < "$W/trace.txt")"
echo "frames: $NF  (~$(python3 -c "print(f'{$NF/$FPS:.1f}')")s @ ${FPS}fps)"

mogrify -density 96 -background '#0d1117' -format png "$W/frames"/f*.svg
ffmpeg -y -framerate "$FPS" -i "$W/frames/f%05d.png" \
  -vf "scale=trunc(iw/2)*2:trunc(ih/2)*2" -c:v libx264 -pix_fmt yuv420p -crf 20 "$W/silent.mp4" 2>/dev/null

python3 iota/morph_music.py "$W/music.wav" "$SECS" "$FPS" "$MUSIC" < "$W/trace.txt"
# hold the final (largest) frame 2.5 s -- the "...and it goes on forever" beat
ffmpeg -y -i "$W/silent.mp4" -i "$W/music.wav" \
  -filter_complex "[0:v]tpad=stop_mode=clone:stop_duration=2.5[v]" \
  -map "[v]" -map 1:a -c:v libx264 -pix_fmt yuv420p -crf 20 -c:a aac -b:a 192k -shortest "$OUT" 2>/dev/null
rm -rf "$W"
echo "wrote $OUT"
