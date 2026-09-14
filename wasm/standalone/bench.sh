#!/usr/bin/env bash
# Compare complete no-gmp compiler runs from one frozen input and source tree.
set -euo pipefail
if (($# != 3)); then
  echo "usage: $0 SOURCE_DIR INPUT.comb RUNTIMES.tsv" >&2
  echo 'TSV columns: label, kind, executable, expected output, Rust GC interval.' >&2
  echo 'Kinds: native-c, native-rust, wasi-c, wasi-rust, wasi-wat. No header.' >&2
  echo 'Use - for the GC interval of a non-Rust runtime.' >&2
  exit 2
fi
[[ $(uname -s) == Linux ]] || { echo 'This runner requires GNU time on Linux.' >&2; exit 2; }
repeat=${MHS_REPEAT:-3}
[[ $repeat =~ ^[1-9][0-9]*$ ]] || { echo 'MHS_REPEAT must be positive.' >&2; exit 2; }
repo=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)
mkdir -p "$repo/target/mhs-wasm"
work=$(mktemp -d "$repo/target/mhs-wasm/selfhost.XXXXXX")
mkdir "$work/source"
for directory in mhs src lib; do cp -a -- "$1/$directory" "$work/source/"; done
cp -- "$2" "$work/input.comb"
cp -- "$3" "$work/runtimes.tsv"
wasmtime --version > "$work/toolchain.txt"
uname -a >> "$work/toolchain.txt"
(
  cd "$work"
  find source -type f -print0 | LC_ALL=C sort -z | xargs -0 sha256sum
  sha256sum input.comb runtimes.tsv
) > "$work/inputs.sha256"
labels=() kinds=() intervals=()
while IFS=$'\t' read -r label kind executable expected interval extra; do
  [[ -n $label ]] || continue
  [[ $label =~ ^[a-zA-Z0-9_.-]+$ && -z $extra && -n $interval ]] || {
    echo 'Each manifest row needs five tab-separated fields and a simple label.' >&2; exit 2;
  }
  case $kind in native-c|native-rust|wasi-c|wasi-rust|wasi-wat) ;;
    *) echo "unknown runtime kind: $kind" >&2; exit 2;; esac
  for previous in "${labels[@]}"; do
    [[ $previous != "$label" ]] || { echo "duplicate label: $label" >&2; exit 2; }
  done
  if [[ $kind == *rust ]]; then
    [[ $interval =~ ^[1-9][0-9]*$ ]] || { echo 'Rust needs an explicit GC interval.' >&2; exit 2; }
  else
    [[ $interval == - ]] || { echo 'Use - for non-Rust GC intervals.' >&2; exit 2; }
  fi
  index=${#labels[@]}
  labels+=("$label"); kinds+=("$kind"); intervals+=("$interval")
  cp -- "$executable" "$work/$index.input"
  cp -- "$expected" "$work/$index.expected"
  sha256sum "$work/$index.input" "$work/$index.expected" >> "$work/inputs.sha256"
  if [[ $kind == wasi-* ]]; then
    wasmtime compile -W exceptions=y -O opt-level=2 "$work/$index.input" -o "$work/$index.run"
  else
    cp -- "$work/$index.input" "$work/$index.run"
    chmod +x "$work/$index.run"
  fi
done < "$work/runtimes.tsv"
((${#labels[@]})) || { echo 'The manifest has no runtimes.' >&2; exit 2; }
printf 'runtime\tround\twall_s\tmax_rss_kib\tsha256\n' > "$work/results.tsv"
echo "results: $work"
cd "$work/source"
for ((round = 1; round <= repeat; round++)); do
  for ((position = 0; position < ${#labels[@]}; position++)); do
    index=$position
    if ((round % 2 == 0)); then index=$((${#labels[@]} - position - 1)); fi
    label=${labels[index]}; kind=${kinds[index]}; stem="$label-$round"
    input="$work/input.comb"; output="$work/$stem.comb"
    if [[ $kind == wasi-* ]]; then
      input=/bench/input.comb; output="/bench/$stem.comb"
      command=(wasmtime run --allow-precompiled -W exceptions=y,max-wasm-stack=8388608
        --dir .::. --dir "$work::/bench")
      if [[ $kind == wasi-rust ]]; then command+=(--env "MHS_GC_NODE_INTERVAL=${intervals[index]}"); fi
      command+=("$work/$index.run")
    elif [[ $kind == native-rust ]]; then
      command=(env "MHS_GC_NODE_INTERVAL=${intervals[index]}" "$work/$index.run")
    else
      command=("$work/$index.run")
    fi
    case $kind in
      *rust) command+=(--input "$input" --mode main --warmup-iters 0 --iters 1);;
      *-c) command+=(--mode main --warmup-iters 0 --iters 1 "$input");;
      wasi-wat) command+=("$input");;
    esac
    command+=(-- ./bin/mhs -i -imhs -isrc -ilib/no-gmp -ilib MicroHs.Main "-o$output")
    /usr/bin/time -f 'wall_s: %e\nmax_rss_kib: %M' \
      timeout "${MHS_BENCH_TIMEOUT:-600s}" "${command[@]}" > "$work/$stem.log" 2>&1
    cmp -s "$work/$index.expected" "$work/$stem.comb" || {
      echo "output mismatch: $work/$stem.comb" >&2; exit 1;
    }
    digest=$(sha256sum "$work/$stem.comb"); digest=${digest%% *}
    awk -F': ' -v label="$label" -v round="$round" -v sha="$digest" '
      $1 == "wall_s" {wall=$2}
      $1 == "max_rss_kib" {rss=$2}
      END {printf "%s\t%d\t%s\t%s\t%s\n", label, round, wall, rss, sha}
    ' "$work/$stem.log" | tee -a "$work/results.tsv"
  done
done
