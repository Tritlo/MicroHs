use super::*;

#[test]
fn serializes_quoted_bytestring_escapes_like_c() {
    let mut special = Vec::new();
    serialize_bytes_quoted(b"?^|\\\"", &mut special);
    assert_eq!(special, b"\"?\\^\\|\\\\\\\"\"");

    for byte in 0u8..=255 {
        let mut input = b"v8.4\n0\n".to_vec();
        serialize_bytes_quoted(&[byte], &mut input);
        input.extend_from_slice(b" }\n");

        let program = parse_program(&input).unwrap();
        match program.node_for_debug(program.root()) {
            Node::Bytes(bytes) => assert_eq!(
                bytes.as_slice(),
                &[byte],
                "byte {byte:#04x} encoded as {}",
                String::from_utf8_lossy(&input)
            ),
            other => panic!(
                "byte {byte:#04x} parsed as {other:?} from {}",
                String::from_utf8_lossy(&input)
            ),
        }
    }
}

#[test]
fn serializes_bigints_with_c_wire_format() {
    let program = parse_program(b"v8.4\n0\n%-123456789\" }\n").unwrap();
    assert_eq!(
        program.serialize_program(program.root()).unwrap(),
        b"v8.4\n0\n%-123456789\" }\n"
    );

    let old_wire = parse_program(b"v8.4\n0\n%\"123456789\" }\n").unwrap();
    match old_wire.node_for_debug(old_wire.root()) {
        Node::BigInt(bytes) => assert_eq!(bytes.as_slice(), b"123456789"),
        other => panic!("old bigint wire format parsed as {other:?}"),
    }
}

#[test]
fn serializes_shared_apps_with_labels() {
    let program = parse_program(b"v8.4\n1\nK #1 @ :0 _0 @ }\n").unwrap();
    let serialized = program.serialize_program(program.root()).unwrap();
    assert!(serialized.starts_with(b"v8.4\n1\n"), "{serialized:?}");
    assert!(
        serialized
            .windows(b":2 ".len())
            .any(|window| window == b":2 "),
        "{}",
        String::from_utf8_lossy(&serialized)
    );
    assert!(
        serialized
            .windows(b"_2 ".len())
            .any(|window| window == b"_2 "),
        "{}",
        String::from_utf8_lossy(&serialized)
    );

    let round_trip = parse_program(&serialized).unwrap();
    assert_eq!(
        round_trip.serialize_program(round_trip.root()).unwrap(),
        serialized
    );
}

#[test]
fn serializes_cyclic_apps_with_labels() {
    let program = parse_program(b"v8.4\n1\nK _0 @ :0 }\n").unwrap();
    let serialized = program.serialize_program(program.root()).unwrap();
    assert!(serialized.starts_with(b"v8.4\n1\n"), "{serialized:?}");
    assert!(
        serialized
            .windows(b":2 ".len())
            .any(|window| window == b":2 "),
        "{}",
        String::from_utf8_lossy(&serialized)
    );
    assert!(
        serialized
            .windows(b"_2 ".len())
            .any(|window| window == b"_2 "),
        "{}",
        String::from_utf8_lossy(&serialized)
    );
    parse_program(&serialized).unwrap();
}

#[test]
fn io_deserialize_reads_one_comb_from_bfile() {
    let mut program = parse_program(b"v8.4\n0\nI }\n").unwrap();
    let (value, ptr) = deserialize_memory(&mut program, b"v8.4\n0\n#42 }tail".to_vec());
    assert_eq!(program.render(value), "42");
    assert_eq!(program.read_bfile_bytes(ptr, 4).unwrap(), b"tail");
}

#[test]
fn io_deserialize_preserves_shared_cycles() {
    let mut program = parse_program(b"v8.4\n0\nI }\n").unwrap();
    let (value, _) = deserialize_memory(&mut program, b"v8.4\n1\nK _0 @ :0 }\n".to_vec());
    let serialized = program.serialize_program(value).unwrap();
    assert!(serialized.starts_with(b"v8.4\n1\n"), "{serialized:?}");
    assert!(
        serialized.contains(&b'_'),
        "{}",
        String::from_utf8_lossy(&serialized)
    );
    assert!(
        serialized.contains(&b':'),
        "{}",
        String::from_utf8_lossy(&serialized)
    );
    parse_program(&serialized).unwrap();
}

fn alloc_read_bfile(program: &mut Program, bytes: Vec<u8>) -> i64 {
    program
        .alloc_bfile(BFile {
            kind: BFileKind::Memory { bytes, pos: 0 },
            readable: true,
            writable: false,
        })
        .unwrap()
}

fn stream_parse_with_tail(input: &[u8], tail_len: usize) -> Result<(Program, Vec<u8>), EvalError> {
    let mut driver = parse_program(b"v8.4\n0\nI }\n").unwrap();
    let ptr = alloc_read_bfile(&mut driver, input.to_vec());
    let parsed = driver.parse_bfile_program_for_test(ptr)?;
    let tail = driver.read_bfile_bytes(ptr, tail_len)?;
    Ok((parsed, tail))
}

fn assert_stream_matches_slice(input: &[u8]) {
    let sliced = parse_program(input).unwrap();
    let (streamed, _) = stream_parse_with_tail(input, 0).unwrap();
    assert_eq!(
        streamed.serialize_program(streamed.root()).unwrap(),
        sliced.serialize_program(sliced.root()).unwrap(),
        "{}",
        String::from_utf8_lossy(input)
    );
}

#[test]
fn stream_parse_matches_slice_parser_cases() {
    let mut raw = b"v8.4\n0\n$6 ".to_vec();
    raw.extend_from_slice(b"ab}\0 c");
    raw.extend_from_slice(b" }\n");

    let cases: &[&[u8]] = &[
        b"v8.4\n0\nI }\n",
        b"v8.4\n1\nK _0 @ :0 }\n",
        b"v8.4\n0\n#1 #2 [2] }\n",
        b"v8.4\n0\n%-12345678901234567890\" }\n",
        b"v8.4\n0\n%\"12345678901234567890\" }\n",
        b"v8.4\n0\n^ffi_name ~tag1,tag2 \"body\" `wrap ;callback !\"tick\" [5] }\n",
        b"v8.4\n0\nI \r\n}\n",
    ];
    for case in cases {
        assert_stream_matches_slice(case);
    }
    assert_stream_matches_slice(&raw);
}

#[test]
fn stream_parse_leaves_js_exports_trailer_readable() {
    let input = b"v8.4\n0\n#42 }##### JS_EXPORTS\nrest";
    let (streamed, tail) = stream_parse_with_tail(input, b"##### JS_EXPORTS\nrest".len()).unwrap();
    let sliced = parse_program(input).unwrap();
    assert_eq!(
        streamed.serialize_program(streamed.root()).unwrap(),
        sliced.serialize_program(sliced.root()).unwrap()
    );
    assert_eq!(tail, b"##### JS_EXPORTS\nrest");
}

#[test]
fn stream_parse_reports_malformed_input_cleanly() {
    for input in [
        b"v8.4\n0\n$4 ab".as_slice(),
        b"v8.4\n0\n# }tail".as_slice(),
        b"v8.4\n0\nNO_SUCH_PRIM }".as_slice(),
        b"bad\n0\nI }".as_slice(),
    ] {
        let mut driver = parse_program(b"v8.4\n0\nI }\n").unwrap();
        let ptr = alloc_read_bfile(&mut driver, input.to_vec());
        assert!(driver.parse_bfile_program_for_test(ptr).is_err());
    }
}

#[test]
fn stream_parse_random_valid_programs_match_slice_parser() {
    let mut seed = 0x1234_5678_9abc_def0_u64;
    for depth in 0..80 {
        let mut input = b"v8.4\n0\n".to_vec();
        emit_random_expr(&mut seed, depth % 5, &mut input);
        input.extend_from_slice(b"}\n");
        assert_stream_matches_slice(&input);
    }
}

fn emit_random_expr(seed: &mut u64, depth: usize, out: &mut Vec<u8>) {
    *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
    let choice = ((*seed >> 32) % if depth == 0 { 6 } else { 8 }) as usize;
    match choice {
        0 => out.extend_from_slice(b"I "),
        1 => {
            out.push(b'#');
            out.extend_from_slice((((*seed >> 16) % 512) as i64 - 128).to_string().as_bytes());
            out.push(b' ');
        }
        2 => {
            out.extend_from_slice(b"##");
            out.extend_from_slice((((*seed >> 8) % 4096) as i64 - 2048).to_string().as_bytes());
            out.push(b' ');
        }
        3 => out.extend_from_slice(b"\"a\\\"b\" "),
        4 => out.extend_from_slice(b"%-123456789\" "),
        5 => out.extend_from_slice(b"&1.25 "),
        6 => {
            emit_random_expr(seed, depth - 1, out);
            emit_random_expr(seed, depth - 1, out);
            out.push(b'@');
        }
        _ => {
            emit_random_expr(seed, depth - 1, out);
            emit_random_expr(seed, depth - 1, out);
            out.extend_from_slice(b"[2] ");
        }
    }
}
