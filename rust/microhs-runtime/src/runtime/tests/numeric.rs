use super::*;

#[test]
fn mpz_get_d_uses_decimal_rounding_like_c() {
    let decimal = b"-299228957055072645483636";
    let value = MpzValue::parse_decimal(decimal).unwrap();
    let expected = std::str::from_utf8(decimal)
        .unwrap()
        .parse::<f64>()
        .unwrap();
    assert_eq!(value.to_f64().to_bits(), expected.to_bits());
}

#[test]
fn reduces_integer_arithmetic() {
    assert_eq!(whnf(b"v8.4\n0\n+ #40 @ #2 @ }"), "42");
    assert_eq!(whnf(b"v8.4\n0\nsubtract #10 @ #3 @ }"), "-7");
    assert_eq!(whnf(b"v8.4\n0\nquot #22 @ #5 @ }"), "4");
    assert_eq!(whnf(b"v8.4\n0\nrem #22 @ #5 @ }"), "2");
}

#[test]
fn reduces_integer_bit_ops() {
    assert_eq!(whnf(b"v8.4\n0\nand #6 @ #3 @ }"), "2");
    assert_eq!(whnf(b"v8.4\n0\nor #4 @ #1 @ }"), "5");
    assert_eq!(whnf(b"v8.4\n0\nshl #3 @ #2 @ }"), "12");
    assert_eq!(whnf(b"v8.4\n0\npopcount #7 @ }"), "3");
}

#[test]
fn reduces_integer_comparisons_to_microhs_bools() {
    assert_eq!(whnf(b"v8.4\n0\n== #2 @ #2 @ }"), "A");
    assert_eq!(whnf(b"v8.4\n0\n< #2 @ #1 @ }"), "K");
    assert_eq!(whnf(b"v8.4\n0\nu> #-1 @ #1 @ }"), "A");
    assert_eq!(whnf(b"v8.4\n0\nicmp #1 @ #2 @ }"), "K2");
    assert_eq!(whnf(b"v8.4\n0\nucmp #-1 @ #1 @ }"), "KA");
}

#[test]
fn reduces_int64_primitives() {
    assert_eq!(whnf(b"v8.4\n0\nI+ ##40 @ ##2 @ }"), "42i64");
    assert_eq!(whnf(b"v8.4\n0\nIsubtract ##10 @ ##3 @ }"), "-7i64");
    assert_eq!(whnf(b"v8.4\n0\nIquot ##22 @ ##5 @ }"), "4i64");
    assert_eq!(whnf(b"v8.4\n0\nIand ##6 @ ##3 @ }"), "2i64");
    assert_eq!(whnf(b"v8.4\n0\nIshl ##3 @ #2 @ }"), "12i64");
    assert_eq!(whnf(b"v8.4\n0\nIpopcount ##7 @ }"), "3");
    assert_eq!(whnf(b"v8.4\n0\nI== ##2 @ ##2 @ }"), "A");
    assert_eq!(whnf(b"v8.4\n0\nIu> ##-1 @ ##1 @ }"), "A");
    assert_eq!(whnf(b"v8.4\n0\nIicmp ##1 @ ##2 @ }"), "K2");
    assert_eq!(whnf(b"v8.4\n0\nIucmp ##-1 @ ##1 @ }"), "KA");
    assert_eq!(whnf(b"v8.4\n0\nitoI #7 @ }"), "7i64");
    assert_eq!(whnf(b"v8.4\n0\nItoi ##7 @ }"), "7");
}

#[test]
fn reduces_float64_primitives() {
    assert_eq!(whnf(b"v8.4\n0\nd+ &1.5 @ &2.25 @ }"), "3.75");
    assert_eq!(whnf(b"v8.4\n0\nd* &3 @ &2.5 @ }"), "7.5");
    assert_eq!(whnf(b"v8.4\n0\ndneg &1.5 @ }"), "-1.5");
    assert_eq!(whnf(b"v8.4\n0\nd< &1.5 @ &2.25 @ }"), "A");
    assert_eq!(whnf(b"v8.4\n0\nd== &1.5 @ &2.25 @ }"), "K");
    assert_eq!(whnf(b"v8.4\n0\nitod #7 @ }"), "7.0");
    assert_eq!(whnf(b"v8.4\n0\nItod ##7 @ }"), "7.0");
    assert_eq!(whnf(b"v8.4\n0\ndtoi &7.75 @ }"), "7");
    assert_eq!(whnf(b"v8.4\n0\nd> utod #-1 @ @ &1000 @ }"), "A");
    assert_eq!(whnf(b"v8.4\n0\ntoDbl fromDbl &1.5 @ @ }"), "1.5");
}

#[test]
fn reduces_float32_primitives() {
    assert_eq!(whnf(b"v8.4\n0\nf+ &&1.5 @ &&2.25 @ }"), "3.75f");
    assert_eq!(whnf(b"v8.4\n0\nf* &&3 @ &&2.5 @ }"), "7.5f");
    assert_eq!(whnf(b"v8.4\n0\nfneg &&1.5 @ }"), "-1.5f");
    assert_eq!(whnf(b"v8.4\n0\nf< &&1.5 @ &&2.25 @ }"), "A");
    assert_eq!(whnf(b"v8.4\n0\nf== &&1.5 @ &&2.25 @ }"), "K");
    assert_eq!(whnf(b"v8.4\n0\nitof #7 @ }"), "7.0f");
    assert_eq!(whnf(b"v8.4\n0\nItof ##7 @ }"), "7.0f");
    assert_eq!(whnf(b"v8.4\n0\nftoi &&7.75 @ }"), "7");
    assert_eq!(whnf(b"v8.4\n0\nf> utof #-1 @ @ &&1000 @ }"), "A");
    assert_eq!(whnf(b"v8.4\n0\nftod dtof &1.5 @ @ }"), "1.5");
    assert_eq!(whnf(b"v8.4\n0\ntoFlt fromFlt &&1.5 @ @ }"), "1.5f");
}
