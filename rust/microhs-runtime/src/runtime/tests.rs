//! Runtime parity and regression tests.
use crate::runtime::{
    BFile, BFileKind, EvalSpine, FORCE_REDUCTION_LIMIT, IGNORED_IO_SHORTCUT_RECURSION_LIMIT,
    MpzValue, bwt_decode, bwt_encode, lz77_compress, lz77_decompress, lzma_compress_payload,
    lzma_decompress_payload, serialize_bytes_quoted,
};
use crate::{EvalError, KnownPrim, Node, NodeId, ParseError, Prim, Program, parse_program};

fn whnf(input: &[u8]) -> String {
    let mut program = parse_program(input).unwrap();
    let (root, _) = program.reduce_whnf(100).unwrap();
    program.render(root)
}

fn collect_for_test(program: &mut Program, root: NodeId) {
    program
        .collect_garbage_between_steps(root, &EvalSpine::default(), &[], &[], None)
        .unwrap();
}

fn deserialize_memory(program: &mut Program, bytes: Vec<u8>) -> (NodeId, i64) {
    let ptr = program
        .alloc_bfile(BFile {
            kind: BFileKind::Memory { bytes, pos: 0 },
            readable: true,
            writable: false,
        })
        .unwrap();
    let ptr_node = program.push_node(Node::Ptr(ptr));
    let world = program.push_node(Node::Int(99_999));
    let deserialize = program.prim("IO.deserialize");
    let deserialize_ptr = program.app(deserialize, ptr_node);
    let action = program.app(deserialize_ptr, world);
    let pair = program
        .reduce_node_whnf(action, FORCE_REDUCTION_LIMIT)
        .unwrap();
    let (left, returned_world) = program.cell(pair).app_fields().unwrap();
    assert_eq!(returned_world, world);
    let (head, value) = program.cell(left).app_fields().unwrap();
    assert!(matches!(
        program.cell(head).prim(),
        Some(Prim::Known(KnownPrim::P))
    ));
    (value, ptr)
}

mod arrays_io;
mod bytes;
mod exceptions;
mod ffi;
mod handles_gc;
mod numeric;
mod reduction;
mod serialization;
