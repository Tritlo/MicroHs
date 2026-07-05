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
