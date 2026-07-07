use super::*;

#[test]
fn reduces_stable_pointer_primitives() {
    assert_eq!(whnf(b"v8.4\n0\nSPnew #42 @ }"), "1");
    assert_eq!(whnf(b"v8.4\n0\nSPderef SPnew #42 @ @ }"), "42");
    assert_eq!(
        whnf(b"v8.4\n2\nseq SPfree SPnew #1 @ :0 @ @ SPnew #2 @ :1 @ }"),
        "1"
    );
    assert_eq!(whnf(b"v8.4\n0\nIO.performIO SPnew #42 @ @ }"), "1");
    assert_eq!(
        whnf(b"v8.4\n0\nIO.performIO SPderef SPnew #42 @ @ @ }"),
        "42"
    );
}

#[test]
fn handle_tables_reuse_lowest_free_slot() {
    let mut program = parse_program(b"v8.4\n0\nI }\n").unwrap();
    let value = program.root();

    let stable_1 = program.new_stable_ptr_handle(value).unwrap();
    let stable_2 = program.new_stable_ptr_handle(value).unwrap();
    let stable_3 = program.new_stable_ptr_handle(value).unwrap();
    assert_eq!((stable_1, stable_2, stable_3), (1, 2, 3));
    program.free_stable_ptr(stable_2 as usize).unwrap();
    assert_eq!(program.new_stable_ptr_handle(value).unwrap(), 2);
    program.free_stable_ptr(stable_1 as usize).unwrap();
    assert_eq!(program.new_stable_ptr_handle(value).unwrap(), 1);
    assert_eq!(program.new_stable_ptr_handle(value).unwrap(), 4);

    let mem_0 = program.alloc_memory(1).unwrap();
    let mem_1 = program.alloc_memory(1).unwrap();
    let mem_2 = program.alloc_memory(1).unwrap();
    assert_eq!(program.decode_allocation_pointer(mem_0).unwrap().0, 0);
    assert_eq!(program.decode_allocation_pointer(mem_1).unwrap().0, 1);
    assert_eq!(program.decode_allocation_pointer(mem_2).unwrap().0, 2);
    program.free_memory(mem_1).unwrap();
    let mem_reused = program.alloc_memory(1).unwrap();
    assert_eq!(program.decode_allocation_pointer(mem_reused).unwrap().0, 1);
    program.free_memory(mem_0).unwrap();
    let mem_reused = program.alloc_memory(1).unwrap();
    assert_eq!(program.decode_allocation_pointer(mem_reused).unwrap().0, 0);
    let mem_new = program.alloc_memory(1).unwrap();
    assert_eq!(program.decode_allocation_pointer(mem_new).unwrap().0, 3);

    let bfile = || BFile {
        kind: BFileKind::Memory {
            bytes: Vec::new(),
            pos: 0,
        },
        readable: true,
        writable: false,
    };
    let bfile_0 = program.alloc_bfile(bfile()).unwrap();
    let bfile_1 = program.alloc_bfile(bfile()).unwrap();
    let bfile_2 = program.alloc_bfile(bfile()).unwrap();
    assert_eq!(program.decode_bfile_pointer(bfile_0).unwrap(), 0);
    assert_eq!(program.decode_bfile_pointer(bfile_1).unwrap(), 1);
    assert_eq!(program.decode_bfile_pointer(bfile_2).unwrap(), 2);
    program.close_bfile(bfile_1).unwrap();
    let bfile_reused = program.alloc_bfile(bfile()).unwrap();
    assert_eq!(program.decode_bfile_pointer(bfile_reused).unwrap(), 1);
    program.close_bfile(bfile_0).unwrap();
    let bfile_reused = program.alloc_bfile(bfile()).unwrap();
    assert_eq!(program.decode_bfile_pointer(bfile_reused).unwrap(), 0);
    let bfile_new = program.alloc_bfile(bfile()).unwrap();
    assert_eq!(program.decode_bfile_pointer(bfile_new).unwrap(), 3);

    let dir_0 = program.alloc_dir(Vec::new()).unwrap();
    let dir_1 = program.alloc_dir(Vec::new()).unwrap();
    let dir_2 = program.alloc_dir(Vec::new()).unwrap();
    assert_eq!(program.decode_dir_pointer(dir_0).unwrap(), 0);
    assert_eq!(program.decode_dir_pointer(dir_1).unwrap(), 1);
    assert_eq!(program.decode_dir_pointer(dir_2).unwrap(), 2);
    program.close_dir(dir_1).unwrap();
    let dir_reused = program.alloc_dir(Vec::new()).unwrap();
    assert_eq!(program.decode_dir_pointer(dir_reused).unwrap(), 1);
    program.close_dir(dir_0).unwrap();
    let dir_reused = program.alloc_dir(Vec::new()).unwrap();
    assert_eq!(program.decode_dir_pointer(dir_reused).unwrap(), 0);
    let dir_new = program.alloc_dir(Vec::new()).unwrap();
    assert_eq!(program.decode_dir_pointer(dir_new).unwrap(), 3);
}

#[test]
fn reduces_pointer_conversion_primitives() {
    assert_eq!(whnf(b"v8.4\n0\ntoPtr #42 @ }"), "Ptr#42");
    assert_eq!(whnf(b"v8.4\n0\ntoInt toPtr #42 @ @ }"), "42");
    assert_eq!(whnf(b"v8.4\n0\ntoFunPtr #7 @ }"), "FunPtr#7");
    assert_eq!(whnf(b"v8.4\n0\ntoInt toFunPtr #7 @ @ }"), "7");
    assert_eq!(whnf(b"v8.4\n0\ntoInt toPtr toFunPtr #9 @ @ @ }"), "9");
}

#[test]
fn reduces_foreign_pointer_primitives() {
    assert_eq!(
        whnf(b"v8.4\n0\nfp2bs bs2fp \"abcdef\" @ @ #3 @ }"),
        "\"abc\""
    );
    assert_eq!(
        whnf(b"v8.4\n0\nfp2bs fp+ bs2fp \"abcdef\" @ @ #2 @ @ #3 @ }"),
        "\"cde\""
    );
    assert_eq!(
        whnf(b"v8.4\n1\n== toInt fp2p bs2fp \"abc\" :0 @ @ @ @ toInt fp2p bs2fp _0 @ @ @ @ }"),
        "A"
    );
    assert_eq!(
        whnf(b"v8.4\n0\nIO.performIO fpnew toPtr #42 @ @ @ }"),
        "ForeignPtr#42"
    );
    assert_eq!(
        whnf(b"v8.4\n0\nseq fpfin toFunPtr #0 @ @ bs2fp \"abc\" @ @ @ #7 @ }"),
        "7"
    );
}

#[test]
fn foreign_pointer_offsets_share_backing_and_serialize() {
    let mut program = parse_program(b"v8.4\n0\nI }\n").unwrap();
    let original_node = program.foreign_ptr_node(Some(b"abcdef".to_vec()), 0, 42);
    let original = program.push_node(original_node);
    let offset = program.offset_foreign_ptr(original, 2).unwrap();
    let original_bytes = match program.cold_node(original) {
        Some(Node::ForeignPtr(foreign_ptr)) => foreign_ptr.bytes.as_ref().unwrap().clone(),
        other => panic!("expected original foreign pointer, got {other:?}"),
    };
    let offset_bytes = match program.cold_node(offset) {
        Some(Node::ForeignPtr(foreign_ptr)) => foreign_ptr.bytes.as_ref().unwrap().clone(),
        other => panic!("expected offset foreign pointer, got {other:?}"),
    };
    assert!(std::rc::Rc::ptr_eq(&original_bytes, &offset_bytes));

    let bytes = program.foreign_ptr_to_bytes(offset, 3).unwrap();
    assert_eq!(program.render(bytes), "\"cde\"");
    assert_eq!(
        program.serialize_program(offset).unwrap(),
        b"v8.4\n0\nfp+ bs2fp \"abcdef\" @ #2 @ }\n"
    );
}

#[test]
fn reduces_weak_pointer_primitives() {
    assert_eq!(
        whnf(b"v8.4\n0\nIO.performIO IO.lazyBind Wknew #0 @ #42 @ @ Wkderef @ @ #0 @ I @ }"),
        "42"
    );
    assert_eq!(
            whnf(b"v8.4\n1\nseq Wkfinal Wknew #0 @ #42 @ :0 @ @ IO.performIO Wkderef _0 @ @ #0 @ I @ @ }"),
            "42"
        );
}

#[test]
fn weak_gc_clears_value_when_key_unreachable() {
    let mut program = parse_program(b"v8.4\n0\nI }\n").unwrap();
    let key = program.push_node(Node::Int(123_456));
    let value = program.push_node(Node::Int(42));
    let weak = program.new_weak_ptr(key, value, None);

    collect_for_test(&mut program, weak);

    match program.cold_node(weak) {
        Some(Node::Weak(weak)) => {
            assert_eq!(weak.key, None);
            assert_eq!(weak.value, None);
        }
        other => panic!("expected weak node, got {other:?}"),
    }
    let deref = program.deref_weak_ptr(weak).unwrap();
    assert_eq!(program.render(deref), "K");
}

#[test]
fn weak_gc_keeps_value_when_key_reachable() {
    let mut program = parse_program(b"v8.4\n0\nI }\n").unwrap();
    let key = program.push_node(Node::Int(123_456));
    let value = program.push_node(Node::Int(42));
    let weak = program.new_weak_ptr(key, value, None);
    let root = program.app(weak, key);

    collect_for_test(&mut program, root);

    match program.cold_node(weak) {
        Some(Node::Weak(weak)) => {
            assert_eq!(weak.key, Some(key));
            assert_eq!(weak.value, Some(value));
        }
        other => panic!("expected weak node, got {other:?}"),
    }
    let deref = program.deref_weak_ptr(weak).unwrap();
    assert_eq!(
        program.cell(deref).app_fields().map(|(_, arg)| arg),
        Some(value)
    );
}

#[test]
fn reduces_mvar_primitives() {
    assert_eq!(
        whnf(b"v8.4\n0\nIO.performIO IO.lazyBind IO.newmvar @ IO.trytakemvar @ @ #0 @ I @ }"),
        "0"
    );
    assert_eq!(
            whnf(b"v8.4\n1\nseq IO.performIO IO.newmvar @ :0 @ IO.performIO IO.>> IO.putmvar _0 @ #42 @ @ IO.takemvar _0 @ @ @ @ }"),
            "42"
        );
    assert_eq!(
            whnf(b"v8.4\n1\nseq IO.performIO IO.newmvar @ :0 @ IO.performIO IO.>> IO.putmvar _0 @ #42 @ @ IO.readmvar _0 @ @ @ @ }"),
            "42"
        );
    assert_eq!(
            whnf(b"v8.4\n1\nseq IO.performIO IO.newmvar @ :0 @ IO.performIO IO.tryputmvar _0 @ #42 @ @ @ }"),
            "A"
        );
}
