#!/usr/bin/env bash
# Compare C-compatible WASI benchmark modules on a frozen compiler source tree.
set -euo pipefail
if (($# < 3)); then
  echo "usage: $0 INPUT.comb EXPECTED.comb MODULE.wasm [MODULE.wasm ...]" >&2
  echo 'MHS_SOURCE_REF selects the source commit. MHS_REPEAT defaults to 3.' >&2
  exit 2
fi
[[ $(uname -s) == Linux ]] || { echo 'This runner requires GNU time on Linux.' >&2; exit 2; }
repo=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
repeat=${MHS_REPEAT:-3}
[[ $repeat =~ ^[1-9][0-9]*$ ]] || { echo 'MHS_REPEAT must be positive.' >&2; exit 2; }
ref=${MHS_SOURCE_REF:-HEAD}
mkdir -p "$repo/target/mhs-wasm"
work=$(mktemp -d "$repo/target/mhs-wasm/bench.XXXXXX")
cp -- "$1" "$work/input.comb"
cp -- "$2" "$work/expected.comb"
shift 2
mkdir "$work/source"
commit=$(git -C "$repo" rev-parse "$ref^{commit}")
printf '%s\n' "$commit" > "$work/source-commit.txt"
git -C "$repo" archive "$commit" mhs src lib | tar -x -C "$work/source"
wasmtime --version > "$work/toolchain.txt"
sha256sum "$work/input.comb" "$work/expected.comb" > "$work/inputs.sha256"
labels=()
index=0
for module in "$@"; do
  label=$(basename -- "$module" .wasm)
  label=${label//[^a-zA-Z0-9_.-]/_}
  for previous in "${labels[@]}"; do
    [[ $previous != "$label" ]] || { echo "duplicate module name: $label" >&2; exit 2; }
  done
  labels+=("$label")
  cp -- "$module" "$work/$index.wasm"
  sha256sum "$work/$index.wasm" >> "$work/inputs.sha256"
  wasmtime compile -W exceptions=y -O opt-level=2 "$work/$index.wasm" -o "$work/$index.cwasm"
  index=$((index + 1))
done
printf 'runtime\tround\tinternal_ms\twall_s\tmax_rss_kib\tsha256\n' > "$work/results.tsv"
echo "results: $work"
cd "$work/source"
for ((round = 1; round <= repeat; round++)); do
  for ((position = 0; position < ${#labels[@]}; position++)); do
    index=$position
    if ((round % 2 == 0)); then index=$((${#labels[@]} - position - 1)); fi
    label=${labels[index]}
    stem="$label-$round"
    /usr/bin/time -f 'wall_s: %e\nmax_rss_kib: %M' \
      timeout "${MHS_BENCH_TIMEOUT:-600s}" \
      wasmtime run --allow-precompiled -W exceptions=y,max-wasm-stack=8388608 \
      --dir .::. --dir "$work::/bench" "$work/$index.cwasm" \
      --mode main --warmup-iters 0 --iters 1 /bench/input.comb \
      -- ./bin/mhs -i -imhs -isrc -ilib MicroHs.Main "-o/bench/$stem.comb" \
      > "$work/$stem.log" 2>&1
    if ! cmp -s "$work/expected.comb" "$work/$stem.comb"; then
      echo "output mismatch: $work/$stem.comb" >&2
      exit 1
    fi
    digest=$(sha256sum "$work/$stem.comb")
    digest=${digest%% *}
    awk -F': ' -v label="$label" -v round="$round" -v sha="$digest" '
      $1 == "c_parse_eval_serialize_total_ms" {ms=$2}
      $1 == "wall_s" {wall=$2}
      $1 == "max_rss_kib" {rss=$2}
      END {printf "%s\t%d\t%s\t%s\t%s\t%s\n", label, round, ms, wall, rss, sha}
    ' "$work/$stem.log" | tee -a "$work/results.tsv"
  done
done
