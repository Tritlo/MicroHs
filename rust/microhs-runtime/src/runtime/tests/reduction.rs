use super::*;

#[test]
fn reduces_identity() {
    assert_eq!(whnf(b"v8.4\n0\nI #42 @ }"), "42");
}

#[test]
fn reduces_skk_identity() {
    assert_eq!(whnf(b"v8.4\n0\nS K @ K @ #7 @ }"), "7");
}

#[test]
fn reduces_optimizer_combinators() {
    assert_eq!(whnf(b"v8.4\n0\nS' K @ K @ K @ #5 @ #0 @ }"), "5");
    assert_eq!(whnf(b"v8.4\n0\nB' K @ #5 @ K @ #0 @ }"), "5");
    assert_eq!(whnf(b"v8.4\n0\nZ K @ #5 @ #0 @ #1 @ }"), "5");
    assert_eq!(whnf(b"v8.4\n0\nJ #5 @ #0 @ I @ }"), "5");
    assert_eq!(whnf(b"v8.4\n0\nL #5 @ I @ #0 @ }"), "5");
    assert_eq!(whnf(b"v8.4\n0\nKK #0 @ #5 @ #1 @ }"), "5");
    assert_eq!(whnf(b"v8.4\n0\nKA #0 @ #1 @ #5 @ }"), "5");
    assert_eq!(whnf(b"v8.4\n0\nC' A @ K @ #5 @ #0 @ }"), "5");
    assert_eq!(whnf(b"v8.4\n0\nR #0 @ K @ #5 @ }"), "5");
    assert_eq!(whnf(b"v8.4\n0\nO #5 @ #0 @ #1 @ K @ }"), "5");
    assert_eq!(whnf(b"v8.4\n0\nC'B K @ K @ #5 @ #0 @ }"), "5");
}

#[test]
fn reduces_partial_arity_specializations() {
    assert_eq!(whnf(b"v8.4\n0\nB' I @ I @ #9 @ }"), "((B (I I)) 9)");
    assert_eq!(whnf(b"v8.4\n0\nZ K @ #5 @ }"), "(K (K 5))");
    assert_eq!(whnf(b"v8.4\n0\nR #1 @ #2 @ }"), "((C 2) 1)");
    assert_eq!(whnf(b"v8.4\n0\nK2 #1 @ #2 @ }"), "(K 1)");
    assert_eq!(whnf(b"v8.4\n0\nK3 #1 @ #2 @ #3 @ }"), "(K 1)");
    assert_eq!(whnf(b"v8.4\n0\nC'B K @ I @ #9 @ }"), "((B (K 9)) I)");
}

#[test]
fn reduces_constructor_tags_and_tuples() {
    assert_eq!(whnf(b"v8.4\n0\nTAG3 #99 @ K @ }"), "3");
    assert_eq!(whnf(b"v8.4\n0\nTAG10 #99 @ A @ }"), "99");
    assert_eq!(whnf(b"v8.4\n0\nT3 #1 @ #2 @ #3 @ K3 @ #0 @ }"), "1");
    assert_eq!(whnf(b"v8.4\n0\nT4 #1 @ #2 @ #3 @ #4 @ K4 @ #0 @ }"), "1");
}

#[test]
fn resolves_shared_labels() {
    assert_eq!(whnf(b"v8.4\n1\nA #42 :0 @ _0 @ }"), "42");
}

#[test]
fn reduces_strict_alias_and_probe_primitives() {
    assert_eq!(whnf(b"v8.4\n0\nord #65 @ }"), "65");
    assert_eq!(whnf(b"v8.4\n0\nchr #65 @ }"), "65");
    assert_eq!(whnf(b"v8.4\n0\nseq + #1 @ #2 @ @ #9 @ }"), "9");
    assert_eq!(whnf(b"v8.4\n0\nisint #7 @ }"), "7");
    assert_eq!(whnf(b"v8.4\n0\nisint \"x\" @ }"), "-1");
}

#[test]
fn rejects_unknown_primitives() {
    assert!(matches!(
        parse_program(b"v8.4\n0\nnot-a-prim }"),
        Err(ParseError::UnknownPrim(name)) if name == "not-a-prim"
    ));

    let mut unsupported = parse_program(b"v8.4\n0\nIO.waitrdfd #1 @ }").unwrap();
    assert!(matches!(
        unsupported.reduce_whnf(10),
        Err(EvalError::UnknownPrim(name)) if name == "IO.waitrdfd"
    ));
}
