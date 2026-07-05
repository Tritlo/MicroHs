use std::env;
use std::fs;
use std::io::Write as _;
use std::process::ExitCode;

use microhs_runtime::{EvalError, EvalProfile, parse_program};

enum Mode {
    Dump,
    Whnf,
}

fn usage() {
    eprintln!("usage: mhs-rust [--dump|--whnf] [--profile] [--profile-top N] FILE");
}

fn main() -> ExitCode {
    let mut mode = Mode::Whnf;
    let mut file = None;
    let mut profile = false;
    let mut profile_top = 25usize;
    let mut args = env::args();
    let program_name = args.next().unwrap_or_else(|| "mhs-rust".to_owned());
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--dump" => mode = Mode::Dump,
            "--whnf" => mode = Mode::Whnf,
            "--profile" => profile = true,
            "--profile-top" => {
                let Some(value) = args.next() else {
                    eprintln!("--profile-top requires a value");
                    usage();
                    return ExitCode::from(2);
                };
                let Ok(value) = value.parse() else {
                    eprintln!("invalid --profile-top value: {value}");
                    usage();
                    return ExitCode::from(2);
                };
                if value == 0 {
                    eprintln!("--profile-top must be greater than zero");
                    usage();
                    return ExitCode::from(2);
                }
                profile_top = value;
            }
            "-h" | "--help" => {
                usage();
                return ExitCode::SUCCESS;
            }
            _ if file.is_none() => file = Some(arg),
            _ => {
                usage();
                return ExitCode::from(2);
            }
        }
    }

    let Some(file) = file else {
        usage();
        return ExitCode::from(2);
    };

    let bytes = match fs::read(&file) {
        Ok(bytes) => bytes,
        Err(err) => {
            eprintln!("{file}: {err}");
            return ExitCode::from(1);
        }
    };

    let mut program = match parse_program(&bytes) {
        Ok(program) => program,
        Err(err) => {
            eprintln!("{file}: {err}");
            return ExitCode::from(1);
        }
    };
    program.set_program_args(vec![program_name.into_bytes()]);
    if profile {
        program.enable_profile();
    }

    match mode {
        Mode::Dump => {
            println!("{}", program.render(program.root()));
            if let Some(profile) = program.take_profile() {
                print_profile(&profile, profile_top);
            }
            ExitCode::SUCCESS
        }
        Mode::Whnf => match program.reduce_whnf(10_000) {
            Ok((root, steps)) => {
                println!("{}", program.render(root));
                eprintln!("{steps} reductions");
                if let Some(profile) = program.take_profile() {
                    print_profile(&profile, profile_top);
                }
                ExitCode::SUCCESS
            }
            Err(err) => {
                let code = report_eval_error(&mut program, &file, err);
                if let Some(profile) = program.take_profile() {
                    print_profile(&profile, profile_top);
                }
                code
            }
        },
    }
}

fn report_eval_error(
    program: &mut microhs_runtime::Program,
    file: &str,
    err: EvalError,
) -> ExitCode {
    match err {
        EvalError::Raised(exn) => match program.uncaught_exception_message_bytes(exn) {
            Ok(message) if message == b"ExitSuccess" => ExitCode::SUCCESS,
            Ok(message) => {
                print_uncaught_exception(file, &message);
                ExitCode::from(1)
            }
            Err(err) => {
                eprintln!("{file}: {err}");
                ExitCode::from(1)
            }
        },
        err => {
            eprintln!("{file}: {err}");
            ExitCode::from(1)
        }
    }
}

fn print_uncaught_exception(program_name: &str, message: &[u8]) {
    let mut stderr = std::io::stderr().lock();
    let _ = stderr.write_all(b"\n");
    let _ = stderr.write_all(program_name.as_bytes());
    let _ = stderr.write_all(b": uncaught exception: ");
    let _ = stderr.write_all(message);
    let _ = stderr.write_all(b"\n");
}

fn print_profile(profile: &EvalProfile, top: usize) {
    eprintln!("profile_step_attempts: {}", profile.step_attempts);
    eprintln!("profile_successful_steps: {}", profile.successful_steps);
    eprintln!("profile_reductions: {}", profile.reductions);
    eprintln!("profile_app_allocations: {}", profile.app_allocations);
    eprintln!(
        "profile_arg_materializations: {}",
        profile.arg_materializations
    );
    eprintln!(
        "profile_arg_materialized_nodes: {}",
        profile.arg_materialized_nodes
    );
    eprintln!("profile_spine_rewrites: {}", profile.spine_rewrites);
    eprintln!(
        "profile_spine_rewrite_extra_args: {}",
        profile.spine_rewrite_extra_args
    );
    eprintln!("profile_app_rewrites: {}", profile.app_rewrites);
    eprintln!(
        "profile_app_rewrite_extra_args: {}",
        profile.app_rewrite_extra_args
    );
    eprintln!(
        "profile_small_int_cache_hits: {}",
        profile.small_int_cache_hits
    );
    eprintln!(
        "profile_small_int_cache_misses: {}",
        profile.small_int_cache_misses
    );
    eprintln!(
        "profile_non_small_int_allocations: {}",
        profile.non_small_int_allocations
    );
    eprintln!("profile_max_spine_arity: {}", profile.max_spine_arity);
    eprintln!("profile_resolve_calls: {}", profile.resolve_calls);
    eprintln!(
        "profile_resolve_indirections: {}",
        profile.resolve_indirections
    );
    eprintln!("profile_max_resolve_chain: {}", profile.max_resolve_chain);
    eprintln!("profile_top_head_attempts:");
    for (head, count) in profile.top_head_attempts(top) {
        eprintln!("  {head}: {count}");
    }
    eprintln!("profile_top_head_reductions:");
    for (head, count) in profile.top_head_reductions(top) {
        eprintln!("  {head}: {count}");
    }
    eprintln!("profile_resolve_chain:");
    for (depth, count) in &profile.resolve_chain {
        eprintln!("  {depth}: {count}");
    }
    eprintln!("profile_shortcut_hits:");
    for (shortcut, count) in profile.top_shortcut_hits(top) {
        eprintln!("  {shortcut}: {count}");
    }
}
