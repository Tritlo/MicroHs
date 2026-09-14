;; Scalar access and result constructors shared by primitive implementations.
;; Arguments have already reached weak head normal form. These helpers must not
;; invoke evaluation or collection. A type mismatch is an invalid runtime call.

(func $tag (param $n i32) (result i32)
  (i32.shr_u (i32.load (local.get $n)) (i32.const 2)))

(func $ival (param $n i32) (result i32)
  (if (i32.ne (call $tag (local.get $n)) (global.get $T_INT))
    (then (call $fail (i32.const 20))))
  (i32.load offset=4 (local.get $n)))

(func $i64val (param $n i32) (result i64)
  (if (i32.ne (call $tag (local.get $n)) (global.get $T_INT64))
    (then (call $fail (i32.const 21))))
  (i64.load (i32.load offset=4 (local.get $n))))

(func $dval (param $n i32) (result f64)
  (if (i32.ne (call $tag (local.get $n)) (global.get $T_DBL))
    (then (call $fail (i32.const 22))))
  (f64.load (i32.load offset=4 (local.get $n))))

(func $fval (param $n i32) (result f32)
  (if (i32.ne (call $tag (local.get $n)) (global.get $T_FLT32))
    (then (call $fail (i32.const 23))))
  (f32.load offset=4 (local.get $n)))

(func $pval (param $n i32) (result i32)
  (local $t i32)
  (local.set $t (call $tag (local.get $n)))
  (if (i32.and (i32.ne (local.get $t) (global.get $T_PTR))
               (i32.ne (local.get $t) (global.get $T_FUNPTR)))
    (then (call $fail (i32.const 24))))
  (i32.load offset=4 (local.get $n)))

;; A bytestring descriptor is {length, data, capacity, finalizer}.
;; Capacity zero means immutable. The last two fields are unused by readers.
(func $bval (param $n i32) (result i32)
  (if (i32.ne (call $tag (local.get $n)) (global.get $T_FORPTR))
    (then (call $fail (i32.const 25))))
  (i32.and (i32.load offset=4 (local.get $n)) (i32.const -2)))

(func $aval (param $n i32) (result i32)
  (if (i32.ne (call $tag (local.get $n)) (global.get $T_ARR))
    (then (call $fail (i32.const 26))))
  (i32.load offset=4 (local.get $n)))

(func $boolean (param $value i32) (result i32)
  (call $prim (select (global.get $T_A) (global.get $T_K) (local.get $value))))

(func $ordering (param $value i32) (result i32)
  (call $prim
    (select (global.get $T_K2)
      (select (global.get $T_KA) (global.get $T_KK)
        (i32.gt_s (local.get $value) (i32.const 0)))
      (i32.lt_s (local.get $value) (i32.const 0)))))

;; The IO result is the graph (P value world). P is the pair constructor.
(func $pair (param $value i32) (param $world i32) (result i32)
  (call $ap
    (call $ap (call $prim (global.get $T_P)) (local.get $value))
    (local.get $world)))

;; Zero is reserved as the primitive result that requests exception unwinding.
;; The exception graph is in the permanent root area, visible to collection.
(func $raise (param $exception i32) (result i32)
  (i32.store (i32.const 0x3004) (local.get $exception))
  (i32.const 0))

(func $raise_rts (param $code i32) (result i32)
  (call $raise (call $int (local.get $code))))

;; Read a node from the primitive argument vector. This does not force it.
(func $value_arg (param $args i32) (param $index i32) (result i32)
  (i32.load (i32.add (local.get $args)
    (i32.shl (local.get $index) (i32.const 2)))))

;; Read a zero-terminated byte string. The caller owns the complete allocation.
(func $strlen (param $p i32) (result i32)
  (local $end i32)
  (local.set $end (local.get $p))
  (block $done
    (loop $next
      (br_if $done (i32.eqz (i32.load8_u (local.get $end))))
      (local.set $end (i32.add (local.get $end) (i32.const 1)))
      (br $next)))
  (i32.sub (local.get $end) (local.get $p)))
