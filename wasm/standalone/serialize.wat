;; Serialize or print a graph without evaluation or collection.
;; $serialize(root, header) returns a byte-string node, including a final LF.
;; Header 1 selects the v8.4 postfix format. Header 0 selects prefix printing.
;; Unsupported values return zero and set the Serialize exception (code 8).
;; The output stays in memory until the complete graph has been checked.
;; An explicit walk stack and two bitmaps preserve sharing and graph cycles.

;; Divide a decimal-conversion big integer by ten. Return its remainder.
;; This uses the 192-limb representation from float-parse.wat.
(func $format_div10 (param $big i32) (result i32)
  (local $index i32) (local $word i32) (local $value i64) (local $remainder i64)
  (local.set $index (i32.load (local.get $big)))
  (block $divided
    (loop $divide
      (br_if $divided (i32.eqz (local.get $index)))
      (local.set $index (i32.sub (local.get $index) (i32.const 1)))
      (local.set $word (call $fp_limb (local.get $big) (local.get $index)))
      (local.set $value (i64.or (i64.shl (local.get $remainder) (i64.const 32))
        (i64.load32_u (local.get $word))))
      (i64.store32 (local.get $word) (i64.div_u (local.get $value) (i64.const 10)))
      (local.set $remainder (i64.rem_u (local.get $value) (i64.const 10)))
      (br $divide)))
  (local.set $index (i32.load (local.get $big)))
  (block $trimmed
    (loop $trim
      (br_if $trimmed (i32.eqz (local.get $index)))
      (br_if $trimmed (i32.load (call $fp_limb (local.get $big)
        (i32.sub (local.get $index) (i32.const 1)))))
      (local.set $index (i32.sub (local.get $index) (i32.const 1)))
      (br $trim)))
  (i32.store (local.get $big) (local.get $index))
  (i32.wrap_i64 (local.get $remainder)))

;; Format like C %.16g, then add .0 when there is no point or exponent.
;; Convert the exact binary value to an integer times a power of ten. Round
;; its decimal digits to 16 significant digits, with ties rounded to even.
;; This avoids host formatting and preserves the established text format.
;; The caller supplies at least 32 bytes. The result excludes the final NUL.
(func $format_f64 (param $value f64) (param $buffer i32) (result i32)
  (local $bits i64) (local $mantissa i64) (local $exponent i32)
  (local $binary_exp i32) (local $decimal_exp i32) (local $high_exp i32)
  (local $scratch i32) (local $reverse i32) (local $digits i32)
  (local $length i32) (local $kept i32) (local $index i32)
  (local $out i32) (local $byte i32) (local $round i32) (local $count i32)
  (local.set $bits (i64.reinterpret_f64 (local.get $value)))
  (if (i64.lt_s (local.get $bits) (i64.const 0))
    (then
      (i32.store8 (local.get $buffer) (i32.const 45))
      (local.set $out (i32.const 1))))
  (local.set $exponent (i32.wrap_i64
    (i64.and (i64.shr_u (local.get $bits) (i64.const 52)) (i64.const 2047))))
  (local.set $mantissa (i64.and (local.get $bits) (i64.const 0xfffffffffffff)))
  (if (i32.eq (local.get $exponent) (i32.const 2047))
    (then
      (i32.store (i32.add (local.get $buffer) (local.get $out))
        (select (i32.const 0x006e616e) (i32.const 0x00666e69)
          (i64.ne (local.get $mantissa) (i64.const 0))))
      (return (i32.add (local.get $out) (i32.const 3)))))
  (if (i32.eqz (local.get $exponent))
    (then
      (if (i64.eqz (local.get $mantissa))
        (then
          (i32.store (i32.add (local.get $buffer) (local.get $out)) (i32.const 0x00302e30))
          (return (i32.add (local.get $out) (i32.const 3)))))
      (local.set $binary_exp (i32.const -1074)))
    (else
      (local.set $mantissa (i64.or (local.get $mantissa) (i64.const 0x10000000000000)))
      (local.set $binary_exp (i32.sub (local.get $exponent) (i32.const 1075)))))
  (local.set $scratch (call $blob_alloc (i32.const 4096) (i32.const 0)))
  (local.set $reverse (i32.add (local.get $scratch) (i32.const 1024)))
  (local.set $digits (i32.add (local.get $scratch) (i32.const 2304)))
  (i32.store (local.get $scratch)
    (select (i32.const 2) (i32.const 1)
      (i64.ne (i64.shr_u (local.get $mantissa) (i64.const 32)) (i64.const 0))))
  (i64.store offset=4 align=4 (local.get $scratch) (local.get $mantissa))
  (if (i32.ge_s (local.get $binary_exp) (i32.const 0))
    (then (call $fp_big_shl (local.get $scratch) (local.get $binary_exp)))
    (else
      ;; M / 2^k = (M * 5^k) * 10^-k, exactly.
      (local.set $decimal_exp (local.get $binary_exp))
      (local.set $count (i32.sub (i32.const 0) (local.get $binary_exp)))
      (block $small_power
        (loop $power13
          (br_if $small_power (i32.lt_u (local.get $count) (i32.const 13)))
          (call $fp_big_mul_add (local.get $scratch) (i32.const 1220703125) (i32.const 0))
          (local.set $count (i32.sub (local.get $count) (i32.const 13)))
          (br $power13)))
      (block $multiplied
        (loop $power5
          (br_if $multiplied (i32.eqz (local.get $count)))
          (call $fp_big_mul_add (local.get $scratch) (i32.const 5) (i32.const 0))
          (local.set $count (i32.sub (local.get $count) (i32.const 1)))
          (br $power5)))))
  (loop $decimal_digits
    (i32.store8 (i32.add (local.get $reverse) (local.get $length))
      (i32.add (call $format_div10 (local.get $scratch)) (i32.const 48)))
    (local.set $length (i32.add (local.get $length) (i32.const 1)))
    (br_if $decimal_digits (i32.load (local.get $scratch))))
  (local.set $high_exp (i32.sub (i32.add (local.get $length) (local.get $decimal_exp)) (i32.const 1)))
  (local.set $kept (select (local.get $length) (i32.const 16)
    (i32.lt_u (local.get $length) (i32.const 16))))
  (local.set $index (i32.const 0))
  (loop $copy_digits
    (i32.store8 (i32.add (local.get $digits) (local.get $index))
      (i32.load8_u (i32.add (local.get $reverse)
        (i32.sub (i32.sub (local.get $length) (i32.const 1)) (local.get $index)))))
    (local.set $index (i32.add (local.get $index) (i32.const 1)))
    (br_if $copy_digits (i32.lt_u (local.get $index) (local.get $kept))))
  (if (i32.gt_u (local.get $length) (i32.const 16))
    (then
      (local.set $byte (i32.load8_u (i32.add (local.get $reverse)
        (i32.sub (local.get $length) (i32.const 17)))))
      (local.set $round (i32.gt_u (local.get $byte) (i32.const 53)))
      (if (i32.eq (local.get $byte) (i32.const 53))
        (then
          (local.set $round (i32.and (i32.load8_u offset=15 (local.get $digits)) (i32.const 1)))
          (local.set $index (i32.const 0))
          (block $sticky_found
            (loop $sticky
              (br_if $sticky_found (local.get $round))
              (br_if $sticky_found (i32.eq (local.get $index)
                (i32.sub (local.get $length) (i32.const 17))))
              (local.set $round (i32.ne (i32.load8_u (i32.add (local.get $reverse) (local.get $index)))
                (i32.const 48)))
              (local.set $index (i32.add (local.get $index) (i32.const 1)))
              (br $sticky)))))
      (if (local.get $round)
        (then
          (local.set $index (local.get $kept))
          (block $rounded
            (loop $carry
              (if (i32.eqz (local.get $index))
                (then
                  (i32.store8 (local.get $digits) (i32.const 49))
                  (local.set $high_exp (i32.add (local.get $high_exp) (i32.const 1)))
                  (br $rounded)))
              (local.set $index (i32.sub (local.get $index) (i32.const 1)))
              (local.set $byte (i32.load8_u (i32.add (local.get $digits) (local.get $index))))
              (if (i32.lt_u (local.get $byte) (i32.const 57))
                (then
                  (i32.store8 (i32.add (local.get $digits) (local.get $index))
                    (i32.add (local.get $byte) (i32.const 1)))
                  (br $rounded)))
              (i32.store8 (i32.add (local.get $digits) (local.get $index)) (i32.const 48))
              (br $carry)))))))
  (block $trimmed
    (loop $trim
      (br_if $trimmed (i32.eq (local.get $kept) (i32.const 1)))
      (br_if $trimmed (i32.ne
        (i32.load8_u (i32.add (local.get $digits) (i32.sub (local.get $kept) (i32.const 1))))
        (i32.const 48)))
      (local.set $kept (i32.sub (local.get $kept) (i32.const 1)))
      (br $trim)))
  (if (i32.or (i32.lt_s (local.get $high_exp) (i32.const -4))
        (i32.ge_s (local.get $high_exp) (i32.const 16)))
    (then
      (i32.store8 (i32.add (local.get $buffer) (local.get $out)) (i32.load8_u (local.get $digits)))
      (local.set $out (i32.add (local.get $out) (i32.const 1)))
      (if (i32.gt_u (local.get $kept) (i32.const 1))
        (then
          (i32.store8 (i32.add (local.get $buffer) (local.get $out)) (i32.const 46))
          (local.set $out (i32.add (local.get $out) (i32.const 1)))
          (memory.copy (i32.add (local.get $buffer) (local.get $out))
            (i32.add (local.get $digits) (i32.const 1)) (i32.sub (local.get $kept) (i32.const 1)))
          (local.set $out (i32.add (local.get $out) (i32.sub (local.get $kept) (i32.const 1))))))
      (i32.store8 (i32.add (local.get $buffer) (local.get $out)) (i32.const 101))
      (local.set $out (i32.add (local.get $out) (i32.const 1)))
      (i32.store8 (i32.add (local.get $buffer) (local.get $out))
        (select (i32.const 45) (i32.const 43) (i32.lt_s (local.get $high_exp) (i32.const 0))))
      (local.set $out (i32.add (local.get $out) (i32.const 1)))
      (if (i32.lt_s (local.get $high_exp) (i32.const 0))
        (then (local.set $high_exp (i32.sub (i32.const 0) (local.get $high_exp)))))
      (if (i32.ge_u (local.get $high_exp) (i32.const 100))
        (then
          (i32.store8 (i32.add (local.get $buffer) (local.get $out))
            (i32.add (i32.div_u (local.get $high_exp) (i32.const 100)) (i32.const 48)))
          (local.set $out (i32.add (local.get $out) (i32.const 1)))
          (local.set $high_exp (i32.rem_u (local.get $high_exp) (i32.const 100)))))
      (i32.store8 (i32.add (local.get $buffer) (local.get $out))
        (i32.add (i32.div_u (local.get $high_exp) (i32.const 10)) (i32.const 48)))
      (i32.store8 (i32.add (local.get $buffer) (i32.add (local.get $out) (i32.const 1)))
        (i32.add (i32.rem_u (local.get $high_exp) (i32.const 10)) (i32.const 48)))
      (local.set $out (i32.add (local.get $out) (i32.const 2))))
    (else
      (if (i32.lt_s (local.get $high_exp) (i32.const 0))
        (then
          (i32.store16 (i32.add (local.get $buffer) (local.get $out)) (i32.const 0x2e30))
          (local.set $out (i32.add (local.get $out) (i32.const 2)))
          (local.set $count (i32.sub (i32.const -1) (local.get $high_exp)))
          (memory.fill (i32.add (local.get $buffer) (local.get $out)) (i32.const 48) (local.get $count))
          (local.set $out (i32.add (local.get $out) (local.get $count)))
          (memory.copy (i32.add (local.get $buffer) (local.get $out)) (local.get $digits) (local.get $kept))
          (local.set $out (i32.add (local.get $out) (local.get $kept))))
        (else
          (local.set $count (i32.add (local.get $high_exp) (i32.const 1)))
          (local.set $index (i32.const 0))
          (block $fixed_done
            (loop $fixed
              (br_if $fixed_done (i32.and (i32.ge_u (local.get $index) (local.get $count))
                (i32.ge_u (local.get $index) (local.get $kept))))
              (if (i32.eq (local.get $index) (local.get $count))
                (then
                  (i32.store8 (i32.add (local.get $buffer) (local.get $out)) (i32.const 46))
                  (local.set $out (i32.add (local.get $out) (i32.const 1)))))
              (i32.store8 (i32.add (local.get $buffer) (local.get $out))
                (if (result i32) (i32.lt_u (local.get $index) (local.get $kept))
                  (then (i32.load8_u (i32.add (local.get $digits) (local.get $index))))
                  (else (i32.const 48))))
              (local.set $out (i32.add (local.get $out) (i32.const 1)))
              (local.set $index (i32.add (local.get $index) (i32.const 1)))
              (br $fixed)))
          (if (i32.le_u (local.get $kept) (local.get $count))
            (then
              (i32.store16 (i32.add (local.get $buffer) (local.get $out)) (i32.const 0x302e))
              (local.set $out (i32.add (local.get $out) (i32.const 2)))))))))
  (i32.store8 (i32.add (local.get $buffer) (local.get $out)) (i32.const 0))
  (call $blob_free (local.get $scratch))
  (local.get $out))

;; One serializer is active at a time. No helper calls into the evaluator.
(global $ser_output (mut i32) (i32.const 0))
(global $ser_length (mut i32) (i32.const 0))
(global $ser_capacity (mut i32) (i32.const 0))
(global $ser_work (mut i32) (i32.const 0))
(global $ser_depth (mut i32) (i32.const 0))
(global $ser_work_capacity (mut i32) (i32.const 0))
(global $ser_marked (mut i32) (i32.const 0))
(global $ser_shared (mut i32) (i32.const 0))
(global $ser_shared_count (mut i32) (i32.const 0))
(global $ser_number (mut i32) (i32.const 0))
(global $ser_header (mut i32) (i32.const 0))
(global $ser_error (mut i32) (i32.const 0))

;; Grow the output without retaining a pointer into its previous allocation.
(func $ser_reserve (param $extra i32)
  (local $needed i64) (local $capacity i32) (local $copy i32)
  (local.set $needed (i64.add (i64.extend_i32_u (global.get $ser_length))
    (i64.extend_i32_u (local.get $extra))))
  (if (i64.gt_u (local.get $needed) (i64.const 0x40000000))
    (then (call $fail (i32.const 89)) (unreachable)))
  (if (i64.le_u (local.get $needed) (i64.extend_i32_u (global.get $ser_capacity)))
    (then (return)))
  (local.set $capacity (global.get $ser_capacity))
  (loop $grow
    (local.set $capacity (i32.shl (local.get $capacity) (i32.const 1)))
    (br_if $grow (i64.lt_u (i64.extend_i32_u (local.get $capacity)) (local.get $needed))))
  (local.set $copy (call $blob_alloc (local.get $capacity) (i32.const 0)))
  (memory.copy (local.get $copy) (global.get $ser_output) (global.get $ser_length))
  (call $blob_free (global.get $ser_output))
  (global.set $ser_output (local.get $copy))
  (global.set $ser_capacity (local.get $capacity)))

(func $ser_byte (param $byte i32)
  (call $ser_reserve (i32.const 1))
  (i32.store8 (i32.add (global.get $ser_output) (global.get $ser_length)) (local.get $byte))
  (global.set $ser_length (i32.add (global.get $ser_length) (i32.const 1))))

(func $ser_write (param $address i32) (param $length i32)
  (call $ser_reserve (local.get $length))
  (memory.copy (i32.add (global.get $ser_output) (global.get $ser_length))
    (local.get $address) (local.get $length))
  (global.set $ser_length (i32.add (global.get $ser_length) (local.get $length))))

(func $ser_name (param $address i32)
  (if (i32.eqz (local.get $address))
    (then (global.set $ser_error (i32.const 1)) (return)))
  (call $ser_write (local.get $address) (call $strlen (local.get $address))))

;; Integer formatting treats MIN as an unsigned magnitude after negation.
(func $ser_unsigned (param $value i64)
  (local $index i32)
  (local.set $index (i32.const 32))
  (loop $digit
    (local.set $index (i32.sub (local.get $index) (i32.const 1)))
    (i32.store8 (i32.add (global.get $ser_number) (local.get $index))
      (i32.add (i32.wrap_i64 (i64.rem_u (local.get $value) (i64.const 10))) (i32.const 48)))
    (local.set $value (i64.div_u (local.get $value) (i64.const 10)))
    (br_if $digit (i64.ne (local.get $value) (i64.const 0))))
  (call $ser_write (i32.add (global.get $ser_number) (local.get $index))
    (i32.sub (i32.const 32) (local.get $index))))

(func $ser_signed (param $value i64)
  (if (i64.lt_s (local.get $value) (i64.const 0))
    (then
      (call $ser_byte (i32.const 45))
      (local.set $value (i64.sub (i64.const 0) (local.get $value)))))
  (call $ser_unsigned (local.get $value)))

;; Each walk frame contains {node, nextChild}. Child -1 means not entered.
;; The print pass uses node bit zero to retain a pending postfix label.
(func $ser_push (param $node i32)
  (local $capacity i32) (local $copy i32) (local $frame i32)
  (if (i32.eq (global.get $ser_depth) (global.get $ser_work_capacity))
    (then
      (if (i32.ge_u (global.get $ser_work_capacity) (i32.const 0x08000000))
        (then (call $fail (i32.const 89)) (unreachable)))
      (local.set $capacity (i32.shl (global.get $ser_work_capacity) (i32.const 1)))
      (local.set $copy (call $blob_alloc (i32.shl (local.get $capacity) (i32.const 3)) (i32.const 0)))
      (memory.copy (local.get $copy) (global.get $ser_work)
        (i32.shl (global.get $ser_depth) (i32.const 3)))
      (call $blob_free (global.get $ser_work))
      (global.set $ser_work (local.get $copy))
      (global.set $ser_work_capacity (local.get $capacity))))
  (local.set $frame (i32.add (global.get $ser_work) (i32.shl (global.get $ser_depth) (i32.const 3))))
  (i32.store (local.get $frame) (local.get $node))
  (i32.store offset=4 (local.get $frame) (i32.const -1))
  (global.set $ser_depth (i32.add (global.get $ser_depth) (i32.const 1))))

(func $ser_pop
  (global.set $ser_depth (i32.sub (global.get $ser_depth) (i32.const 1))))

(func $ser_label (param $node i32) (result i32)
  (i32.div_u (i32.sub (local.get $node) (i32.const 0x02000000)) (i32.const 8)))

(func $ser_flag (param $bitmap i32) (param $node i32) (result i32)
  (local $index i32)
  (local.set $index (call $ser_label (local.get $node)))
  (i32.and (i32.load8_u (i32.add (local.get $bitmap) (i32.shr_u (local.get $index) (i32.const 3))))
    (i32.shl (i32.const 1) (i32.and (local.get $index) (i32.const 7)))))

(func $ser_set_flag (param $bitmap i32) (param $node i32) (param $set i32)
  (local $index i32) (local $address i32) (local $bit i32)
  (local.set $index (call $ser_label (local.get $node)))
  (local.set $address (i32.add (local.get $bitmap) (i32.shr_u (local.get $index) (i32.const 3))))
  (local.set $bit (i32.shl (i32.const 1) (i32.and (local.get $index) (i32.const 7))))
  (i32.store8 (local.get $address)
    (if (result i32) (local.get $set)
      (then (i32.or (i32.load8_u (local.get $address)) (local.get $bit)))
      (else (i32.and (i32.load8_u (local.get $address)) (i32.xor (local.get $bit) (i32.const -1)))))))

;; Applications have an address in word zero, rather than a tagged word.
(func $ser_kind (param $node i32) (result i32)
  (if (result i32) (i32.eqz (i32.and (i32.load (local.get $node)) (i32.const 3)))
    (then (global.get $T_AP))
    (else (call $tag (local.get $node)))))

;; Standard streams have stable root slots. This keeps their serialization
;; independent of the host's handle values and the WASI implementation.
(func $ser_standard (param $node i32) (param $raw_pointer i32) (result i32)
  (local $index i32) (local $standard i32) (local $matches i32)
  (loop $streams
    (local.set $standard (i32.load (i32.add (i32.const 0x3010)
      (i32.shl (local.get $index) (i32.const 2)))))
    (if (local.get $standard)
      (then
        (local.set $matches
          (if (result i32) (local.get $raw_pointer)
            (then (i32.eq (i32.load offset=4 (local.get $node))
              (i32.load offset=4 (i32.load offset=4 (local.get $standard)))))
            (else
              (i32.or (i32.eq (local.get $node) (local.get $standard))
                (i32.eq (i32.load offset=4 (local.get $node))
                  (i32.load offset=4 (local.get $standard)))))))
        (if (local.get $matches)
          (then (return (i32.add (global.get $T_IO_STDIN) (local.get $index)))))))
    (local.set $index (i32.add (local.get $index) (i32.const 1)))
    (br_if $streams (i32.lt_u (local.get $index) (i32.const 3))))
  (i32.const 0))

;; Reject graph values that have no stable serialized representation.
(func $ser_supported (param $node i32) (param $kind i32) (result i32)
  (if (i32.or (i32.eq (local.get $kind) (global.get $T_AP))
        (i32.eq (local.get $kind) (global.get $T_ARR)))
    (then (return (i32.const 1))))
  (if (i32.and (i32.ge_u (local.get $kind) (global.get $T_INT))
        (i32.le_u (local.get $kind) (global.get $T_FLT32)))
    (then (return (i32.const 1))))
  (if (i32.eq (local.get $kind) (global.get $T_PTR))
    (then (return (i32.or (i32.eqz (i32.load offset=4 (local.get $node)))
      (call $ser_standard (local.get $node) (i32.const 1))))))
  (if (i32.eq (local.get $kind) (global.get $T_FUNPTR))
    (then (return (i32.or (i32.eqz (i32.load offset=4 (local.get $node)))
      (i32.eq (i32.load offset=4 (local.get $node))
        (i32.add (global.get $svc_closeb) (i32.const 1)))))))
  (if (i32.eq (local.get $kind) (global.get $T_FORPTR))
    (then (return (i32.or (i32.eq (i32.and (i32.load offset=4 (local.get $node)) (i32.const 1)) (i32.const 1))
      (call $ser_standard (local.get $node) (i32.const 0))))))
  (if (i32.eq (local.get $kind) (global.get $T_IO_CCALL))
    (then (return (i32.ne (call $service_name (i32.load offset=4 (local.get $node))) (i32.const 0)))))
  (if (i32.eq (local.get $kind) (global.get $T_TICK))
    (then (return (i32.const 1))))
  (i32.ne (call $prim_name (local.get $kind)) (i32.const 0)))

;; Find sharing in function-before-argument order. The explicit child cursor
;; visits an array one element at a time, without pushing its whole contents.
(func $ser_find_sharing (param $root i32)
  (local $frame i32) (local $node i32) (local $kind i32)
  (local $child i32) (local $descriptor i32)
  (call $ser_push (local.get $root))
  (block $done
    (loop $walk
      (br_if $done (i32.eqz (global.get $ser_depth)))
      (br_if $done (global.get $ser_error))
      (local.set $frame (i32.add (global.get $ser_work)
        (i32.shl (i32.sub (global.get $ser_depth) (i32.const 1)) (i32.const 3))))
      (local.set $node (i32.load (local.get $frame)))
      (local.set $child (i32.load offset=4 (local.get $frame)))
      (if (i32.eq (local.get $child) (i32.const -1))
        (then
          (local.set $node (call $resolve (local.get $node)))
          (if (i32.or
                (i32.or (i32.lt_u (local.get $node) (i32.const 0x02000000))
                  (i32.ge_u (local.get $node) (i32.const 0x25c34600)))
                (i32.ne (i32.rem_u (i32.sub (local.get $node) (i32.const 0x02000000)) (i32.const 8)) (i32.const 0)))
            (then (global.set $ser_error (i32.const 1)) (br $done)))
          (i32.store (local.get $frame) (local.get $node))
          (local.set $kind (call $ser_kind (local.get $node)))
          (if (i32.eqz (call $ser_supported (local.get $node) (local.get $kind)))
            (then (global.set $ser_error (i32.const 1)) (br $done)))
          (if (i32.eqz (i32.or (i32.eq (local.get $kind) (global.get $T_AP))
                (i32.or (i32.eq (local.get $kind) (global.get $T_ARR))
                  (i32.eq (local.get $kind) (global.get $T_FORPTR)))))
            (then (call $ser_pop) (br $walk)))
          (if (call $ser_flag (global.get $ser_shared) (local.get $node))
            (then (call $ser_pop) (br $walk)))
          (if (call $ser_flag (global.get $ser_marked) (local.get $node))
            (then
              (call $ser_set_flag (global.get $ser_shared) (local.get $node) (i32.const 1))
              (global.set $ser_shared_count (i32.add (global.get $ser_shared_count) (i32.const 1)))
              (call $ser_pop) (br $walk)))
          (call $ser_set_flag (global.get $ser_marked) (local.get $node) (i32.const 1))
          (local.set $child (i32.const 0))
          (i32.store offset=4 (local.get $frame) (local.get $child))))
      (local.set $kind (call $ser_kind (local.get $node)))
      (if (i32.eq (local.get $kind) (global.get $T_AP))
        (then
          (if (i32.lt_u (local.get $child) (i32.const 2))
            (then
              (i32.store offset=4 (local.get $frame) (i32.add (local.get $child) (i32.const 1)))
              (call $ser_push (i32.load (i32.add (local.get $node) (i32.shl (local.get $child) (i32.const 2)))))
              (br $walk)))))
      (if (i32.eq (local.get $kind) (global.get $T_ARR))
        (then
          (local.set $descriptor (call $aval (local.get $node)))
          (if (i32.lt_u (local.get $child) (i32.load (local.get $descriptor)))
            (then
              (i32.store offset=4 (local.get $frame) (i32.add (local.get $child) (i32.const 1)))
              (call $ser_push (i32.load offset=4 (i32.add (local.get $descriptor)
                (i32.shl (local.get $child) (i32.const 2)))))
              (br $walk)))))
      (call $ser_pop)
      (br $walk))))

;; Encode all byte values with the established printable escape alphabet.
(func $ser_quote (param $descriptor i32)
  (local $index i32) (local $byte i32)
  (call $ser_byte (i32.const 34))
  (block $done
    (loop $bytes
      (br_if $done (i32.eq (local.get $index) (i32.load (local.get $descriptor))))
      (local.set $byte (i32.load8_u (i32.add (i32.load offset=4 (local.get $descriptor)) (local.get $index))))
      (if (i32.lt_u (local.get $byte) (i32.const 32))
        (then
          (call $ser_byte (i32.const 94))
          (call $ser_byte (i32.add (local.get $byte) (i32.const 32))))
        (else
          (if (i32.or (i32.eq (local.get $byte) (i32.const 34))
                (i32.or (i32.eq (local.get $byte) (i32.const 94))
                  (i32.or (i32.eq (local.get $byte) (i32.const 124))
                    (i32.eq (local.get $byte) (i32.const 92)))))
            (then (call $ser_byte (i32.const 92)) (call $ser_byte (local.get $byte)))
            (else
              (if (i32.lt_u (local.get $byte) (i32.const 127))
                (then (call $ser_byte (local.get $byte)))
                (else
                  (if (i32.eq (local.get $byte) (i32.const 127))
                    (then (call $ser_byte (i32.const 92)) (call $ser_byte (i32.const 63)))
                    (else
                      (if (i32.lt_u (local.get $byte) (i32.const 160))
                        (then
                          (call $ser_byte (i32.const 94))
                          (call $ser_byte (i32.sub (local.get $byte) (i32.const 64))))
                        (else
                          (if (i32.lt_u (local.get $byte) (i32.const 255))
                            (then
                              (call $ser_byte (i32.const 124))
                              (call $ser_byte (i32.sub (local.get $byte) (i32.const 128))))
                            (else (call $ser_byte (i32.const 92)) (call $ser_byte (i32.const 95))))))))))))))
      (local.set $index (i32.add (local.get $index) (i32.const 1)))
      (br $bytes)))
  (call $ser_byte (i32.const 34)))

(func $ser_array_size (param $descriptor i32)
  (call $ser_byte (i32.const 91))
  (call $ser_unsigned (i64.extend_i32_u (i32.load (local.get $descriptor))))
  (call $ser_byte (i32.const 93)))

;; Print a leaf. All stable-representation checks ran in the first pass.
(func $ser_leaf (param $node i32) (param $kind i32)
  (local $descriptor i32) (local $standard i32) (local $length i32)
  (if (i32.eq (local.get $kind) (global.get $T_INT))
    (then
      (call $ser_byte (i32.const 35))
      (call $ser_signed (i64.load32_s offset=4 (local.get $node))) (return)))
  (if (i32.eq (local.get $kind) (global.get $T_INT64))
    (then
      (call $ser_byte (i32.const 35)) (call $ser_byte (i32.const 35))
      (call $ser_signed (call $i64val (local.get $node))) (return)))
  (if (i32.or (i32.eq (local.get $kind) (global.get $T_DBL))
        (i32.eq (local.get $kind) (global.get $T_FLT32)))
    (then
      (call $ser_byte (i32.const 38))
      (if (i32.eq (local.get $kind) (global.get $T_FLT32))
        (then (call $ser_byte (i32.const 38))))
      (local.set $length (call $format_f64
        (if (result f64) (i32.eq (local.get $kind) (global.get $T_DBL))
          (then (call $dval (local.get $node)))
          (else (f64.promote_f32 (f32.load offset=4 (local.get $node)))))
        (global.get $ser_number)))
      (call $ser_write (global.get $ser_number) (local.get $length)) (return)))
  (if (i32.eq (local.get $kind) (global.get $T_FUNPTR))
    (then
      (call $ser_byte (i32.const 59))
      (if (i32.eqz (i32.load offset=4 (local.get $node)))
        (then (call $ser_byte (i32.const 48)))
        (else (call $ser_name (call $service_name (global.get $svc_closeb)))))
      (call $ser_byte (i32.const 32)) (return)))
  (if (i32.eq (local.get $kind) (global.get $T_PTR))
    (then
      (local.set $standard (call $ser_standard (local.get $node) (i32.const 1)))
      (if (i32.eqz (global.get $ser_header)) (then (call $ser_byte (i32.const 40))))
      (call $ser_name (call $prim_name (select (global.get $T_FP2P) (global.get $T_TOPTR) (local.get $standard))))
      (call $ser_byte (i32.const 32))
      (if (local.get $standard)
        (then (call $ser_name (call $prim_name (local.get $standard))))
        (else (call $ser_byte (i32.const 35)) (call $ser_byte (i32.const 48))))
      (if (global.get $ser_header)
        (then (call $ser_byte (i32.const 32)) (call $ser_byte (i32.const 64)))
        (else (call $ser_byte (i32.const 41))))
      (return)))
  (if (i32.eq (local.get $kind) (global.get $T_FORPTR))
    (then
      (local.set $standard (call $ser_standard (local.get $node) (i32.const 0)))
      (if (local.get $standard)
        (then (call $ser_name (call $prim_name (local.get $standard))) (return)))
      (local.set $descriptor (call $bval (local.get $node)))
      (local.set $length (i32.load (local.get $descriptor)))
      (if (i32.gt_u (local.get $length) (i32.const 100))
        (then
          (call $ser_byte (i32.const 36))
          (call $ser_unsigned (i64.extend_i32_u (local.get $length)))
          (call $ser_byte (i32.const 32))
          (call $ser_write (i32.load offset=4 (local.get $descriptor)) (local.get $length)))
        (else (call $ser_quote (local.get $descriptor))))
      (return)))
  (if (i32.eq (local.get $kind) (global.get $T_IO_CCALL))
    (then
      (call $ser_byte (i32.const 94))
      (call $ser_name (call $service_name (i32.load offset=4 (local.get $node)))) (return)))
  (if (i32.eq (local.get $kind) (global.get $T_TICK))
    (then
      (if (i32.load offset=4 (local.get $node))
        (then
          (call $ser_byte (i32.const 33))
          (call $ser_quote (call $bval (i32.load offset=4 (local.get $node))))
          (return)))))
  (call $ser_name (call $prim_name (local.get $kind))))

;; Finish one postfix node, including its optional sharing definition.
(func $ser_finish (param $node i32) (param $kind i32) (param $shared i32)
  (if (i32.eqz (global.get $ser_header)) (then (return)))
  (if (i32.ne (local.get $kind) (global.get $T_AP))
    (then (call $ser_byte (i32.const 32))))
  (if (local.get $shared)
    (then
      (call $ser_byte (i32.const 58))
      (call $ser_unsigned (i64.extend_i32_u (call $ser_label (local.get $node))))
      (call $ser_byte (i32.const 32)))))

;; Print with the same function-before-argument order as the sharing pass.
;; Clear a shared node's visited bit before its children, so a cycle becomes
;; a forward reference followed by the node's postfix definition.
(func $ser_print (param $root i32)
  (local $frame i32) (local $node i32) (local $kind i32) (local $shared i32)
  (local $child i32) (local $descriptor i32)
  (call $ser_push (local.get $root))
  (block $done
    (loop $walk
      (br_if $done (i32.eqz (global.get $ser_depth)))
      (br_if $done (global.get $ser_error))
      (local.set $frame (i32.add (global.get $ser_work)
        (i32.shl (i32.sub (global.get $ser_depth) (i32.const 1)) (i32.const 3))))
      (local.set $node (i32.and (i32.load (local.get $frame)) (i32.const -4)))
      (local.set $shared (i32.and (i32.load (local.get $frame)) (i32.const 1)))
      (local.set $child (i32.load offset=4 (local.get $frame)))
      (if (i32.eq (local.get $child) (i32.const -1))
        (then
          (local.set $node (call $resolve (local.get $node)))
          (if (call $ser_flag (global.get $ser_shared) (local.get $node))
            (then
              (if (i32.eqz (call $ser_flag (global.get $ser_marked) (local.get $node)))
                (then
                  (call $ser_byte (i32.const 95))
                  (call $ser_unsigned (i64.extend_i32_u (call $ser_label (local.get $node))))
                  (if (global.get $ser_header) (then (call $ser_byte (i32.const 32))))
                  (call $ser_pop) (br $walk)))
              (call $ser_set_flag (global.get $ser_marked) (local.get $node) (i32.const 0))
              (if (global.get $ser_header)
                (then (local.set $shared (i32.const 1)))
                (else
                  (call $ser_byte (i32.const 58))
                  (call $ser_unsigned (i64.extend_i32_u (call $ser_label (local.get $node))))
                  (call $ser_byte (i32.const 32))))))
          (i32.store (local.get $frame) (i32.or (local.get $node) (local.get $shared)))
          (local.set $child (i32.const 0))
          (i32.store offset=4 (local.get $frame) (local.get $child))
          (local.set $kind (call $ser_kind (local.get $node)))
          (if (i32.eqz (global.get $ser_header))
            (then
              (if (i32.eq (local.get $kind) (global.get $T_AP))
                (then (call $ser_byte (i32.const 40))))
              (if (i32.eq (local.get $kind) (global.get $T_ARR))
                (then (call $ser_array_size (call $aval (local.get $node)))))))))
      (local.set $kind (call $ser_kind (local.get $node)))
      (if (i32.eq (local.get $kind) (global.get $T_AP))
        (then
          (if (i32.lt_u (local.get $child) (i32.const 2))
            (then
              (if (i32.and (i32.eqz (global.get $ser_header)) (i32.eq (local.get $child) (i32.const 1)))
                (then (call $ser_byte (i32.const 32))))
              (i32.store offset=4 (local.get $frame) (i32.add (local.get $child) (i32.const 1)))
              (call $ser_push (i32.load (i32.add (local.get $node) (i32.shl (local.get $child) (i32.const 2)))))
              (br $walk)))
          (call $ser_byte (select (i32.const 64) (i32.const 41) (global.get $ser_header))))
        (else
          (if (i32.eq (local.get $kind) (global.get $T_ARR))
            (then
              (local.set $descriptor (call $aval (local.get $node)))
              (if (i32.lt_u (local.get $child) (i32.load (local.get $descriptor)))
                (then
                  (if (i32.eqz (global.get $ser_header)) (then (call $ser_byte (i32.const 32))))
                  (i32.store offset=4 (local.get $frame) (i32.add (local.get $child) (i32.const 1)))
                  (call $ser_push (i32.load offset=4 (i32.add (local.get $descriptor)
                    (i32.shl (local.get $child) (i32.const 2)))))
                  (br $walk)))
              (if (global.get $ser_header) (then (call $ser_array_size (local.get $descriptor)))))
            (else (call $ser_leaf (local.get $node) (local.get $kind))))))
      (call $ser_finish (local.get $node) (local.get $kind) (local.get $shared))
      (call $ser_pop)
      (br $walk))))

;; Allocate temporary state, validate the whole graph, then emit its bytes.
;; The temporary output is released on failure. No host can see partial data.
(func $serialize (param $root i32) (param $header i32) (result i32)
  (local $result i32)
  (global.set $ser_header (i32.ne (local.get $header) (i32.const 0)))
  (global.set $ser_error (i32.const 0))
  (global.set $ser_shared_count (i32.const 0))
  (global.set $ser_depth (i32.const 0))
  (global.set $ser_length (i32.const 0))
  (global.set $ser_capacity (i32.const 4096))
  (global.set $ser_output (call $blob_alloc (global.get $ser_capacity) (i32.const 0)))
  (global.set $ser_work_capacity (i32.const 1024))
  (global.set $ser_work (call $blob_alloc (i32.const 8192) (i32.const 0)))
  (global.set $ser_number (call $blob_alloc (i32.const 64) (i32.const 0)))
  ;; The arena has 75,000,000 nodes. Each bitmap needs 9,375,000 bytes.
  (global.set $ser_marked (call $blob_alloc (i32.const 18750000) (i32.const 0)))
  (global.set $ser_shared (i32.add (global.get $ser_marked) (i32.const 9375000)))
  (call $ser_find_sharing (local.get $root))
  (if (i32.eqz (global.get $ser_error))
    (then
      (if (global.get $ser_header)
        (then
          (call $ser_byte (i32.const 118)) (call $ser_byte (i32.const 56))
          (call $ser_byte (i32.const 46)) (call $ser_byte (i32.const 52))
          (call $ser_byte (i32.const 10))
          (call $ser_unsigned (i64.extend_i32_u (global.get $ser_shared_count)))
          (call $ser_byte (i32.const 10))))
      (call $ser_print (local.get $root))
      (if (global.get $ser_header) (then (call $ser_byte (i32.const 125))))
      (call $ser_byte (i32.const 10))))
  (call $blob_free (global.get $ser_marked))
  (call $blob_free (global.get $ser_work))
  (call $blob_free (global.get $ser_number))
  (if (global.get $ser_error)
    (then
      (call $blob_free (global.get $ser_output))
      (return (call $raise_rts (i32.const 8)))))
  (local.set $result (call $bytes_view (global.get $ser_output) (global.get $ser_length)))
  (global.set $ser_output (i32.const 0))
  (local.get $result))
