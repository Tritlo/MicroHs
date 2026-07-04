use std::env;
use std::fs;
use std::hint::black_box;
use std::mem::size_of;
use std::process::Command;
use std::process::ExitCode;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use microhs_runtime::{
    EvalError, EvalProfile, GcStats, Node, NodeId, Prim, Program, parse_program,
};

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
    c_mhseval: Option<String>,
    c_mhsbench: Option<String>,
    c_mhsbench_mode: BenchMode,
    profile: bool,
    profile_top: usize,
    step_limit: Option<usize>,
}

#[derive(Clone, Copy)]
enum BenchMode {
    Whnf,
    Main,
}

impl BenchMode {
    fn parse(text: &str) -> Result<Self, String> {
        match text {
            "whnf" => Ok(Self::Whnf),
            "main" => Ok(Self::Main),
            _ => Err(format!("invalid benchmark mode: {text}")),
        }
    }

    fn as_str(self) -> &'static str {
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
                                  [--c-mhseval PATH] [--c-mhsbench PATH] [--c-mhsbench-mode whnf|main]\n\
                                  [--profile] [--profile-top N] [--step-limit N]\n\
                                  [-- PROGRAM ARGS...]\n\
         default: --scenario {DEFAULT_SCENARIO} --iters {DEFAULT_ITERS}"
    );
}

fn main() -> ExitCode {
    let config = match parse_args(env::args().skip(1)) {
        Ok(config) => config,
        Err(message) => {
            eprintln!("{message}");
            usage();
            return ExitCode::from(2);
        }
    };

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
        println!("gc_red_i_opportunities: {}", eval.gc.red_i_opportunities);
        println!("gc_red_k_opportunities: {}", eval.gc.red_k_opportunities);
        println!("gc_red_a_opportunities: {}", eval.gc.red_a_opportunities);
        println!("gc_red_bi_opportunities: {}", eval.gc.red_bi_opportunities);
        println!(
            "gc_red_bxi_opportunities: {}",
            eval.gc.red_bxi_opportunities
        );
        println!(
            "gc_red_ccbi_opportunities: {}",
            eval.gc.red_ccbi_opportunities
        );
        println!("gc_red_cc_opportunities: {}", eval.gc.red_cc_opportunities);
        println!(
            "gc_red_cci_opportunities: {}",
            eval.gc.red_cci_opportunities
        );
        println!(
            "gc_red_ccbbcp_opportunities: {}",
            eval.gc.red_ccbbcp_opportunities
        );
        println!(
            "gc_red_flip_opportunities: {}",
            eval.gc.red_flip_opportunities
        );
        println!(
            "gc_young_profile_last_slots: {}",
            eval.gc.young_profile_last_slots
        );
        println!(
            "gc_young_profile_last_live: {}",
            eval.gc.young_profile_last_live
        );
        println!(
            "gc_young_profile_last_dead: {}",
            eval.gc.young_profile_last_dead
        );
        println!(
            "gc_young_profile_last_old_to_young_sources: {}",
            eval.gc.young_profile_last_old_to_young_sources
        );
        println!(
            "gc_young_profile_last_old_to_young_edges: {}",
            eval.gc.young_profile_last_old_to_young_edges
        );
        println!(
            "gc_young_profile_total_slots: {}",
            eval.gc.young_profile_total_slots
        );
        println!(
            "gc_young_profile_total_live: {}",
            eval.gc.young_profile_total_live
        );
        println!(
            "gc_young_profile_total_dead: {}",
            eval.gc.young_profile_total_dead
        );
        println!(
            "gc_young_profile_total_old_to_young_sources: {}",
            eval.gc.young_profile_total_old_to_young_sources
        );
        println!(
            "gc_young_profile_total_old_to_young_edges: {}",
            eval.gc.young_profile_total_old_to_young_edges
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
        let profile = profile_eval(
            &config.input,
            config.mode,
            &config.program_args,
            config.executable_path.as_deref(),
            config.step_limit,
        );
        print_profile(&profile, config.profile_top);
    }

    if let Some(c_mhseval) = &config.c_mhseval {
        match bench_c_mhseval(&config.input, c_mhseval, config.iters) {
            Ok(c) => {
                println!("c_mhseval: {c_mhseval}");
                println!("c_parse_serialize_total_ms: {:.3}", millis(c.elapsed));
                println!(
                    "c_parse_serialize_ns_per_iter: {:.1}",
                    nanos_per_iter(c.elapsed, config.iters)
                );
            }
            Err(err) => {
                eprintln!("C mhseval benchmark failed: {err}");
                return ExitCode::from(1);
            }
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
    let mut c_mhseval = None;
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
            "--c-mhseval" => {
                c_mhseval = Some(args.next().ok_or("--c-mhseval requires a path")?);
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
        c_mhseval,
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

fn make_scenario(scenario: &str) -> Result<Vec<u8>, String> {
    if let Some(size) = scenario.strip_prefix("identity-chain:") {
        let size = parse_scenario_size("identity-chain", size)?;
        let mut out = String::from("v8.4\n0\nI");
        for _ in 1..size {
            out.push_str(" I @");
        }
        out.push_str(" #1 @ }\n");
        return Ok(out.into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("arith-chain:") {
        let size = parse_scenario_size("arith-chain", size)?;
        let mut expr = String::from("#0");
        for _ in 0..size {
            expr = format!("+ {expr} @ #1 @");
        }
        return Ok(format!("v8.4\n0\n{expr} }}\n").into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("int64-chain:") {
        let size = parse_scenario_size("int64-chain", size)?;
        let mut expr = String::from("##0");
        for _ in 0..size {
            expr = format!("I+ {expr} @ ##1 @");
        }
        return Ok(format!("v8.4\n0\n{expr} }}\n").into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("float64-chain:") {
        let size = parse_scenario_size("float64-chain", size)?;
        let mut expr = String::from("&0");
        for _ in 0..size {
            expr = format!("d+ {expr} @ &1.25 @");
        }
        return Ok(format!("v8.4\n0\n{expr} }}\n").into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("float32-chain:") {
        let size = parse_scenario_size("float32-chain", size)?;
        let mut expr = String::from("&&0");
        for _ in 0..size {
            expr = format!("f+ {expr} @ &&1.25 @");
        }
        return Ok(format!("v8.4\n0\n{expr} }}\n").into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("bytes-chain:") {
        let size = parse_scenario_size("bytes-chain", size)?;
        let mut expr = String::from("\"\"");
        for idx in 0..size {
            let byte = (b'a' + (idx % 26) as u8) as char;
            expr = format!("bs++ {expr} @ \"{byte}\" @");
        }
        return Ok(format!("v8.4\n0\n{expr} }}\n").into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("foreignptr-slice:") {
        let bytes = ascii_payload("foreignptr-slice", size)?;
        let len = bytes.len() - 1;
        return Ok(
            format!("v8.4\n0\nfp2bs fp+ bs2fp \"{bytes}\" @ @ #1 @ @ #{len} @ }}\n").into_bytes(),
        );
    }
    if let Some(size) = scenario.strip_prefix("cstring-pack:") {
        let bytes = ascii_payload("cstring-pack", size)?;
        let len = bytes.len();
        return Ok(format!(
            "v8.4\n0\nIO.performIO packCStringLen fp2p bs2fp \"{bytes}\" @ @ @ #{len} @ @ }}\n"
        )
        .into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("unpack-chain:") {
        let bytes = ascii_payload("unpack-chain", size)?;
        return Ok(format!("v8.4\n0\nbsunpack \"{bytes}\" @ #0 @ K @ }}\n").into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("fromutf8-chain:") {
        let bytes = ascii_payload("fromutf8-chain", size)?;
        return Ok(format!("v8.4\n0\nfromUTF8 \"{bytes}\" @ #0 @ K @ }}\n").into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("array-chain:") {
        let size = parse_scenario_size("array-chain", size)?;
        let last = size - 1;
        let mut items = String::new();
        for _ in 0..size {
            items.push_str("#0 ");
        }
        return Ok(format!(
            "v8.4\n1\nIO.performIO IO.>> A.write {items}[{size}] :0 @ #{last} @ #42 @ @ A.read _0 @ #{last} @ @ @ }}\n"
        )
        .into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("io-chain:") {
        let size = parse_scenario_size("io-chain", size)?;
        let mut expr = String::from("IO.return #0 @");
        for idx in 1..size {
            expr = format!("IO.>> {expr} @ IO.return #{idx} @ @");
        }
        return Ok(format!("v8.4\n0\nIO.performIO {expr} @ }}\n").into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("io-array-chain:") {
        let size = parse_scenario_size("io-array-chain", size)?;
        let last = size - 1;
        let mut items = String::new();
        for _ in 0..size {
            items.push_str("#0 ");
        }
        return Ok(format!(
            "v8.4\n1\nIO.performIO IO.>> A.write {items}[{size}] :0 @ #{last} @ #42 @ @ A.read _0 @ #{last} @ @ @ }}\n"
        )
        .into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("io-bytes-chain:") {
        let bytes = ascii_payload("io-bytes-chain", size)?;
        let last = bytes.len() - 1;
        return Ok(format!(
            "v8.4\n1\nIO.performIO IO.>> bswrite \"{bytes}\" :0 @ #{last} @ #42 @ @ bsread _0 @ #{last} @ @ @ }}\n"
        )
        .into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("io-control-chain:") {
        let size = parse_scenario_size("io-control-chain", size)?;
        let mut expr = String::from("IO.getmaskingstate");
        for idx in 0..size {
            expr = format!("IO.>> IO.setmaskingstate #{} @ @ {expr} @", idx % 3);
        }
        return Ok(format!("v8.4\n0\nIO.performIO {expr} @ }}\n").into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("performio-apply-chain:") {
        let size = parse_scenario_size("performio-apply-chain", size)?;
        let mut expr = String::from("#0");
        for _ in 0..size {
            expr = format!("IO.performIO IO.return I @ @ {expr} @");
        }
        return Ok(format!("v8.4\n0\n{expr} }}\n").into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("argref-chain:") {
        let size = parse_scenario_size("argref-chain", size)?;
        let mut expr = String::from("IO.getArgRef");
        for _ in 1..size {
            expr = format!("IO.>> {expr} @ IO.getArgRef @");
        }
        return Ok(format!("v8.4\n0\nIO.performIO {expr} @ }}\n").into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("stdio-chain:") {
        let size = parse_scenario_size("stdio-chain", size)?;
        let mut expr = String::from("#0");
        for _ in 0..size {
            expr = format!("seq toInt fp2p IO.stdout @ @ @ {expr} @");
        }
        return Ok(format!("v8.4\n0\n{expr} }}\n").into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("ffi-chain:") {
        let size = parse_scenario_size("ffi-chain", size)?;
        let mut expr = String::from("^islinux");
        for _ in 1..size {
            expr = format!("IO.>> {expr} @ ^islinux @");
        }
        return Ok(format!("v8.4\n0\nIO.performIO {expr} @ }}\n").into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("ffi-math-chain:") {
        let size = parse_scenario_size("ffi-math-chain", size)?;
        let mut expr = String::from("^sqrt &9 @");
        for _ in 1..size {
            expr = format!("IO.>> {expr} @ ^sqrt &9 @ @");
        }
        return Ok(format!("v8.4\n0\nIO.performIO {expr} @ }}\n").into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("ffi-const-chain:") {
        let size = parse_scenario_size("ffi-const-chain", size)?;
        let mut expr = String::from("^sizeof_int");
        for _ in 1..size {
            expr = format!("IO.>> {expr} @ ^sizeof_int @");
        }
        return Ok(format!("v8.4\n0\nIO.performIO {expr} @ }}\n").into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("ffi-mem-chain:") {
        let size = parse_scenario_size("ffi-mem-chain", size)?;
        let action = "IO.lazyBind ^calloc #1 @ #1 @ @ ^peek_uint8 @";
        let mut expr = action.to_owned();
        for _ in 1..size {
            expr = format!("IO.>> {expr} @ {action} @");
        }
        return Ok(format!("v8.4\n0\nIO.performIO {expr} @ }}\n").into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("ffi-wide-mem-chain:") {
        let size = parse_scenario_size("ffi-wide-mem-chain", size)?;
        let action = "IO.lazyBind ^calloc #1 @ #8 @ @ S S K IO.>> @ @ S S K ^poke_uint64 @ @ I @ @ K ##123456789 @ @ @ @ S K ^peek_uint64 @ @ I @ @ @";
        let mut expr = action.to_owned();
        for _ in 1..size {
            expr = format!("IO.>> {expr} @ {action} @");
        }
        return Ok(format!("v8.4\n0\nIO.performIO {expr} @ }}\n").into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("ffi-word-mem-chain:") {
        let size = parse_scenario_size("ffi-word-mem-chain", size)?;
        let action = "IO.lazyBind ^calloc #1 @ #8 @ @ S S K IO.>> @ @ S S K ^pokeWord @ @ I @ @ K #123456789 @ @ @ @ S K ^peekWord @ @ I @ @ @";
        let mut expr = action.to_owned();
        for _ in 1..size {
            expr = format!("IO.>> {expr} @ {action} @");
        }
        return Ok(format!("v8.4\n0\nIO.performIO {expr} @ }}\n").into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("ffi-ptr-mem-chain:") {
        let size = parse_scenario_size("ffi-ptr-mem-chain", size)?;
        let ptr_action = "IO.lazyBind ^calloc #1 @ #8 @ @ S S K IO.>> @ @ S S K ^pokePtr @ @ I @ @ K toPtr #42 @ @ @ @ @ S K ^peekPtr @ @ I @ @ @";
        let convert = "S K IO.return @ @ S K toInt @ @ I @ @";
        let action = format!("IO.lazyBind {ptr_action} @ {convert} @");
        let mut expr = action.to_owned();
        for _ in 1..size {
            expr = format!("IO.>> {expr} @ {action} @");
        }
        return Ok(format!("v8.4\n0\nIO.performIO {expr} @ }}\n").into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("ffi-strcpy-chain:") {
        let size = parse_scenario_size("ffi-strcpy-chain", size)?;
        let mut source = String::from("fp2p bs2fp $17 microhs-rust-ffi");
        source.push('\0');
        source.push_str(" @ @");
        let action = format!(
            "IO.lazyBind ^calloc #1 @ #17 @ @ S S K IO.>> @ @ S S K ^strcpy @ @ I @ @ K {source} @ @ @ @ S K ^strlen @ @ I @ @ @"
        );
        let mut expr = action.clone();
        for _ in 1..size {
            expr = format!("IO.>> {expr} @ {action} @");
        }
        return Ok(format!("v8.4\n0\nIO.performIO {expr} @ }}\n").into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("md5-string-chain:") {
        let size = parse_scenario_size("md5-string-chain", size)?;
        let text = "The quick fox jumps over the lazy dog.";
        let mut source = format!("fp2p bs2fp ${} {text}", text.len() + 1);
        source.push('\0');
        source.push_str(" @ @");
        let action = format!(
            "IO.lazyBind ^calloc #1 @ #16 @ @ S S K IO.>> @ @ S S K ^md5String @ @ K {source} @ @ @ I @ @ @ S K ^peek_uint8 @ @ I @ @ @"
        );
        let mut expr = action.clone();
        for _ in 1..size {
            expr = format!("IO.>> {expr} @ {action} @");
        }
        return Ok(format!("v8.4\n0\nIO.performIO {expr} @ }}\n").into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("getenv-chain:") {
        let size = parse_scenario_size("getenv-chain", size)?;
        let mut key = String::from("fp2p bs2fp $5 PATH");
        key.push('\0');
        key.push_str(" @ @");
        let action = format!("IO.lazyBind ^getenv {key} @ @ ^strlen @");
        let mut expr = action.clone();
        for _ in 1..size {
            expr = format!("IO.>> {expr} @ {action} @");
        }
        return Ok(format!("v8.4\n0\nIO.performIO {expr} @ }}\n").into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("env-set-chain:") {
        let size = parse_scenario_size("env-set-chain", size)?;
        let mut key = String::from("fp2p bs2fp $23 MICROHS_RUST_ENV_BENCH");
        key.push('\0');
        key.push_str(" @ @");
        let mut value = String::from("fp2p bs2fp $4 xyz");
        value.push('\0');
        value.push_str(" @ @");
        let action = format!(
            "IO.>> ^setenv {key} @ {value} @ #1 @ @ IO.lazyBind ^getenv {key} @ @ ^strlen @ @"
        );
        let mut expr = action.clone();
        for _ in 1..size {
            expr = format!("IO.>> {expr} @ {action} @");
        }
        return Ok(format!("v8.4\n0\nIO.performIO {expr} @ }}\n").into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("errno-chain:") {
        let size = parse_scenario_size("errno-chain", size)?;
        let action = "IO.lazyBind ^&errno @ S S K IO.>> @ @ S S K ^poke_int @ @ I @ @ K #2 @ @ @ @ S K ^peek_int @ @ I @ @ @";
        let mut expr = action.to_owned();
        for _ in 1..size {
            expr = format!("IO.>> {expr} @ {action} @");
        }
        return Ok(format!("v8.4\n0\nIO.performIO {expr} @ }}\n").into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("getcwd-chain:") {
        let size = parse_scenario_size("getcwd-chain", size)?;
        let action = "IO.lazyBind ^calloc #1 @ #4096 @ @ S S K IO.>> @ @ S S K ^getcwd @ @ I @ @ K #4096 @ @ @ @ S K ^strlen @ @ I @ @ @";
        let mut expr = action.to_owned();
        for _ in 1..size {
            expr = format!("IO.>> {expr} @ {action} @");
        }
        return Ok(format!("v8.4\n0\nIO.performIO {expr} @ }}\n").into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("dir-read-chain:") {
        let size = parse_scenario_size("dir-read-chain", size)?;
        let mut path = String::from("fp2p bs2fp $2 .");
        path.push('\0');
        path.push_str(" @ @");
        let action = format!(
            "IO.lazyBind ^opendir {path} @ @ S S K IO.>> @ @ S S K IO.lazyBind @ @ S K ^readdir @ @ I @ @ @ K S S K IO.lazyBind @ @ S K ^c_d_name @ @ I @ @ @ K ^strlen @ @ @ @ @ @ S K ^closedir @ @ I @ @ @"
        );
        let mut expr = action.clone();
        for _ in 1..size {
            expr = format!("IO.>> {expr} @ {action} @");
        }
        return Ok(format!("v8.4\n0\nIO.performIO {expr} @ }}\n").into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("remove-missing-chain:") {
        let size = parse_scenario_size("remove-missing-chain", size)?;
        let mut path = String::from("fp2p bs2fp $28 microhs-rust-missing-remove");
        path.push('\0');
        path.push_str(" @ @");
        let action = format!("^remove {path} @");
        let mut expr = action.clone();
        for _ in 1..size {
            expr = format!("IO.>> {expr} @ {action} @");
        }
        return Ok(format!("v8.4\n0\nIO.performIO {expr} @ }}\n").into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("file-read-close-chain:") {
        let size = parse_scenario_size("file-read-close-chain", size)?;
        let mut path = String::from("fp2p bs2fp $11 Cargo.toml");
        path.push('\0');
        path.push_str(" @ @");
        let mut mode = String::from("fp2p bs2fp $3 rb");
        mode.push('\0');
        mode.push_str(" @ @");
        let open =
            format!("IO.lazyBind IO.lazyBind ^fopen {path} @ {mode} @ @ ^add_FILE @ @ ^add_utf8 @");
        let read_close = "S S K IO.>> @ @ ^getb @ @ ^closeb @";
        let action = format!("IO.lazyBind {open} @ {read_close} @");
        let mut expr = action.clone();
        for _ in 1..size {
            expr = format!("IO.>> {expr} @ {action} @");
        }
        return Ok(format!("v8.4\n0\nIO.performIO {expr} @ }}\n").into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("utf8-bfile-read-chain:") {
        let size = parse_scenario_size("utf8-bfile-read-chain", size)?;
        let mut open = b"IO.lazyBind ^openb_rd_mem fp2p bs2fp $3 ".to_vec();
        open.extend_from_slice(&[0xc0, 0x80, b'A']);
        open.extend_from_slice(b" @ @ @ #3 @ @ ^add_utf8 @");
        let mut action = b"IO.lazyBind ".to_vec();
        action.extend_from_slice(&open);
        action.extend_from_slice(b" @ ^getb @");
        let mut expr = action.clone();
        for _ in 1..size {
            let mut next = b"IO.>> ".to_vec();
            next.extend_from_slice(&expr);
            next.extend_from_slice(b" @ ");
            next.extend_from_slice(&action);
            next.extend_from_slice(b" @");
            expr = next;
        }
        let mut out = b"v8.4\n0\nIO.performIO ".to_vec();
        out.extend_from_slice(&expr);
        out.extend_from_slice(b" @ }\n");
        return Ok(out);
    }
    if let Some(size) = scenario.strip_prefix("crlf-bfile-read-chain:") {
        let size = parse_scenario_size("crlf-bfile-read-chain", size)?;
        let mut open = b"IO.lazyBind ^openb_rd_mem fp2p bs2fp $3 ".to_vec();
        open.extend_from_slice(b"\r\nA");
        open.extend_from_slice(b" @ @ @ #3 @ @ ^add_crlf @");
        let mut action = b"IO.lazyBind ".to_vec();
        action.extend_from_slice(&open);
        action.extend_from_slice(b" @ ^getb @");
        let mut expr = action.clone();
        for _ in 1..size {
            let mut next = b"IO.>> ".to_vec();
            next.extend_from_slice(&expr);
            next.extend_from_slice(b" @ ");
            next.extend_from_slice(&action);
            next.extend_from_slice(b" @");
            expr = next;
        }
        let mut out = b"v8.4\n0\nIO.performIO ".to_vec();
        out.extend_from_slice(&expr);
        out.extend_from_slice(b" @ }\n");
        return Ok(out);
    }
    if let Some(size) = scenario.strip_prefix("base64-bfile-read-chain:") {
        let size = parse_scenario_size("base64-bfile-read-chain", size)?;
        let mut open = b"IO.lazyBind ^openb_rd_mem fp2p bs2fp $6 ".to_vec();
        open.extend_from_slice(b"Q Q==\n");
        open.extend_from_slice(b" @ @ @ #6 @ @ ^add_base64_decoder @");
        let mut action = b"IO.lazyBind ".to_vec();
        action.extend_from_slice(&open);
        action.extend_from_slice(b" @ ^getb @");
        let mut expr = action.clone();
        for _ in 1..size {
            let mut next = b"IO.>> ".to_vec();
            next.extend_from_slice(&expr);
            next.extend_from_slice(b" @ ");
            next.extend_from_slice(&action);
            next.extend_from_slice(b" @");
            expr = next;
        }
        let mut out = b"v8.4\n0\nIO.performIO ".to_vec();
        out.extend_from_slice(&expr);
        out.extend_from_slice(b" @ }\n");
        return Ok(out);
    }
    if let Some(size) = scenario.strip_prefix("rle-bfile-read-chain:") {
        let size = parse_scenario_size("rle-bfile-read-chain", size)?;
        let mut open = b"IO.lazyBind ^openb_rd_mem fp2p bs2fp $2 ".to_vec();
        open.extend_from_slice(&[0x81, 0x01]);
        open.extend_from_slice(b" @ @ @ #2 @ @ ^add_rle_decompressor @");
        let mut action = b"IO.lazyBind ".to_vec();
        action.extend_from_slice(&open);
        action.extend_from_slice(b" @ ^getb @");
        let mut expr = action.clone();
        for _ in 1..size {
            let mut next = b"IO.>> ".to_vec();
            next.extend_from_slice(&expr);
            next.extend_from_slice(b" @ ");
            next.extend_from_slice(&action);
            next.extend_from_slice(b" @");
            expr = next;
        }
        let mut out = b"v8.4\n0\nIO.performIO ".to_vec();
        out.extend_from_slice(&expr);
        out.extend_from_slice(b" @ }\n");
        return Ok(out);
    }
    if let Some(size) = scenario.strip_prefix("lz77-bfile-read-chain:") {
        let size = parse_scenario_size("lz77-bfile-read-chain", size)?;
        let mut open = b"IO.lazyBind ^openb_rd_mem fp2p bs2fp $9 ".to_vec();
        open.extend_from_slice(b"LZ1");
        open.extend_from_slice(&[2, 0, 0, 0, 0, b'A']);
        open.extend_from_slice(b" @ @ @ #9 @ @ ^add_lz77_decompressor @");
        let mut action = b"IO.lazyBind ".to_vec();
        action.extend_from_slice(&open);
        action.extend_from_slice(b" @ ^getb @");
        let mut expr = action.clone();
        for _ in 1..size {
            let mut next = b"IO.>> ".to_vec();
            next.extend_from_slice(&expr);
            next.extend_from_slice(b" @ ");
            next.extend_from_slice(&action);
            next.extend_from_slice(b" @");
            expr = next;
        }
        let mut out = b"v8.4\n0\nIO.performIO ".to_vec();
        out.extend_from_slice(&expr);
        out.extend_from_slice(b" @ }\n");
        return Ok(out);
    }
    if let Some(size) = scenario.strip_prefix("bwt-bfile-read-chain:") {
        let size = parse_scenario_size("bwt-bfile-read-chain", size)?;
        let mut open = b"IO.lazyBind ^openb_rd_mem fp2p bs2fp $12 ".to_vec();
        open.extend_from_slice(b"BW1");
        open.extend_from_slice(&[1, 0, 0, 0, 0, 0, 0, 0, b'A']);
        open.extend_from_slice(b" @ @ @ #12 @ @ ^add_bwt_decompressor @");
        let mut action = b"IO.lazyBind ".to_vec();
        action.extend_from_slice(&open);
        action.extend_from_slice(b" @ ^getb @");
        let mut expr = action.clone();
        for _ in 1..size {
            let mut next = b"IO.>> ".to_vec();
            next.extend_from_slice(&expr);
            next.extend_from_slice(b" @ ");
            next.extend_from_slice(&action);
            next.extend_from_slice(b" @");
            expr = next;
        }
        let mut out = b"v8.4\n0\nIO.performIO ".to_vec();
        out.extend_from_slice(&expr);
        out.extend_from_slice(b" @ }\n");
        return Ok(out);
    }
    if let Some(size) = scenario.strip_prefix("lzma-bfile-read-chain:") {
        let size = parse_scenario_size("lzma-bfile-read-chain", size)?;
        let payload = lzma_envelope_for_bytes(b"A")?;
        let mut open =
            format!("IO.lazyBind ^openb_rd_mem fp2p bs2fp ${} ", payload.len()).into_bytes();
        open.extend_from_slice(&payload);
        open.extend_from_slice(
            format!(" @ @ @ #{} @ @ ^add_lzma_decompressor @", payload.len()).as_bytes(),
        );
        let mut action = b"IO.lazyBind ".to_vec();
        action.extend_from_slice(&open);
        action.extend_from_slice(b" @ ^getb @");
        let mut expr = action.clone();
        for _ in 1..size {
            let mut next = b"IO.>> ".to_vec();
            next.extend_from_slice(&expr);
            next.extend_from_slice(b" @ ");
            next.extend_from_slice(&action);
            next.extend_from_slice(b" @");
            expr = next;
        }
        let mut out = b"v8.4\n0\nIO.performIO ".to_vec();
        out.extend_from_slice(&expr);
        out.extend_from_slice(b" @ }\n");
        return Ok(out);
    }
    if let Some(size) = scenario.strip_prefix("buf-bfile-read-chain:") {
        let size = parse_scenario_size("buf-bfile-read-chain", size)?;
        let payload = "microhs-rust-buffered";
        let open = format!(
            "IO.lazyBind ^openb_rd_mem fp2p bs2fp \"{payload}\" @ @ @ #{} @ @ R #4 @ ^add_buf @ @",
            payload.len()
        );
        let action = format!("IO.lazyBind {open} @ ^getb @");
        let mut expr = action.clone();
        for _ in 1..size {
            expr = format!("IO.>> {expr} @ {action} @");
        }
        return Ok(format!("v8.4\n0\nIO.performIO {expr} @ }}\n").into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("bfile-read-chain:") {
        let size = parse_scenario_size("bfile-read-chain", size)?;
        let payload = "microhs-rust-bfile";
        let action = format!(
            "IO.lazyBind ^openb_rd_mem fp2p bs2fp \"{payload}\" @ @ @ #{} @ @ ^getb @",
            payload.len()
        );
        let mut expr = action.clone();
        for _ in 1..size {
            expr = format!("IO.>> {expr} @ {action} @");
        }
        return Ok(format!("v8.4\n0\nIO.performIO {expr} @ }}\n").into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("ptr-chain:") {
        let size = parse_scenario_size("ptr-chain", size)?;
        let mut expr = String::from("#0");
        for _ in 0..size {
            expr = format!("toInt toPtr {expr} @ @");
        }
        return Ok(format!("v8.4\n0\n{expr} }}\n").into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("mvar-chain:") {
        let size = parse_scenario_size("mvar-chain", size)?;
        let mut expr = String::from("#0");
        for idx in 0..size {
            expr = format!("O #{idx} @ {expr} @");
        }
        return Ok(format!(
            "v8.4\n0\nIO.performIO IO.lazyBind IO.newmvar @ IO.trytakemvar @ @ {expr} @ I @ }}\n"
        )
        .into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("rnf-chain:") {
        let size = parse_scenario_size("rnf-chain", size)?;
        let mut expr = String::from("#0");
        for idx in 0..size {
            expr = format!("O #{idx} @ {expr} @");
        }
        return Ok(format!("v8.4\n0\nrnf #0 @ {expr} @ }}\n").into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("stableptr-chain:") {
        let size = parse_scenario_size("stableptr-chain", size)?;
        let mut expr = String::from("#0");
        for idx in 0..size {
            expr = format!("O #{idx} @ {expr} @");
        }
        return Ok(format!("v8.4\n0\nIO.performIO SPnew {expr} @ @ }}\n").into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("weak-chain:") {
        let size = parse_scenario_size("weak-chain", size)?;
        let mut expr = String::from("#0");
        for idx in 0..size {
            expr = format!("O #{idx} @ {expr} @");
        }
        return Ok(format!(
            "v8.4\n0\nIO.performIO IO.lazyBind Wknew #0 @ {expr} @ @ Wkderef @ @ #0 @ I @ }}\n"
        )
        .into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("zoo-chain:") {
        let size = parse_scenario_size("zoo-chain", size)?;
        let mut expr = String::from("#1");
        for idx in 0..size {
            expr = match idx % 11 {
                0 => format!("S' K @ K @ K @ {expr} @ #0 @"),
                1 => format!("B' K @ {expr} @ K @ #0 @"),
                2 => format!("Z K @ {expr} @ #0 @ #1 @"),
                3 => format!("J {expr} @ #0 @ I @"),
                4 => format!("L {expr} @ I @ #0 @"),
                5 => format!("KK #0 @ {expr} @ #1 @"),
                6 => format!("KA #0 @ #1 @ {expr} @"),
                7 => format!("C' A @ K @ {expr} @ #0 @"),
                8 => format!("R #0 @ K @ {expr} @"),
                9 => format!("O {expr} @ #0 @ #1 @ K @"),
                _ => format!("C'B K @ K @ {expr} @ #0 @"),
            };
        }
        return Ok(format!("v8.4\n0\n{expr} }}\n").into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("data-chain:") {
        let size = parse_scenario_size("data-chain", size)?;
        let mut expr = String::from("#1");
        for idx in 0..size {
            expr = if idx % 3 == 0 {
                format!("TAG{} {expr} @ A @", idx % 33)
            } else if idx % 3 == 1 {
                format!("T3 {expr} @ #0 @ #1 @ K3 @ #0 @")
            } else {
                format!("T4 {expr} @ #0 @ #1 @ #2 @ K4 @ #0 @")
            };
        }
        return Ok(format!("v8.4\n0\n{expr} }}\n").into_bytes());
    }
    Err(format!("unsupported scenario: {scenario}"))
}

fn parse_scenario_size(name: &str, size: &str) -> Result<usize, String> {
    let size: usize = size
        .parse()
        .map_err(|_| format!("invalid {name} size: {size}"))?;
    if size == 0 {
        return Err(format!("{name} size must be greater than zero"));
    }
    Ok(size)
}

fn ascii_payload(name: &str, size: &str) -> Result<String, String> {
    let size = parse_scenario_size(name, size)?;
    let mut bytes = String::with_capacity(size);
    for idx in 0..size {
        bytes.push((b'a' + (idx % 26) as u8) as char);
    }
    Ok(bytes)
}

fn lzma_envelope_for_bytes(input: &[u8]) -> Result<Vec<u8>, String> {
    let props = lzma_sdk_rs::LzmaProps::for_level(5, u32::MAX);
    let raw = lzma_sdk_rs::encode(input, &props);
    let mut payload = Vec::with_capacity(20 + raw.len());
    payload.extend_from_slice(b"LZ2");
    let compressed_len = 13usize
        .checked_add(raw.len())
        .ok_or("lzma payload length overflow")?;
    let compressed_len =
        u32::try_from(compressed_len).map_err(|_| "lzma payload too large for envelope")?;
    payload.extend_from_slice(&compressed_len.to_le_bytes());
    payload.extend_from_slice(&lzma_sdk_rs::decoder_props(&props));
    let input_len = u64::try_from(input.len()).map_err(|_| "lzma input length overflow")?;
    payload.extend_from_slice(&input_len.to_le_bytes());
    payload.extend_from_slice(&raw);
    Ok(payload)
}

struct ParseBench {
    elapsed: Duration,
}

fn bench_parse(input: &[u8], warmup_iters: usize, iters: usize) -> ParseBench {
    for _ in 0..warmup_iters {
        black_box(parse_once(input));
    }

    let started = Instant::now();
    for _ in 0..iters {
        black_box(parse_once(input));
    }
    ParseBench {
        elapsed: started.elapsed(),
    }
}

fn parse_once(input: &[u8]) -> usize {
    let program = parse_program(black_box(input)).expect("parse benchmark input");
    program.node_count()
}

struct EvalBench {
    elapsed: Duration,
    steps: usize,
    serialize_sink: usize,
    step_limited_iters: usize,
    gc: GcStats,
}

struct ProfileBench {
    elapsed: Duration,
    steps: usize,
    serialize_sink: usize,
    step_limited: bool,
    nodes_before: usize,
    nodes_after: usize,
    gc: GcStats,
    profile: EvalProfile,
}

fn bench_eval(
    input: &[u8],
    mode: BenchMode,
    program_args: &[Vec<u8>],
    executable_path: Option<&[u8]>,
    warmup_iters: usize,
    iters: usize,
    step_limit: Option<usize>,
) -> EvalBench {
    for _ in 0..warmup_iters {
        black_box(eval_once(
            input,
            mode,
            program_args,
            executable_path,
            step_limit,
        ));
    }

    let started = Instant::now();
    let mut steps = 0;
    let mut serialize_sink = 0usize;
    let mut step_limited_iters = 0usize;
    let mut gc = GcStats::default();
    for _ in 0..iters {
        let run = eval_once(input, mode, program_args, executable_path, step_limit);
        steps += run.steps;
        if run.step_limited {
            step_limited_iters += 1;
        }
        let sink = run.serialize_sink;
        serialize_sink = serialize_sink.wrapping_add(sink);
        gc.collections = gc.collections.saturating_add(run.gc.collections);
        gc.freed_nodes_total = gc
            .freed_nodes_total
            .saturating_add(run.gc.freed_nodes_total);
        gc.last_live_nodes = run.gc.last_live_nodes;
        gc.last_free_nodes = run.gc.last_free_nodes;
        gc.high_water_nodes = gc.high_water_nodes.max(run.gc.high_water_nodes);
        gc.current_nodes = run.gc.current_nodes;
        gc.current_free_nodes = run.gc.current_free_nodes;
        gc.last_pause_nanos = run.gc.last_pause_nanos;
        gc.total_pause_nanos = gc
            .total_pause_nanos
            .saturating_add(run.gc.total_pause_nanos);
        #[cfg(feature = "gc-phase-profile")]
        {
            gc.last_mark_nanos = run.gc.last_mark_nanos;
            gc.total_mark_nanos = gc.total_mark_nanos.saturating_add(run.gc.total_mark_nanos);
            gc.last_sweep_nanos = run.gc.last_sweep_nanos;
            gc.total_sweep_nanos = gc
                .total_sweep_nanos
                .saturating_add(run.gc.total_sweep_nanos);
            gc.young_profile_last_slots = run.gc.young_profile_last_slots;
            gc.young_profile_last_live = run.gc.young_profile_last_live;
            gc.young_profile_last_dead = run.gc.young_profile_last_dead;
            gc.young_profile_last_old_to_young_sources =
                run.gc.young_profile_last_old_to_young_sources;
            gc.young_profile_last_old_to_young_edges = run.gc.young_profile_last_old_to_young_edges;
            gc.young_profile_total_slots = gc
                .young_profile_total_slots
                .saturating_add(run.gc.young_profile_total_slots);
            gc.young_profile_total_live = gc
                .young_profile_total_live
                .saturating_add(run.gc.young_profile_total_live);
            gc.young_profile_total_dead = gc
                .young_profile_total_dead
                .saturating_add(run.gc.young_profile_total_dead);
            gc.young_profile_total_old_to_young_sources = gc
                .young_profile_total_old_to_young_sources
                .saturating_add(run.gc.young_profile_total_old_to_young_sources);
            gc.young_profile_total_old_to_young_edges = gc
                .young_profile_total_old_to_young_edges
                .saturating_add(run.gc.young_profile_total_old_to_young_edges);
        }
        gc.last_allocations_since_collect = run.gc.last_allocations_since_collect;
        gc.current_allocations_since_collect = run.gc.current_allocations_since_collect;
        gc.events.extend(run.gc.events);
    }
    black_box(serialize_sink);
    EvalBench {
        elapsed: started.elapsed(),
        steps,
        serialize_sink,
        step_limited_iters,
        gc,
    }
}

struct RunOnce {
    steps: usize,
    serialize_sink: usize,
    step_limited: bool,
    gc: GcStats,
}

fn eval_once(
    input: &[u8],
    mode: BenchMode,
    program_args: &[Vec<u8>],
    executable_path: Option<&[u8]>,
    step_limit: Option<usize>,
) -> RunOnce {
    let mut program = parse_program(black_box(input)).expect("reduce benchmark input");
    program.set_program_args(program_args.to_vec());
    program.set_executable_path(executable_path.map(Vec::from));
    let limit = step_limit.unwrap_or(usize::MAX);
    let mut run = match mode {
        BenchMode::Whnf => {
            let reductions = program.reduction_count();
            match program.reduce_whnf(limit) {
                Ok((root, steps)) => {
                    let serialized = program
                        .serialize_program(root)
                        .expect("serialize benchmark result");
                    let sink = bytes_sink(&serialized);
                    black_box(&serialized);
                    RunOnce {
                        steps,
                        serialize_sink: sink,
                        step_limited: false,
                        gc: GcStats::default(),
                    }
                }
                Err(EvalError::StepLimit { .. }) => RunOnce {
                    steps: program.reduction_count().saturating_sub(reductions),
                    serialize_sink: main_input_sink(input),
                    step_limited: true,
                    gc: GcStats::default(),
                },
                Err(err) => panic!("reduce benchmark input: {err}"),
            }
        }
        BenchMode::Main => reduce_main_or_panic(&mut program, limit, "run benchmark main", input),
    };
    run.gc = program.gc_stats();
    run
}

fn profile_eval(
    input: &[u8],
    mode: BenchMode,
    program_args: &[Vec<u8>],
    executable_path: Option<&[u8]>,
    step_limit: Option<usize>,
) -> ProfileBench {
    let started = Instant::now();
    let mut program = parse_program(black_box(input)).expect("profile benchmark input");
    program.set_program_args(program_args.to_vec());
    program.set_executable_path(executable_path.map(Vec::from));
    let nodes_before = program.node_count();
    program.enable_profile();
    let limit = step_limit.unwrap_or(usize::MAX);
    let run = match mode {
        BenchMode::Whnf => {
            let reductions = program.reduction_count();
            match program.reduce_whnf(limit) {
                Ok((root, steps)) => {
                    let serialized = program
                        .serialize_program(root)
                        .expect("profile serialize benchmark result");
                    let sink = bytes_sink(&serialized);
                    black_box(&serialized);
                    RunOnce {
                        steps,
                        serialize_sink: sink,
                        step_limited: false,
                        gc: GcStats::default(),
                    }
                }
                Err(EvalError::StepLimit { .. }) => RunOnce {
                    steps: program.reduction_count().saturating_sub(reductions),
                    serialize_sink: main_input_sink(input),
                    step_limited: true,
                    gc: GcStats::default(),
                },
                Err(err) => panic!("profile reduce benchmark input: {err}"),
            }
        }
        BenchMode::Main => {
            reduce_main_or_panic(&mut program, limit, "profile run benchmark main", input)
        }
    };
    let elapsed = started.elapsed();
    let nodes_after = program.node_count();
    let gc = program.gc_stats();
    let profile = program.take_profile().expect("profile enabled");
    ProfileBench {
        elapsed,
        steps: run.steps,
        serialize_sink: run.serialize_sink,
        step_limited: run.step_limited,
        nodes_before,
        nodes_after,
        gc,
        profile,
    }
}

fn print_profile(profile: &ProfileBench, top: usize) {
    println!("profile_total_ms: {:.3}", millis(profile.elapsed));
    println!("profile_steps: {}", profile.steps);
    println!("profile_sink: {}", profile.serialize_sink);
    println!("profile_step_limited: {}", profile.step_limited);
    println!("profile_nodes_before: {}", profile.nodes_before);
    println!("profile_nodes_after: {}", profile.nodes_after);
    println!("profile_gc_collections: {}", profile.gc.collections);
    println!(
        "profile_gc_freed_nodes_total: {}",
        profile.gc.freed_nodes_total
    );
    println!("profile_gc_last_live_nodes: {}", profile.gc.last_live_nodes);
    println!("profile_gc_last_free_nodes: {}", profile.gc.last_free_nodes);
    println!(
        "profile_gc_high_water_nodes: {}",
        profile.gc.high_water_nodes
    );
    println!("profile_gc_current_nodes: {}", profile.gc.current_nodes);
    println!(
        "profile_gc_current_free_nodes: {}",
        profile.gc.current_free_nodes
    );
    println!(
        "profile_gc_last_pause_ms: {:.3}",
        nanos_millis(profile.gc.last_pause_nanos)
    );
    println!(
        "profile_gc_total_pause_ms: {:.3}",
        nanos_millis(profile.gc.total_pause_nanos)
    );
    #[cfg(feature = "gc-phase-profile")]
    {
        println!(
            "profile_gc_last_mark_ms: {:.3}",
            nanos_millis(profile.gc.last_mark_nanos)
        );
        println!(
            "profile_gc_total_mark_ms: {:.3}",
            nanos_millis(profile.gc.total_mark_nanos)
        );
        println!(
            "profile_gc_last_sweep_ms: {:.3}",
            nanos_millis(profile.gc.last_sweep_nanos)
        );
        println!(
            "profile_gc_total_sweep_ms: {:.3}",
            nanos_millis(profile.gc.total_sweep_nanos)
        );
        println!(
            "profile_gc_red_i_opportunities: {}",
            profile.gc.red_i_opportunities
        );
        println!(
            "profile_gc_red_k_opportunities: {}",
            profile.gc.red_k_opportunities
        );
        println!(
            "profile_gc_red_a_opportunities: {}",
            profile.gc.red_a_opportunities
        );
        println!(
            "profile_gc_red_bi_opportunities: {}",
            profile.gc.red_bi_opportunities
        );
        println!(
            "profile_gc_red_bxi_opportunities: {}",
            profile.gc.red_bxi_opportunities
        );
        println!(
            "profile_gc_red_ccbi_opportunities: {}",
            profile.gc.red_ccbi_opportunities
        );
        println!(
            "profile_gc_red_cc_opportunities: {}",
            profile.gc.red_cc_opportunities
        );
        println!(
            "profile_gc_red_cci_opportunities: {}",
            profile.gc.red_cci_opportunities
        );
        println!(
            "profile_gc_red_ccbbcp_opportunities: {}",
            profile.gc.red_ccbbcp_opportunities
        );
        println!(
            "profile_gc_red_flip_opportunities: {}",
            profile.gc.red_flip_opportunities
        );
        println!(
            "profile_gc_young_profile_last_slots: {}",
            profile.gc.young_profile_last_slots
        );
        println!(
            "profile_gc_young_profile_last_live: {}",
            profile.gc.young_profile_last_live
        );
        println!(
            "profile_gc_young_profile_last_dead: {}",
            profile.gc.young_profile_last_dead
        );
        println!(
            "profile_gc_young_profile_last_old_to_young_sources: {}",
            profile.gc.young_profile_last_old_to_young_sources
        );
        println!(
            "profile_gc_young_profile_last_old_to_young_edges: {}",
            profile.gc.young_profile_last_old_to_young_edges
        );
        println!(
            "profile_gc_young_profile_total_slots: {}",
            profile.gc.young_profile_total_slots
        );
        println!(
            "profile_gc_young_profile_total_live: {}",
            profile.gc.young_profile_total_live
        );
        println!(
            "profile_gc_young_profile_total_dead: {}",
            profile.gc.young_profile_total_dead
        );
        println!(
            "profile_gc_young_profile_total_old_to_young_sources: {}",
            profile.gc.young_profile_total_old_to_young_sources
        );
        println!(
            "profile_gc_young_profile_total_old_to_young_edges: {}",
            profile.gc.young_profile_total_old_to_young_edges
        );
    }
    println!(
        "profile_gc_last_allocations_since_collect: {}",
        profile.gc.last_allocations_since_collect
    );
    println!(
        "profile_gc_current_allocations_since_collect: {}",
        profile.gc.current_allocations_since_collect
    );
    print_gc_events("profile_gc_events", &profile.gc);
    println!(
        "profile_node_growth: {}",
        profile.nodes_after.saturating_sub(profile.nodes_before)
    );
    println!("profile_step_attempts: {}", profile.profile.step_attempts);
    println!(
        "profile_successful_steps: {}",
        profile.profile.successful_steps
    );
    println!("profile_reductions: {}", profile.profile.reductions);
    println!(
        "profile_app_allocations: {}",
        profile.profile.app_allocations
    );
    println!(
        "profile_arg_materializations: {}",
        profile.profile.arg_materializations
    );
    println!(
        "profile_arg_materialized_nodes: {}",
        profile.profile.arg_materialized_nodes
    );
    println!("profile_spine_rewrites: {}", profile.profile.spine_rewrites);
    println!(
        "profile_spine_rewrite_extra_args: {}",
        profile.profile.spine_rewrite_extra_args
    );
    println!("profile_app_rewrites: {}", profile.profile.app_rewrites);
    println!(
        "profile_app_rewrite_extra_args: {}",
        profile.profile.app_rewrite_extra_args
    );
    println!("profile_stack_rewrites: {}", profile.profile.stack_rewrites);
    println!(
        "profile_stack_rewrite_apps: {}",
        profile.profile.stack_rewrite_apps
    );
    println!(
        "profile_stack_rewrite_indirections: {}",
        profile.profile.stack_rewrite_indirections
    );
    println!(
        "profile_stack_app_updates: {}",
        profile.profile.stack_app_updates
    );
    println!(
        "profile_stack_app_update_apps: {}",
        profile.profile.stack_app_update_apps
    );
    println!(
        "profile_stack_app_update_allocations: {}",
        profile.profile.stack_app_update_allocations
    );
    println!(
        "profile_stack_rethreads: {}",
        profile.profile.stack_rethreads
    );
    println!(
        "profile_stack_rethread_apps: {}",
        profile.profile.stack_rethread_apps
    );
    println!(
        "profile_stack_descent_pushes: {}",
        profile.profile.stack_descent_pushes
    );
    println!(
        "profile_stack_arg_reads: {}",
        profile.profile.stack_arg_reads
    );
    println!(
        "profile_stack_arg_batches: {}",
        profile.profile.stack_arg_batches
    );
    #[cfg(feature = "eval-phase-profile")]
    print_phase_profile(profile, top);
    let spine_arity_entries: usize = profile
        .profile
        .spine_arity
        .iter()
        .map(|(arity, count)| arity.saturating_mul(*count))
        .sum();
    println!("profile_spine_arity_entries: {spine_arity_entries}");
    println!(
        "profile_persistent_forces: {}",
        profile.profile.persistent_forces
    );
    println!(
        "profile_persistent_fallbacks: {}",
        profile.profile.persistent_fallbacks
    );
    println!(
        "profile_fallback_eval_loop_steps: {}",
        profile.profile.fallback_eval_loop_steps
    );
    println!(
        "profile_strict_redex_snapshots: {}",
        profile.profile.strict_redex_snapshots
    );
    println!(
        "profile_strict_redex_snapshot_apps: {}",
        profile.profile.strict_redex_snapshot_apps
    );
    println!(
        "profile_remaining_app_scans: {}",
        profile.profile.remaining_app_scans
    );
    println!(
        "profile_remaining_app_scan_apps: {}",
        profile.profile.remaining_app_scan_apps
    );
    println!(
        "profile_eval_frame_pushes: {}",
        profile.profile.eval_frame_pushes
    );
    println!(
        "profile_small_int_cache_hits: {}",
        profile.profile.small_int_cache_hits
    );
    println!(
        "profile_small_int_cache_misses: {}",
        profile.profile.small_int_cache_misses
    );
    println!(
        "profile_non_small_int_allocations: {}",
        profile.profile.non_small_int_allocations
    );
    println!("profile_heap_spines: {}", profile.profile.heap_spines);
    println!(
        "profile_max_spine_arity: {}",
        profile.profile.max_spine_arity
    );
    println!("profile_resolve_calls: {}", profile.profile.resolve_calls);
    println!(
        "profile_resolve_indirections: {}",
        profile.profile.resolve_indirections
    );
    println!(
        "profile_max_resolve_chain: {}",
        profile.profile.max_resolve_chain
    );
    println!("profile_top_head_attempts:");
    for (head, count) in profile.profile.top_head_attempts(top) {
        println!("  {head}: {count}");
    }
    println!("profile_top_head_reductions:");
    for (head, count) in profile.profile.top_head_reductions(top) {
        println!("  {head}: {count}");
    }
    println!("profile_spine_arity:");
    for (arity, count) in &profile.profile.spine_arity {
        println!("  {arity}: {count}");
    }
    println!("profile_resolve_chain:");
    for (depth, count) in &profile.profile.resolve_chain {
        println!("  {depth}: {count}");
    }
    println!("profile_shortcut_hits:");
    for (shortcut, count) in profile.profile.top_shortcut_hits(top) {
        println!("  {shortcut}: {count}");
    }
    println!("profile_primitive_dispatch_probes:");
    for (probe, count) in profile.profile.top_primitive_dispatch_probes(top) {
        println!("  {probe}: {count}");
    }
    println!("profile_primitive_dispatch_hits:");
    for (hit, count) in profile.profile.top_primitive_dispatch_hits(top) {
        println!("  {hit}: {count}");
    }
    println!("profile_node_allocations:");
    for (kind, count) in profile.profile.top_node_allocations(top) {
        println!("  {kind}: {count}");
    }
    println!("profile_app_allocation_sites:");
    for (site, count) in profile.profile.top_app_allocation_sites(top) {
        println!("  {site}: {count}");
    }
    println!("profile_eval_frame_push_kinds:");
    for (kind, count) in profile.profile.top_eval_frame_push_kinds(top) {
        println!("  {kind}: {count}");
    }
    println!("profile_stack_fallback_heads:");
    for (head, count) in profile.profile.top_stack_fallback_heads(top) {
        println!("  {head}: {count}");
    }
}

#[cfg(feature = "eval-phase-profile")]
fn print_phase_profile(profile: &ProfileBench, top: usize) {
    println!(
        "profile_stack_loop_iterations: {}",
        profile.profile.stack_loop_iterations
    );
    println!(
        "profile_stack_ready_checks: {}",
        profile.profile.stack_ready_checks
    );
    println!(
        "profile_stack_ready_successes: {}",
        profile.profile.stack_ready_successes
    );
    println!(
        "profile_stack_eval_step_calls: {}",
        profile.profile.stack_eval_step_calls
    );
    println!(
        "profile_stack_step_reduced: {}",
        profile.profile.stack_step_reduced
    );
    println!(
        "profile_stack_step_force: {}",
        profile.profile.stack_step_force
    );
    println!(
        "profile_stack_step_whnf: {}",
        profile.profile.stack_step_whnf
    );
    println!(
        "profile_stack_step_fallback: {}",
        profile.profile.stack_step_fallback
    );
    println!(
        "profile_stack_gc_check_ms: {:.3}",
        nanos_millis(profile.profile.stack_gc_check_nanos)
    );
    println!(
        "profile_stack_resolve_ms: {:.3}",
        nanos_millis(profile.profile.stack_resolve_nanos)
    );
    println!(
        "profile_stack_ready_frame_ms: {:.3}",
        nanos_millis(profile.profile.stack_ready_frame_nanos)
    );
    println!(
        "profile_stack_descent_ms: {:.3}",
        nanos_millis(profile.profile.stack_descent_nanos)
    );
    println!(
        "profile_stack_eval_step_ms: {:.3}",
        nanos_millis(profile.profile.stack_eval_step_nanos)
    );
    println!(
        "profile_stack_whnf_finish_ms: {:.3}",
        nanos_millis(profile.profile.stack_whnf_finish_nanos)
    );
    println!(
        "profile_stack_arg_read_ms: {:.3}",
        nanos_millis(profile.profile.stack_arg_read_nanos)
    );
    println!(
        "profile_stack_app_alloc_ms: {:.3}",
        nanos_millis(profile.profile.stack_app_alloc_nanos)
    );
    println!(
        "profile_app_alloc_reused: {}",
        profile.profile.app_alloc_reused
    );
    println!(
        "profile_app_alloc_fresh: {}",
        profile.profile.app_alloc_fresh
    );
    println!(
        "profile_app_alloc_free_pop_ms: {:.3}",
        nanos_millis(profile.profile.app_alloc_free_pop_nanos)
    );
    println!(
        "profile_app_alloc_reused_write_ms: {:.3}",
        nanos_millis(profile.profile.app_alloc_reused_write_nanos)
    );
    println!(
        "profile_app_alloc_fresh_push_ms: {:.3}",
        nanos_millis(profile.profile.app_alloc_fresh_push_nanos)
    );
    println!(
        "profile_stack_apply_rewrite_ms: {:.3}",
        nanos_millis(profile.profile.stack_apply_rewrite_nanos)
    );
    println!(
        "profile_stack_apply_app_ms: {:.3}",
        nanos_millis(profile.profile.stack_apply_app_nanos)
    );
    println!(
        "profile_stack_force_frame_ms: {:.3}",
        nanos_millis(profile.profile.stack_force_frame_nanos)
    );
    println!(
        "profile_stack_inner_descent_ms: {:.3}",
        nanos_millis(profile.profile.stack_inner_descent_nanos)
    );
    println!(
        "profile_profile_step_ms: {:.3}",
        nanos_millis(profile.profile.profile_step_nanos)
    );
    println!(
        "profile_profile_reduction_ms: {:.3}",
        nanos_millis(profile.profile.profile_reduction_nanos)
    );
    println!(
        "profile_profile_stack_head_time_ms: {:.3}",
        nanos_millis(profile.profile.profile_stack_head_time_nanos)
    );
    println!(
        "profile_profile_app_alloc_bookkeeping_ms: {:.3}",
        nanos_millis(profile.profile.profile_app_alloc_bookkeeping_nanos)
    );
    println!("profile_stack_eval_step_head_ms:");
    for (head, nanos) in profile.profile.top_stack_eval_step_head_times(top) {
        println!("  {head}: {:.3}", nanos_millis(nanos));
    }
    println!("profile_stack_arg_read_head_ms:");
    for (head, nanos) in profile.profile.top_stack_arg_read_head_times(top) {
        println!("  {head}: {:.3}", nanos_millis(nanos));
    }
    println!("profile_stack_app_alloc_site_ms:");
    for (site, nanos) in profile.profile.top_stack_app_alloc_site_times(top) {
        println!("  {site}: {:.3}", nanos_millis(nanos));
    }
    println!("profile_stack_apply_rewrite_head_ms:");
    for (head, nanos) in profile.profile.top_stack_apply_rewrite_head_times(top) {
        println!("  {head}: {:.3}", nanos_millis(nanos));
    }
    println!("profile_stack_apply_app_head_ms:");
    for (head, nanos) in profile.profile.top_stack_apply_app_head_times(top) {
        println!("  {head}: {:.3}", nanos_millis(nanos));
    }
    println!("profile_stack_force_frame_head_ms:");
    for (head, nanos) in profile.profile.top_stack_force_frame_head_times(top) {
        println!("  {head}: {:.3}", nanos_millis(nanos));
    }
    println!("profile_stack_inner_descent_head_ms:");
    for (head, nanos) in profile.profile.top_stack_inner_descent_head_times(top) {
        println!("  {head}: {:.3}", nanos_millis(nanos));
    }
}

fn bytes_sink(bytes: &[u8]) -> usize {
    let mut sink = bytes.len();
    if let Some(first) = bytes.first() {
        sink = sink.wrapping_add(*first as usize);
    }
    sink
}

fn main_input_sink(input: &[u8]) -> usize {
    bytes_sink(input)
}

fn reduce_main_or_panic(
    program: &mut Program,
    limit: usize,
    context: &str,
    input: &[u8],
) -> RunOnce {
    let reductions = program.reduction_count();
    match program.reduce_main(limit) {
        Ok((_, steps)) => RunOnce {
            steps,
            serialize_sink: main_input_sink(input),
            step_limited: false,
            gc: GcStats::default(),
        },
        Err(EvalError::Raised(exn)) => {
            let message = program
                .uncaught_exception_message_bytes(exn)
                .unwrap_or_else(|err| err.to_string().into_bytes());
            if message == b"ExitSuccess" {
                RunOnce {
                    steps: program.reduction_count().saturating_sub(reductions),
                    serialize_sink: main_input_sink(input),
                    step_limited: false,
                    gc: GcStats::default(),
                }
            } else {
                panic!("{context}: {}", String::from_utf8_lossy(&message));
            }
        }
        Err(EvalError::StepLimit { .. }) => RunOnce {
            steps: program.reduction_count().saturating_sub(reductions),
            serialize_sink: main_input_sink(input),
            step_limited: true,
            gc: GcStats::default(),
        },
        Err(err) => panic!("{context}: {err}"),
    }
}

struct CBench {
    elapsed: Duration,
}

struct CInProcessBench {
    mode: BenchMode,
    ns_per_iter: f64,
    sink: usize,
}

fn bench_c_mhseval(input: &[u8], c_mhseval: &str, iters: usize) -> Result<CBench, String> {
    let file = temp_comb_file();
    fs::write(&file, input).map_err(|err| format!("{}: {err}", file.display()))?;
    let file_arg = format!("-r{}", file.display());

    let started = Instant::now();
    for _ in 0..iters {
        let status = Command::new(c_mhseval)
            .args(["+RTS", "-H1M", &file_arg, "-o/dev/null", "-RTS"])
            .status()
            .map_err(|err| format!("{c_mhseval}: {err}"))?;
        if !status.success() {
            let _ = fs::remove_file(&file);
            return Err(format!("{c_mhseval} exited with {status}"));
        }
    }
    let elapsed = started.elapsed();
    let _ = fs::remove_file(&file);
    Ok(CBench { elapsed })
}

fn bench_c_mhsbench(
    input: &[u8],
    c_mhsbench: &str,
    mode: BenchMode,
    program_args: &[Vec<u8>],
    warmup_iters: usize,
    iters: usize,
) -> Result<CInProcessBench, String> {
    let file = temp_comb_file();
    fs::write(&file, input).map_err(|err| format!("{}: {err}", file.display()))?;
    let iters_arg = iters.to_string();
    let warmup_iters_arg = warmup_iters.to_string();
    let file_arg = file.to_str().ok_or("non-utf8 temp file")?;
    let mut command = Command::new(c_mhsbench);
    command.args([
        "--mode",
        mode.as_str(),
        "--warmup-iters",
        &warmup_iters_arg,
        "--iters",
        &iters_arg,
        file_arg,
    ]);
    if !program_args.is_empty() {
        command.arg("--");
        for arg in program_args {
            command.arg(String::from_utf8_lossy(arg).as_ref());
        }
    }
    let output = command
        .output()
        .map_err(|err| format!("{c_mhsbench}: {err}"))?;
    let _ = fs::remove_file(&file);
    if !output.status.success() {
        return Err(format!(
            "{c_mhsbench} exited with {}\n{}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let stdout = String::from_utf8(output.stdout).map_err(|_| "non-utf8 mhsbench output")?;
    let ns_per_iter = stdout
        .lines()
        .find_map(|line| {
            line.strip_prefix("c_parse_eval_serialize_ns_per_iter: ")
                .or_else(|| line.strip_prefix("c_parse_reduce_serialize_ns_per_iter: "))
        })
        .ok_or_else(|| format!("missing timing in mhsbench output:\n{stdout}"))?
        .parse()
        .map_err(|_| format!("invalid timing in mhsbench output:\n{stdout}"))?;
    let sink = stdout
        .lines()
        .find_map(|line| line.strip_prefix("c_bench_sink: "))
        .ok_or_else(|| format!("missing sink in mhsbench output:\n{stdout}"))?
        .parse()
        .map_err(|_| format!("invalid sink in mhsbench output:\n{stdout}"))?;
    Ok(CInProcessBench {
        mode,
        ns_per_iter,
        sink,
    })
}

fn temp_comb_file() -> std::path::PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    env::temp_dir().join(format!(
        "mhs-rust-bench-{}-{stamp}.comb",
        std::process::id()
    ))
}

fn millis(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1_000.0
}

fn nanos_millis(nanos: u128) -> f64 {
    nanos as f64 / 1_000_000.0
}

fn print_gc_events(label: &str, gc: &GcStats) {
    println!("{label}:");
    for event in &gc.events {
        #[cfg(feature = "gc-phase-profile")]
        println!(
            "  collection={} pause_ms={:.3} mark_ms={:.3} sweep_ms={:.3} live_nodes={} free_nodes={} arena_nodes={} freed_nodes={} allocations_since_collect={} young_slots={} young_live={} young_dead={} old_to_young_sources={} old_to_young_edges={}",
            event.collection,
            nanos_millis(event.pause_nanos),
            nanos_millis(event.mark_nanos),
            nanos_millis(event.sweep_nanos),
            event.live_nodes,
            event.free_nodes,
            event.arena_nodes,
            event.freed_nodes,
            event.allocations_since_collect,
            event.young_profile_slots,
            event.young_profile_live,
            event.young_profile_dead,
            event.young_profile_old_to_young_sources,
            event.young_profile_old_to_young_edges
        );
        #[cfg(not(feature = "gc-phase-profile"))]
        println!(
            "  collection={} pause_ms={:.3} live_nodes={} free_nodes={} arena_nodes={} freed_nodes={} allocations_since_collect={}",
            event.collection,
            nanos_millis(event.pause_nanos),
            event.live_nodes,
            event.free_nodes,
            event.arena_nodes,
            event.freed_nodes,
            event.allocations_since_collect
        );
    }
}

fn nanos_per_iter(duration: Duration, iters: usize) -> f64 {
    duration.as_secs_f64() * 1_000_000_000.0 / iters as f64
}

fn mib_per_s(bytes: usize, iters: usize, duration: Duration) -> f64 {
    let mib = (bytes as f64 * iters as f64) / (1024.0 * 1024.0);
    mib / duration.as_secs_f64()
}
