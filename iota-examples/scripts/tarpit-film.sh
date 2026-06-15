#!/usr/bin/env bash
# Render a "long reduction" from the Turing Tarpit (Hemann & Holk, "Visualizing the
# Turing Tarpit") as a morph film.  Iota/Jot programs are pure S/K terms, so a
# program is just a combinator term we reduce (capped at N steps), rendering each
# step as an iota tree and morphing between them.  A divergent program never
# resolves -- the mandala just grows; a terminating one balloons then settles.
#
#   tarpit-film.sh DUMP ROOT OUT.mp4 [steps] [seconds] [fps] [musicmode]
#     e.g. tarpit-film.sh iota-examples/tarpit/programs.dump Tarpit.grow out.mp4 40 55 30 comb
#   TARPIT_AO=1  -> applicative (eager / call-by-value) order instead of normal order
#
# Same streaming pipeline as scripts/morph-film.sh (no gmhs -- the program is given
# directly as a combinator dump).  Needs ghc, python3 (numpy), ffmpeg.
set -uo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
DUMP="$1"; ROOT="$2"; OUT="${3:-iota-examples/films/tarpit.mp4}"
STEPS="${4:-40}"; SECS="${5:-55}"; FPS="${6:-30}"; MUSIC_MODE="${7:-comb}"
FONT="${MORPH_FONT:-/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf}"
W="$(mktemp -d)"; mkdir -p "$(dirname "$OUT")"

[ -x iota/morph ] || ghc -O0 -outputdir "$W/b" -o iota/morph iota/Morph.hs

AOFLAG=(); [ -n "${TARPIT_AO:-}" ] && AOFLAG=(--ao)
iota/morph --iota "${AOFLAG[@]}" "$DUMP" "$ROOT" "$STEPS" > "$W/trace.txt"
echo "steps: $(wc -l < "$W/trace.txt")  (cap $STEPS${TARPIT_AO:+, applicative order})"

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
