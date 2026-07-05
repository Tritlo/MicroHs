use super::*;

#[test]
fn reduces_bytestring_primitives() {
    assert_eq!(whnf(b"v8.4\n0\nbs++ \"foo\" @ \"bar\" @ }"), "\"foobar\"");
    assert_eq!(whnf(b"v8.4\n0\nbs++. \"foo\" @ \"bar\" @ }"), "\"foo.bar\"");
    assert_eq!(whnf(b"v8.4\n0\nbs== \"x\" @ \"x\" @ }"), "A");
    assert_eq!(whnf(b"v8.4\n0\nbs< \"abc\" @ \"abd\" @ }"), "A");
    assert_eq!(whnf(b"v8.4\n0\nbscmp \"abd\" @ \"abc\" @ }"), "KA");
    assert_eq!(whnf(b"v8.4\n0\nbslength \"hello\" @ }"), "5");
    assert_eq!(whnf(b"v8.4\n0\nbsreplicate #3 @ #65 @ }"), "\"AAA\"");
    assert_eq!(whnf(b"v8.4\n0\nbsindex \"ABC\" @ #1 @ }"), "66");
    assert_eq!(
        whnf(b"v8.4\n0\nbssubstr \"abcdef\" @ #2 @ #3 @ }"),
        "\"cde\""
    );
    assert_eq!(whnf(b"v8.4\n0\nheadUTF8 $2 \xc3\xa5 @ }"), "229");
    assert_eq!(whnf(b"v8.4\n0\ntailUTF8 $3 \xc3\xa5x @ }"), "\"x\"");
    assert_eq!(whnf(b"v8.4\n0\nbsunpack \"AB\" @ #0 @ K @ }"), "65");
    assert_eq!(
        whnf(b"v8.4\n0\nbsunpack \"AB\" @ #0 @ A @ #0 @ K @ }"),
        "66"
    );
    assert_eq!(whnf(b"v8.4\n0\nfromUTF8 $3 \xc3\xa5x @ #0 @ K @ }"), "229");
    assert_eq!(
        whnf(b"v8.4\n0\nfromUTF8 $3 \xc3\xa5x @ #0 @ A @ #0 @ K @ }"),
        "120"
    );
    assert_eq!(whnf(b"v8.4\n0\nfromUTF8 $2 \xc0\x80 @ #0 @ K @ }"), "0");
    assert_eq!(whnf(b"v8.4\n0\nfromUTF8 $1 \xc3 @ }"), "K");

    let mut program = parse_program(b"v8.4\n0\nfromUTF8 $2 \xc1\x81 @ }").unwrap();
    assert!(matches!(
        program.reduce_whnf(100),
        Err(EvalError::InvalidByteString)
    ));
}

#[test]
fn reduces_mutable_bytestring_primitives() {
    assert_eq!(whnf(b"v8.4\n0\nbsnew #2 @ #4 @ }"), "\"\\x00\\x00\"");
    assert_eq!(
        whnf(b"v8.4\n1\nseq bswrite bsnew #2 @ #4 @ :0 @ #1 @ #65 @ @ bsread _0 @ #1 @ @ }"),
        "65"
    );
    assert_eq!(
        whnf(b"v8.4\n1\nseq bsappbyte bsnew #0 @ #0 @ :0 @ #65 @ @ bsfreeze _0 @ @ }"),
        "\"A\""
    );
    assert_eq!(
        whnf(b"v8.4\n1\nseq bsappchar bsnew #0 @ #0 @ :0 @ #229 @ @ bsfreeze _0 @ @ }"),
        "\"\\xc3\\xa5\""
    );
    assert_eq!(
        whnf(b"v8.4\n1\nseq bswrite \"abc\" :0 @ #1 @ #88 @ @ bsread _0 @ #1 @ @ }"),
        "88"
    );
}

#[test]
fn reduces_mutable_bytestring_primitives_as_io_actions() {
    assert_eq!(
        whnf(b"v8.4\n0\nIO.performIO bsnew #2 @ #4 @ @ }"),
        "\"\\x00\\x00\""
    );
    assert_eq!(
            whnf(b"v8.4\n1\nIO.performIO IO.>> bswrite bsnew #2 @ #4 @ :0 @ #1 @ #65 @ @ bsread _0 @ #1 @ @ @ }"),
            "65"
        );
    assert_eq!(
            whnf(b"v8.4\n1\nIO.performIO IO.>> bsappbyte bsnew #0 @ #0 @ :0 @ #65 @ @ bsfreeze _0 @ @ @ }"),
            "\"A\""
        );
}
