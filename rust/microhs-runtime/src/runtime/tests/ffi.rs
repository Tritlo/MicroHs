use super::*;

#[test]
fn reduces_builtin_ffi_calls() {
    let is_linux = if cfg!(target_os = "linux") { "1" } else { "0" };
    assert_eq!(whnf(b"v8.4\n0\nIO.performIO ^islinux @ }"), is_linux);
    assert_eq!(whnf(b"v8.4\n0\nIO.performIO ^GETRAW @ }"), "-1");
    assert_eq!(whnf(b"v8.4\n0\nIO.performIO ^putchar #10 @ @ }"), "I");
    assert_eq!(whnf(b"v8.4\n0\nIO.performIO ^sizeof_char @ }"), "1");
    assert_eq!(
        whnf(b"v8.4\n0\nIO.performIO ^sizeof_int @ }"),
        std::mem::size_of::<std::os::raw::c_int>().to_string()
    );
    assert_eq!(whnf(b"v8.4\n0\nIO.performIO ^want_gmp @ }"), "0");
    assert_eq!(whnf(b"v8.4\n0\nIO.performIO ^want_imath @ }"), "1");
    assert_eq!(
        whnf(b"v8.4\n0\nIO.performIO dynsym \"islinux\" @ @ }"),
        is_linux
    );
    assert_eq!(
            whnf(b"v8.4\n0\nIO.performIO dynsym O #105 @ O #115 @ O #108 @ O #105 @ O #110 @ O #117 @ O #120 @ K @ @ @ @ @ @ @ @ @ }"),
            is_linux
        );

    let mut program = parse_program(b"v8.4\n0\nIO.performIO ^does_not_exist @ }").unwrap();
    assert!(matches!(
        program.reduce_whnf(100),
        Err(EvalError::UnknownFfi(name)) if name == "does_not_exist"
    ));

    let mut program = parse_program(b"v8.4\n0\nIO.performIO ^GETTIMEMICRO @ }").unwrap();
    let (root, _) = program.reduce_whnf(100).unwrap();
    let root = program.resolve(root).unwrap();
    match program.node_for_debug(root) {
        Node::Int(n) => assert!(n >= 0),
        _ => panic!("GETTIMEMICRO did not return an Int"),
    }
}

#[test]
fn returns_program_boot_time() {
    let mut program = parse_program(b"v8.4\n0\nIO.performIO ^GETBOOTTIMEMICRO @ }").unwrap();
    let boot_time = program.boot_time_micro;
    let (root, _) = program.reduce_whnf(100).unwrap();
    let root = program.resolve(root).unwrap();
    assert!(matches!(program.node_for_debug(root), Node::Int(n) if n == boot_time));
}

#[test]
fn lz77c_ffi_compresses_to_guest_buffer() {
    let mut program = parse_program(b"v8.4\n0\nI }\n").unwrap();
    let input = b"AAAAAAAAAAAAAAAAzzzzzzzzzzzzzzzz";
    let src = program.alloc_memory(input.len()).unwrap();
    program.write_pointer_bytes(src, input).unwrap();
    let out_ptr = program.alloc_memory(8).unwrap();

    let src_node = program.push_node(Node::Ptr(src));
    let len_node = program.push_node(Node::Int(input.len() as i64));
    let out_ptr_node = program.push_node(Node::Ptr(out_ptr));
    let world = program.prim("I");
    let Some((used, pair)) = program
        .ffi_call("lz77c", &[src_node, len_node, out_ptr_node, world])
        .unwrap()
    else {
        panic!("lz77c did not reduce");
    };
    assert_eq!(used, 4);

    let Some((compressed_len, returned_world)) = program.pair_fields(pair).unwrap() else {
        panic!("lz77c did not return a pair");
    };
    assert_eq!(returned_world, world);
    let compressed_len = match program.node_for_debug(compressed_len) {
        Node::Int(n) => usize::try_from(n).unwrap(),
        other => panic!("lz77c length returned {other:?}"),
    };
    let compressed_ptr = program.peek_signed(out_ptr, 8).unwrap();
    let compressed = program
        .read_pointer_bytes(compressed_ptr, compressed_len)
        .unwrap();
    assert_eq!(lz77_decompress(&compressed).unwrap(), input);
}

#[test]
fn decodes_c_runtime_compression_fixtures() {
    const INPUT: &[u8] = b"AAAAAAAAAAAAAAAABABABABABABA\xff\0end";
    const C_LZ77: &[u8] = &[
        76, 90, 49, 16, 0, 0, 0, 0, 65, 224, 0, 6, 0, 66, 224, 1, 2, 4, 255, 0, 101, 110, 100,
    ];
    const C_BWT: &[u8] = &[
        66, 87, 49, 33, 0, 0, 0, 1, 0, 0, 0, 255, 100, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65,
        65, 65, 65, 65, 66, 66, 66, 66, 66, 66, 65, 65, 65, 65, 65, 65, 110, 0, 101, 65,
    ];
    const C_LZMA: &[u8] = &[
        76, 90, 50, 28, 0, 0, 0, 93, 0, 0, 0, 1, 33, 0, 0, 0, 0, 0, 0, 0, 0, 32, 237, 68, 84, 65,
        127, 132, 12, 164, 143, 145, 248, 248, 0,
    ];

    assert_eq!(&C_LZ77[..3], b"LZ1");
    let lz77_len = u32::from_le_bytes(C_LZ77[3..7].try_into().unwrap()) as usize;
    assert_eq!(lz77_len, C_LZ77.len() - 7);
    assert_eq!(lz77_decompress(&C_LZ77[7..]).unwrap(), INPUT);
    assert_eq!(
        lz77_decompress(&lz77_compress(INPUT).unwrap()).unwrap(),
        INPUT
    );

    assert_eq!(&C_BWT[..3], b"BW1");
    let bwt_len = u32::from_le_bytes(C_BWT[3..7].try_into().unwrap()) as usize;
    let bwt_zero = u32::from_le_bytes(C_BWT[7..11].try_into().unwrap()) as usize;
    assert_eq!(bwt_len, C_BWT.len() - 11);
    assert_eq!(bwt_decode(&C_BWT[11..], bwt_zero).unwrap(), INPUT);
    let (rust_bwt_zero, rust_bwt_last) = bwt_encode(INPUT).unwrap();
    assert_eq!(bwt_decode(&rust_bwt_last, rust_bwt_zero).unwrap(), INPUT);

    assert_eq!(&C_LZMA[..3], b"LZ2");
    let lzma_len = u32::from_le_bytes(C_LZMA[3..7].try_into().unwrap()) as usize;
    assert_eq!(lzma_len, C_LZMA.len() - 7);
    assert_eq!(lzma_decompress_payload(&C_LZMA[7..]).unwrap(), INPUT);
    assert_eq!(
        lzma_decompress_payload(&lzma_compress_payload(INPUT).unwrap()).unwrap(),
        INPUT
    );
}

#[test]
fn reduces_math_ffi_calls() {
    assert_eq!(whnf(b"v8.4\n0\nIO.performIO ^sqrt &9 @ @ }"), "3.0");
    assert_eq!(whnf(b"v8.4\n0\nIO.performIO ^pow &2 @ &8 @ @ }"), "256.0");
    assert_eq!(
        whnf(b"v8.4\n0\nIO.performIO ^scalbn &1.5 @ #2 @ @ }"),
        "6.0"
    );
    assert_eq!(whnf(b"v8.4\n0\nIO.performIO ^sqrtf &&9 @ @ }"), "3.0f");
    assert_eq!(
        whnf(b"v8.4\n0\nIO.performIO ^powf &&2 @ &&8 @ @ }"),
        "256.0f"
    );
    assert_eq!(
        whnf(b"v8.4\n0\nIO.performIO ^scalbnf &&1.5 @ #2 @ @ }"),
        "6.0f"
    );
}
