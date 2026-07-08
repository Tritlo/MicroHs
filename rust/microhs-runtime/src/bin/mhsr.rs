//! `mhsr` — the MicroHs compiler as a self-contained binary on the Rust runtime,
//! a drop-in for the C `bin/mhs`.
//!
//! It bakes the compiler comb (`generated/mhs.comb`) in with `include_bytes!`
//! (the way the C `bin/mhs` embeds its comb) and drives it as an IO `main` with
//! `argv[0] = "mhs"`. Because the same comb self-hosts byte-identically on the C
//! and Rust runtimes, `mhsr` compiles identically to `bin/mhs`, so it can stand
//! in wherever `mhs` is invoked (e.g. as mcabal's sub-compiler).
//!
//! Installed at `<repo>/bin/mhsr`, beside the C `bin/mhs` — one level below the
//! checkout root — so the compiler's `getExecutablePath` finds the inplace
//! `lib/` exactly as `bin/mhs` does, with no path plumbing here. Rebuild whenever
//! `generated/mhs.comb` is rebaselined — `include_bytes!` bakes it in at build
//! time.

use std::env;
use std::io::Write as _;
use std::os::unix::ffi::{OsStrExt as _, OsStringExt as _};
use std::process::ExitCode;

use microhs_runtime::{EvalError, Program, parse_program};

/// The compiler comb, baked in at build time like the C `bin/mhs` embeds its own.
static COMB: &[u8] =
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../generated/mhs.comb"));

fn main() -> ExitCode {
    // argv[0] is deliberately "mhs" so getProgName and the compiler's diagnostic
    // prefixes match the C binary; the rest is our command line (raw bytes, since
    // paths need not be UTF-8) minus any `+RTS ... -RTS` block. The C runtime
    // strips that block before handing argv to the program; the Rust runtime is
    // tuned via MHS_GC_NODE_INTERVAL and has no use for those flags, so it is
    // simply dropped.
    let mut argv: Vec<Vec<u8>> = vec![b"mhs".to_vec()];
    let mut in_rts = false;
    for arg in env::args_os().skip(1) {
        if in_rts {
            if arg.as_bytes() == b"-RTS" {
                in_rts = false;
            }
        } else if arg.as_bytes() == b"+RTS" {
            in_rts = true;
        } else {
            argv.push(arg.into_vec());
        }
    }

    let mut program = match parse_program(COMB) {
        Ok(program) => program,
        Err(err) => {
            eprintln!("mhsr: embedded comb is invalid: {err}");
            return ExitCode::from(1);
        }
    };
    program.set_program_args(argv);

    match program.reduce_main(usize::MAX) {
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => report_eval_error(&mut program, err),
    }
}

/// Map a terminal evaluation result to a process exit code, matching the C
/// runtime: a clean `ExitSuccess` is 0, and any other uncaught exception
/// (including `ExitFailure n` and ordinary compiler errors) is 1. `ExitFailure n`
/// is not preserved because this runtime does not currently carry the code.
fn report_eval_error(program: &mut Program, err: EvalError) -> ExitCode {
    match err {
        EvalError::Raised(exn) => match program.uncaught_exception_message_bytes(exn) {
            Ok(message) if message == b"ExitSuccess" => ExitCode::SUCCESS,
            Ok(message) => {
                let mut stderr = std::io::stderr().lock();
                let _ = stderr.write_all(b"mhs: uncaught exception: ");
                let _ = stderr.write_all(&message);
                let _ = stderr.write_all(b"\n");
                ExitCode::from(1)
            }
            Err(err) => {
                eprintln!("mhs: {err}");
                ExitCode::from(1)
            }
        },
        err => {
            eprintln!("mhs: {err}");
            ExitCode::from(1)
        }
    }
}
