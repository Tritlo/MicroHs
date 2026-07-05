use super::*;

#[test]
fn reduces_array_primitives() {
    assert_eq!(whnf(b"v8.4\n0\nA.alloc #3 @ #7 @ }"), "[7, 7, 7]");
    assert_eq!(whnf(b"v8.4\n0\nA.size A.alloc #3 @ #7 @ @ }"), "3");
    assert_eq!(whnf(b"v8.4\n0\nA.read A.alloc #3 @ #7 @ @ #1 @ }"), "7");
    assert_eq!(whnf(b"v8.4\n1\nA.== #0 [1] :0 @ _0 @ }"), "A");
    assert_eq!(whnf(b"v8.4\n1\nA.== #0 [1] :0 @ A.copy _0 @ @ }"), "K");
    assert_eq!(
        whnf(b"v8.4\n1\nseq A.write #0 #0 #0 [3] :0 @ #1 @ #42 @ @ A.read _0 @ #1 @ @ }"),
        "42"
    );
    assert_eq!(
        whnf(b"v8.4\n1\nseq A.trunc #0 #0 #0 [3] :0 @ #1 @ @ A.size _0 @ @ }"),
        "1"
    );
    assert_eq!(whnf(b"v8.4\n1\nA.== A.alloc #1 @ #0 @ :0 @ _0 @ }"), "A");
    assert_eq!(
        whnf(b"v8.4\n1\nseq A.write A.alloc #3 @ #0 @ :0 @ #1 @ #42 @ @ A.read _0 @ #1 @ @ }"),
        "42"
    );
}

#[test]
fn reduces_io_control_primitives() {
    assert_eq!(whnf(b"v8.4\n0\nIO.performIO IO.return #5 @ @ }"), "5");
    assert_eq!(
        whnf(b"v8.4\n0\nIO.performIO IO.>> IO.return #1 @ @ IO.return #7 @ @ @ }"),
        "7"
    );
    assert_eq!(
        whnf(b"v8.4\n0\nIO.performIO IO.>>= IO.return #1 @ @ K IO.return #7 @ @ @ @ }"),
        "7"
    );
    assert_eq!(
        whnf(b"v8.4\n0\nIO.performIO IO.lazyBind IO.return #3 @ @ IO.return @ @ }"),
        "3"
    );
    assert_eq!(
        whnf(b"v8.4\n0\nIO.performIO IO.strict IO.return #4 @ @ @ }"),
        "4"
    );
    assert_eq!(
        whnf(b"v8.4\n0\nIO.performIO IO.atomic IO.return #6 @ @ @ }"),
        "6"
    );
    assert_eq!(whnf(b"v8.4\n0\nIO.performIO IO.gc #0 @ @ }"), "I");
    assert_eq!(whnf(b"v8.4\n0\nIO.performIO IO.yield @ }"), "I");
    assert_eq!(whnf(b"v8.4\n0\nIO.performIO IO.getmaskingstate @ }"), "0");
    assert_eq!(
        whnf(b"v8.4\n0\nIO.performIO IO.>> IO.setmaskingstate #2 @ @ IO.getmaskingstate @ @ }"),
        "2"
    );
    assert_eq!(whnf(b"v8.4\n0\nthnum IO.performIO IO.thid @ @ }"), "1");
    assert_eq!(
        whnf(b"v8.4\n0\nIO.performIO IO.threadstatus IO.performIO IO.thid @ @ @ }"),
        "0"
    );
}

#[test]
fn ignored_io_shortcut_has_depth_cap() {
    fn ignored_chain(program: &mut Program, len: usize) -> NodeId {
        let unit = program.prim("I");
        let ret = program.prim("IO.return");
        let unit_action = program.app(ret, unit);
        let result = program.push_node(Node::Int(7));
        let ret = program.prim("IO.return");
        let mut action = program.app(ret, result);
        for _ in 0..len {
            let then = program.prim("IO.>>");
            let left = program.app(then, unit_action);
            action = program.app(left, action);
        }
        action
    }

    let mut program = parse_program(b"v8.4\n0\nI }\n").unwrap();
    let bounded = ignored_chain(&mut program, IGNORED_IO_SHORTCUT_RECURSION_LIMIT / 2);
    assert!(
        program
            .ignored_io_action_reductions(bounded, usize::MAX)
            .unwrap()
            .is_some()
    );

    let mut program = parse_program(b"v8.4\n0\nI }\n").unwrap();
    let over_cap = ignored_chain(&mut program, IGNORED_IO_SHORTCUT_RECURSION_LIMIT + 1);
    assert_eq!(
        program
            .ignored_io_action_reductions(over_cap, usize::MAX)
            .unwrap(),
        None
    );

    let perform_io = program.prim("IO.performIO");
    program.root = program.app(perform_io, over_cap);
    let (root, _) = program.reduce_whnf(20_000).unwrap();
    assert_eq!(program.render(root), "7");
}

#[test]
fn reduces_array_primitives_as_io_actions() {
    assert_eq!(
        whnf(b"v8.4\n0\nIO.performIO A.alloc #3 @ #7 @ @ }"),
        "[7, 7, 7]"
    );
    assert_eq!(whnf(b"v8.4\n0\nIO.performIO A.size #0 #0 [2] @ @ }"), "2");
    assert_eq!(whnf(b"v8.4\n0\nIO.performIO A.copy #0 [1] @ @ }"), "[0]");
    assert_eq!(
            whnf(b"v8.4\n1\nIO.performIO IO.>> A.write #0 #0 #0 [3] :0 @ #1 @ #42 @ @ A.read _0 @ #1 @ @ @ }"),
            "42"
        );
}
