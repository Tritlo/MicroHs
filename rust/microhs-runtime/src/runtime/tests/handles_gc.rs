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
