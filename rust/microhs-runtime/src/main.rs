use std::env;
use std::fs;
use std::process::ExitCode;

use microhs_runtime::parse_program;

enum Mode {
    Dump,
    Whnf,
}

fn usage() {
    eprintln!("usage: mhs-rust [--dump|--whnf] FILE");
}

fn main() -> ExitCode {
    let mut mode = Mode::Whnf;
    let mut file = None;
    let mut args = env::args();
    let program_name = args.next().unwrap_or_else(|| "mhs-rust".to_owned());
    for arg in args {
        match arg.as_str() {
            "--dump" => mode = Mode::Dump,
            "--whnf" => mode = Mode::Whnf,
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

    match mode {
        Mode::Dump => {
            println!("{}", program.render(program.root()));
            ExitCode::SUCCESS
        }
        Mode::Whnf => match program.reduce_whnf(10_000) {
            Ok((root, steps)) => {
                println!("{}", program.render(root));
                eprintln!("{steps} reductions");
                ExitCode::SUCCESS
            }
            Err(err) => {
                eprintln!("{file}: {err}");
                ExitCode::from(1)
            }
        },
    }
}
