use super::c_compare::bench_c_mhsbench;
use super::metrics::{mib_per_s, millis, nanos_millis, nanos_per_iter, print_gc_events};
#[cfg(feature = "profile")]
use super::profile_output::print_profile;
#[cfg(feature = "profile")]
use super::runner::profile_eval;
use super::runner::{bench_eval, bench_parse};
use super::scenarios::make_scenario;
use super::*;

const DEFAULT_ITERS: usize = 1_000;
const DEFAULT_WARMUP_ITERS: usize = 0;
const DEFAULT_SCENARIO: &str = "identity-chain:1000";

struct Config {
    input: Vec<u8>,
    name: String,
    mode: BenchMode,
    iters: usize,
    warmup_iters: usize,
    program_args: Vec<Vec<u8>>,
    executable_path: Option<Vec<u8>>,
    c_mhsbench: Option<String>,
    c_mhsbench_mode: BenchMode,
    profile: bool,
    profile_top: usize,
    step_limit: Option<usize>,
}

#[derive(Clone, Copy)]
pub(super) enum BenchMode {
    Whnf,
    Main,
}

impl BenchMode {
    pub(super) fn parse(text: &str) -> Result<Self, String> {
        match text {
            "whnf" => Ok(Self::Whnf),
            "main" => Ok(Self::Main),
            _ => Err(format!("invalid benchmark mode: {text}")),
        }
    }

    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Whnf => "whnf",
            Self::Main => "main",
        }
    }
}

fn usage() {
    eprintln!(
        "usage: mhs-rust-bench [--iters N] [--input FILE | --scenario identity-chain:N|arith-chain:N|int64-chain:N|float64-chain:N|float32-chain:N|bytes-chain:N|foreignptr-slice:N|cstring-pack:N|unpack-chain:N|fromutf8-chain:N|array-chain:N|io-chain:N|io-array-chain:N|io-bytes-chain:N|io-control-chain:N|performio-apply-chain:N|argref-chain:N|stdio-chain:N|ffi-chain:N|ffi-math-chain:N|ffi-const-chain:N|ffi-mem-chain:N|ffi-wide-mem-chain:N|ffi-word-mem-chain:N|ffi-ptr-mem-chain:N|ffi-strcpy-chain:N|md5-string-chain:N|getenv-chain:N|env-set-chain:N|errno-chain:N|getcwd-chain:N|dir-read-chain:N|remove-missing-chain:N|file-read-close-chain:N|utf8-bfile-read-chain:N|crlf-bfile-read-chain:N|base64-bfile-read-chain:N|lz77-bfile-read-chain:N|bwt-bfile-read-chain:N|lzma-bfile-read-chain:N|rle-bfile-read-chain:N|buf-bfile-read-chain:N|bfile-read-chain:N|mvar-chain:N|ptr-chain:N|rnf-chain:N|stableptr-chain:N|weak-chain:N|zoo-chain:N|data-chain:N]\n\
                                  [--mode whnf|main]\n\
                                  [--warmup-iters N]\n\
                                  [--c-mhsbench PATH] [--c-mhsbench-mode whnf|main]\n\
                                  [--profile] [--profile-top N] [--step-limit N]\n\
                                  [-- PROGRAM ARGS...]\n\
         default: --scenario {DEFAULT_SCENARIO} --iters {DEFAULT_ITERS}"
    );
}

pub(super) fn main() -> ExitCode {
    let config = match parse_args(env::args().skip(1)) {
        Ok(config) => config,
        Err(message) => {
            eprintln!("{message}");
            usage();
            return ExitCode::from(2);
        }
    };
    #[cfg(not(feature = "profile"))]
    if config.profile {
        let _ = config.profile_top;
        eprintln!("--profile requires rebuilding mhs-rust-bench with --features profile");
        return ExitCode::from(2);
    }

    let parse = bench_parse(&config.input, config.warmup_iters, config.iters);
    let eval = bench_eval(
        &config.input,
        config.mode,
        &config.program_args,
        config.executable_path.as_deref(),
        config.warmup_iters,
        config.iters,
        config.step_limit,
    );
    let bytes = config.input.len();

    println!("input: {}", config.name);
    println!("mode: {}", config.mode.as_str());
    println!("bytes: {bytes}");
    println!("node_size_bytes: {}", size_of::<Node>());
    println!("cell_size_bytes: {}", cell_size_bytes());
    println!("node_id_size_bytes: {}", size_of::<NodeId>());
    println!("prim_size_bytes: {}", size_of::<Prim>());
    println!("iters: {}", config.iters);
    println!("warmup_iters: {}", config.warmup_iters);
    match config.step_limit {
        Some(limit) => println!("step_limit: {limit}"),
        None => println!("step_limit: none"),
    }
    println!("parse_total_ms: {:.3}", millis(parse.elapsed));
    println!(
        "parse_ns_per_iter: {:.1}",
        nanos_per_iter(parse.elapsed, config.iters)
    );
    println!(
        "parse_mib_per_s: {:.1}",
        mib_per_s(bytes, config.iters, parse.elapsed)
    );
    println!("parse_reduce_render_total_ms: {:.3}", millis(eval.elapsed));
    println!(
        "parse_reduce_render_ns_per_iter: {:.1}",
        nanos_per_iter(eval.elapsed, config.iters)
    );
    println!(
        "whnf_steps_per_iter: {:.1}",
        eval.steps as f64 / config.iters as f64
    );
    println!(
        "whnf_steps_per_s: {:.1}",
        eval.steps as f64 / eval.elapsed.as_secs_f64()
    );
    println!("step_limited_iters: {}", eval.step_limited_iters);
    println!("serialize_sink: {}", eval.serialize_sink);
    println!("gc_collections: {}", eval.gc.collections);
    println!("gc_freed_nodes_total: {}", eval.gc.freed_nodes_total);
    println!("gc_last_live_nodes: {}", eval.gc.last_live_nodes);
    println!("gc_last_free_nodes: {}", eval.gc.last_free_nodes);
    println!("gc_high_water_nodes: {}", eval.gc.high_water_nodes);
    println!("gc_current_nodes: {}", eval.gc.current_nodes);
    println!("gc_current_free_nodes: {}", eval.gc.current_free_nodes);
    println!(
        "gc_last_pause_ms: {:.3}",
        nanos_millis(eval.gc.last_pause_nanos)
    );
    println!(
        "gc_total_pause_ms: {:.3}",
        nanos_millis(eval.gc.total_pause_nanos)
    );
    #[cfg(feature = "gc-phase-profile")]
    {
        println!(
            "gc_last_mark_ms: {:.3}",
            nanos_millis(eval.gc.last_mark_nanos)
        );
        println!(
            "gc_total_mark_ms: {:.3}",
            nanos_millis(eval.gc.total_mark_nanos)
        );
        println!(
            "gc_last_sweep_ms: {:.3}",
            nanos_millis(eval.gc.last_sweep_nanos)
        );
        println!(
            "gc_total_sweep_ms: {:.3}",
            nanos_millis(eval.gc.total_sweep_nanos)
        );
    }
    println!(
        "gc_last_allocations_since_collect: {}",
        eval.gc.last_allocations_since_collect
    );
    println!(
        "gc_current_allocations_since_collect: {}",
        eval.gc.current_allocations_since_collect
    );
    print_gc_events("gc_events", &eval.gc);

    if config.profile {
        #[cfg(feature = "profile")]
        {
            let profile = profile_eval(
                &config.input,
                config.mode,
                &config.program_args,
                config.executable_path.as_deref(),
                config.step_limit,
            );
            print_profile(&profile, config.profile_top);
        }
    }

    if let Some(c_mhsbench) = &config.c_mhsbench {
        match bench_c_mhsbench(
            &config.input,
            c_mhsbench,
            config.c_mhsbench_mode,
            &config.program_args,
            config.warmup_iters,
            config.iters,
        ) {
            Ok(c) => {
                println!("c_mhsbench: {c_mhsbench}");
                println!("c_mhsbench_mode: {}", c.mode.as_str());
                println!("c_parse_eval_serialize_ns_per_iter: {:.1}", c.ns_per_iter);
                println!("c_bench_sink: {}", c.sink);
            }
            Err(err) => {
                eprintln!("C mhsbench benchmark failed: {err}");
                return ExitCode::from(1);
            }
        }
    }

    ExitCode::SUCCESS
}

fn parse_args(args: impl Iterator<Item = String>) -> Result<Config, String> {
    let mut iters = DEFAULT_ITERS;
    let mut warmup_iters = DEFAULT_WARMUP_ITERS;
    let mut mode = BenchMode::Whnf;
    let mut input = None;
    let mut name = None;
    let mut program_args = vec![b"mhsbench".to_vec()];
    let mut executable_path = None;
    let mut c_mhsbench = None;
    let mut c_mhsbench_mode = BenchMode::Whnf;
    let mut profile = false;
    let mut profile_top = 25usize;
    let mut step_limit = None;
    let mut args = args.peekable();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--" => {
                let raw_args: Vec<String> = args.collect();
                if raw_args.is_empty() {
                    return Err("-- requires at least argv[0]".to_owned());
                }
                executable_path = raw_args.first().map(|arg| canonical_program_path(arg));
                program_args = raw_args.into_iter().map(String::into_bytes).collect();
                break;
            }
            "--iters" => {
                let value = args.next().ok_or("--iters requires a value")?;
                iters = value
                    .parse()
                    .map_err(|_| format!("invalid --iters value: {value}"))?;
                if iters == 0 {
                    return Err("--iters must be greater than zero".to_owned());
                }
            }
            "--warmup-iters" => {
                let value = args.next().ok_or("--warmup-iters requires a value")?;
                warmup_iters = value
                    .parse()
                    .map_err(|_| format!("invalid --warmup-iters value: {value}"))?;
            }
            "--mode" => {
                let value = args.next().ok_or("--mode requires a value")?;
                mode = BenchMode::parse(&value)?;
            }
            "--input" => {
                let file = args.next().ok_or("--input requires a file")?;
                let bytes = fs::read(&file).map_err(|err| format!("{file}: {err}"))?;
                input = Some(bytes);
                name = Some(file);
            }
            "--scenario" => {
                let scenario = args.next().ok_or("--scenario requires a value")?;
                input = Some(make_scenario(&scenario)?);
                name = Some(scenario);
            }
            "--c-mhsbench" => {
                c_mhsbench = Some(args.next().ok_or("--c-mhsbench requires a path")?);
            }
            "--c-mhsbench-mode" => {
                let mode = args.next().ok_or("--c-mhsbench-mode requires a value")?;
                c_mhsbench_mode = BenchMode::parse(&mode)?;
            }
            "--profile" => {
                profile = true;
            }
            "--profile-top" => {
                let value = args.next().ok_or("--profile-top requires a value")?;
                profile_top = value
                    .parse()
                    .map_err(|_| format!("invalid --profile-top value: {value}"))?;
                if profile_top == 0 {
                    return Err("--profile-top must be greater than zero".to_owned());
                }
            }
            "--step-limit" => {
                let value = args.next().ok_or("--step-limit requires a value")?;
                let limit = value
                    .parse()
                    .map_err(|_| format!("invalid --step-limit value: {value}"))?;
                if limit == 0 {
                    return Err("--step-limit must be greater than zero".to_owned());
                }
                step_limit = Some(limit);
            }
            "-h" | "--help" => return Err(String::new()),
            _ => return Err(format!("unknown argument: {arg}")),
        }
    }

    let (input, name) = match (input, name) {
        (Some(input), Some(name)) => (input, name),
        (None, None) => (
            make_scenario(DEFAULT_SCENARIO)?,
            DEFAULT_SCENARIO.to_owned(),
        ),
        _ => unreachable!(),
    };

    Ok(Config {
        input,
        name,
        mode,
        iters,
        warmup_iters,
        program_args,
        executable_path,
        c_mhsbench,
        c_mhsbench_mode,
        profile,
        profile_top,
        step_limit,
    })
}

fn canonical_program_path(arg: &str) -> Vec<u8> {
    fs::canonicalize(arg)
        .map(|path| path.to_string_lossy().into_owned().into_bytes())
        .unwrap_or_else(|_| arg.as_bytes().to_vec())
}
