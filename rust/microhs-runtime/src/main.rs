use std::env;
use std::fs;
use std::io::Write as _;
use std::process::ExitCode;

#[cfg(feature = "profile")]
use microhs_runtime::EvalProfile;
use microhs_runtime::{EvalError, parse_program};

enum Mode {
    Dump,
    Whnf,
    Main,
}

fn usage() {
    eprintln!("usage: mhs-rust [--dump|--whnf|--main] [--profile] [--profile-top N] FILE");
}

fn main() -> ExitCode {
    let mut mode = Mode::Whnf;
    let mut file = None;
    let mut profile = false;
    let mut profile_top = 25usize;
    let mut program_args: Vec<String> = Vec::new();
    let mut args = env::args();
    let program_name = args.next().unwrap_or_else(|| "mhs-rust".to_owned());
    while let Some(arg) = args.next() {
        // Once the FILE is set, remaining tokens are the program's own argv (mhseval
        // convention: `mhs-rust --main prog.comb arg0 arg1 ...`), so don't treat a
        // leading '-' as an mhs-rust flag.
        if file.is_some() {
            program_args.push(arg);
            continue;
        }
        match arg.as_str() {
            "--dump" => mode = Mode::Dump,
            "--whnf" => mode = Mode::Whnf,
            "--main" => mode = Mode::Main,
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

    #[cfg(not(feature = "profile"))]
    if profile {
        eprintln!("--profile requires rebuilding mhs-rust with --features profile");
        return ExitCode::from(2);
    }

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
    // The trailing CLI tokens are the program's own argv (argv[0] = program name,
    // argv[1..] = getArgs), matching mhseval / the bench harness. With no trailing
    // tokens, expose just this binary's name so getProgName works.
    let argv = if program_args.is_empty() {
        vec![program_name.into_bytes()]
    } else {
        program_args.into_iter().map(String::into_bytes).collect()
    };
    program.set_program_args(argv);
    #[cfg(feature = "profile")]
    if profile {
        program.enable_profile();
    }

    match mode {
        Mode::Dump => {
            println!("{}", program.render(program.root()));
            maybe_print_profile(&mut program, profile_top);
            ExitCode::SUCCESS
        }
        Mode::Whnf => match program.reduce_whnf(10_000) {
            Ok((root, steps)) => {
                println!("{}", program.render(root));
                eprintln!("{steps} reductions");
                maybe_print_profile(&mut program, profile_top);
                ExitCode::SUCCESS
            }
            Err(err) => {
                let code = report_eval_error(&mut program, &file, err);
                maybe_print_profile(&mut program, profile_top);
                code
            }
        },
        // Drive the program as a full IO `main` (like `mhseval`): the program's own
        // output goes straight to stdout during reduction; we print nothing extra.
        Mode::Main => match program.reduce_main(usize::MAX) {
            Ok(_) => {
                maybe_print_profile(&mut program, profile_top);
                ExitCode::SUCCESS
            }
            Err(err) => {
                let code = report_eval_error(&mut program, &file, err);
                maybe_print_profile(&mut program, profile_top);
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

#[cfg(feature = "profile")]
fn maybe_print_profile(program: &mut microhs_runtime::Program, profile_top: usize) {
    if let Some(profile) = program.take_profile() {
        print_profile(&profile, profile_top);
    }
}

#[cfg(not(feature = "profile"))]
fn maybe_print_profile(_program: &mut microhs_runtime::Program, _profile_top: usize) {}

#[cfg(feature = "profile")]
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
