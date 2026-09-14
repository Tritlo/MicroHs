;; services-math.wat: floating-point math services for the standalone runtime.
;;
;; This fragment is part of the standalone WAT support module. The parent
;; concatenates it into that module. It defines this entry point:
;;
;;   $service_math(id, args) -> node
;;
;; The evaluator calls it for a fixed runtime service. `id` is a service
;; index from names.wat. `args` is the primitive argument vector. Each
;; argument is a node pointer that is already in weak head normal form. The
;; result is a boxed Double or Float node. The result is zero when this
;; fragment does not own the service. This fragment never raises an
;; exception, never invokes the evaluator, and never invokes the collector.
;; It calls $double and $float to box results. Those allocate one node and
;; do not collect.
;;
;; Services owned here (names.wat service IDs):
;;
;;   Double -> Double        acos asin atan cos exp log sin sqrt tan
;;   Double -> Double -> Double   atan2 pow
;;   Double -> Int -> Double      scalbn
;;   Float versions of the same    acosf ... scalbnf powf
;;
;; The Haskell side is lib/Data/Double.hs and lib/Data/Float.hs. They call
;; these names through `foreign import ccall`. The argument order is the C
;; order: atan2(y, x), pow(x, y), scalbn(x, n).
;;
;; PROVENANCE AND LICENSE
;;
;; The Double algorithms are ports of the FreeBSD msun / Sun fdlibm sources
;; as they appear in musl libc and in the rust-lang/libm crate (MIT). The
;; original notice for those files:
;;
;;   Copyright (C) 1993, 2004 by Sun Microsystems, Inc. All rights reserved.
;;   Developed at SunSoft, a Sun Microsystems, Inc. business.
;;   Permission to use, copy, modify, and distribute this software is freely
;;   granted, provided that this notice is preserved.
;;
;; Each function below names its source file. The polynomial coefficients
;; and split constants are copied exactly. The hex words in comments are the
;; intended bit patterns. The decimal literals produce those bit patterns
;; under correct decimal-to-binary conversion, which wasm-as performs.
;;
;; ACCURACY
;;
;; The fdlibm error analysis bounds every Double function here to less than
;; 1 ulp. sqrt uses the native f64.sqrt instruction and is correctly rounded.
;; scalbn is exact except for one final rounding when the result is subnormal.
;; See MATH.md for the measured ulp errors against MPFR.
;;
;; The Float functions compute in Double and round once to Float, except
;; sqrtf (native f32.sqrt) and scalbnf (exact). A Double result carries at
;; least 29 extra bits, so the final rounding is correct except when the
;; true value lies within about 2^-29 ulp of a Float rounding boundary.
;;
;; IEEE SEMANTICS
;;
;; Special cases follow the C standard and fdlibm: NaN arguments give NaN,
;; signed zeros are preserved where the function is odd, log(0) is -inf,
;; log(negative) is NaN, exp overflows to +inf, pow follows Annex F including
;; pow(x, 0) = 1 and pow(1, y) = 1 for NaN x or y. Huge arguments to sin, cos,
;; and tan use Payne-Hanek reduction with 66 words of 2/pi, which covers the
;; whole Double range. No floating-point exception flags exist in Wasm, so the
;; flag-raising expressions from the sources are omitted.
;;
;; STATIC MEMORY
;;
;; This fragment uses part of the static range 0xe800..0xf7ff. The table is
;; read-only after instantiation. The scratch areas are written and read
;; within one call. Nothing in them survives a call, so the collector does
;; not scan them.
;;
;;   0xe800  ipio2[66]  i32   bits of 2/pi, 24 bits per word (data segment)
;;   0xea00  f[20]      f64   rem_pio2_large scratch
;;   0xeaa0  q[20]      f64   rem_pio2_large scratch
;;   0xeb40  fq[20]     f64   rem_pio2_large scratch
;;   0xebe0  iq[20]     i32   rem_pio2_large scratch
;;   0xec40  tx[3]      f64   24-bit pieces of |x| for rem_pio2_large
;;   0xec60  y[2]       f64   rem_pio2 result y0, y1 (double-double)
;;
;; wasm-as 108 does not accept stack-style consumption of multi-value
;; results, so $math_rem_pio2 returns n and writes y0, y1 to 0xec60.

(data (i32.const 0xe800)
  "\83\f9\a2\00\44\4e\6e\00\fc\29\15\00\d1\57\27\00\dd\34\f5\00\62\db\c0\00" ;; ipio2[0..5]
  "\3c\99\95\00\41\90\43\00\63\51\fe\00\bb\de\ab\00\b7\61\c5\00\3a\6e\24\00" ;; ipio2[6..11]
  "\d2\4d\42\00\49\06\e0\00\09\ea\2e\00\1c\92\d1\00\eb\1d\fe\00\29\b1\1c\00" ;; ipio2[12..17]
  "\e8\3e\a7\00\f5\35\82\00\44\bb\2e\00\9c\e9\84\00\b4\26\70\00\41\7e\5f\00" ;; ipio2[18..23]
  "\d6\91\39\00\53\83\39\00\9c\f4\39\00\8b\5f\84\00\28\f9\bd\00\f8\1f\3b\00" ;; ipio2[24..29]
  "\de\ff\97\00\0f\98\05\00\11\2f\ef\00\0a\5a\8b\00\6d\1f\6d\00\cf\7e\36\00" ;; ipio2[30..35]
  "\09\cb\27\00\46\4f\b7\00\9e\66\3f\00\2d\ea\5f\00\ba\27\75\00\e5\eb\c7\00" ;; ipio2[36..41]
  "\3d\7b\f1\00\f7\39\07\00\92\52\8a\00\fb\6b\ea\00\1f\b1\5f\00\08\5d\8d\00" ;; ipio2[42..47]
  "\30\03\56\00\7b\fc\46\00\f0\ab\6b\00\20\bc\cf\00\36\f4\9a\00\e3\a9\1d\00" ;; ipio2[48..53]
  "\5e\61\91\00\08\1b\e6\00\85\99\65\00\a0\14\5f\00\8d\40\68\00\80\d8\ff\00" ;; ipio2[54..59]
  "\27\73\4d\00\06\06\31\00\ca\56\15\00\c9\a8\73\00\7b\e2\60\00\6b\8c\c0\00" ;; ipio2[60..65]
)

;; ---------------------------------------------------------------------------
;; Service dispatch
;; ---------------------------------------------------------------------------

(func $service_math (param $id i32) (param $args i32) (result i32)
  (local $a i32) (local $b i32)
  (local.set $a (call $value_arg (local.get $args) (i32.const 0)))
  (local.set $b (call $value_arg (local.get $args) (i32.const 1)))
  ;; Double
  (if (i32.eq (local.get $id) (global.get $svc_sqrt))
    (then (return (call $double (f64.sqrt (call $dval (local.get $a)))))))
  (if (i32.eq (local.get $id) (global.get $svc_scalbn))
    (then (return (call $double (call $math_scalbn (call $dval (local.get $a)) (call $ival (local.get $b)))))))
  (if (i32.eq (local.get $id) (global.get $svc_exp))
    (then (return (call $double (call $math_exp (call $dval (local.get $a)))))))
  (if (i32.eq (local.get $id) (global.get $svc_log))
    (then (return (call $double (call $math_log (call $dval (local.get $a)))))))
  (if (i32.eq (local.get $id) (global.get $svc_sin))
    (then (return (call $double (call $math_sin (call $dval (local.get $a)))))))
  (if (i32.eq (local.get $id) (global.get $svc_cos))
    (then (return (call $double (call $math_cos (call $dval (local.get $a)))))))
  (if (i32.eq (local.get $id) (global.get $svc_tan))
    (then (return (call $double (call $math_tan (call $dval (local.get $a)))))))
  (if (i32.eq (local.get $id) (global.get $svc_atan))
    (then (return (call $double (call $math_atan (call $dval (local.get $a)))))))
  (if (i32.eq (local.get $id) (global.get $svc_asin))
    (then (return (call $double (call $math_asin (call $dval (local.get $a)))))))
  (if (i32.eq (local.get $id) (global.get $svc_acos))
    (then (return (call $double (call $math_acos (call $dval (local.get $a)))))))
  (if (i32.eq (local.get $id) (global.get $svc_atan2))
    (then (return (call $double (call $math_atan2 (call $dval (local.get $a)) (call $dval (local.get $b)))))))
  (if (i32.eq (local.get $id) (global.get $svc_pow))
    (then (return (call $double (call $math_pow (call $dval (local.get $a)) (call $dval (local.get $b)))))))
  ;; Float
  (if (i32.eq (local.get $id) (global.get $svc_sqrtf))
    (then (return (call $float (f32.sqrt (call $fval (local.get $a)))))))
  (if (i32.eq (local.get $id) (global.get $svc_scalbnf))
    (then (return (call $float (call $math_scalbnf (call $fval (local.get $a)) (call $ival (local.get $b)))))))
  (if (i32.eq (local.get $id) (global.get $svc_expf))
    (then (return (call $float (f32.demote_f64 (call $math_exp (f64.promote_f32 (call $fval (local.get $a)))))))))
  (if (i32.eq (local.get $id) (global.get $svc_logf))
    (then (return (call $float (f32.demote_f64 (call $math_log (f64.promote_f32 (call $fval (local.get $a)))))))))
  (if (i32.eq (local.get $id) (global.get $svc_sinf))
    (then (return (call $float (f32.demote_f64 (call $math_sin (f64.promote_f32 (call $fval (local.get $a)))))))))
  (if (i32.eq (local.get $id) (global.get $svc_cosf))
    (then (return (call $float (f32.demote_f64 (call $math_cos (f64.promote_f32 (call $fval (local.get $a)))))))))
  (if (i32.eq (local.get $id) (global.get $svc_tanf))
    (then (return (call $float (f32.demote_f64 (call $math_tan (f64.promote_f32 (call $fval (local.get $a)))))))))
  (if (i32.eq (local.get $id) (global.get $svc_atanf))
    (then (return (call $float (f32.demote_f64 (call $math_atan (f64.promote_f32 (call $fval (local.get $a)))))))))
  (if (i32.eq (local.get $id) (global.get $svc_asinf))
    (then (return (call $float (f32.demote_f64 (call $math_asin (f64.promote_f32 (call $fval (local.get $a)))))))))
  (if (i32.eq (local.get $id) (global.get $svc_acosf))
    (then (return (call $float (f32.demote_f64 (call $math_acos (f64.promote_f32 (call $fval (local.get $a)))))))))
  (if (i32.eq (local.get $id) (global.get $svc_atan2f))
    (then (return (call $float (f32.demote_f64 (call $math_atan2
      (f64.promote_f32 (call $fval (local.get $a)))
      (f64.promote_f32 (call $fval (local.get $b)))))))))
  (if (i32.eq (local.get $id) (global.get $svc_powf))
    (then (return (call $float (f32.demote_f64 (call $math_pow
      (f64.promote_f32 (call $fval (local.get $a)))
      (f64.promote_f32 (call $fval (local.get $b)))))))))
  (i32.const 0))

;; ---------------------------------------------------------------------------
;; Bit-level helpers. A Double is {high word, low word}. The high word holds
;; the sign, the 11 exponent bits, and the top 20 significand bits.
;; ---------------------------------------------------------------------------

(func $math_hw (param $x f64) (result i32)
  (i32.wrap_i64 (i64.shr_u (i64.reinterpret_f64 (local.get $x)) (i64.const 32))))

(func $math_lw (param $x f64) (result i32)
  (i32.wrap_i64 (i64.reinterpret_f64 (local.get $x))))

;; Clear the low word. This keeps the top 21 significand bits.
(func $math_trunc_lw (param $x f64) (result f64)
  (f64.reinterpret_i64 (i64.and (i64.reinterpret_f64 (local.get $x))
    (i64.const 0xffffffff00000000))))

;; Replace the high word.
(func $math_set_hw (param $x f64) (param $h i32) (result f64)
  (f64.reinterpret_i64 (i64.or
    (i64.and (i64.reinterpret_f64 (local.get $x)) (i64.const 0xffffffff))
    (i64.shl (i64.extend_i32_u (local.get $h)) (i64.const 32)))))

;; Build a Double from a high word and a zero low word.
(func $math_from_hw (param $h i32) (result f64)
  (f64.reinterpret_i64 (i64.shl (i64.extend_i32_u (local.get $h)) (i64.const 32))))

;; 2^n for -1022 <= n <= 1023.
(func $math_pow2 (param $n i32) (result f64)
  (call $math_from_hw (i32.shl (i32.add (local.get $n) (i32.const 0x3ff)) (i32.const 20))))

;; ---------------------------------------------------------------------------
;; scalbn(x, n) = x * 2^n, exact. Source: rust-lang/libm generic/scalbn.rs.
;;
;; A direct multiply by 2^n fails when 2^n is not representable but the
;; result is. The scale is applied in at most three steps. Each step uses a
;; power of two that is representable. Only the last step can round, and it
;; rounds only when the result is subnormal, so the result is the correctly
;; rounded value of x * 2^n. Large |n| is clamped after two steps; the clamp
;; cannot change the result because the value has already overflowed or
;; underflowed at that point.
;; ---------------------------------------------------------------------------

(func $math_scalbn (param $x f64) (param $n i32) (result f64)
  (if (i32.gt_s (local.get $n) (i32.const 1023))
    (then
      (local.set $x (f64.mul (local.get $x) (f64.const 0x1p1023)))
      (local.set $n (i32.sub (local.get $n) (i32.const 1023)))
      (if (i32.gt_s (local.get $n) (i32.const 1023))
        (then
          (local.set $x (f64.mul (local.get $x) (f64.const 0x1p1023)))
          (local.set $n (i32.sub (local.get $n) (i32.const 1023)))
          (if (i32.gt_s (local.get $n) (i32.const 1023))
            (then (local.set $n (i32.const 1023)))))))
    (else
      (if (i32.lt_s (local.get $n) (i32.const -1022))
        (then
          ;; 2^-1022 * 2^53: keeps x normal for one more step.
          (local.set $x (f64.mul (local.get $x) (f64.const 0x1p-969)))
          (local.set $n (i32.add (local.get $n) (i32.const 969)))
          (if (i32.lt_s (local.get $n) (i32.const -1022))
            (then
              (local.set $x (f64.mul (local.get $x) (f64.const 0x1p-969)))
              (local.set $n (i32.add (local.get $n) (i32.const 969)))
              (if (i32.lt_s (local.get $n) (i32.const -1022))
                (then (local.set $n (i32.const -1022))))))))))
  (f64.mul (local.get $x) (call $math_pow2 (local.get $n))))

;; scalbnf(x, n) = x * 2^n for Float. Same scheme with the Float limits:
;; exponent range -126..127, 24 significand bits, so the small step is
;; 2^-126 * 2^24 = 2^-102.
(func $math_scalbnf (param $x f32) (param $n i32) (result f32)
  (if (i32.gt_s (local.get $n) (i32.const 127))
    (then
      (local.set $x (f32.mul (local.get $x) (f32.const 0x1p127)))
      (local.set $n (i32.sub (local.get $n) (i32.const 127)))
      (if (i32.gt_s (local.get $n) (i32.const 127))
        (then
          (local.set $x (f32.mul (local.get $x) (f32.const 0x1p127)))
          (local.set $n (i32.sub (local.get $n) (i32.const 127)))
          (if (i32.gt_s (local.get $n) (i32.const 127))
            (then (local.set $n (i32.const 127)))))))
    (else
      (if (i32.lt_s (local.get $n) (i32.const -126))
        (then
          (local.set $x (f32.mul (local.get $x) (f32.const 0x1p-102)))
          (local.set $n (i32.add (local.get $n) (i32.const 102)))
          (if (i32.lt_s (local.get $n) (i32.const -126))
            (then
              (local.set $x (f32.mul (local.get $x) (f32.const 0x1p-102)))
              (local.set $n (i32.add (local.get $n) (i32.const 102)))
              (if (i32.lt_s (local.get $n) (i32.const -126))
                (then (local.set $n (i32.const -126))))))))))
  (f32.mul (local.get $x)
    (f32.reinterpret_i32 (i32.shl (i32.add (local.get $n) (i32.const 127)) (i32.const 23)))))

;; ---------------------------------------------------------------------------
;; exp(x). Source: FreeBSD msun e_exp.c (Sun, 2004) via musl exp.c.
;;
;; Method
;;   1. Argument reduction: x = k*ln2 + r, |r| <= 0.5*ln2. r is kept as
;;      hi - lo, where hi = x - k*ln2hi is exact and lo = k*ln2lo.
;;   2. exp(r) on [0, 0.34658] from the Remez polynomial c(r) of degree 5 in
;;      r^2 with |error| <= 2^-59:
;;        exp(r) = 1 + r + r*c(r)/(2 - c(r)),  c(r) = r - r^2*P(r^2)
;;   3. exp(x) = 2^k * exp(r) through scalbn.
;;
;; Special cases: exp(+inf) = +inf, exp(-inf) = 0, exp(NaN) = NaN.
;; Overflow when x > 709.782712893383973096; underflow to zero when
;; x < -745.133219101941108420. Error < 1 ulp.
;; ---------------------------------------------------------------------------

(func $math_exp (param $x f64) (result f64)
  (local $hx i32) (local $sign i32) (local $k i32)
  (local $hi f64) (local $lo f64) (local $c f64) (local $xx f64) (local $y f64)
  (local.set $hx (call $math_hw (local.get $x)))
  (local.set $sign (i32.shr_u (local.get $hx) (i32.const 31)))
  (local.set $hx (i32.and (local.get $hx) (i32.const 0x7fffffff)))
  ;; |x| >= 708.39...: NaN, overflow, underflow.
  (if (i32.ge_u (local.get $hx) (i32.const 0x4086232b))
    (then
      (if (f64.ne (local.get $x) (local.get $x)) (then (return (local.get $x))))
      (if (f64.gt (local.get $x) (f64.const 709.782712893383973096))
        (then (return (f64.mul (local.get $x) (f64.const 0x1p1023)))))
      (if (f64.lt (local.get $x) (f64.const -745.13321910194110842))
        (then (return (f64.const 0))))))
  ;; Argument reduction.
  (if (i32.gt_u (local.get $hx) (i32.const 0x3fd62e42)) ;; |x| > 0.5 ln2
    (then
      (if (i32.ge_u (local.get $hx) (i32.const 0x3ff0a2b2)) ;; |x| >= 1.5 ln2
        (then
          (local.set $k (i32.trunc_f64_s (f64.add
            (f64.mul (f64.const 1.44269504088896338700e+00) (local.get $x)) ;; 0x3ff71547_652b82fe 1/ln2
            (select (f64.const -0.5) (f64.const 0.5) (local.get $sign))))))
        (else
          (local.set $k (i32.sub (i32.const 1) (i32.shl (local.get $sign) (i32.const 1))))))
      ;; k*ln2hi is exact because ln2hi has trailing zero bits.
      (local.set $hi (f64.sub (local.get $x)
        (f64.mul (f64.convert_i32_s (local.get $k)) (f64.const 6.93147180369123816490e-01)))) ;; 0x3fe62e42_fee00000
      (local.set $lo (f64.mul (f64.convert_i32_s (local.get $k)) (f64.const 1.90821492927058770002e-10))) ;; 0x3dea39ef_35793c76
      (local.set $x (f64.sub (local.get $hi) (local.get $lo))))
    (else
      (if (i32.gt_u (local.get $hx) (i32.const 0x3e300000)) ;; |x| > 2^-28
        (then
          (local.set $k (i32.const 0))
          (local.set $hi (local.get $x))
          (local.set $lo (f64.const 0)))
        (else
          (return (f64.add (f64.const 1) (local.get $x)))))))
  ;; x is in the primary range.
  (local.set $xx (f64.mul (local.get $x) (local.get $x)))
  (local.set $c (f64.sub (local.get $x) (f64.mul (local.get $xx)
    (f64.add (f64.const 1.66666666666666019037e-01) (f64.mul (local.get $xx)          ;; P1 0x3fc55555_5555553e
    (f64.add (f64.const -2.77777777770155933842e-03) (f64.mul (local.get $xx)         ;; P2 0xbf66c16c_16bebd93
    (f64.add (f64.const 6.61375632143793436117e-05) (f64.mul (local.get $xx)          ;; P3 0x3f11566a_af25de2c
    (f64.add (f64.const -1.65339022054652515390e-06) (f64.mul (local.get $xx)         ;; P4 0xbebbbd41_c5d26bf1
             (f64.const 4.13813679705723846039e-08))))))))))))                        ;; P5 0x3e663769_72bea4d0
  (local.set $y (f64.add (f64.const 1)
    (f64.add
      (f64.sub (f64.div (f64.mul (local.get $x) (local.get $c)) (f64.sub (f64.const 2) (local.get $c)))
               (local.get $lo))
      (local.get $hi))))
  (if (i32.eqz (local.get $k)) (then (return (local.get $y))))
  (call $math_scalbn (local.get $y) (local.get $k)))

;; ---------------------------------------------------------------------------
;; log(x). Source: FreeBSD msun e_log.c (Sun, 1993) via musl log.c.
;;
;; Method
;;   1. Argument reduction: x = 2^k * (1+f), sqrt(2)/2 < 1+f < sqrt(2).
;;   2. With s = f/(2+f), log(1+f) = 2s + s*R(s^2). R is a degree-14 Remez
;;      polynomial in s with |error| <= 2^-58.45. To keep the error below
;;      1 ulp the code evaluates
;;        log(1+f) = f - (hfsq - s*(hfsq + R)),  hfsq = f*f/2.
;;   3. log(x) = k*ln2_hi + (f - (hfsq - (s*(hfsq+R) + k*ln2_lo))).
;;      k*ln2_hi is exact for |k| < 2000.
;;
;; Special cases: log(+-0) = -inf, log(x<0) = NaN, log(+inf) = +inf,
;; log(NaN) = NaN, log(1) = +0. Error < 1 ulp.
;; ---------------------------------------------------------------------------

(func $math_log (param $x f64) (result f64)
  (local $hx i32) (local $k i32)
  (local $f f64) (local $hfsq f64) (local $s f64) (local $z f64) (local $w f64)
  (local $t1 f64) (local $t2 f64) (local $r f64) (local $dk f64)
  (local.set $hx (call $math_hw (local.get $x)))
  (local.set $k (i32.const 0))
  (if (i32.or (i32.lt_u (local.get $hx) (i32.const 0x00100000))
              (i32.shr_u (local.get $hx) (i32.const 31)))
    (then
      ;; x < 2^-1022, or x is negative
      (if (i64.eqz (i64.shl (i64.reinterpret_f64 (local.get $x)) (i64.const 1)))
        (then (return (f64.const -inf))))
      (if (i32.shr_u (local.get $hx) (i32.const 31))
        (then (return (f64.div (f64.sub (local.get $x) (local.get $x)) (f64.const 0)))))
      ;; subnormal: scale x up by 2^54
      (local.set $k (i32.const -54))
      (local.set $x (f64.mul (local.get $x) (f64.const 0x1p54)))
      (local.set $hx (call $math_hw (local.get $x))))
    (else
      (if (i32.ge_u (local.get $hx) (i32.const 0x7ff00000))
        (then (return (local.get $x))))
      (if (i32.and (i32.eq (local.get $hx) (i32.const 0x3ff00000))
                   (i32.eqz (call $math_lw (local.get $x))))
        (then (return (f64.const 0))))))
  ;; Reduce x into [sqrt(2)/2, sqrt(2)].
  (local.set $hx (i32.add (local.get $hx) (i32.const 0x00095f62))) ;; 0x3ff00000 - 0x3fe6a09e
  (local.set $k (i32.add (local.get $k)
    (i32.sub (i32.shr_u (local.get $hx) (i32.const 20)) (i32.const 0x3ff))))
  (local.set $hx (i32.add (i32.and (local.get $hx) (i32.const 0x000fffff)) (i32.const 0x3fe6a09e)))
  (local.set $x (call $math_set_hw (local.get $x) (local.get $hx)))
  (local.set $f (f64.sub (local.get $x) (f64.const 1)))
  (local.set $hfsq (f64.mul (f64.const 0.5) (f64.mul (local.get $f) (local.get $f))))
  (local.set $s (f64.div (local.get $f) (f64.add (f64.const 2) (local.get $f))))
  (local.set $z (f64.mul (local.get $s) (local.get $s)))
  (local.set $w (f64.mul (local.get $z) (local.get $z)))
  (local.set $t1 (f64.mul (local.get $w)
    (f64.add (f64.const 3.999999999940941908e-01) (f64.mul (local.get $w)      ;; Lg2 0x3fd99999_9997fa04
    (f64.add (f64.const 2.222219843214978396e-01) (f64.mul (local.get $w)      ;; Lg4 0x3fcc71c5_1d8e78af
             (f64.const 1.531383769920937332e-01)))))))                        ;; Lg6 0x3fc39a09_d078c69f
  (local.set $t2 (f64.mul (local.get $z)
    (f64.add (f64.const 6.666666666666735130e-01) (f64.mul (local.get $w)      ;; Lg1 0x3fe55555_55555593
    (f64.add (f64.const 2.857142874366239149e-01) (f64.mul (local.get $w)      ;; Lg3 0x3fd24924_94229359
    (f64.add (f64.const 1.818357216161805012e-01) (f64.mul (local.get $w)      ;; Lg5 0x3fc74664_96cb03de
             (f64.const 1.479819860511658591e-01)))))))))                      ;; Lg7 0x3fc2f112_df3e5244
  (local.set $r (f64.add (local.get $t2) (local.get $t1)))
  (local.set $dk (f64.convert_i32_s (local.get $k)))
  ;; s*(hfsq+r) + dk*ln2_lo - hfsq + f + dk*ln2_hi, evaluated left to right.
  (f64.add
    (f64.add
      (f64.sub
        (f64.add
          (f64.mul (local.get $s) (f64.add (local.get $hfsq) (local.get $r)))
          (f64.mul (local.get $dk) (f64.const 1.90821492927058770002e-10)))   ;; ln2_lo 0x3dea39ef_35793c76
        (local.get $hfsq))
      (local.get $f))
    (f64.mul (local.get $dk) (f64.const 6.93147180369123816490e-01))))        ;; ln2_hi 0x3fe62e42_fee00000

;; ---------------------------------------------------------------------------
;; Trigonometric kernels. Source: FreeBSD msun k_sin.c, k_cos.c, k_tan.c via
;; musl __sin.c, __cos.c, __tan.c. Each kernel takes the reduced argument as
;; a double-double x + y with |x| <= pi/4 and |y| tiny.
;; ---------------------------------------------------------------------------

;; sin(x+y) on [-pi/4, pi/4]. iy = 0 means y is zero.
;; sin(x) ~ x + S1*x^3 + ... + S6*x^13, |error| < 2^-58 on the interval.
;; With y: sin(x+y) ~ sin(x) + (1 - x^2/2)*y.
(func $math_k_sin (param $x f64) (param $y f64) (param $iy i32) (result f64)
  (local $z f64) (local $w f64) (local $r f64) (local $v f64)
  (local.set $z (f64.mul (local.get $x) (local.get $x)))
  (local.set $w (f64.mul (local.get $z) (local.get $z)))
  (local.set $r (f64.add
    (f64.add (f64.const 8.33333333332248946124e-03)                            ;; S2 0x3f811111_1110f8a6
      (f64.mul (local.get $z) (f64.add (f64.const -1.98412698298579493134e-04) ;; S3 0xbf2a01a0_19c161d5
        (f64.mul (local.get $z) (f64.const 2.75573137070700676789e-06)))))     ;; S4 0x3ec71de3_57b1fe7d
    (f64.mul (f64.mul (local.get $z) (local.get $w))
      (f64.add (f64.const -2.50507602534068634195e-08)                         ;; S5 0xbe5ae5e6_8a2b9ceb
        (f64.mul (local.get $z) (f64.const 1.58969099521155010221e-10))))))    ;; S6 0x3de5d93a_5acfd57c
  (local.set $v (f64.mul (local.get $z) (local.get $x)))
  (if (i32.eqz (local.get $iy))
    (then (return (f64.add (local.get $x) (f64.mul (local.get $v)
      (f64.add (f64.const -1.66666666666666324348e-01)                         ;; S1 0xbfc55555_55555549
        (f64.mul (local.get $z) (local.get $r))))))))
  (f64.sub (local.get $x)
    (f64.sub
      (f64.sub
        (f64.mul (local.get $z) (f64.sub (f64.mul (f64.const 0.5) (local.get $y))
                                         (f64.mul (local.get $v) (local.get $r))))
        (local.get $y))
      (f64.mul (local.get $v) (f64.const -1.66666666666666324348e-01)))))      ;; S1

;; cos(x+y) on [-pi/4, pi/4].
;; cos(x) ~ 1 - x^2/2 + C1*x^4 + ... + C6*x^14, |error| < 2^-58.
;; The 1 - x^2/2 part is formed as w + (((1-w) - hz) + tail) to keep the
;; rounding error of 1 - hz out of the result.
(func $math_k_cos (param $x f64) (param $y f64) (result f64)
  (local $z f64) (local $w f64) (local $r f64) (local $hz f64)
  (local.set $z (f64.mul (local.get $x) (local.get $x)))
  (local.set $w (f64.mul (local.get $z) (local.get $z)))
  (local.set $r (f64.add
    (f64.mul (local.get $z)
      (f64.add (f64.const 4.16666666666666019037e-02)                          ;; C1 0x3fa55555_5555554c
        (f64.mul (local.get $z) (f64.add (f64.const -1.38888888888741095749e-03) ;; C2 0xbf56c16c_16c15177
          (f64.mul (local.get $z) (f64.const 2.48015872894767294178e-05))))))  ;; C3 0x3efa01a0_19cb1590
    (f64.mul (f64.mul (local.get $w) (local.get $w))
      (f64.add (f64.const -2.75573143513906633035e-07)                         ;; C4 0xbe927e4f_809c52ad
        (f64.mul (local.get $z) (f64.add (f64.const 2.08757232129817482790e-09) ;; C5 0x3e21ee9e_bdb4b1c4
          (f64.mul (local.get $z) (f64.const -1.13596475577881948265e-11)))))))) ;; C6 0xbda8fae9_be8838d4
  (local.set $hz (f64.mul (f64.const 0.5) (local.get $z)))
  (local.set $w (f64.sub (f64.const 1) (local.get $hz)))
  (f64.add (local.get $w)
    (f64.add
      (f64.sub (f64.sub (f64.const 1) (local.get $w)) (local.get $hz))
      (f64.sub (f64.mul (local.get $z) (local.get $r)) (f64.mul (local.get $x) (local.get $y))))))

;; Odd-indexed and even-indexed parts of the k_tan polynomial table T[0..12].
;; r = T[1] + w*(T[3] + w*(T[5] + w*(T[7] + w*(T[9] + w*T[11]))))
(func $math_k_tan_odd (param $w f64) (result f64)
  (f64.add (f64.const 1.33333333333201242699e-01) (f64.mul (local.get $w)    ;; T1  0x3fc11111_1110fe7a
  (f64.add (f64.const 2.18694882948595424599e-02) (f64.mul (local.get $w)    ;; T3  0x3f9664f4_8406d637
  (f64.add (f64.const 3.59207910759131235356e-03) (f64.mul (local.get $w)    ;; T5  0x3f6d6d22_c9560328
  (f64.add (f64.const 5.88041240820264096874e-04) (f64.mul (local.get $w)    ;; T7  0x3f4344d8_f2f26501
  (f64.add (f64.const 7.81794442939557092300e-05) (f64.mul (local.get $w)    ;; T9  0x3f147e88_a03792a6
           (f64.const -1.85586374855275456654e-05))))))))))))                 ;; T11 0xbef375cb_db605373

;; v = z*(T[2] + w*(T[4] + w*(T[6] + w*(T[8] + w*(T[10] + w*T[12])))))
(func $math_k_tan_even (param $z f64) (param $w f64) (result f64)
  (f64.mul (local.get $z)
  (f64.add (f64.const 5.39682539762260521377e-02) (f64.mul (local.get $w)    ;; T2  0x3faba1ba_1bb341fe
  (f64.add (f64.const 8.86323982359930005737e-03) (f64.mul (local.get $w)    ;; T4  0x3f8226e3_e96e8493
  (f64.add (f64.const 1.45620945432529025516e-03) (f64.mul (local.get $w)    ;; T6  0x3f57dbc8_fee08315
  (f64.add (f64.const 2.46463134818469906812e-04) (f64.mul (local.get $w)    ;; T8  0x3f3026f7_1a8d1068
  (f64.add (f64.const 7.14072491382608190305e-05) (f64.mul (local.get $w)    ;; T10 0x3f12b80f_32f0a7e9
           (f64.const 2.59073051863633712884e-05)))))))))))))                 ;; T12 0x3efb2a70_74bf7ad4

;; tan(x+y) on [-pi/4, pi/4]. odd = 1 returns -1/tan(x+y) instead.
;; tan(x) ~ x + T0*x^3 + ... + T12*x^27, |error| < 2^-59.2 on [0, 0.67434].
;; For |x| >= 0.6744 the code uses tan(pi/4 - x) and the identity
;; tan(x) = 1/tan(pi/4 - x) via 1 - 2*(w - w^2/(w+1)).
(func $math_k_tan (param $x f64) (param $y f64) (param $odd i32) (result f64)
  (local $hx i32) (local $big i32) (local $sign i32)
  (local $z f64) (local $w f64) (local $r f64) (local $v f64) (local $s f64)
  (local $w0 f64) (local $a f64) (local $a0 f64)
  (local.set $hx (call $math_hw (local.get $x)))
  (local.set $big (i32.ge_u (i32.and (local.get $hx) (i32.const 0x7fffffff)) (i32.const 0x3fe59428))) ;; |x| >= 0.6744
  (local.set $sign (i32.shr_u (local.get $hx) (i32.const 31)))
  (if (local.get $big)
    (then
      (if (local.get $sign)
        (then
          (local.set $x (f64.neg (local.get $x)))
          (local.set $y (f64.neg (local.get $y)))))
      (local.set $x (f64.add
        (f64.sub (f64.const 7.85398163397448278999e-01) (local.get $x))       ;; pio4    0x3fe921fb_54442d18
        (f64.sub (f64.const 3.06161699786838301793e-17) (local.get $y))))     ;; pio4_lo 0x3c81a626_33145c07
      (local.set $y (f64.const 0))))
  (local.set $z (f64.mul (local.get $x) (local.get $x)))
  (local.set $w (f64.mul (local.get $z) (local.get $z)))
  (local.set $r (call $math_k_tan_odd (local.get $w)))
  (local.set $v (call $math_k_tan_even (local.get $z) (local.get $w)))
  (local.set $s (f64.mul (local.get $z) (local.get $x)))
  ;; r = y + z*(s*(r+v) + y) + s*T0
  (local.set $r (f64.add
    (f64.add (local.get $y)
      (f64.mul (local.get $z) (f64.add (f64.mul (local.get $s) (f64.add (local.get $r) (local.get $v)))
                                       (local.get $y))))
    (f64.mul (local.get $s) (f64.const 3.33333333333334091986e-01))))         ;; T0 0x3fd55555_55555563
  (local.set $w (f64.add (local.get $x) (local.get $r)))
  (if (local.get $big)
    (then
      (local.set $s (f64.sub (f64.const 1) (f64.mul (f64.const 2) (f64.convert_i32_s (local.get $odd)))))
      (local.set $v (f64.sub (local.get $s)
        (f64.mul (f64.const 2)
          (f64.add (local.get $x)
            (f64.sub (local.get $r)
              (f64.div (f64.mul (local.get $w) (local.get $w)) (f64.add (local.get $w) (local.get $s))))))))
      (return (select (f64.neg (local.get $v)) (local.get $v) (local.get $sign)))))
  (if (i32.eqz (local.get $odd)) (then (return (local.get $w))))
  ;; -1/(x+r) has up to 2 ulp error, so compute it with a correction step.
  (local.set $w0 (call $math_trunc_lw (local.get $w)))
  (local.set $v (f64.sub (local.get $r) (f64.sub (local.get $w0) (local.get $x)))) ;; w0 + v = r + x
  (local.set $a (f64.div (f64.const -1) (local.get $w)))
  (local.set $a0 (call $math_trunc_lw (local.get $a)))
  (f64.add (local.get $a0)
    (f64.mul (local.get $a)
      (f64.add (f64.add (f64.const 1) (f64.mul (local.get $a0) (local.get $w0)))
               (f64.mul (local.get $a0) (local.get $v))))))

;; ---------------------------------------------------------------------------
;; Argument reduction: x = n*(pi/2) + (y0 + y1). Source: FreeBSD msun
;; e_rem_pio2.c and k_rem_pio2.c (Sun, 1993) via musl __rem_pio2.c and
;; __rem_pio2_large.c.
;;
;; $math_rem_pio2(x) returns n. It stores y0 at 0xec60 and y1 at 0xec68.
;; y0 + y1 is the reduced argument with about 33 extra bits, |y0| <= pi/4.
;;
;; Ranges:
;;   |x| <= 5pi/4, 7pi/4, 9pi/4: subtract 1..4 multiples of a 3-part pi/2
;;   |x| < 2^20 * pi/2:          n = rint(x * 2/pi), 1 to 3 rounds of
;;                               subtraction with 33-bit pieces of pi/2
;;                               (good to 85, 118, or 151 bits)
;;   otherwise:                  Payne-Hanek with the 2/pi table
;; ---------------------------------------------------------------------------

;; pio2_1 is the first 33 bits of pi/2. pio2_1t = pi/2 - pio2_1.
;; pio2_2 is the second 33 bits, pio2_3 the third.
(func $math_rem_pio2_medium (param $x f64) (param $ix i32) (result i32)
  (local $n i32) (local $ex i32) (local $ey i32)
  (local $fn f64) (local $r f64) (local $w f64) (local $t f64) (local $y0 f64)
  ;; rint(x / (pi/2)) by adding and subtracting 1.5 * 2^52.
  (local.set $fn (f64.sub
    (f64.add (f64.mul (local.get $x) (f64.const 6.36619772367581382433e-01))  ;; invpio2 0x3fe45f30_6dc9c883
             (f64.const 0x1.8p52))
    (f64.const 0x1.8p52)))
  (local.set $n (i32.trunc_f64_s (local.get $fn)))
  (local.set $r (f64.sub (local.get $x) (f64.mul (local.get $fn) (f64.const 1.57079632673412561417e+00))))  ;; pio2_1 0x3ff921fb_54400000
  (local.set $w (f64.mul (local.get $fn) (f64.const 6.07710050650619224932e-11)))  ;; pio2_1t 0x3dd0b461_1a626331
  ;; First round, good to 85 bits.
  (local.set $y0 (f64.sub (local.get $r) (local.get $w)))
  (local.set $ey (i32.and (i32.shr_u (call $math_hw (local.get $y0)) (i32.const 20)) (i32.const 0x7ff)))
  (local.set $ex (i32.shr_u (local.get $ix) (i32.const 20)))
  (if (i32.gt_s (i32.sub (local.get $ex) (local.get $ey)) (i32.const 16))
    (then
      ;; Second round, good to 118 bits.
      (local.set $t (local.get $r))
      (local.set $w (f64.mul (local.get $fn) (f64.const 6.07710050630396597660e-11)))  ;; pio2_2 0x3dd0b461_1a600000
      (local.set $r (f64.sub (local.get $t) (local.get $w)))
      (local.set $w (f64.sub
        (f64.mul (local.get $fn) (f64.const 2.02226624879595063154e-21))            ;; pio2_2t 0x3ba3198a_2e037073
        (f64.sub (f64.sub (local.get $t) (local.get $r)) (local.get $w))))
      (local.set $y0 (f64.sub (local.get $r) (local.get $w)))
      (local.set $ey (i32.and (i32.shr_u (call $math_hw (local.get $y0)) (i32.const 20)) (i32.const 0x7ff)))
      (if (i32.gt_s (i32.sub (local.get $ex) (local.get $ey)) (i32.const 49))
        (then
          ;; Third round, good to 151 bits. Covers all cases.
          (local.set $t (local.get $r))
          (local.set $w (f64.mul (local.get $fn) (f64.const 2.02226624871116645580e-21)))  ;; pio2_3 0x3ba3198a_2e000000
          (local.set $r (f64.sub (local.get $t) (local.get $w)))
          (local.set $w (f64.sub
            (f64.mul (local.get $fn) (f64.const 8.47842766036889956997e-32))            ;; pio2_3t 0x397b839a_252049c1
            (f64.sub (f64.sub (local.get $t) (local.get $r)) (local.get $w))))
          (local.set $y0 (f64.sub (local.get $r) (local.get $w)))))))
  (f64.store (i32.const 0xec60) (local.get $y0))
  (f64.store (i32.const 0xec68) (f64.sub (f64.sub (local.get $r) (local.get $y0)) (local.get $w)))
  (local.get $n))

;; Subtract m * (pio2_1 + pio2_1t) from x, m in 1..4. One round is good to
;; 85 bits, which is enough because these ranges exclude near-multiples.
(func $math_rem_pio2_small (param $x f64) (param $m i32) (param $sign i32) (result i32)
  (local $fm f64) (local $z f64) (local $y0 f64)
  (local.set $fm (f64.convert_i32_s (local.get $m)))
  (if (local.get $sign)
    (then
      (local.set $z (f64.add (local.get $x) (f64.mul (local.get $fm) (f64.const 1.57079632673412561417e+00))))
      (local.set $y0 (f64.add (local.get $z) (f64.mul (local.get $fm) (f64.const 6.07710050650619224932e-11))))
      (f64.store (i32.const 0xec60) (local.get $y0))
      (f64.store (i32.const 0xec68) (f64.add (f64.sub (local.get $z) (local.get $y0))
        (f64.mul (local.get $fm) (f64.const 6.07710050650619224932e-11))))
      (return (i32.sub (i32.const 0) (local.get $m)))))
  (local.set $z (f64.sub (local.get $x) (f64.mul (local.get $fm) (f64.const 1.57079632673412561417e+00))))
  (local.set $y0 (f64.sub (local.get $z) (f64.mul (local.get $fm) (f64.const 6.07710050650619224932e-11))))
  (f64.store (i32.const 0xec60) (local.get $y0))
  (f64.store (i32.const 0xec68) (f64.sub (f64.sub (local.get $z) (local.get $y0))
    (f64.mul (local.get $fm) (f64.const 6.07710050650619224932e-11))))
  (local.get $m))

(func $math_rem_pio2 (param $x f64) (result i32)
  (local $ix i32) (local $sign i32) (local $n i32) (local $i i32) (local $nx i32)
  (local $z f64) (local $t f64)
  (local.set $ix (i32.and (call $math_hw (local.get $x)) (i32.const 0x7fffffff)))
  (local.set $sign (i32.wrap_i64 (i64.shr_u (i64.reinterpret_f64 (local.get $x)) (i64.const 63))))
  (if (i32.le_u (local.get $ix) (i32.const 0x400f6a7a)) ;; |x| ~<= 5pi/4
    (then
      ;; |x| ~= pi/2 or pi: cancellation, use the medium path.
      (if (i32.eq (i32.and (local.get $ix) (i32.const 0xfffff)) (i32.const 0x921fb))
        (then (return (call $math_rem_pio2_medium (local.get $x) (local.get $ix)))))
      (if (i32.le_u (local.get $ix) (i32.const 0x4002d97c)) ;; |x| ~<= 3pi/4
        (then (return (call $math_rem_pio2_small (local.get $x) (i32.const 1) (local.get $sign)))))
      (return (call $math_rem_pio2_small (local.get $x) (i32.const 2) (local.get $sign)))))
  (if (i32.le_u (local.get $ix) (i32.const 0x401c463b)) ;; |x| ~<= 9pi/4
    (then
      (if (i32.le_u (local.get $ix) (i32.const 0x4015fdbc)) ;; |x| ~<= 7pi/4
        (then
          (if (i32.eq (local.get $ix) (i32.const 0x4012d97c)) ;; |x| ~= 3pi/2
            (then (return (call $math_rem_pio2_medium (local.get $x) (local.get $ix)))))
          (return (call $math_rem_pio2_small (local.get $x) (i32.const 3) (local.get $sign)))))
      (if (i32.eq (local.get $ix) (i32.const 0x401921fb)) ;; |x| ~= 2pi
        (then (return (call $math_rem_pio2_medium (local.get $x) (local.get $ix)))))
      (return (call $math_rem_pio2_small (local.get $x) (i32.const 4) (local.get $sign)))))
  (if (i32.lt_u (local.get $ix) (i32.const 0x413921fb)) ;; |x| ~< 2^20 * pi/2
    (then (return (call $math_rem_pio2_medium (local.get $x) (local.get $ix)))))
  ;; inf or NaN
  (if (i32.ge_u (local.get $ix) (i32.const 0x7ff00000))
    (then
      (f64.store (i32.const 0xec60) (f64.sub (local.get $x) (local.get $x)))
      (f64.store (i32.const 0xec68) (f64.sub (local.get $x) (local.get $x)))
      (return (i32.const 0))))
  ;; Large: split |x| * 2^(23 - ilogb(x)) into three 24-bit integers.
  (local.set $z (f64.reinterpret_i64 (i64.or
    (i64.and (i64.reinterpret_f64 (local.get $x)) (i64.const 0x000fffffffffffff))
    (i64.const 0x4160000000000000)))) ;; exponent 0x3ff + 23
  (local.set $i (i32.const 0))
  (loop $split
    (local.set $t (f64.convert_i32_s (i32.trunc_f64_s (local.get $z))))
    (f64.store (i32.add (i32.const 0xec40) (i32.shl (local.get $i) (i32.const 3))) (local.get $t))
    (local.set $z (f64.mul (f64.sub (local.get $z) (local.get $t)) (f64.const 0x1p24)))
    (local.set $i (i32.add (local.get $i) (i32.const 1)))
    (br_if $split (i32.lt_u (local.get $i) (i32.const 2))))
  (f64.store (i32.const 0xec50) (local.get $z))
  ;; Skip trailing zero terms. The first term is nonzero.
  (local.set $nx (i32.const 3))
  (block $trimmed
    (loop $trim
      (br_if $trimmed (i32.eq (local.get $nx) (i32.const 1)))
      (br_if $trimmed (f64.ne
        (f64.load (i32.add (i32.const 0xec40) (i32.shl (i32.sub (local.get $nx) (i32.const 1)) (i32.const 3))))
        (f64.const 0)))
      (local.set $nx (i32.sub (local.get $nx) (i32.const 1)))
      (br $trim)))
  (local.set $n (call $math_rem_pio2_large (local.get $nx)
    (i32.sub (i32.shr_u (local.get $ix) (i32.const 20)) (i32.const 1046)))) ;; 0x3ff + 23
  (if (local.get $sign)
    (then
      (f64.store (i32.const 0xec60) (f64.neg (f64.load (i32.const 0xec60))))
      (f64.store (i32.const 0xec68) (f64.neg (f64.load (i32.const 0xec68))))
      (return (i32.sub (i32.const 0) (local.get $n)))))
  (local.get $n))

;; The pieces of pi/2 used by rem_pio2_large, each with 24 significant bits.
(func $math_pio2_piece (param $k i32) (result f64)
  (if (i32.eq (local.get $k) (i32.const 0)) (then (return (f64.const 1.57079625129699707031e+00)))) ;; 0x3ff921fb_40000000
  (if (i32.eq (local.get $k) (i32.const 1)) (then (return (f64.const 7.54978941586159635335e-08)))) ;; 0x3e74442d_00000000
  (if (i32.eq (local.get $k) (i32.const 2)) (then (return (f64.const 5.39030252995776476554e-15)))) ;; 0x3cf84698_80000000
  (if (i32.eq (local.get $k) (i32.const 3)) (then (return (f64.const 3.28200341580791294123e-22)))) ;; 0x3b78cc51_60000000
  (if (i32.eq (local.get $k) (i32.const 4)) (then (return (f64.const 1.27065575308067607349e-29)))) ;; 0x39f01b83_80000000
  (if (i32.eq (local.get $k) (i32.const 5)) (then (return (f64.const 1.22933308981111328932e-36)))) ;; 0x387a2520_40000000
  (if (i32.eq (local.get $k) (i32.const 6)) (then (return (f64.const 2.73370053816464559624e-44)))) ;; 0x36e38222_80000000
  (f64.const 2.16741683877804819444e-51))                                                           ;; 0x3569f31d_00000000

;; Payne-Hanek reduction, the double-precision case (prec = 1, jk = 4) of
;; k_rem_pio2.c. The input is tx[0..nx-1] at 0xec40: |x| = sum tx[i]*2^(e0 - 24i)
;; with each tx[i] a 24-bit integer value. It multiplies by the 2/pi table,
;; discards the integer part modulo 8, and converts the fraction back with
;; the pi/2 pieces. It returns n mod 8 and stores y0, y1 at 0xec60.
;;
;; Array addresses: f 0xea00, q 0xeaa0, fq 0xeb40 (f64); iq 0xebe0 (i32).
;; 66 table words serve exponents up to 1024: jv <= 42 and jv + jk + 1 <= 48
;; before recomputation, which adds at most a few more words.
(func $math_rem_pio2_large (param $nx i32) (param $e0 i32) (result i32)
  (local $jx i32) (local $jv i32) (local $q0 i32) (local $jz i32) (local $jp i32)
  (local $i i32) (local $j i32) (local $k i32) (local $m i32) (local $n i32) (local $ih i32)
  (local $carry i32) (local $iqv i32)
  (local $fw f64) (local $z f64)
  (local.set $jp (i32.const 4)) ;; jk
  (local.set $jx (i32.sub (local.get $nx) (i32.const 1)))
  ;; jv = max(0, (e0-3)/24), q0 = e0 - 24*(jv+1)
  (local.set $jv (i32.div_s (i32.sub (local.get $e0) (i32.const 3)) (i32.const 24)))
  (if (i32.lt_s (local.get $jv) (i32.const 0)) (then (local.set $jv (i32.const 0))))
  (local.set $q0 (i32.sub (local.get $e0) (i32.mul (i32.const 24) (i32.add (local.get $jv) (i32.const 1)))))
  ;; f[0..jx+jk] = ipio2[jv-jx .. jv+jk], zero below index 0
  (local.set $j (i32.sub (local.get $jv) (local.get $jx)))
  (local.set $m (i32.add (local.get $jx) (i32.const 4)))
  (local.set $i (i32.const 0))
  (loop $fill_f
    (f64.store (i32.add (i32.const 0xea00) (i32.shl (local.get $i) (i32.const 3)))
      (if (result f64) (i32.lt_s (local.get $j) (i32.const 0))
        (then (f64.const 0))
        (else (f64.convert_i32_s (i32.load (i32.add (i32.const 0xe800) (i32.shl (local.get $j) (i32.const 2))))))))
    (local.set $j (i32.add (local.get $j) (i32.const 1)))
    (local.set $i (i32.add (local.get $i) (i32.const 1)))
    (br_if $fill_f (i32.le_s (local.get $i) (local.get $m))))
  ;; q[i] = sum_{j<=jx} x[j] * f[jx+i-j] for i in 0..jk
  (local.set $i (i32.const 0))
  (loop $fill_q
    (local.set $fw (f64.const 0))
    (local.set $j (i32.const 0))
    (loop $dot
      (local.set $fw (f64.add (local.get $fw) (f64.mul
        (f64.load (i32.add (i32.const 0xec40) (i32.shl (local.get $j) (i32.const 3))))
        (f64.load (i32.add (i32.const 0xea00) (i32.shl
          (i32.sub (i32.add (local.get $jx) (local.get $i)) (local.get $j)) (i32.const 3)))))))
      (local.set $j (i32.add (local.get $j) (i32.const 1)))
      (br_if $dot (i32.le_s (local.get $j) (local.get $jx))))
    (f64.store (i32.add (i32.const 0xeaa0) (i32.shl (local.get $i) (i32.const 3))) (local.get $fw))
    (local.set $i (i32.add (local.get $i) (i32.const 1)))
    (br_if $fill_q (i32.le_s (local.get $i) (i32.const 4))))
  (local.set $jz (i32.const 4))
  (block $computed
    (loop $recompute
      ;; Distill q[] into iq[] reversingly: iq[i] = low 24 bits, carry up.
      (local.set $i (i32.const 0))
      (local.set $z (f64.load (i32.add (i32.const 0xeaa0) (i32.shl (local.get $jz) (i32.const 3)))))
      (local.set $j (local.get $jz))
      (loop $distill
        (local.set $fw (f64.convert_i32_s (i32.trunc_f64_s (f64.mul (f64.const 0x1p-24) (local.get $z)))))
        (i32.store (i32.add (i32.const 0xebe0) (i32.shl (local.get $i) (i32.const 2)))
          (i32.trunc_f64_s (f64.sub (local.get $z) (f64.mul (f64.const 0x1p24) (local.get $fw)))))
        (local.set $z (f64.add
          (f64.load (i32.add (i32.const 0xeaa0) (i32.shl (i32.sub (local.get $j) (i32.const 1)) (i32.const 3))))
          (local.get $fw)))
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (local.set $j (i32.sub (local.get $j) (i32.const 1)))
        (br_if $distill (i32.ge_s (local.get $j) (i32.const 1))))
      ;; Compute n: the integer part of z * 2^q0 modulo 8.
      (local.set $z (call $math_scalbn (local.get $z) (local.get $q0)))
      (local.set $z (f64.sub (local.get $z) (f64.mul (f64.const 8) (f64.floor (f64.mul (local.get $z) (f64.const 0.125))))))
      (local.set $n (i32.trunc_f64_s (local.get $z)))
      (local.set $z (f64.sub (local.get $z) (f64.convert_i32_s (local.get $n))))
      (local.set $ih (i32.const 0))
      (if (i32.gt_s (local.get $q0) (i32.const 0))
        (then
          ;; iq[jz-1] holds more integer bits.
          (local.set $iqv (i32.load (i32.add (i32.const 0xebe0) (i32.shl (i32.sub (local.get $jz) (i32.const 1)) (i32.const 2)))))
          (local.set $i (i32.shr_s (local.get $iqv) (i32.sub (i32.const 24) (local.get $q0))))
          (local.set $n (i32.add (local.get $n) (local.get $i)))
          (local.set $iqv (i32.sub (local.get $iqv) (i32.shl (local.get $i) (i32.sub (i32.const 24) (local.get $q0)))))
          (i32.store (i32.add (i32.const 0xebe0) (i32.shl (i32.sub (local.get $jz) (i32.const 1)) (i32.const 2))) (local.get $iqv))
          (local.set $ih (i32.shr_s (local.get $iqv) (i32.sub (i32.const 23) (local.get $q0)))))
        (else
          (if (i32.eqz (local.get $q0))
            (then
              (local.set $ih (i32.shr_s
                (i32.load (i32.add (i32.const 0xebe0) (i32.shl (i32.sub (local.get $jz) (i32.const 1)) (i32.const 2))))
                (i32.const 23))))
            (else
              (if (f64.ge (local.get $z) (f64.const 0.5)) (then (local.set $ih (i32.const 2))))))))
      (if (i32.gt_s (local.get $ih) (i32.const 0))
        (then
          ;; The fraction is > 0.5: round n up and take 1 - fraction.
          (local.set $n (i32.add (local.get $n) (i32.const 1)))
          (local.set $carry (i32.const 0))
          (local.set $i (i32.const 0))
          (block $negated
            (loop $negate
              (br_if $negated (i32.ge_s (local.get $i) (local.get $jz)))
              (local.set $j (i32.load (i32.add (i32.const 0xebe0) (i32.shl (local.get $i) (i32.const 2)))))
              (if (i32.eqz (local.get $carry))
                (then
                  (if (local.get $j)
                    (then
                      (local.set $carry (i32.const 1))
                      (i32.store (i32.add (i32.const 0xebe0) (i32.shl (local.get $i) (i32.const 2)))
                        (i32.sub (i32.const 0x1000000) (local.get $j))))))
                (else
                  (i32.store (i32.add (i32.const 0xebe0) (i32.shl (local.get $i) (i32.const 2)))
                    (i32.sub (i32.const 0xffffff) (local.get $j)))))
              (local.set $i (i32.add (local.get $i) (i32.const 1)))
              (br $negate)))
          (if (i32.gt_s (local.get $q0) (i32.const 0))
            (then
              ;; Clear the integer bits that were moved into n.
              (if (i32.eq (local.get $q0) (i32.const 1))
                (then (i32.store (i32.add (i32.const 0xebe0) (i32.shl (i32.sub (local.get $jz) (i32.const 1)) (i32.const 2)))
                  (i32.and (i32.load (i32.add (i32.const 0xebe0) (i32.shl (i32.sub (local.get $jz) (i32.const 1)) (i32.const 2))))
                           (i32.const 0x7fffff)))))
              (if (i32.eq (local.get $q0) (i32.const 2))
                (then (i32.store (i32.add (i32.const 0xebe0) (i32.shl (i32.sub (local.get $jz) (i32.const 1)) (i32.const 2)))
                  (i32.and (i32.load (i32.add (i32.const 0xebe0) (i32.shl (i32.sub (local.get $jz) (i32.const 1)) (i32.const 2))))
                           (i32.const 0x3fffff)))))))
          (if (i32.eq (local.get $ih) (i32.const 2))
            (then
              (local.set $z (f64.sub (f64.const 1) (local.get $z)))
              (if (local.get $carry)
                (then (local.set $z (f64.sub (local.get $z) (call $math_scalbn (f64.const 1) (local.get $q0))))))))))
      ;; If every remaining chunk is zero, more table words are needed.
      (br_if $computed (f64.ne (local.get $z) (f64.const 0)))
      (local.set $j (i32.const 0))
      (local.set $i (i32.sub (local.get $jz) (i32.const 1)))
      (block $ored
        (loop $or
          (br_if $ored (i32.lt_s (local.get $i) (i32.const 4)))
          (local.set $j (i32.or (local.get $j) (i32.load (i32.add (i32.const 0xebe0) (i32.shl (local.get $i) (i32.const 2))))))
          (local.set $i (i32.sub (local.get $i) (i32.const 1)))
          (br $or)))
      (br_if $computed (local.get $j))
      ;; k = number of extra terms needed
      (local.set $k (i32.const 1))
      (loop $count
        (if (i32.eqz (i32.load (i32.add (i32.const 0xebe0) (i32.shl (i32.sub (i32.const 4) (local.get $k)) (i32.const 2)))))
          (then
            (local.set $k (i32.add (local.get $k) (i32.const 1)))
            (br $count))))
      ;; Add q[jz+1] .. q[jz+k].
      (local.set $i (i32.add (local.get $jz) (i32.const 1)))
      (loop $extend
        (f64.store (i32.add (i32.const 0xea00) (i32.shl (i32.add (local.get $jx) (local.get $i)) (i32.const 3)))
          (f64.convert_i32_s (i32.load (i32.add (i32.const 0xe800) (i32.shl (i32.add (local.get $jv) (local.get $i)) (i32.const 2))))))
        (local.set $fw (f64.const 0))
        (local.set $j (i32.const 0))
        (loop $dot2
          (local.set $fw (f64.add (local.get $fw) (f64.mul
            (f64.load (i32.add (i32.const 0xec40) (i32.shl (local.get $j) (i32.const 3))))
            (f64.load (i32.add (i32.const 0xea00) (i32.shl
              (i32.sub (i32.add (local.get $jx) (local.get $i)) (local.get $j)) (i32.const 3)))))))
          (local.set $j (i32.add (local.get $j) (i32.const 1)))
          (br_if $dot2 (i32.le_s (local.get $j) (local.get $jx))))
        (f64.store (i32.add (i32.const 0xeaa0) (i32.shl (local.get $i) (i32.const 3))) (local.get $fw))
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br_if $extend (i32.le_s (local.get $i) (i32.add (local.get $jz) (local.get $k)))))
      (local.set $jz (i32.add (local.get $jz) (local.get $k)))
      (br $recompute)))
  ;; Chop off zero terms, or break z into 24-bit chunks.
  (if (f64.eq (local.get $z) (f64.const 0))
    (then
      (local.set $jz (i32.sub (local.get $jz) (i32.const 1)))
      (local.set $q0 (i32.sub (local.get $q0) (i32.const 24)))
      (loop $chop
        (if (i32.eqz (i32.load (i32.add (i32.const 0xebe0) (i32.shl (local.get $jz) (i32.const 2)))))
          (then
            (local.set $jz (i32.sub (local.get $jz) (i32.const 1)))
            (local.set $q0 (i32.sub (local.get $q0) (i32.const 24)))
            (br $chop)))))
    (else
      (local.set $z (call $math_scalbn (local.get $z) (i32.sub (i32.const 0) (local.get $q0))))
      (if (f64.ge (local.get $z) (f64.const 0x1p24))
        (then
          (local.set $fw (f64.convert_i32_s (i32.trunc_f64_s (f64.mul (f64.const 0x1p-24) (local.get $z)))))
          (i32.store (i32.add (i32.const 0xebe0) (i32.shl (local.get $jz) (i32.const 2)))
            (i32.trunc_f64_s (f64.sub (local.get $z) (f64.mul (f64.const 0x1p24) (local.get $fw)))))
          (local.set $jz (i32.add (local.get $jz) (i32.const 1)))
          (local.set $q0 (i32.add (local.get $q0) (i32.const 24)))
          (i32.store (i32.add (i32.const 0xebe0) (i32.shl (local.get $jz) (i32.const 2)))
            (i32.trunc_f64_s (local.get $fw))))
        (else
          (i32.store (i32.add (i32.const 0xebe0) (i32.shl (local.get $jz) (i32.const 2)))
            (i32.trunc_f64_s (local.get $z)))))))
  ;; Convert the integer chunks back to floating-point: q[i] = iq[i] * 2^(q0 - 24(jz-i)).
  (local.set $fw (call $math_scalbn (f64.const 1) (local.get $q0)))
  (local.set $i (local.get $jz))
  (loop $convert
    (f64.store (i32.add (i32.const 0xeaa0) (i32.shl (local.get $i) (i32.const 3)))
      (f64.mul (local.get $fw) (f64.convert_i32_s (i32.load (i32.add (i32.const 0xebe0) (i32.shl (local.get $i) (i32.const 2)))))))
    (local.set $fw (f64.mul (local.get $fw) (f64.const 0x1p-24)))
    (local.set $i (i32.sub (local.get $i) (i32.const 1)))
    (br_if $convert (i32.ge_s (local.get $i) (i32.const 0))))
  ;; fq[jz-i] = sum_k pio2[k] * q[i+k] for k <= jp and k <= jz-i
  (local.set $i (local.get $jz))
  (loop $multiply
    (local.set $fw (f64.const 0))
    (local.set $k (i32.const 0))
    (block $row_done
      (loop $row
        (br_if $row_done (i32.gt_s (local.get $k) (local.get $jp)))
        (br_if $row_done (i32.gt_s (local.get $k) (i32.sub (local.get $jz) (local.get $i))))
        (local.set $fw (f64.add (local.get $fw) (f64.mul
          (call $math_pio2_piece (local.get $k))
          (f64.load (i32.add (i32.const 0xeaa0) (i32.shl (i32.add (local.get $i) (local.get $k)) (i32.const 3)))))))
        (local.set $k (i32.add (local.get $k) (i32.const 1)))
        (br $row)))
    (f64.store (i32.add (i32.const 0xeb40) (i32.shl (i32.sub (local.get $jz) (local.get $i)) (i32.const 3))) (local.get $fw))
    (local.set $i (i32.sub (local.get $i) (i32.const 1)))
    (br_if $multiply (i32.ge_s (local.get $i) (i32.const 0))))
  ;; Compress fq[] into y0 + y1.
  (local.set $fw (f64.const 0))
  (local.set $i (local.get $jz))
  (loop $sum
    (local.set $fw (f64.add (local.get $fw) (f64.load (i32.add (i32.const 0xeb40) (i32.shl (local.get $i) (i32.const 3))))))
    (local.set $i (i32.sub (local.get $i) (i32.const 1)))
    (br_if $sum (i32.ge_s (local.get $i) (i32.const 0))))
  (f64.store (i32.const 0xec60) (select (f64.neg (local.get $fw)) (local.get $fw) (local.get $ih)))
  (local.set $fw (f64.sub (f64.load (i32.const 0xeb40)) (local.get $fw)))
  (local.set $i (i32.const 1))
  (block $tail_done
    (loop $tail
      (br_if $tail_done (i32.gt_s (local.get $i) (local.get $jz)))
      (local.set $fw (f64.add (local.get $fw) (f64.load (i32.add (i32.const 0xeb40) (i32.shl (local.get $i) (i32.const 3))))))
      (local.set $i (i32.add (local.get $i) (i32.const 1)))
      (br $tail)))
  (f64.store (i32.const 0xec68) (select (f64.neg (local.get $fw)) (local.get $fw) (local.get $ih)))
  (i32.and (local.get $n) (i32.const 7)))

;; ---------------------------------------------------------------------------
;; sin, cos, tan. Source: FreeBSD msun s_sin.c, s_cos.c, s_tan.c via musl.
;; For |x| <= pi/4 the kernel is applied directly. Otherwise the argument is
;; reduced to n*(pi/2) + y and the quadrant n selects the kernel and sign.
;; sin(+-0) = +-0, tan(+-0) = +-0, cos(0) = 1, all NaN for inf or NaN.
;; ---------------------------------------------------------------------------

(func $math_sin (param $x f64) (result f64)
  (local $ix i32) (local $n i32) (local $y0 f64) (local $y1 f64)
  (local.set $ix (i32.and (call $math_hw (local.get $x)) (i32.const 0x7fffffff)))
  (if (i32.le_u (local.get $ix) (i32.const 0x3fe921fb)) ;; |x| ~<= pi/4
    (then
      (if (i32.lt_u (local.get $ix) (i32.const 0x3e500000)) ;; |x| < 2^-26
        (then (return (local.get $x))))
      (return (call $math_k_sin (local.get $x) (f64.const 0) (i32.const 0)))))
  (if (i32.ge_u (local.get $ix) (i32.const 0x7ff00000))
    (then (return (f64.sub (local.get $x) (local.get $x)))))
  (local.set $n (i32.and (call $math_rem_pio2 (local.get $x)) (i32.const 3)))
  (local.set $y0 (f64.load (i32.const 0xec60)))
  (local.set $y1 (f64.load (i32.const 0xec68)))
  (if (i32.eq (local.get $n) (i32.const 0))
    (then (return (call $math_k_sin (local.get $y0) (local.get $y1) (i32.const 1)))))
  (if (i32.eq (local.get $n) (i32.const 1))
    (then (return (call $math_k_cos (local.get $y0) (local.get $y1)))))
  (if (i32.eq (local.get $n) (i32.const 2))
    (then (return (f64.neg (call $math_k_sin (local.get $y0) (local.get $y1) (i32.const 1))))))
  (f64.neg (call $math_k_cos (local.get $y0) (local.get $y1))))

(func $math_cos (param $x f64) (result f64)
  (local $ix i32) (local $n i32) (local $y0 f64) (local $y1 f64)
  (local.set $ix (i32.and (call $math_hw (local.get $x)) (i32.const 0x7fffffff)))
  (if (i32.le_u (local.get $ix) (i32.const 0x3fe921fb)) ;; |x| ~<= pi/4
    (then
      (if (i32.lt_u (local.get $ix) (i32.const 0x3e46a09e)) ;; |x| < 2^-27 * sqrt(2)
        (then (return (f64.const 1))))
      (return (call $math_k_cos (local.get $x) (f64.const 0)))))
  (if (i32.ge_u (local.get $ix) (i32.const 0x7ff00000))
    (then (return (f64.sub (local.get $x) (local.get $x)))))
  (local.set $n (i32.and (call $math_rem_pio2 (local.get $x)) (i32.const 3)))
  (local.set $y0 (f64.load (i32.const 0xec60)))
  (local.set $y1 (f64.load (i32.const 0xec68)))
  (if (i32.eq (local.get $n) (i32.const 0))
    (then (return (call $math_k_cos (local.get $y0) (local.get $y1)))))
  (if (i32.eq (local.get $n) (i32.const 1))
    (then (return (f64.neg (call $math_k_sin (local.get $y0) (local.get $y1) (i32.const 1))))))
  (if (i32.eq (local.get $n) (i32.const 2))
    (then (return (f64.neg (call $math_k_cos (local.get $y0) (local.get $y1))))))
  (call $math_k_sin (local.get $y0) (local.get $y1) (i32.const 1)))

(func $math_tan (param $x f64) (result f64)
  (local $ix i32) (local $n i32)
  (local.set $ix (i32.and (call $math_hw (local.get $x)) (i32.const 0x7fffffff)))
  (if (i32.le_u (local.get $ix) (i32.const 0x3fe921fb)) ;; |x| ~<= pi/4
    (then
      (if (i32.lt_u (local.get $ix) (i32.const 0x3e400000)) ;; |x| < 2^-27
        (then (return (local.get $x))))
      (return (call $math_k_tan (local.get $x) (f64.const 0) (i32.const 0)))))
  (if (i32.ge_u (local.get $ix) (i32.const 0x7ff00000))
    (then (return (f64.sub (local.get $x) (local.get $x)))))
  (local.set $n (call $math_rem_pio2 (local.get $x)))
  (call $math_k_tan (f64.load (i32.const 0xec60)) (f64.load (i32.const 0xec68))
    (i32.and (local.get $n) (i32.const 1))))

;; ---------------------------------------------------------------------------
;; atan(x). Source: FreeBSD msun s_atan.c (Sun, 1993) via musl atan.c.
;;
;; Method
;;   1. atan(x) = -atan(-x), so work with |x|.
;;   2. Reduce |x| to one of five intervals and use
;;        [0, 7/16)       atan(t) = t - t^3*(a1 + t^2*(a2 + ... a11))
;;        [7/16, 11/16)   atan(t) = atan(1/2) + atan((t - 0.5)/(1 + t/2))
;;        [11/16, 19/16)  atan(t) = atan(1)   + atan((t - 1)/(1 + t))
;;        [19/16, 39/16)  atan(t) = atan(3/2) + atan((t - 1.5)/(1 + 1.5t))
;;        [39/16, inf)    atan(t) = atan(inf) + atan(-1/t)
;;      The four base values are stored as hi + lo pairs.
;;
;; atan(+-0) = +-0, atan(+-inf) = +-pi/2, atan(NaN) = NaN. Error < 1 ulp.
;; ---------------------------------------------------------------------------

(func $math_atan_hi (param $id i32) (result f64)
  (if (i32.eq (local.get $id) (i32.const 0)) (then (return (f64.const 4.63647609000806093515e-01)))) ;; atan(0.5) 0x3fddac67_0561bb4f
  (if (i32.eq (local.get $id) (i32.const 1)) (then (return (f64.const 7.85398163397448278999e-01)))) ;; atan(1.0) 0x3fe921fb_54442d18
  (if (i32.eq (local.get $id) (i32.const 2)) (then (return (f64.const 9.82793723247329054082e-01)))) ;; atan(1.5) 0x3fef730b_d281f69b
  (f64.const 1.57079632679489655800e+00))                                                          ;; atan(inf) 0x3ff921fb_54442d18

(func $math_atan_lo (param $id i32) (result f64)
  (if (i32.eq (local.get $id) (i32.const 0)) (then (return (f64.const 2.26987774529616870924e-17)))) ;; 0x3c7a2b7f_222f65e2
  (if (i32.eq (local.get $id) (i32.const 1)) (then (return (f64.const 3.06161699786838301793e-17)))) ;; 0x3c81a626_33145c07
  (if (i32.eq (local.get $id) (i32.const 2)) (then (return (f64.const 1.39033110312309984516e-17)))) ;; 0x3c700788_7af0cbbd
  (f64.const 6.12323399573676603587e-17))                                                          ;; 0x3c91a626_33145c07

(func $math_atan (param $x f64) (result f64)
  (local $ix i32) (local $sign i32) (local $id i32)
  (local $z f64) (local $w f64) (local $s1 f64) (local $s2 f64)
  (local.set $ix (call $math_hw (local.get $x)))
  (local.set $sign (i32.shr_u (local.get $ix) (i32.const 31)))
  (local.set $ix (i32.and (local.get $ix) (i32.const 0x7fffffff)))
  (if (i32.ge_u (local.get $ix) (i32.const 0x44100000)) ;; |x| >= 2^66
    (then
      (if (f64.ne (local.get $x) (local.get $x)) (then (return (local.get $x))))
      (local.set $z (f64.add (f64.const 1.57079632679489655800e+00) (f64.const 0x1p-120)))
      (return (select (f64.neg (local.get $z)) (local.get $z) (local.get $sign)))))
  (if (i32.lt_u (local.get $ix) (i32.const 0x3fdc0000)) ;; |x| < 0.4375
    (then
      (if (i32.lt_u (local.get $ix) (i32.const 0x3e400000)) ;; |x| < 2^-27
        (then (return (local.get $x))))
      (local.set $id (i32.const -1)))
    (else
      (local.set $x (f64.abs (local.get $x)))
      (if (i32.lt_u (local.get $ix) (i32.const 0x3ff30000)) ;; |x| < 1.1875
        (then
          (if (i32.lt_u (local.get $ix) (i32.const 0x3fe60000)) ;; 7/16 <= |x| < 11/16
            (then
              (local.set $x (f64.div (f64.sub (f64.mul (f64.const 2) (local.get $x)) (f64.const 1))
                                     (f64.add (f64.const 2) (local.get $x))))
              (local.set $id (i32.const 0)))
            (else ;; 11/16 <= |x| < 19/16
              (local.set $x (f64.div (f64.sub (local.get $x) (f64.const 1)) (f64.add (local.get $x) (f64.const 1))))
              (local.set $id (i32.const 1)))))
        (else
          (if (i32.lt_u (local.get $ix) (i32.const 0x40038000)) ;; |x| < 2.4375
            (then
              (local.set $x (f64.div (f64.sub (local.get $x) (f64.const 1.5))
                                     (f64.add (f64.const 1) (f64.mul (f64.const 1.5) (local.get $x)))))
              (local.set $id (i32.const 2)))
            (else ;; 2.4375 <= |x| < 2^66
              (local.set $x (f64.div (f64.const -1) (local.get $x)))
              (local.set $id (i32.const 3))))))))
  (local.set $z (f64.mul (local.get $x) (local.get $x)))
  (local.set $w (f64.mul (local.get $z) (local.get $z)))
  ;; Odd and even parts of sum_{i=0..10} AT[i] z^(i+1).
  (local.set $s1 (f64.mul (local.get $z)
    (f64.add (f64.const 3.33333333333329318027e-01) (f64.mul (local.get $w)   ;; AT0  0x3fd55555_5555550d
    (f64.add (f64.const 1.42857142725034663711e-01) (f64.mul (local.get $w)   ;; AT2  0x3fc24924_920083ff
    (f64.add (f64.const 9.09088713343650656196e-02) (f64.mul (local.get $w)   ;; AT4  0x3fb745cd_c54c206e
    (f64.add (f64.const 6.66107313738753120669e-02) (f64.mul (local.get $w)   ;; AT6  0x3fb10d66_a0d03d51
    (f64.add (f64.const 4.97687799461593236017e-02) (f64.mul (local.get $w)   ;; AT8  0x3fa97b4b_24760deb
             (f64.const 1.62858201153657823623e-02)))))))))))))               ;; AT10 0x3f90ad3a_e322da11
  (local.set $s2 (f64.mul (local.get $w)
    (f64.add (f64.const -1.99999999998764832476e-01) (f64.mul (local.get $w)  ;; AT1  0xbfc99999_9998ebc4
    (f64.add (f64.const -1.11111104054623557880e-01) (f64.mul (local.get $w)  ;; AT3  0xbfbc71c6_fe231671
    (f64.add (f64.const -7.69187620504482999495e-02) (f64.mul (local.get $w)  ;; AT5  0xbfb3b0f2_af749a6d
    (f64.add (f64.const -5.83357013379057348645e-02) (f64.mul (local.get $w)  ;; AT7  0xbfadde2d_52defd9a
             (f64.const -3.65315727442169155270e-02)))))))))))                ;; AT9  0xbfa2b444_2c6a6c2f
  (if (i32.lt_s (local.get $id) (i32.const 0))
    (then (return (f64.sub (local.get $x) (f64.mul (local.get $x) (f64.add (local.get $s1) (local.get $s2)))))))
  (local.set $z (f64.sub (call $math_atan_hi (local.get $id))
    (f64.sub
      (f64.sub (f64.mul (local.get $x) (f64.add (local.get $s1) (local.get $s2)))
               (call $math_atan_lo (local.get $id)))
      (local.get $x))))
  (select (f64.neg (local.get $z)) (local.get $z) (local.get $sign)))

;; ---------------------------------------------------------------------------
;; asin(x) and acos(x). Source: FreeBSD msun e_asin.c, e_acos.c (Sun, 1993)
;; via musl asin.c, acos.c.
;;
;; Both use R(z) = P(z)/Q(z), a rational approximation of (asin(s)-s)/s^3
;; with |error| < 2^(-58.75) on [0, 0.25]. Below 0.5:
;;   asin(x) = x + x*x^2*R(x^2)
;; For 0.5 <= |x| < 1, with z = (1-|x|)/2 and s = sqrt(z):
;;   asin(x) = pi/2 - 2*(s + s*z*R(z))
;; The sqrt is split into a high part f (low word cleared) and a correction
;; c = (z - f*f)/(s + f) so that f + c = sqrt(z) to extra precision.
;;
;; asin(+-1) = +-pi/2, acos(1) = 0, acos(-1) = pi, |x| > 1 gives NaN.
;; ---------------------------------------------------------------------------

(func $math_asin_r (param $z f64) (result f64)
  (local $p f64) (local $q f64)
  (local.set $p (f64.mul (local.get $z)
    (f64.add (f64.const 1.66666666666666657415e-01) (f64.mul (local.get $z)   ;; pS0 0x3fc55555_55555555
    (f64.add (f64.const -3.25565818622400915405e-01) (f64.mul (local.get $z)  ;; pS1 0xbfd4d612_03eb6f7d
    (f64.add (f64.const 2.01212532134862925881e-01) (f64.mul (local.get $z)   ;; pS2 0x3fc9c155_0e884455
    (f64.add (f64.const -4.00555345006794114027e-02) (f64.mul (local.get $z)  ;; pS3 0xbfa48228_b5688f3b
    (f64.add (f64.const 7.91534994289814532176e-04) (f64.mul (local.get $z)   ;; pS4 0x3f49efe0_7501b288
             (f64.const 3.47933107596021167570e-05)))))))))))))               ;; pS5 0x3f023de1_0dfdf709
  (local.set $q (f64.add (f64.const 1) (f64.mul (local.get $z)
    (f64.add (f64.const -2.40339491173441421878e+00) (f64.mul (local.get $z)  ;; qS1 0xc0033a27_1c8a2d4b
    (f64.add (f64.const 2.02094576023350569471e+00) (f64.mul (local.get $z)   ;; qS2 0x40002ae5_9c598ac8
    (f64.add (f64.const -6.88283971605453293030e-01) (f64.mul (local.get $z)  ;; qS3 0xbfe6066c_1b8d0159
             (f64.const 7.70381505559019352791e-02))))))))))                 ;; qS4 0x3fb3b8c5_b12e9282
  (f64.div (local.get $p) (local.get $q)))

(func $math_asin (param $x f64) (result f64)
  (local $hx i32) (local $ix i32)
  (local $z f64) (local $r f64) (local $s f64) (local $f f64) (local $c f64)
  (local.set $hx (call $math_hw (local.get $x)))
  (local.set $ix (i32.and (local.get $hx) (i32.const 0x7fffffff)))
  (if (i32.ge_u (local.get $ix) (i32.const 0x3ff00000)) ;; |x| >= 1 or NaN
    (then
      (if (i32.eqz (i32.or (i32.sub (local.get $ix) (i32.const 0x3ff00000)) (call $math_lw (local.get $x))))
        (then ;; asin(+-1) = +-pi/2
          (return (f64.add (f64.mul (local.get $x) (f64.const 1.57079632679489655800e+00)) (f64.const 0x1p-120)))))
      (return (f64.div (f64.const 0) (f64.sub (local.get $x) (local.get $x))))))
  (if (i32.lt_u (local.get $ix) (i32.const 0x3fe00000)) ;; |x| < 0.5
    (then
      (if (i32.lt_u (local.get $ix) (i32.const 0x3e500000)) ;; |x| < 2^-26
        (then (return (local.get $x))))
      (return (f64.add (local.get $x) (f64.mul (local.get $x)
        (call $math_asin_r (f64.mul (local.get $x) (local.get $x))))))))
  ;; 0.5 <= |x| < 1
  (local.set $z (f64.mul (f64.sub (f64.const 1) (f64.abs (local.get $x))) (f64.const 0.5)))
  (local.set $s (f64.sqrt (local.get $z)))
  (local.set $r (call $math_asin_r (local.get $z)))
  (if (i32.ge_u (local.get $ix) (i32.const 0x3fef3333)) ;; |x| > 0.975
    (then
      (local.set $x (f64.sub (f64.const 1.57079632679489655800e+00)              ;; pio2_hi 0x3ff921fb_54442d18
        (f64.sub (f64.mul (f64.const 2) (f64.add (local.get $s) (f64.mul (local.get $s) (local.get $r))))
                 (f64.const 6.12323399573676603587e-17)))))                      ;; pio2_lo 0x3c91a626_33145c07
    (else
      (local.set $f (call $math_trunc_lw (local.get $s)))
      (local.set $c (f64.div (f64.sub (local.get $z) (f64.mul (local.get $f) (local.get $f)))
                             (f64.add (local.get $s) (local.get $f))))
      ;; pio4_hi - (2*s*r - (pio2_lo - 2*c) - (pio4_hi - 2*f))
      (local.set $x (f64.sub (f64.const 7.85398163397448278999e-01)
        (f64.sub
          (f64.sub (f64.mul (f64.mul (f64.const 2) (local.get $s)) (local.get $r))
                   (f64.sub (f64.const 6.12323399573676603587e-17) (f64.mul (f64.const 2) (local.get $c))))
          (f64.sub (f64.const 7.85398163397448278999e-01) (f64.mul (f64.const 2) (local.get $f))))))))
  (select (f64.neg (local.get $x)) (local.get $x) (i32.shr_u (local.get $hx) (i32.const 31))))

(func $math_acos (param $x f64) (result f64)
  (local $hx i32) (local $ix i32)
  (local $z f64) (local $w f64) (local $s f64) (local $c f64) (local $df f64)
  (local.set $hx (call $math_hw (local.get $x)))
  (local.set $ix (i32.and (local.get $hx) (i32.const 0x7fffffff)))
  (if (i32.ge_u (local.get $ix) (i32.const 0x3ff00000)) ;; |x| >= 1 or NaN
    (then
      (if (i32.eqz (i32.or (i32.sub (local.get $ix) (i32.const 0x3ff00000)) (call $math_lw (local.get $x))))
        (then
          (if (i32.shr_u (local.get $hx) (i32.const 31))
            (then (return (f64.add (f64.mul (f64.const 2) (f64.const 1.57079632679489655800e+00)) (f64.const 0x1p-120)))))
          (return (f64.const 0))))
      (return (f64.div (f64.const 0) (f64.sub (local.get $x) (local.get $x))))))
  (if (i32.lt_u (local.get $ix) (i32.const 0x3fe00000)) ;; |x| < 0.5
    (then
      (if (i32.le_u (local.get $ix) (i32.const 0x3c600000)) ;; |x| < 2^-57
        (then (return (f64.add (f64.const 1.57079632679489655800e+00) (f64.const 0x1p-120)))))
      ;; pio2_hi - (x - (pio2_lo - x*R(x^2)))
      (return (f64.sub (f64.const 1.57079632679489655800e+00)
        (f64.sub (local.get $x)
          (f64.sub (f64.const 6.12323399573676603587e-17)
            (f64.mul (local.get $x) (call $math_asin_r (f64.mul (local.get $x) (local.get $x))))))))))
  (if (i32.shr_u (local.get $hx) (i32.const 31))
    (then ;; x < -0.5: acos(x) = pi - 2*asin(sqrt((1+x)/2))
      (local.set $z (f64.mul (f64.add (f64.const 1) (local.get $x)) (f64.const 0.5)))
      (local.set $s (f64.sqrt (local.get $z)))
      (local.set $w (f64.sub (f64.mul (call $math_asin_r (local.get $z)) (local.get $s))
                             (f64.const 6.12323399573676603587e-17)))
      (return (f64.mul (f64.const 2)
        (f64.sub (f64.const 1.57079632679489655800e+00) (f64.add (local.get $s) (local.get $w)))))))
  ;; x > 0.5: acos(x) = 2*asin(sqrt((1-x)/2)) = 2f + (2c + 2*s*z*R(z))
  (local.set $z (f64.mul (f64.sub (f64.const 1) (local.get $x)) (f64.const 0.5)))
  (local.set $s (f64.sqrt (local.get $z)))
  (local.set $df (call $math_trunc_lw (local.get $s)))
  (local.set $c (f64.div (f64.sub (local.get $z) (f64.mul (local.get $df) (local.get $df)))
                         (f64.add (local.get $s) (local.get $df))))
  (local.set $w (f64.add (f64.mul (call $math_asin_r (local.get $z)) (local.get $s)) (local.get $c)))
  (f64.mul (f64.const 2) (f64.add (local.get $df) (local.get $w))))

;; ---------------------------------------------------------------------------
;; atan2(y, x). Source: FreeBSD msun e_atan2.c (Sun, 1993) via musl atan2.c.
;;
;; Method: reduce to atan(|y/x|) and fix the quadrant from the signs.
;;   x > 0: arg = atan(y/x);  x < 0: arg = pi - atan(y/(-x)), with pi as a
;;   hi + lo pair. The sign of y is applied last.
;;
;; Special cases (from the source):
;;   atan2(anything, NaN) and atan2(NaN, anything) are NaN
;;   atan2(+-0, +x) = +-0;  atan2(+-0, -x) = +-pi
;;   atan2(+-y, 0) = +-pi/2 for y != 0
;;   atan2(+-y, +inf) = +-0;  atan2(+-y, -inf) = +-pi for finite y
;;   atan2(+-inf, +inf) = +-pi/4;  atan2(+-inf, -inf) = +-3pi/4
;;   atan2(+-inf, x) = +-pi/2 for finite x
;; ---------------------------------------------------------------------------

(func $math_atan2 (param $y f64) (param $x f64) (result f64)
  (local $ix i32) (local $lx i32) (local $iy i32) (local $ly i32) (local $m i32)
  (local $z f64)
  (if (i32.or (f64.ne (local.get $x) (local.get $x)) (f64.ne (local.get $y) (local.get $y)))
    (then (return (f64.add (local.get $x) (local.get $y)))))
  (local.set $ix (call $math_hw (local.get $x)))
  (local.set $lx (call $math_lw (local.get $x)))
  (local.set $iy (call $math_hw (local.get $y)))
  (local.set $ly (call $math_lw (local.get $y)))
  ;; x = 1.0
  (if (i32.eqz (i32.or (i32.sub (local.get $ix) (i32.const 0x3ff00000)) (local.get $lx)))
    (then (return (call $math_atan (local.get $y)))))
  ;; m = 2*sign(x) + sign(y)
  (local.set $m (i32.or (i32.and (i32.shr_u (local.get $iy) (i32.const 31)) (i32.const 1))
                        (i32.and (i32.shr_u (local.get $ix) (i32.const 30)) (i32.const 2))))
  (local.set $ix (i32.and (local.get $ix) (i32.const 0x7fffffff)))
  (local.set $iy (i32.and (local.get $iy) (i32.const 0x7fffffff)))
  ;; y = 0
  (if (i32.eqz (i32.or (local.get $iy) (local.get $ly)))
    (then
      (if (i32.lt_u (local.get $m) (i32.const 2)) (then (return (local.get $y))))
      (if (i32.eq (local.get $m) (i32.const 2)) (then (return (f64.const 3.1415926535897931160E+00))))
      (return (f64.const -3.1415926535897931160E+00))))
  ;; x = 0
  (if (i32.eqz (i32.or (local.get $ix) (local.get $lx)))
    (then (return (select (f64.const -1.57079632679489655800e+00) (f64.const 1.57079632679489655800e+00)
                          (i32.and (local.get $m) (i32.const 1))))))
  ;; x is inf
  (if (i32.eq (local.get $ix) (i32.const 0x7ff00000))
    (then
      (if (i32.eq (local.get $iy) (i32.const 0x7ff00000))
        (then
          (if (i32.eq (local.get $m) (i32.const 0)) (then (return (f64.const 7.85398163397448278999e-01))))  ;; pi/4
          (if (i32.eq (local.get $m) (i32.const 1)) (then (return (f64.const -7.85398163397448278999e-01))))
          (if (i32.eq (local.get $m) (i32.const 2)) ;; 3pi/4
            (then (return (f64.div (f64.mul (f64.const 3) (f64.const 3.1415926535897931160E+00)) (f64.const 4)))))
          (return (f64.neg (f64.div (f64.mul (f64.const 3) (f64.const 3.1415926535897931160E+00)) (f64.const 4))))))
      (if (i32.eq (local.get $m) (i32.const 0)) (then (return (f64.const 0))))
      (if (i32.eq (local.get $m) (i32.const 1)) (then (return (f64.const -0))))
      (if (i32.eq (local.get $m) (i32.const 2)) (then (return (f64.const 3.1415926535897931160E+00))))
      (return (f64.const -3.1415926535897931160E+00))))
  ;; |y/x| > 2^64, or y is inf
  (if (i32.or (i32.lt_u (i32.add (local.get $ix) (i32.const 0x04000000)) (local.get $iy))
              (i32.eq (local.get $iy) (i32.const 0x7ff00000)))
    (then (return (select (f64.const -1.57079632679489655800e+00) (f64.const 1.57079632679489655800e+00)
                          (i32.and (local.get $m) (i32.const 1))))))
  ;; z = atan(|y/x|) without spurious underflow
  (if (i32.and (i32.and (local.get $m) (i32.const 2))
               (i32.lt_u (i32.add (local.get $iy) (i32.const 0x04000000)) (local.get $ix)))
    (then (local.set $z (f64.const 0))) ;; |y/x| < 2^-64, x < 0
    (else (local.set $z (call $math_atan (f64.abs (f64.div (local.get $y) (local.get $x)))))))
  (if (i32.eq (local.get $m) (i32.const 0)) (then (return (local.get $z))))
  (if (i32.eq (local.get $m) (i32.const 1)) (then (return (f64.neg (local.get $z)))))
  (if (i32.eq (local.get $m) (i32.const 2))
    (then (return (f64.sub (f64.const 3.1415926535897931160E+00)                 ;; pi    0x400921fb_54442d18
      (f64.sub (local.get $z) (f64.const 1.2246467991473531772E-16))))))        ;; pi_lo 0x3ca1a626_33145c07
  (f64.sub (f64.sub (local.get $z) (f64.const 1.2246467991473531772E-16)) (f64.const 3.1415926535897931160E+00)))

;; ---------------------------------------------------------------------------
;; pow(x, y). Source: FreeBSD msun e_pow.c (Sun, 2004) via musl pow.c.
;;
;; Method: compute log2(x) in extra precision as t1 + t2, then
;; z = y * log2(x) as p_h + p_l, then 2^z = 2^n * exp(z' * ln2) with the same
;; polynomial as exp. The log2 uses ss = (x-1)/(x+1) or (x-1.5)/(x+1.5) with
;; the degree-6 polynomial L1..L6 for log2(1+s)/s.
;;
;; Special cases (C Annex F, as implemented by the source):
;;   pow(x, +-0) = 1 for any x, including NaN
;;   pow(1, y) = 1 for any y, including NaN
;;   pow(NaN, y) and pow(x, NaN) are NaN otherwise
;;   pow(+-0, y<0 odd int) = +-inf;  pow(+-0, y<0 else) = +inf
;;   pow(+-0, y>0 odd int) = +-0;    pow(+-0, y>0 else) = +0
;;   pow(-1, +-inf) = 1
;;   pow(|x|<1, -inf) = +inf;  pow(|x|>1, -inf) = +0;  and the reverse for +inf
;;   pow(+inf, y) = +inf for y > 0, +0 for y < 0
;;   pow(-inf, y) = pow(-0, -y)
;;   pow(x<0, non-integer y) = NaN
;;
;; Accuracy: the source states the error is below 1 ulp. Integer results
;; that are representable are exact.
;; ---------------------------------------------------------------------------

(func $math_pow (param $x f64) (param $y f64) (result f64)
  (local $hx i32) (local $lx i32) (local $hy i32) (local $ly i32)
  (local $ix i32) (local $iy i32) (local $yisint i32) (local $k i32) (local $j i32)
  (local $i i32) (local $n i32)
  (local $ax f64) (local $s f64) (local $t f64) (local $t1 f64) (local $t2 f64)
  (local $u f64) (local $v f64) (local $w f64) (local $z f64)
  (local $ss f64) (local $s_h f64) (local $s_l f64) (local $t_h f64) (local $t_l f64)
  (local $r f64) (local $s2 f64) (local $p_h f64) (local $p_l f64) (local $z_h f64) (local $z_l f64)
  (local $y1 f64) (local $bp f64)
  (local.set $hx (call $math_hw (local.get $x)))
  (local.set $lx (call $math_lw (local.get $x)))
  (local.set $hy (call $math_hw (local.get $y)))
  (local.set $ly (call $math_lw (local.get $y)))
  (local.set $ix (i32.and (local.get $hx) (i32.const 0x7fffffff)))
  (local.set $iy (i32.and (local.get $hy) (i32.const 0x7fffffff)))
  ;; x**0 = 1, even for NaN x
  (if (i32.eqz (i32.or (local.get $iy) (local.get $ly))) (then (return (f64.const 1))))
  ;; 1**y = 1, even for NaN y
  (if (i32.and (i32.eq (local.get $hx) (i32.const 0x3ff00000)) (i32.eqz (local.get $lx)))
    (then (return (f64.const 1))))
  ;; NaN if either argument is NaN
  (if (i32.or
        (i32.or (i32.gt_s (local.get $ix) (i32.const 0x7ff00000))
                (i32.and (i32.eq (local.get $ix) (i32.const 0x7ff00000)) (i32.ne (local.get $lx) (i32.const 0))))
        (i32.or (i32.gt_s (local.get $iy) (i32.const 0x7ff00000))
                (i32.and (i32.eq (local.get $iy) (i32.const 0x7ff00000)) (i32.ne (local.get $ly) (i32.const 0)))))
    (then (return (f64.add (local.get $x) (local.get $y)))))
  ;; yisint: 0 = y is not an integer, 1 = odd integer, 2 = even integer.
  ;; Only needed when x < 0.
  (local.set $yisint (i32.const 0))
  (if (i32.lt_s (local.get $hx) (i32.const 0))
    (then
      (if (i32.ge_s (local.get $iy) (i32.const 0x43400000)) ;; |y| >= 2^53: even integer
        (then (local.set $yisint (i32.const 2)))
        (else
          (if (i32.ge_s (local.get $iy) (i32.const 0x3ff00000)) ;; |y| >= 1
            (then
              (local.set $k (i32.sub (i32.shr_s (local.get $iy) (i32.const 20)) (i32.const 0x3ff))) ;; exponent
              (if (i32.gt_s (local.get $k) (i32.const 20))
                (then
                  (local.set $j (i32.shr_u (local.get $ly) (i32.sub (i32.const 52) (local.get $k))))
                  (if (i32.eq (i32.shl (local.get $j) (i32.sub (i32.const 52) (local.get $k))) (local.get $ly))
                    (then (local.set $yisint (i32.sub (i32.const 2) (i32.and (local.get $j) (i32.const 1)))))))
                (else
                  (if (i32.eqz (local.get $ly))
                    (then
                      (local.set $j (i32.shr_s (local.get $iy) (i32.sub (i32.const 20) (local.get $k))))
                      (if (i32.eq (i32.shl (local.get $j) (i32.sub (i32.const 20) (local.get $k))) (local.get $iy))
                        (then (local.set $yisint (i32.sub (i32.const 2) (i32.and (local.get $j) (i32.const 1))))))))))))))))
  ;; Special values of y.
  (if (i32.eqz (local.get $ly))
    (then
      (if (i32.eq (local.get $iy) (i32.const 0x7ff00000)) ;; y is +-inf
        (then
          (if (i32.eqz (i32.or (i32.sub (local.get $ix) (i32.const 0x3ff00000)) (local.get $lx)))
            (then (return (f64.const 1)))) ;; (-1)**+-inf = 1
          (if (i32.ge_s (local.get $ix) (i32.const 0x3ff00000))
            (then ;; (|x|>1)**+-inf = inf, 0
              (return (select (local.get $y) (f64.const 0) (i32.ge_s (local.get $hy) (i32.const 0))))))
          ;; (|x|<1)**+-inf = 0, inf
          (return (select (f64.const 0) (f64.neg (local.get $y)) (i32.ge_s (local.get $hy) (i32.const 0))))))
      (if (i32.eq (local.get $iy) (i32.const 0x3ff00000)) ;; y is +-1
        (then (return (select (local.get $x) (f64.div (f64.const 1) (local.get $x)) (i32.ge_s (local.get $hy) (i32.const 0))))))
      (if (i32.eq (local.get $hy) (i32.const 0x40000000)) ;; y is 2
        (then (return (f64.mul (local.get $x) (local.get $x)))))
      (if (i32.eq (local.get $hy) (i32.const 0x3fe00000)) ;; y is 0.5
        (then
          (if (i32.ge_s (local.get $hx) (i32.const 0)) ;; x >= +0
            (then (return (f64.sqrt (local.get $x)))))))))
  (local.set $ax (f64.abs (local.get $x)))
  ;; Special values of x: +-0, +-inf, +-1.
  (if (i32.eqz (local.get $lx))
    (then
      (if (i32.or (i32.or (i32.eq (local.get $ix) (i32.const 0x7ff00000)) (i32.eqz (local.get $ix)))
                  (i32.eq (local.get $ix) (i32.const 0x3ff00000)))
        (then
          (local.set $z (local.get $ax))
          (if (i32.lt_s (local.get $hy) (i32.const 0))
            (then (local.set $z (f64.div (f64.const 1) (local.get $z)))))
          (if (i32.lt_s (local.get $hx) (i32.const 0))
            (then
              (if (i32.eqz (i32.or (i32.sub (local.get $ix) (i32.const 0x3ff00000)) (local.get $yisint)))
                (then ;; (-1)**non-int is NaN
                  (local.set $z (f64.div (f64.sub (local.get $z) (local.get $z)) (f64.sub (local.get $z) (local.get $z)))))
                (else
                  (if (i32.eq (local.get $yisint) (i32.const 1))
                    (then (local.set $z (f64.neg (local.get $z)))))))))
          (return (local.get $z))))))
  ;; Sign of the result.
  (local.set $s (f64.const 1))
  (if (i32.lt_s (local.get $hx) (i32.const 0))
    (then
      (if (i32.eqz (local.get $yisint)) ;; (x<0)**(non-int) is NaN
        (then (return (f64.div (f64.sub (local.get $x) (local.get $x)) (f64.sub (local.get $x) (local.get $x))))))
      (if (i32.eq (local.get $yisint) (i32.const 1))
        (then (local.set $s (f64.const -1))))))
  ;; |y| is huge.
  (if (i32.gt_s (local.get $iy) (i32.const 0x41e00000)) ;; |y| > 2^31
    (then
      (if (i32.gt_s (local.get $iy) (i32.const 0x43f00000)) ;; |y| > 2^64: must overflow or underflow
        (then
          (if (i32.le_s (local.get $ix) (i32.const 0x3fefffff))
            (then (return (select (f64.const inf) (f64.const 0) (i32.lt_s (local.get $hy) (i32.const 0))))))
          (if (i32.ge_s (local.get $ix) (i32.const 0x3ff00000))
            (then (return (select (f64.const inf) (f64.const 0) (i32.gt_s (local.get $hy) (i32.const 0))))))))
      ;; Overflow or underflow if x is not close to one.
      (if (i32.lt_s (local.get $ix) (i32.const 0x3fefffff))
        (then (return (f64.mul (local.get $s) (select (f64.const inf) (f64.const 0) (i32.lt_s (local.get $hy) (i32.const 0)))))))
      (if (i32.gt_s (local.get $ix) (i32.const 0x3ff00000))
        (then (return (f64.mul (local.get $s) (select (f64.const inf) (f64.const 0) (i32.gt_s (local.get $hy) (i32.const 0)))))))
      ;; Now |1-x| <= 2^-20: log2(x) from x - x^2/2 + x^3/3 - x^4/4.
      (local.set $t (f64.sub (local.get $ax) (f64.const 1))) ;; t has 20 trailing zeros
      (local.set $w (f64.mul (f64.mul (local.get $t) (local.get $t))
        (f64.sub (f64.const 0.5) (f64.mul (local.get $t)
          (f64.sub (f64.const 0.3333333333333333333333) (f64.mul (local.get $t) (f64.const 0.25)))))))
      (local.set $u (f64.mul (f64.const 1.44269502162933349609e+00) (local.get $t)))   ;; ivln2_h 0x3ff71547_60000000, 21 bits
      (local.set $v (f64.sub (f64.mul (local.get $t) (f64.const 1.92596299112661746887e-08))  ;; ivln2_l 0x3e54ae0b_f85ddf44
                             (f64.mul (local.get $w) (f64.const 1.44269504088896338700e+00)))) ;; ivln2 0x3ff71547_652b82fe
      (local.set $t1 (call $math_trunc_lw (f64.add (local.get $u) (local.get $v))))
      (local.set $t2 (f64.sub (local.get $v) (f64.sub (local.get $t1) (local.get $u)))))
    (else
      (local.set $n (i32.const 0))
      (if (i32.lt_s (local.get $ix) (i32.const 0x00100000)) ;; subnormal x
        (then
          (local.set $ax (f64.mul (local.get $ax) (f64.const 0x1p53)))
          (local.set $n (i32.const -53))
          (local.set $ix (call $math_hw (local.get $ax)))))
      (local.set $n (i32.add (local.get $n) (i32.sub (i32.shr_s (local.get $ix) (i32.const 20)) (i32.const 0x3ff))))
      (local.set $j (i32.and (local.get $ix) (i32.const 0x000fffff)))
      ;; Determine the interval.
      (local.set $ix (i32.or (local.get $j) (i32.const 0x3ff00000)))
      (if (i32.le_s (local.get $j) (i32.const 0x3988e)) ;; |x| < sqrt(3/2)
        (then (local.set $k (i32.const 0)))
        (else
          (if (i32.lt_s (local.get $j) (i32.const 0xbb67a)) ;; |x| < sqrt(3)
            (then (local.set $k (i32.const 1)))
            (else
              (local.set $k (i32.const 0))
              (local.set $n (i32.add (local.get $n) (i32.const 1)))
              (local.set $ix (i32.sub (local.get $ix) (i32.const 0x00100000)))))))
      (local.set $ax (call $math_set_hw (local.get $ax) (local.get $ix)))
      (local.set $bp (select (f64.const 1.5) (f64.const 1) (local.get $k)))
      ;; ss = s_h + s_l = (x-1)/(x+1) or (x-1.5)/(x+1.5)
      (local.set $u (f64.sub (local.get $ax) (local.get $bp)))
      (local.set $v (f64.div (f64.const 1) (f64.add (local.get $ax) (local.get $bp))))
      (local.set $ss (f64.mul (local.get $u) (local.get $v)))
      (local.set $s_h (call $math_trunc_lw (local.get $ss)))
      ;; t_h = ax + bp, high part
      (local.set $t_h (call $math_from_hw (i32.add
        (i32.add (i32.or (i32.shr_u (local.get $ix) (i32.const 1)) (i32.const 0x20000000)) (i32.const 0x00080000))
        (i32.shl (local.get $k) (i32.const 18)))))
      (local.set $t_l (f64.sub (local.get $ax) (f64.sub (local.get $t_h) (local.get $bp))))
      (local.set $s_l (f64.mul (local.get $v)
        (f64.sub (f64.sub (local.get $u) (f64.mul (local.get $s_h) (local.get $t_h)))
                 (f64.mul (local.get $s_h) (local.get $t_l)))))
      ;; log(ax)
      (local.set $s2 (f64.mul (local.get $ss) (local.get $ss)))
      (local.set $r (f64.mul (f64.mul (local.get $s2) (local.get $s2))
        (f64.add (f64.const 5.99999999999994648725e-01) (f64.mul (local.get $s2)   ;; L1 0x3fe33333_33333303
        (f64.add (f64.const 4.28571428578550184252e-01) (f64.mul (local.get $s2)   ;; L2 0x3fdb6db6_db6fabff
        (f64.add (f64.const 3.33333329818377432918e-01) (f64.mul (local.get $s2)   ;; L3 0x3fd55555_518f264d
        (f64.add (f64.const 2.72728123808534006489e-01) (f64.mul (local.get $s2)   ;; L4 0x3fd17460_a91d4101
        (f64.add (f64.const 2.30660745775561754067e-01) (f64.mul (local.get $s2)   ;; L5 0x3fcd864a_93c9db65
                 (f64.const 2.06975017800338417784e-01)))))))))))))               ;; L6 0x3fca7e28_4a454eef
      (local.set $r (f64.add (local.get $r) (f64.mul (local.get $s_l) (f64.add (local.get $s_h) (local.get $ss)))))
      (local.set $s2 (f64.mul (local.get $s_h) (local.get $s_h)))
      (local.set $t_h (call $math_trunc_lw (f64.add (f64.add (f64.const 3) (local.get $s2)) (local.get $r))))
      (local.set $t_l (f64.sub (local.get $r) (f64.sub (f64.sub (local.get $t_h) (f64.const 3)) (local.get $s2))))
      ;; u + v = ss*(1+...)
      (local.set $u (f64.mul (local.get $s_h) (local.get $t_h)))
      (local.set $v (f64.add (f64.mul (local.get $s_l) (local.get $t_h)) (f64.mul (local.get $t_l) (local.get $ss))))
      ;; 2/(3log2)*(ss+...)
      (local.set $p_h (call $math_trunc_lw (f64.add (local.get $u) (local.get $v))))
      (local.set $p_l (f64.sub (local.get $v) (f64.sub (local.get $p_h) (local.get $u))))
      (local.set $z_h (f64.mul (f64.const 9.61796700954437255859e-01) (local.get $p_h)))   ;; cp_h 0x3feec709_e0000000
      (local.set $z_l (f64.add
        (f64.add (f64.mul (f64.const -7.02846165095275826516e-09) (local.get $p_h))       ;; cp_l 0xbe3e2fe0_145b01f5
                 (f64.mul (local.get $p_l) (f64.const 9.61796693925975554329e-01)))       ;; cp   0x3feec709_dc3a03fd
        (select (f64.const 1.35003920212974897128e-08) (f64.const 0) (local.get $k))))    ;; dp_l[1] 0x3e4cfdeb_43cfd006
      ;; log2(ax) = (ss+..)*2/(3*log2) = n + dp_h + z_h + z_l
      (local.set $t (f64.convert_i32_s (local.get $n)))
      (local.set $bp (select (f64.const 5.84962487220764160156e-01) (f64.const 0) (local.get $k))) ;; dp_h[1] 0x3fe2b803_40000000
      (local.set $t1 (call $math_trunc_lw (f64.add (f64.add (f64.add (local.get $z_h) (local.get $z_l)) (local.get $bp)) (local.get $t))))
      (local.set $t2 (f64.sub (local.get $z_l)
        (f64.sub (f64.sub (f64.sub (local.get $t1) (local.get $t)) (local.get $bp)) (local.get $z_h))))))
  ;; Split y into y1 + y2 and compute (y1+y2)*(t1+t2).
  (local.set $y1 (call $math_trunc_lw (local.get $y)))
  (local.set $p_l (f64.add (f64.mul (f64.sub (local.get $y) (local.get $y1)) (local.get $t1))
                           (f64.mul (local.get $y) (local.get $t2))))
  (local.set $p_h (f64.mul (local.get $y1) (local.get $t1)))
  (local.set $z (f64.add (local.get $p_l) (local.get $p_h)))
  (local.set $j (call $math_hw (local.get $z)))
  (local.set $i (call $math_lw (local.get $z)))
  (if (i32.ge_s (local.get $j) (i32.const 0x40900000)) ;; z >= 1024
    (then
      (if (i32.or (i32.sub (local.get $j) (i32.const 0x40900000)) (local.get $i)) ;; z > 1024
        (then (return (f64.mul (local.get $s) (f64.const inf)))))
      (if (f64.gt (f64.add (local.get $p_l) (f64.const 8.0085662595372944372e-017)) ;; ovt
                  (f64.sub (local.get $z) (local.get $p_h)))
        (then (return (f64.mul (local.get $s) (f64.const inf))))))
    (else
      (if (i32.ge_s (i32.and (local.get $j) (i32.const 0x7fffffff)) (i32.const 0x4090cc00)) ;; z <= -1075
        (then
          (if (i32.or (i32.sub (local.get $j) (i32.const 0xc090cc00)) (local.get $i)) ;; z < -1075
            (then (return (f64.mul (local.get $s) (f64.const 0)))))
          (if (f64.le (local.get $p_l) (f64.sub (local.get $z) (local.get $p_h)))
            (then (return (f64.mul (local.get $s) (f64.const 0)))))))))
  ;; Compute 2**(p_h+p_l).
  (local.set $i (i32.and (local.get $j) (i32.const 0x7fffffff)))
  (local.set $k (i32.sub (i32.shr_s (local.get $i) (i32.const 20)) (i32.const 0x3ff)))
  (local.set $n (i32.const 0))
  (if (i32.gt_s (local.get $i) (i32.const 0x3fe00000)) ;; |z| > 0.5: n = [z + 0.5]
    (then
      (local.set $n (i32.add (local.get $j) (i32.shr_s (i32.const 0x00100000) (i32.add (local.get $k) (i32.const 1)))))
      (local.set $k (i32.sub (i32.shr_s (i32.and (local.get $n) (i32.const 0x7fffffff)) (i32.const 20)) (i32.const 0x3ff)))
      (local.set $t (call $math_from_hw
        (i32.and (local.get $n) (i32.xor (i32.shr_s (i32.const 0x000fffff) (local.get $k)) (i32.const -1)))))
      (local.set $n (i32.shr_s (i32.or (i32.and (local.get $n) (i32.const 0x000fffff)) (i32.const 0x00100000))
                               (i32.sub (i32.const 20) (local.get $k))))
      (if (i32.lt_s (local.get $j) (i32.const 0))
        (then (local.set $n (i32.sub (i32.const 0) (local.get $n)))))
      (local.set $p_h (f64.sub (local.get $p_h) (local.get $t)))))
  (local.set $t (call $math_trunc_lw (f64.add (local.get $p_l) (local.get $p_h))))
  (local.set $u (f64.mul (local.get $t) (f64.const 6.93147182464599609375e-01)))         ;; lg2_h 0x3fe62e43_00000000
  (local.set $v (f64.add
    (f64.mul (f64.sub (local.get $p_l) (f64.sub (local.get $t) (local.get $p_h)))
             (f64.const 6.93147180559945286227e-01))                                    ;; lg2   0x3fe62e42_fefa39ef
    (f64.mul (local.get $t) (f64.const -1.90465429995776804525e-09))))                  ;; lg2_l 0xbe205c61_0ca86c39
  (local.set $z (f64.add (local.get $u) (local.get $v)))
  (local.set $w (f64.sub (local.get $v) (f64.sub (local.get $z) (local.get $u))))
  (local.set $t (f64.mul (local.get $z) (local.get $z)))
  (local.set $t1 (f64.sub (local.get $z) (f64.mul (local.get $t)
    (f64.add (f64.const 1.66666666666666019037e-01) (f64.mul (local.get $t)          ;; P1
    (f64.add (f64.const -2.77777777770155933842e-03) (f64.mul (local.get $t)         ;; P2
    (f64.add (f64.const 6.61375632143793436117e-05) (f64.mul (local.get $t)          ;; P3
    (f64.add (f64.const -1.65339022054652515390e-06) (f64.mul (local.get $t)         ;; P4
             (f64.const 4.13813679705723846039e-08))))))))))))                      ;; P5
  (local.set $r (f64.sub (f64.div (f64.mul (local.get $z) (local.get $t1)) (f64.sub (local.get $t1) (f64.const 2)))
                         (f64.add (local.get $w) (f64.mul (local.get $z) (local.get $w)))))
  (local.set $z (f64.sub (f64.const 1) (f64.sub (local.get $r) (local.get $z))))
  (local.set $j (i32.add (call $math_hw (local.get $z)) (i32.shl (local.get $n) (i32.const 20))))
  (if (i32.le_s (i32.shr_s (local.get $j) (i32.const 20)) (i32.const 0))
    (then (local.set $z (call $math_scalbn (local.get $z) (local.get $n)))) ;; subnormal output
    (else (local.set $z (call $math_set_hw (local.get $z) (local.get $j)))))
  (f64.mul (local.get $s) (local.get $z)))
