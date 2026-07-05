use super::*;

#[test]
fn reduces_rnf_and_exception_primitives() {
    assert_eq!(whnf(b"v8.4\n0\nrnf #0 @ O #1 @ #2 @ @ }"), "I");
    assert_eq!(whnf(b"v8.4\n0\nrnf #1 @ raise #7 @ @ }"), "I");

    let mut program = parse_program(b"v8.4\n0\nraise #7 @ }").unwrap();
    assert!(matches!(
        program.reduce_whnf(100),
        Err(EvalError::Raised(_))
    ));

    assert_eq!(
        whnf(b"v8.4\n0\nIO.performIO catch IO.return #5 @ @ K IO.return #42 @ @ @ @ }"),
        "5"
    );
    assert_eq!(
        whnf(b"v8.4\n0\nIO.performIO catch raise #7 @ @ K IO.return #42 @ @ @ @ }"),
        "42"
    );
    assert_eq!(
        whnf(
            b"v8.4\n0\nIO.performIO catch IO.strict IO.return @ quot #1 @ #0 @ @ @ IO.return @ @ }"
        ),
        "4"
    );
    assert_eq!(
            whnf(b"v8.4\n0\nIO.performIO catch IO.strict IO.return @ + #9223372036854775807 @ #1 @ @ @ IO.return @ @ }"),
            "7"
        );
    assert_eq!(
            whnf(b"v8.4\n0\nIO.performIO catch IO.strict IO.return @ Iuquot ##1 @ ##0 @ @ @ IO.return @ @ }"),
            "4"
        );
    assert_eq!(
            whnf(b"v8.4\n0\nIO.performIO catch IO.strict IO.return @ Ineg ##-9223372036854775808 @ @ @ IO.return @ @ }"),
            "7"
        );

    let mut program = parse_program(b"v8.4\n0\nshl #1 @ #64 @ }").unwrap();
    assert!(matches!(
        program.reduce_whnf(100),
        Err(EvalError::InvalidShift(64))
    ));
}

#[test]
fn formats_uncaught_rts_exceptions_like_c() {
    let mut program = parse_program(b"v8.4\n0\nraise #4 @ }").unwrap();
    let Err(EvalError::Raised(exn)) = program.reduce_whnf(100) else {
        panic!("raise did not produce an exception");
    };
    assert_eq!(
        program.uncaught_exception_message_bytes(exn).unwrap(),
        b"DivideByZero"
    );
}
