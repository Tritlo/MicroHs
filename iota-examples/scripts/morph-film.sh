#!/usr/bin/env bash
# Render a reduction as a *smooth, semantic* morphing film with a generated
# soundtrack.  Each combinator animates its meaning (discarded branches grey out and
# drift, copies flash then emerge, the redex twinkles); captions track the ongoing
# reduction and a 2.5 s hold lets the result + chord ring.
#
#   morph-film.sh MODULE ROOT OUT.mp4 [seconds] [fps]
#     defaults: Lt  Lt.test  iota-examples/films/morph_lt23.mp4  120  30
#
# Two-level parallel, segmented pipeline (also keeps each segment's caption
# filtergraph small, so long reductions don't blow ffmpeg's command line):
#   * split the timeline into SEG_SECS chunks; render up to SEG_JOBS chunks at once;
#   * each chunk streams its frames from morph_render.py --raw (MORPH_JOBS frame
#     workers) into its own ffmpeg, burning just that chunk's captions;
#   * concat the chunks (stream copy) and mux the soundtrack.
# Knobs: SEG_SECS (30), SEG_JOBS (4), MORPH_JOBS (4), MUSIC_MODE (comb).
# Needs: bin/gmhs, ghc, python3 (numpy), ffmpeg.  (No ImageMagick.)
set -uo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
MOD="${1:-Lt}"; DEF="${2:-Lt.test}"; OUT="${3:-iota-examples/films/morph_lt23.mp4}"
SECS="${4:-120}"; FPS="${5:-30}"
SEG_SECS="${SEG_SECS:-30}"; SEG_JOBS="${SEG_JOBS:-4}"; export MORPH_JOBS="${MORPH_JOBS:-4}"
MUSIC_MODE="${MUSIC_MODE:-comb}"
FONT="${MORPH_FONT:-/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf}"
PROG=iota-examples/programs
W="$(mktemp -d)"; mkdir -p "$(dirname "$OUT")"

[ -x iota/morph ] || ghc -O0 -outputdir "$W/b" -o iota/morph iota/Morph.hs
"$ROOT_DIR/bin/gmhs" -i"$PROG" -ilib -ddump-combinator "$MOD" > "$W/raw" 2>/dev/null || true
grep "^$MOD\." "$W/raw" > "$W/dump"
iota/morph --iota "$W/dump" "$DEF" > "$W/trace.txt"

read VW VH NF GS < <(python3 iota/morph_render.py --dims "$W/caps.tsv" "$SECS" "$FPS" < "$W/trace.txt")
TOTAL="$(python3 -c "print(f'{$NF/$FPS:.3f}')")"
SEGF=$((SEG_SECS*FPS)); NSEG=$(( (NF + SEGF - 1) / SEGF ))
echo "frames: $NF (~${TOTAL}s @ ${FPS}fps), ${VW}x${VH}; $NSEG segment(s) x ${SEG_SECS}s, ${SEG_JOBS} parallel x ${MORPH_JOBS} workers"

python3 iota/morph_music.py "$W/music.wav" "$SECS" "$FPS" "$MUSIC_MODE" < "$W/trace.txt" >/dev/null

render_seg () {                      # render_seg <k>
  local k="$1" start count tstart tend last pre body filt
  start=$((k*SEGF)); count=$((NF-start)); [ "$count" -gt "$SEGF" ] && count=$SEGF
  tstart="$(python3 -c "print(f'{$start/$FPS:.3f}')")"
  if [ "$k" -eq $((NSEG-1)) ]; then last=1; tend="$(python3 -c "print($TOTAL+5)")"; pre="tpad=stop_mode=clone:stop_duration=2.5"
  else last=0; tend="$(python3 -c "print(f'{($start+$count)/$FPS:.3f}')")"; pre=""; fi
  filt="$(python3 iota/caps_filter.py "$W/caps.tsv" "$W/td$k" "$FONT" "$TOTAL" "$GS" "$tstart" "$tend")"
  if   [ -n "$pre" ] && [ -n "$filt" ]; then body="$pre,$filt"
  elif [ -n "$filt" ]; then body="$filt"
  elif [ -n "$pre" ];  then body="$pre"
  else body="null"; fi
  printf '[0:v]%s[v]' "$body" > "$W/fc$k"
  python3 iota/morph_render.py --raw "$SECS" "$FPS" "$start" "$count" < "$W/trace.txt" \
    | ffmpeg -y -f rawvideo -pix_fmt rgb24 -s "${VW}x${VH}" -framerate "$FPS" -i - \
        -filter_complex_script "$W/fc$k" -map "[v]" \
        -c:v libx264 -crf 20 -preset medium -pix_fmt yuv420p -an "$W/seg$k.mp4" 2>"$W/ff$k.log"
}

running=0
for k in $(seq 0 $((NSEG-1))); do
  render_seg "$k" &
  running=$((running+1))
  if [ "$running" -ge "$SEG_JOBS" ]; then wait -n; running=$((running-1)); fi
done
wait

: > "$W/list.txt"
for k in $(seq 0 $((NSEG-1))); do
  [ -s "$W/seg$k.mp4" ] || { echo "segment $k failed:"; tail -3 "$W/ff$k.log"; rm -rf "$W"; exit 1; }
  echo "file '$W/seg$k.mp4'" >> "$W/list.txt"
done
ffmpeg -y -f concat -safe 0 -i "$W/list.txt" -c copy "$W/silent.mp4" 2>/dev/null
ffmpeg -y -i "$W/silent.mp4" -i "$W/music.wav" -map 0:v -map 1:a -c:v copy -c:a aac -b:a 192k -shortest "$OUT" 2>/dev/null
rm -rf "$W"
echo "wrote $OUT"
