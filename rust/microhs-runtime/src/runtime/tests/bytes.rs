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
    assert_matches!(program.reduce_whnf(100), Err(EvalError::InvalidByteString));
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

#[cfg(any(not(target_arch = "wasm32"), target_os = "wasi"))]
fn native_bfile_test_path(name: &str) -> std::path::PathBuf {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "microhs-runtime-{name}-{}-{now}.bin",
        std::process::id()
    ))
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
fn native_bfile_path_bytes(path: &std::path::Path) -> Vec<u8> {
    use std::os::unix::ffi::OsStrExt as _;

    path.as_os_str().as_bytes().to_vec()
}

#[cfg(any(target_os = "wasi", not(any(unix, target_arch = "wasm32"))))]
fn native_bfile_path_bytes(path: &std::path::Path) -> Vec<u8> {
    path.to_string_lossy().into_owned().into_bytes()
}

#[cfg(any(not(target_arch = "wasm32"), target_os = "wasi"))]
fn native_bfile_payload(len: usize) -> Vec<u8> {
    (0..len).map(|i| ((i * 37 + 11) % 251) as u8).collect()
}

#[cfg(any(not(target_arch = "wasm32"), target_os = "wasi"))]
#[test]
fn native_fopen_bfile_byte_writes_round_trip() {
    let path = native_bfile_test_path("byte-round-trip");
    let path_bytes = native_bfile_path_bytes(&path);
    let payload = native_bfile_payload(17 * 1024 + 19);
    let mut program = parse_program(b"v8.4\n0\nI }\n").unwrap();

    let ptr = program
        .alloc_bfile(native_fopen_bfile(&path_bytes, b"w").unwrap())
        .unwrap();
    for byte in &payload {
        program.put_bfile_byte(ptr, i64::from(*byte)).unwrap();
    }
    program.close_bfile(ptr).unwrap();

    let ptr = program
        .alloc_bfile(native_fopen_bfile(&path_bytes, b"r").unwrap())
        .unwrap();
    let mut actual = Vec::with_capacity(payload.len());
    loop {
        match program.get_bfile_byte(ptr).unwrap() {
            -1 => break,
            byte => actual.push(byte as u8),
        }
    }
    program.close_bfile(ptr).unwrap();
    assert_eq!(actual, payload);

    let _ = std::fs::remove_file(path);
}

#[cfg(any(not(target_arch = "wasm32"), target_os = "wasi"))]
#[test]
fn native_fopen_bfile_drop_flushes_without_close() {
    let path = native_bfile_test_path("drop-flush");
    let path_bytes = native_bfile_path_bytes(&path);
    let payload = native_bfile_payload(9 * 1024 + 7);
    {
        let mut program = parse_program(b"v8.4\n0\nI }\n").unwrap();
        let ptr = program
            .alloc_bfile(native_fopen_bfile(&path_bytes, b"w").unwrap())
            .unwrap();
        for byte in &payload {
            program.put_bfile_byte(ptr, i64::from(*byte)).unwrap();
        }
    }

    assert_eq!(std::fs::read(&path).unwrap(), payload);
    let _ = std::fs::remove_file(path);
}

#[cfg(any(not(target_arch = "wasm32"), target_os = "wasi"))]
#[test]
fn native_fopen_bfile_bulk_write_preserves_pending_byte_order() {
    let path = native_bfile_test_path("interleaved-write");
    let path_bytes = native_bfile_path_bytes(&path);
    let prefix = native_bfile_payload(4091);
    let middle = native_bfile_payload(8197);
    let suffix = native_bfile_payload(413);
    let mut expected = Vec::new();
    expected.extend_from_slice(&prefix);
    expected.extend_from_slice(&middle);
    expected.extend_from_slice(&suffix);

    let mut program = parse_program(b"v8.4\n0\nI }\n").unwrap();
    let ptr = program
        .alloc_bfile(native_fopen_bfile(&path_bytes, b"w").unwrap())
        .unwrap();
    for byte in &prefix {
        program.put_bfile_byte(ptr, i64::from(*byte)).unwrap();
    }
    assert_eq!(
        program.write_bfile_bytes(ptr, &middle).unwrap(),
        middle.len()
    );
    for byte in &suffix {
        program.put_bfile_byte(ptr, i64::from(*byte)).unwrap();
    }
    program.close_bfile(ptr).unwrap();

    assert_eq!(std::fs::read(&path).unwrap(), expected);
    let _ = std::fs::remove_file(path);
}
