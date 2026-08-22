use super::runner::RunOnce;
use super::*;

pub(super) fn bytes_sink(bytes: &[u8]) -> usize {
    let mut sink = bytes.len();
    if let Some(first) = bytes.first() {
        sink = sink.wrapping_add(*first as usize);
    }
    sink
}

pub(super) fn reduce_main_or_panic(
    program: &mut Program,
    limit: usize,
    context: &str,
    input: &[u8],
) -> RunOnce {
    let reductions = program.reduction_count();
    match program.reduce_main(limit) {
        Ok((_, steps)) => RunOnce {
            steps,
            serialize_sink: bytes_sink(input),
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
                    serialize_sink: bytes_sink(input),
                    step_limited: false,
                    gc: GcStats::default(),
                }
            } else {
                panic!("{context}: {}", String::from_utf8_lossy(&message));
            }
        }
        Err(EvalError::StepLimit { .. }) => RunOnce {
            steps: program.reduction_count().saturating_sub(reductions),
            serialize_sink: bytes_sink(input),
            step_limited: true,
            gc: GcStats::default(),
        },
        Err(err) => panic!("{context}: {err}"),
    }
}
