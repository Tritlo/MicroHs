#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
repo_root=$(cd -- "$script_dir/../.." && pwd)
manifest="$repo_root/rust/microhs-runtime/Cargo.toml"

input=${MHS_SELFHOST_COMB:-/tmp/mhs-selfhost.comb}
work_dir=${MHS_NEUTRALITY_DIR:-/tmp/mhs-rust-neutrality}
gc_interval=${MHS_GC_NODE_INTERVAL:-33554432}
step_limit=${MHS_STEP_LIMIT:-100000000}
bench_timeout=${MHS_BENCH_TIMEOUT:-900s}
repeat=${MHS_REPEAT:-1}

base_target=${MHS_BASE_TARGET_DIR:-"$work_dir/base-target"}
candidate_target=${MHS_CANDIDATE_TARGET_DIR:-"$work_dir/candidate-target"}
base_rustflags=${MHS_BASE_RUSTFLAGS:-}
candidate_rustflags=${MHS_CANDIDATE_RUSTFLAGS:-}
candidate_features=${MHS_CANDIDATE_FEATURES:-}

if [[ ! -f "$input" ]]; then
  echo "self-host input not found: $input" >&2
  exit 1
fi

mkdir -p "$work_dir"
rm -f "$work_dir"/base-*.comb "$work_dir"/base-*.log \
  "$work_dir"/cand-*.comb "$work_dir"/cand-*.log

build_bench() {
  local target_dir=$1
  local rustflags=$2
  local features=$3
  local -a cargo_cmd=(
    cargo build --release
    --manifest-path "$manifest"
    --bin mhs-rust-bench
  )
  if [[ -n "$features" ]]; then
    cargo_cmd+=(--features "$features")
  fi
  env CARGO_TARGET_DIR="$target_dir" RUSTFLAGS="$rustflags" "${cargo_cmd[@]}"
}

run_bench() {
  local label=$1
  local bin=$2
  local out="$work_dir/$label.comb"
  local log="$work_dir/$label.log"
  local -a bench_args=(
    --input "$input"
    --mode main
    --warmup-iters 0
    --iters 1
  )
  if [[ "$step_limit" != "none" ]]; then
    bench_args+=(--step-limit "$step_limit")
  fi

  env MHS_GC_NODE_INTERVAL="$gc_interval" \
    timeout "$bench_timeout" \
    "$bin" \
    "${bench_args[@]}" \
    -- \
    "$repo_root/bin/mhs" -i -imhs -isrc -ilib MicroHs.Main -o"$out" \
    >"$log" 2>&1
}

metric() {
  local log=$1
  local key=$2
  awk -F': ' -v key="$key" '$1 == key { print $2; exit }' "$log"
}

delta_pct() {
  local base=$1
  local candidate=$2
  awk -v base="$base" -v candidate="$candidate" 'BEGIN {
    if (base == 0) {
      print "n/a"
    } else {
      printf "%+.2f%%", ((candidate - base) * 100.0) / base
    }
  }'
}

avg_metric() {
  local key=$1
  shift
  awk -F': ' -v key="$key" '$1 == key { sum += $2; count += 1 } END {
    if (count == 0) {
      print ""
    } else {
      printf "%.3f", sum / count
    }
  }' "$@"
}

avg_metric_int() {
  local key=$1
  shift
  awk -F': ' -v key="$key" '$1 == key { sum += $2; count += 1 } END {
    if (count == 0) {
      print ""
    } else {
      printf "%.0f", sum / count
    }
  }' "$@"
}

print_row() {
  local label=$1
  local log=$2
  local delta=$3

  echo "| $label | $(metric "$log" parse_reduce_render_total_ms) | $delta | $(metric "$log" whnf_steps_per_s) | $(metric "$log" whnf_steps_per_iter) | $(metric "$log" gc_collections) | $(metric "$log" gc_total_pause_ms) | $(metric "$log" gc_high_water_nodes) | $(metric "$log" serialize_sink) |"
}

build_bench "$base_target" "$base_rustflags" ""
build_bench "$candidate_target" "$candidate_rustflags" "$candidate_features"

base_bin="$base_target/release/mhs-rust-bench"
candidate_bin="$candidate_target/release/mhs-rust-bench"

base_logs=()
candidate_logs=()
for idx in $(seq 1 "$repeat"); do
  base_logs+=("$work_dir/base-$idx.log")
  candidate_logs+=("$work_dir/cand-$idx.log")
  if (( idx % 2 == 1 )); then
    run_bench "base-$idx" "$base_bin"
    run_bench "cand-$idx" "$candidate_bin"
  else
    run_bench "cand-$idx" "$candidate_bin"
    run_bench "base-$idx" "$base_bin"
  fi
done

cmp_result=0
for idx in $(seq 1 "$repeat"); do
  if [[ -f "$work_dir/base-$idx.comb" && -f "$work_dir/cand-$idx.comb" ]]; then
    if cmp -s "$work_dir/base-$idx.comb" "$work_dir/cand-$idx.comb"; then
      :
    else
      cmp_result=$?
      break
    fi
  else
    cmp_result=missing-output
    break
  fi
done

base_avg_ms=$(avg_metric parse_reduce_render_total_ms "${base_logs[@]}")
candidate_avg_ms=$(avg_metric parse_reduce_render_total_ms "${candidate_logs[@]}")

cat <<EOF
| run | ms | delta | steps/s | steps | GCs | GC ms | high-water | sink |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
EOF

for idx in $(seq 1 "$repeat"); do
  base_log="$work_dir/base-$idx.log"
  candidate_log="$work_dir/cand-$idx.log"
  base_ms=$(metric "$base_log" parse_reduce_render_total_ms)
  candidate_ms=$(metric "$candidate_log" parse_reduce_render_total_ms)
  print_row "base-$idx" "$base_log" ""
  print_row "candidate-$idx" "$candidate_log" "$(delta_pct "$base_ms" "$candidate_ms")"
done

cat <<EOF
| base avg | $base_avg_ms |  | $(avg_metric whnf_steps_per_s "${base_logs[@]}") | $(avg_metric whnf_steps_per_iter "${base_logs[@]}") | $(avg_metric gc_collections "${base_logs[@]}") | $(avg_metric gc_total_pause_ms "${base_logs[@]}") | $(avg_metric_int gc_high_water_nodes "${base_logs[@]}") | $(avg_metric_int serialize_sink "${base_logs[@]}") |
| candidate avg | $candidate_avg_ms | $(delta_pct "$base_avg_ms" "$candidate_avg_ms") | $(avg_metric whnf_steps_per_s "${candidate_logs[@]}") | $(avg_metric whnf_steps_per_iter "${candidate_logs[@]}") | $(avg_metric gc_collections "${candidate_logs[@]}") | $(avg_metric gc_total_pause_ms "${candidate_logs[@]}") | $(avg_metric_int gc_high_water_nodes "${candidate_logs[@]}") | $(avg_metric_int serialize_sink "${candidate_logs[@]}") |

work_dir: $work_dir
cmp_result: $cmp_result
EOF
