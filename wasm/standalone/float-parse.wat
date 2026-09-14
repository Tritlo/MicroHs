;; float-parse.wat: decimal floating-point literal parser.
;;
;; This fragment is part of the standalone WAT support module. The parent
;; concatenates it into that module. It defines these entry points:
;;
;;   $parse_f64(address, length) -> f64
;;   $parse_f32(address, length) -> f32
;;
;; The serialized program stores a Double literal as `&text` and a Float
;; literal as `&&text`. The loader passes the address and byte length of
;; `text` in linear memory. The text is read-only. The result is the IEEE 754
;; value nearest to the decimal value, with ties rounded to even. This is the
;; same value that a correct `strtod` or `strtof` returns. The Float result is
;; rounded once, directly from the decimal value. It is not rounded through
;; Double first.
;;
;; This fragment calls three functions that the parent provides:
;;
;;   $blob_alloc(size, mask) -> address   scratch allocation, no pointer words
;;   $blob_free(address)                  release the scratch allocation
;;   $fail(code)                          report a malformed literal, no return
;;
;; No function in this fragment invokes the evaluator or the collector.
;; The fragment does not use data segments, globals, or tables. All state
;; lives in locals and in one scratch allocation that each call frees.
;;
;; INPUT GRAMMAR
;;
;;   literal   ::= sign? (number | special)
;;   sign      ::= '+' | '-'
;;   number    ::= digits ('.' digits?)? exponent?
;;               | '.' digits exponent?
;;   digits    ::= ('0'..'9')+
;;   exponent  ::= ('e' | 'E') sign? digits
;;   special   ::= "inf" | "infinity" | "nan"       (case-insensitive)
;;
;; The whole input must match. No white space is allowed. This accepts every
;; form that the Haskell `show` instances produce ("1.0e-2", "Infinity",
;; "-Infinity", "NaN") and every form that the C runtime writes with "%.16g"
;; ("1e+20", "inf", "-nan"). A malformed literal calls $fail with the syntax
;; failure code (see $fp_fail_syntax). "nan" and "-nan" both give a quiet NaN.
;; A sign on "nan" sets the sign bit.
;;
;; ROUNDING ALGORITHM
;;
;; Step 1. Read the significant digits into a big integer M and track the
;; decimal exponent E so that the value is v = M * 10^E. Leading zeros are not
;; significant. Only the first 800 significant digits enter M. Each later
;; digit shifts E up by one. If any dropped digit is not zero, one more digit
;; with the value 1 is appended to M (the sticky digit). The sticky value
;; v' = (M*10 + 1) * 10^(E-1) lies strictly inside the same open interval
;; (M*10^E, (M+1)*10^E) as v. Every Double and every Double rounding
;; boundary has at most 768 significant decimal digits, so every boundary is
;; a multiple of 10^E when M holds 800 digits. The open interval contains no
;; boundary. Therefore v and v' round to the same result, and v' is never a
;; tie. The same holds for Float, whose boundaries have fewer digits.
;;
;; Step 2. Clamp the magnitude. With nd significant digits, v lies in
;; [10^(nd+E-1), 10^(nd+E)). If nd+E > 400, v is above every finite value
;; and the result is infinity. If nd+E < -400, v is below half of the
;; smallest subnormal and the result is zero. These bounds keep every big
;; integer below the buffer capacity. The general path handles all other
;; overflow and underflow through the exponent checks in step 5.
;;
;; Step 3. Form the exact fraction v = N / D with N = M * 10^max(E,0) and
;; D = 10^max(-E,0). Both are big integers.
;;
;; Step 4. Find e = floor(log2 v) from the bit lengths of N and D and one
;; exact comparison. Let P be the precision (53 for Double, 24 for Float) and
;; e2 = e - (P - 1), so that floor(v / 2^e2) has exactly P bits. If e2 is
;; below the subnormal exponent (-1074 for Double, -149 for Float), set e2 to
;; that minimum; the quotient then has fewer than P bits. Scale N or D by a
;; power of two so that q = floor(N / D) is the P-bit integer part of
;; v / 2^e2, and compute q with restoring binary division. The remainder r
;; satisfies v / 2^e2 = q + r / D with 0 <= r < D.
;;
;; Step 5. Round to nearest, ties to even: increment q when 2r > D, or when
;; 2r = D and q is odd. If q reaches 2^P, replace it by 2^(P-1) and add one
;; to e2. The value is q * 2^e2. If q = 0 the result is a signed zero. If
;; q < 2^(P-1) the result is subnormal with biased exponent zero. If
;; e2 + (P - 1) exceeds the maximum exponent (1023 for Double, 127 for
;; Float) the result is infinity. Otherwise the biased exponent is
;; e2 + (P - 1) + bias and the stored fraction is q - 2^(P-1).
;;
;; Every step is exact integer arithmetic. There are no tables and no
;; floating-point operations before the final bit assembly.
;;
;; BIG INTEGER REPRESENTATION
;;
;; A big integer occupies 4 + 4 * 192 bytes. Offset 0 holds the limb count n.
;; Offsets 4 .. 4 + 4n hold n little-endian 32-bit limbs, least significant
;; first. The top limb is not zero. The value zero has n = 0. The capacity is
;; 192 limbs (6144 bits). The largest value that the clamped inputs produce is
;; about 4100 bits. Capacity exhaustion calls $fail with the capacity failure
;; code (see $fp_fail_capacity). It cannot happen for valid input.
;;
;; SCRATCH LAYOUT
;;
;;   offset    0  i32  kind: 0 finite, 1 infinity, 2 NaN
;;   offset    4  i32  sign: 0 positive, 1 negative
;;   offset    8  i32  e2: binary exponent of the result
;;   offset   16  i64  q: integer significand, 0 <= q < 2^P
;;   offset   24  big  A: M, then N, then the dividend and remainder
;;   offset  796  big  B: D, then the scaled divisor
;;   offset 1568  big  T: temporary for the exponent comparison
;;   size   2340

  ;; Failure codes. The parent runtime owns the code space. Change the two
  ;; constants here when the parent assigns codes.
  (func $fp_fail_syntax
    (call $fail (i32.const 70))
    (unreachable))

  (func $fp_fail_capacity
    (call $fail (i32.const 71))
    (unreachable))

  ;; Address of limb $i of the big integer at $b.
  (func $fp_limb (param $b i32) (param $i i32) (result i32)
    (i32.add (local.get $b)
      (i32.add (i32.const 4) (i32.shl (local.get $i) (i32.const 2)))))

  ;; Set the big integer at $b to the small value $v.
  (func $fp_big_set (param $b i32) (param $v i32)
    (i32.store (local.get $b) (i32.ne (local.get $v) (i32.const 0)))
    (i32.store offset=4 (local.get $b) (local.get $v)))

  ;; Multiply the big integer at $b by $mul and add $add, in place.
  (func $fp_big_mul_add (param $b i32) (param $mul i32) (param $add i32)
    (local $n i32) (local $i i32) (local $p i32) (local $carry i64) (local $t i64)
    (local.set $n (i32.load (local.get $b)))
    (local.set $carry (i64.extend_i32_u (local.get $add)))
    (local.set $i (i32.const 0))
    (block $done
      (loop $next
        (br_if $done (i32.ge_u (local.get $i) (local.get $n)))
        (local.set $p (call $fp_limb (local.get $b) (local.get $i)))
        ;; limb * mul + carry < 2^64 because each factor is below 2^32.
        (local.set $t
          (i64.add
            (i64.mul (i64.load32_u (local.get $p))
              (i64.extend_i32_u (local.get $mul)))
            (local.get $carry)))
        (i64.store32 (local.get $p) (local.get $t))
        (local.set $carry (i64.shr_u (local.get $t) (i64.const 32)))
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $next)))
    (if (i64.ne (local.get $carry) (i64.const 0))
      (then
        (if (i32.ge_u (local.get $n) (i32.const 192))
          (then (call $fp_fail_capacity)))
        (i64.store32 (call $fp_limb (local.get $b) (local.get $n)) (local.get $carry))
        (i32.store (local.get $b) (i32.add (local.get $n) (i32.const 1))))))

  ;; Multiply the big integer at $b by 10^$k, in place.
  (func $fp_big_mul_pow10 (param $b i32) (param $k i32)
    (block $done9
      (loop $next9
        (br_if $done9 (i32.lt_s (local.get $k) (i32.const 9)))
        (call $fp_big_mul_add (local.get $b) (i32.const 1000000000) (i32.const 0))
        (local.set $k (i32.sub (local.get $k) (i32.const 9)))
        (br $next9)))
    (block $done
      (loop $next
        (br_if $done (i32.le_s (local.get $k) (i32.const 0)))
        (call $fp_big_mul_add (local.get $b) (i32.const 10) (i32.const 0))
        (local.set $k (i32.sub (local.get $k) (i32.const 1)))
        (br $next))))

  ;; Bit length of the big integer at $b. Zero has bit length 0.
  (func $fp_big_bits (param $b i32) (result i32)
    (local $n i32)
    (local.set $n (i32.load (local.get $b)))
    (if (result i32) (i32.eqz (local.get $n))
      (then (i32.const 0))
      (else
        (i32.sub
          (i32.shl (local.get $n) (i32.const 5))
          (i32.clz
            (i32.load
              (call $fp_limb (local.get $b)
                (i32.sub (local.get $n) (i32.const 1)))))))))

  ;; Copy the big integer at $src to $dst.
  (func $fp_big_copy (param $dst i32) (param $src i32)
    (local $n i32) (local $i i32)
    (local.set $n (i32.load (local.get $src)))
    (i32.store (local.get $dst) (local.get $n))
    (local.set $i (i32.const 0))
    (block $done
      (loop $next
        (br_if $done (i32.ge_u (local.get $i) (local.get $n)))
        (i32.store (call $fp_limb (local.get $dst) (local.get $i))
          (i32.load (call $fp_limb (local.get $src) (local.get $i))))
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $next))))

  ;; Shift the big integer at $b left by $bits, in place.
  (func $fp_big_shl (param $b i32) (param $bits i32)
    (local $n i32) (local $i i32) (local $r i32) (local $limbs i32) (local $top i32)
    (local.set $n (i32.load (local.get $b)))
    (if (i32.eqz (local.get $n)) (then (return)))
    (local.set $limbs (i32.shr_u (local.get $bits) (i32.const 5)))
    (local.set $r (i32.and (local.get $bits) (i32.const 31)))
    (if (local.get $r)
      (then
        ;; First shift inside the limbs. The top limb can spill over.
        (local.set $top
          (i32.shr_u
            (i32.load (call $fp_limb (local.get $b) (i32.sub (local.get $n) (i32.const 1))))
            (i32.sub (i32.const 32) (local.get $r))))
        (local.set $i (i32.sub (local.get $n) (i32.const 1)))
        (block $done
          (loop $next
            (br_if $done (i32.eqz (local.get $i)))
            (i32.store (call $fp_limb (local.get $b) (local.get $i))
              (i32.or
                (i32.shl (i32.load (call $fp_limb (local.get $b) (local.get $i)))
                  (local.get $r))
                (i32.shr_u
                  (i32.load (call $fp_limb (local.get $b) (i32.sub (local.get $i) (i32.const 1))))
                  (i32.sub (i32.const 32) (local.get $r)))))
            (local.set $i (i32.sub (local.get $i) (i32.const 1)))
            (br $next)))
        (i32.store offset=4 (local.get $b)
          (i32.shl (i32.load offset=4 (local.get $b)) (local.get $r)))
        (if (local.get $top)
          (then
            (if (i32.ge_u (local.get $n) (i32.const 192))
              (then (call $fp_fail_capacity)))
            (i32.store (call $fp_limb (local.get $b) (local.get $n)) (local.get $top))
            (local.set $n (i32.add (local.get $n) (i32.const 1)))))))
    (if (local.get $limbs)
      (then
        ;; Then move whole limbs up and clear the low limbs.
        (if (i32.gt_u (i32.add (local.get $n) (local.get $limbs)) (i32.const 192))
          (then (call $fp_fail_capacity)))
        (local.set $i (local.get $n))
        (block $done_move
          (loop $move
            (br_if $done_move (i32.eqz (local.get $i)))
            (local.set $i (i32.sub (local.get $i) (i32.const 1)))
            (i32.store
              (call $fp_limb (local.get $b) (i32.add (local.get $i) (local.get $limbs)))
              (i32.load (call $fp_limb (local.get $b) (local.get $i))))
            (br $move)))
        (local.set $i (i32.const 0))
        (block $done_clear
          (loop $clear
            (br_if $done_clear (i32.ge_u (local.get $i) (local.get $limbs)))
            (i32.store (call $fp_limb (local.get $b) (local.get $i)) (i32.const 0))
            (local.set $i (i32.add (local.get $i) (i32.const 1)))
            (br $clear)))
        (local.set $n (i32.add (local.get $n) (local.get $limbs)))))
    (i32.store (local.get $b) (local.get $n)))

  ;; Shift the big integer at $b right by one bit, in place.
  (func $fp_big_shr1 (param $b i32)
    (local $n i32) (local $i i32) (local $last i32)
    (local.set $n (i32.load (local.get $b)))
    (if (i32.eqz (local.get $n)) (then (return)))
    (local.set $last (i32.sub (local.get $n) (i32.const 1)))
    (local.set $i (i32.const 0))
    (block $done
      (loop $next
        (br_if $done (i32.ge_u (local.get $i) (local.get $last)))
        (i32.store (call $fp_limb (local.get $b) (local.get $i))
          (i32.or
            (i32.shr_u (i32.load (call $fp_limb (local.get $b) (local.get $i))) (i32.const 1))
            (i32.shl
              (i32.load (call $fp_limb (local.get $b) (i32.add (local.get $i) (i32.const 1))))
              (i32.const 31))))
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $next)))
    (i32.store (call $fp_limb (local.get $b) (local.get $last))
      (i32.shr_u (i32.load (call $fp_limb (local.get $b) (local.get $last))) (i32.const 1)))
    (if (i32.eqz (i32.load (call $fp_limb (local.get $b) (local.get $last))))
      (then (i32.store (local.get $b) (local.get $last)))))

  ;; Compare two big integers. Return -1, 0, or 1.
  (func $fp_big_cmp (param $a i32) (param $b i32) (result i32)
    (local $na i32) (local $nb i32) (local $i i32) (local $x i32) (local $y i32)
    (local.set $na (i32.load (local.get $a)))
    (local.set $nb (i32.load (local.get $b)))
    (if (i32.ne (local.get $na) (local.get $nb))
      (then
        (return
          (select (i32.const 1) (i32.const -1)
            (i32.gt_u (local.get $na) (local.get $nb))))))
    (local.set $i (local.get $na))
    (block $done
      (loop $next
        (br_if $done (i32.eqz (local.get $i)))
        (local.set $i (i32.sub (local.get $i) (i32.const 1)))
        (local.set $x (i32.load (call $fp_limb (local.get $a) (local.get $i))))
        (local.set $y (i32.load (call $fp_limb (local.get $b) (local.get $i))))
        (if (i32.ne (local.get $x) (local.get $y))
          (then
            (return
              (select (i32.const 1) (i32.const -1)
                (i32.gt_u (local.get $x) (local.get $y))))))
        (br $next)))
    (i32.const 0))

  ;; Subtract the big integer at $b from the one at $a, in place.
  ;; The caller guarantees $a >= $b.
  (func $fp_big_sub (param $a i32) (param $b i32)
    (local $na i32) (local $nb i32) (local $i i32) (local $p i32) (local $t i64)
    (local $borrow i64)
    (local.set $na (i32.load (local.get $a)))
    (local.set $nb (i32.load (local.get $b)))
    (local.set $i (i32.const 0))
    (block $done
      (loop $next
        (br_if $done (i32.ge_u (local.get $i) (local.get $na)))
        ;; After the last limb of $b only the borrow remains.
        (br_if $done
          (i32.and (i32.ge_u (local.get $i) (local.get $nb))
            (i64.eqz (local.get $borrow))))
        (local.set $p (call $fp_limb (local.get $a) (local.get $i)))
        (local.set $t
          (i64.sub
            (i64.sub (i64.load32_u (local.get $p)) (local.get $borrow))
            (if (result i64) (i32.lt_u (local.get $i) (local.get $nb))
              (then (i64.load32_u (call $fp_limb (local.get $b) (local.get $i))))
              (else (i64.const 0)))))
        (i64.store32 (local.get $p) (local.get $t))
        (local.set $borrow (i64.extend_i32_u (i64.lt_s (local.get $t) (i64.const 0))))
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $next)))
    ;; Drop zero limbs at the top.
    (block $trimmed
      (loop $trim
        (br_if $trimmed (i32.eqz (local.get $na)))
        (br_if $trimmed
          (i32.load (call $fp_limb (local.get $a) (i32.sub (local.get $na) (i32.const 1)))))
        (local.set $na (i32.sub (local.get $na) (i32.const 1)))
        (br $trim)))
    (i32.store (local.get $a) (local.get $na)))

  ;; Byte $i of the input, folded to lower case for ASCII letters.
  (func $fp_lower (param $ptr i32) (param $i i32) (result i32)
    (i32.or (i32.load8_u (i32.add (local.get $ptr) (local.get $i))) (i32.const 32)))

  ;; Parse the literal and fill the scratch record. Return the scratch
  ;; address. The caller reads the record and frees it.
  ;; $prec is the significand width P. $e2min is the subnormal exponent.
  (func $fp_parse_core (param $ptr i32) (param $len i32) (param $prec i32) (param $e2min i32)
      (result i32)
    (local $s i32) (local $a i32) (local $b i32) (local $t i32)
    (local $i i32) (local $c i32) (local $sign i32)
    (local $seen_digit i32) (local $seen_point i32) (local $nd i32) (local $sticky i32)
    (local $dexp i32) (local $exp i32) (local $esign i32)
    (local $l i32) (local $e i32) (local $e2 i32) (local $k i32) (local $cmp i32)
    (local $q i64)
    (local.set $s (call $blob_alloc (i32.const 2340) (i32.const 0)))
    (local.set $a (i32.add (local.get $s) (i32.const 24)))
    (local.set $b (i32.add (local.get $s) (i32.const 796)))
    (local.set $t (i32.add (local.get $s) (i32.const 1568)))
    (i32.store (local.get $s) (i32.const 0))
    (i32.store offset=8 (local.get $s) (i32.const 0))
    (i64.store offset=16 (local.get $s) (i64.const 0))
    (call $fp_big_set (local.get $a) (i32.const 0))

    ;; Optional sign.
    (if (i32.lt_u (local.get $i) (local.get $len))
      (then
        (local.set $c (i32.load8_u (i32.add (local.get $ptr) (local.get $i))))
        (if (i32.eq (local.get $c) (i32.const 45))
          (then
            (local.set $sign (i32.const 1))
            (local.set $i (i32.const 1)))
          (else
            (if (i32.eq (local.get $c) (i32.const 43))
              (then (local.set $i (i32.const 1))))))))
    (i32.store offset=4 (local.get $s) (local.get $sign))

    ;; Special values: inf, infinity, nan.
    (local.set $k (i32.sub (local.get $len) (local.get $i)))
    (if (i32.or (i32.eq (local.get $k) (i32.const 3)) (i32.eq (local.get $k) (i32.const 8)))
      (then
        (if (i32.and
              (i32.and
                (i32.eq (call $fp_lower (local.get $ptr) (local.get $i)) (i32.const 105))
                (i32.eq (call $fp_lower (local.get $ptr) (i32.add (local.get $i) (i32.const 1)))
                  (i32.const 110)))
              (i32.eq (call $fp_lower (local.get $ptr) (i32.add (local.get $i) (i32.const 2)))
                (i32.const 102)))
          (then
            (if (i32.eq (local.get $k) (i32.const 8))
              (then
                (if (i32.eqz
                      (i32.and
                        (i32.and
                          (i32.eq (call $fp_lower (local.get $ptr) (i32.add (local.get $i) (i32.const 3)))
                            (i32.const 105))
                          (i32.eq (call $fp_lower (local.get $ptr) (i32.add (local.get $i) (i32.const 4)))
                            (i32.const 110)))
                        (i32.and
                          (i32.and
                            (i32.eq (call $fp_lower (local.get $ptr) (i32.add (local.get $i) (i32.const 5)))
                              (i32.const 105))
                            (i32.eq (call $fp_lower (local.get $ptr) (i32.add (local.get $i) (i32.const 6)))
                              (i32.const 116)))
                          (i32.eq (call $fp_lower (local.get $ptr) (i32.add (local.get $i) (i32.const 7)))
                            (i32.const 121)))))
                  (then (call $fp_fail_syntax)))))
            (i32.store (local.get $s) (i32.const 1))
            (return (local.get $s))))
        (if (i32.and
              (i32.and
                (i32.eq (local.get $k) (i32.const 3))
                (i32.eq (call $fp_lower (local.get $ptr) (local.get $i)) (i32.const 110)))
              (i32.and
                (i32.eq (call $fp_lower (local.get $ptr) (i32.add (local.get $i) (i32.const 1)))
                  (i32.const 97))
                (i32.eq (call $fp_lower (local.get $ptr) (i32.add (local.get $i) (i32.const 2)))
                  (i32.const 110))))
          (then
            (i32.store (local.get $s) (i32.const 2))
            (return (local.get $s))))))

    ;; Step 1: integer digits, optional point, fraction digits.
    (block $mantissa_done
      (loop $mantissa
        (br_if $mantissa_done (i32.ge_u (local.get $i) (local.get $len)))
        (local.set $c (i32.load8_u (i32.add (local.get $ptr) (local.get $i))))
        (if (i32.eq (local.get $c) (i32.const 46))
          (then
            (if (local.get $seen_point) (then (call $fp_fail_syntax)))
            (local.set $seen_point (i32.const 1))
            (local.set $i (i32.add (local.get $i) (i32.const 1)))
            (br $mantissa)))
        (local.set $c (i32.sub (local.get $c) (i32.const 48)))
        (br_if $mantissa_done (i32.gt_u (local.get $c) (i32.const 9)))
        (local.set $seen_digit (i32.const 1))
        (if (local.get $seen_point)
          (then (local.set $dexp (i32.sub (local.get $dexp) (i32.const 1)))))
        (if (i32.or (local.get $nd) (local.get $c))
          (then
            (if (i32.lt_u (local.get $nd) (i32.const 800))
              (then
                (call $fp_big_mul_add (local.get $a) (i32.const 10) (local.get $c))
                (local.set $nd (i32.add (local.get $nd) (i32.const 1))))
              (else
                ;; A dropped digit scales the kept digits by ten.
                (if (local.get $c) (then (local.set $sticky (i32.const 1))))
                (local.set $dexp (i32.add (local.get $dexp) (i32.const 1)))))))
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $mantissa)))
    (if (i32.eqz (local.get $seen_digit)) (then (call $fp_fail_syntax)))

    ;; Optional exponent. The magnitude saturates at 1000000, which is far
    ;; outside the clamp bounds of step 2.
    (if (i32.lt_u (local.get $i) (local.get $len))
      (then
        (if (i32.ne (i32.or (i32.load8_u (i32.add (local.get $ptr) (local.get $i))) (i32.const 32))
              (i32.const 101))
          (then (call $fp_fail_syntax)))
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (if (i32.lt_u (local.get $i) (local.get $len))
          (then
            (local.set $c (i32.load8_u (i32.add (local.get $ptr) (local.get $i))))
            (if (i32.eq (local.get $c) (i32.const 45))
              (then
                (local.set $esign (i32.const 1))
                (local.set $i (i32.add (local.get $i) (i32.const 1))))
              (else
                (if (i32.eq (local.get $c) (i32.const 43))
                  (then (local.set $i (i32.add (local.get $i) (i32.const 1)))))))))
        (if (i32.ge_u (local.get $i) (local.get $len)) (then (call $fp_fail_syntax)))
        (block $exp_done
          (loop $exp_loop
            (br_if $exp_done (i32.ge_u (local.get $i) (local.get $len)))
            (local.set $c
              (i32.sub (i32.load8_u (i32.add (local.get $ptr) (local.get $i))) (i32.const 48)))
            (if (i32.gt_u (local.get $c) (i32.const 9)) (then (call $fp_fail_syntax)))
            (if (i32.lt_s (local.get $exp) (i32.const 1000000))
              (then
                (local.set $exp
                  (i32.add (i32.mul (local.get $exp) (i32.const 10)) (local.get $c)))))
            (local.set $i (i32.add (local.get $i) (i32.const 1)))
            (br $exp_loop)))))

    ;; Append the sticky digit, then combine the exponents.
    (if (local.get $sticky)
      (then
        (call $fp_big_mul_add (local.get $a) (i32.const 10) (i32.const 1))
        (local.set $nd (i32.add (local.get $nd) (i32.const 1)))
        (local.set $dexp (i32.sub (local.get $dexp) (i32.const 1)))))
    (local.set $dexp
      (i32.add (local.get $dexp)
        (select (i32.sub (i32.const 0) (local.get $exp)) (local.get $exp) (local.get $esign))))

    ;; A zero significand gives a signed zero. q is already 0.
    (if (i32.eqz (local.get $nd)) (then (return (local.get $s))))

    ;; Step 2: clamp the decimal magnitude.
    (local.set $k (i32.add (local.get $nd) (local.get $dexp)))
    (if (i32.gt_s (local.get $k) (i32.const 400))
      (then
        (i32.store (local.get $s) (i32.const 1))
        (return (local.get $s))))
    (if (i32.lt_s (local.get $k) (i32.const -400))
      (then (return (local.get $s))))

    ;; Step 3: v = A / B with A = M * 10^max(E,0) and B = 10^max(-E,0).
    (call $fp_big_set (local.get $b) (i32.const 1))
    (if (i32.ge_s (local.get $dexp) (i32.const 0))
      (then (call $fp_big_mul_pow10 (local.get $a) (local.get $dexp)))
      (else (call $fp_big_mul_pow10 (local.get $b) (i32.sub (i32.const 0) (local.get $dexp)))))

    ;; Step 4: e = floor(log2 v). The bit lengths give l with
    ;; 2^(l-1) < v < 2^(l+1), so e is l or l - 1. One comparison decides.
    (local.set $l
      (i32.sub (call $fp_big_bits (local.get $a)) (call $fp_big_bits (local.get $b))))
    (if (i32.ge_s (local.get $l) (i32.const 0))
      (then
        (call $fp_big_copy (local.get $t) (local.get $b))
        (call $fp_big_shl (local.get $t) (local.get $l))
        (local.set $cmp (call $fp_big_cmp (local.get $a) (local.get $t))))
      (else
        (call $fp_big_copy (local.get $t) (local.get $a))
        (call $fp_big_shl (local.get $t) (i32.sub (i32.const 0) (local.get $l)))
        (local.set $cmp (call $fp_big_cmp (local.get $t) (local.get $b)))))
    (local.set $e
      (select (local.get $l) (i32.sub (local.get $l) (i32.const 1))
        (i32.ge_s (local.get $cmp) (i32.const 0))))
    (local.set $e2 (i32.sub (local.get $e) (i32.sub (local.get $prec) (i32.const 1))))
    (if (i32.lt_s (local.get $e2) (local.get $e2min))
      (then (local.set $e2 (local.get $e2min))))
    ;; Scale so that floor(A / B) = floor(v / 2^e2) < 2^P.
    (if (i32.le_s (local.get $e2) (i32.const 0))
      (then (call $fp_big_shl (local.get $a) (i32.sub (i32.const 0) (local.get $e2))))
      (else (call $fp_big_shl (local.get $b) (local.get $e2))))
    ;; Restoring division, one quotient bit per step, from bit P-1 down to
    ;; bit 0. B holds the divisor shifted by the current bit. After the last
    ;; step B is the unshifted divisor and A is the remainder.
    (call $fp_big_shl (local.get $b) (i32.sub (local.get $prec) (i32.const 1)))
    (local.set $k (local.get $prec))
    (block $div_done
      (loop $div
        (local.set $q (i64.shl (local.get $q) (i64.const 1)))
        (if (i32.ge_s (call $fp_big_cmp (local.get $a) (local.get $b)) (i32.const 0))
          (then
            (call $fp_big_sub (local.get $a) (local.get $b))
            (local.set $q (i64.or (local.get $q) (i64.const 1)))))
        (local.set $k (i32.sub (local.get $k) (i32.const 1)))
        (br_if $div_done (i32.eqz (local.get $k)))
        (call $fp_big_shr1 (local.get $b))
        (br $div)))

    ;; Step 5: round to nearest, ties to even. Compare 2r with D.
    (call $fp_big_shl (local.get $a) (i32.const 1))
    (local.set $cmp (call $fp_big_cmp (local.get $a) (local.get $b)))
    (if (i32.or (i32.gt_s (local.get $cmp) (i32.const 0))
          (i32.and (i32.eqz (local.get $cmp))
            (i32.wrap_i64 (i64.and (local.get $q) (i64.const 1)))))
      (then
        (local.set $q (i64.add (local.get $q) (i64.const 1)))
        (if (i64.eq (local.get $q) (i64.shl (i64.const 1) (i64.extend_i32_u (local.get $prec))))
          (then
            (local.set $q
              (i64.shl (i64.const 1) (i64.extend_i32_u (i32.sub (local.get $prec) (i32.const 1)))))
            (local.set $e2 (i32.add (local.get $e2) (i32.const 1)))))))
    (i32.store offset=8 (local.get $s) (local.get $e2))
    (i64.store offset=16 (local.get $s) (local.get $q))
    (local.get $s))

  ;; Parse a Double literal. P = 53, subnormal exponent -1074, bias 1023.
  (func $parse_f64 (param $ptr i32) (param $len i32) (result f64)
    (local $s i32) (local $kind i32) (local $e2 i32) (local $q i64) (local $bits i64)
    (local.set $s (call $fp_parse_core (local.get $ptr) (local.get $len) (i32.const 53) (i32.const -1074)))
    (local.set $kind (i32.load (local.get $s)))
    (local.set $bits (i64.shl (i64.extend_i32_u (i32.load offset=4 (local.get $s))) (i64.const 63)))
    (local.set $e2 (i32.load offset=8 (local.get $s)))
    (local.set $q (i64.load offset=16 (local.get $s)))
    (call $blob_free (local.get $s))
    (if (i32.eq (local.get $kind) (i32.const 2))
      (then (return (f64.reinterpret_i64 (i64.or (local.get $bits) (i64.const 0x7ff8000000000000))))))
    ;; The value is q * 2^e2. It overflows when e2 + 52 > 1023.
    (if (i32.or (i32.eq (local.get $kind) (i32.const 1)) (i32.gt_s (local.get $e2) (i32.const 971)))
      (then (return (f64.reinterpret_i64 (i64.or (local.get $bits) (i64.const 0x7ff0000000000000))))))
    (if (i64.lt_u (local.get $q) (i64.const 0x10000000000000))
      (then
        ;; Zero or subnormal: biased exponent 0, fraction q.
        (return (f64.reinterpret_i64 (i64.or (local.get $bits) (local.get $q))))))
    (f64.reinterpret_i64
      (i64.or (local.get $bits)
        (i64.or
          (i64.shl (i64.extend_i32_u (i32.add (local.get $e2) (i32.const 1075))) (i64.const 52))
          (i64.sub (local.get $q) (i64.const 0x10000000000000))))))

  ;; Parse a Float literal. P = 24, subnormal exponent -149, bias 127.
  (func $parse_f32 (param $ptr i32) (param $len i32) (result f32)
    (local $s i32) (local $kind i32) (local $e2 i32) (local $q i32) (local $bits i32)
    (local.set $s (call $fp_parse_core (local.get $ptr) (local.get $len) (i32.const 24) (i32.const -149)))
    (local.set $kind (i32.load (local.get $s)))
    (local.set $bits (i32.shl (i32.load offset=4 (local.get $s)) (i32.const 31)))
    (local.set $e2 (i32.load offset=8 (local.get $s)))
    (local.set $q (i32.wrap_i64 (i64.load offset=16 (local.get $s))))
    (call $blob_free (local.get $s))
    (if (i32.eq (local.get $kind) (i32.const 2))
      (then (return (f32.reinterpret_i32 (i32.or (local.get $bits) (i32.const 0x7fc00000))))))
    ;; The value is q * 2^e2. It overflows when e2 + 23 > 127.
    (if (i32.or (i32.eq (local.get $kind) (i32.const 1)) (i32.gt_s (local.get $e2) (i32.const 104)))
      (then (return (f32.reinterpret_i32 (i32.or (local.get $bits) (i32.const 0x7f800000))))))
    (if (i32.lt_u (local.get $q) (i32.const 0x800000))
      (then
        ;; Zero or subnormal: biased exponent 0, fraction q.
        (return (f32.reinterpret_i32 (i32.or (local.get $bits) (local.get $q))))))
    (f32.reinterpret_i32
      (i32.or (local.get $bits)
        (i32.or
          (i32.shl (i32.add (local.get $e2) (i32.const 150)) (i32.const 23))
          (i32.sub (local.get $q) (i32.const 0x800000))))))
