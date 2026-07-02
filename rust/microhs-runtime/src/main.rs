use std::env;
use std::fs;
use std::process::ExitCode;

use microhs_runtime::{EvalProfile, parse_program};

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
                eprintln!("{file}: {err}");
                if let Some(profile) = program.take_profile() {
                    print_profile(&profile, profile_top);
                }
                ExitCode::from(1)
            }
        },
    }
}

fn print_profile(profile: &EvalProfile, top: usize) {
    eprintln!("profile_step_attempts: {}", profile.step_attempts);
    eprintln!("profile_successful_steps: {}", profile.successful_steps);
    eprintln!("profile_reductions: {}", profile.reductions);
    eprintln!("profile_heap_spines: {}", profile.heap_spines);
    eprintln!("profile_max_spine_arity: {}", profile.max_spine_arity);
    eprintln!("profile_top_head_attempts:");
    for (head, count) in profile.top_head_attempts(top) {
        eprintln!("  {head}: {count}");
    }
    eprintln!("profile_top_head_reductions:");
    for (head, count) in profile.top_head_reductions(top) {
        eprintln!("  {head}: {count}");
    }
    eprintln!("profile_spine_arity:");
    for (arity, count) in &profile.spine_arity {
        eprintln!("  {arity}: {count}");
    }
}
