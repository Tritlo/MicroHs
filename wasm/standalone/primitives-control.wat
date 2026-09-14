;; IO graph constructors and runtime control primitives.
;; Strict arguments arrive in the frame vector. Lazy arguments remain nodes.
;; These functions never evaluate. Graph results return to the main loop.

(func $primitive_control (param $tag i32) (param $args i32)
  (param $head i32) (result i32)
  (local $x i32) (local $y i32) (local $z i32) (local $w i32)
  (local $p i32) (local $q i32) (local $id i32) (local $arity i32)
  (local.set $x (call $value_arg (local.get $args) (i32.const 0)))
  (local.set $y (call $value_arg (local.get $args) (i32.const 1)))
  (local.set $z (call $value_arg (local.get $args) (i32.const 2)))
  (local.set $w (call $value_arg (local.get $args) (i32.const 3)))

  ;; IO.bind and IO.return retain the C and P combinator equations.
  (if (i32.eq (local.get $tag) (global.get $T_IO_BIND))
    (then (return (call $ap (call $ap (local.get $x) (local.get $z)) (local.get $y)))))
  (if (i32.eq (local.get $tag) (global.get $T_IO_RETURN))
    (then (return (call $ap (call $ap (local.get $z) (local.get $x)) (local.get $y)))))
  (if (i32.eq (local.get $tag) (global.get $T_IO_THEN))
    (then
      (return (call $ap
        (call $ap (call $prim (global.get $T_IO_BIND)) (local.get $x))
        (call $ap (call $prim (global.get $T_K)) (local.get $y))))))
  (if (i32.eq (local.get $tag) (global.get $T_IO_PERFORMIO))
    (then
      (return (call $ap
        (call $ap (local.get $x) (i32.load (i32.const 0x3000)))
        (call $prim (global.get $T_K))))))
  (if (i32.eq (local.get $tag) (global.get $T_IO_STRICT))
    (then (return (call $ap (local.get $x) (local.get $y)))))
  (if (i32.eq (local.get $tag) (global.get $T_IO_LAZYBIND))
    (then
      (local.set $p (call $ap (local.get $x) (local.get $z)))
      (return (call $ap
        (call $ap (local.get $y) (call $ap (local.get $p) (call $prim (global.get $T_K))))
        (call $ap (local.get $p) (call $prim (global.get $T_A)))))))
  (if (i32.eq (local.get $tag) (global.get $T_SEQ))
    (then (return (local.get $y))))
  (if (i32.eq (local.get $tag) (global.get $T_RAISE))
    (then (return (call $raise (local.get $x)))))
  (if (i32.eq (local.get $tag) (global.get $T_TICK))
    (then (return (local.get $x))))
  (if (i32.eq (local.get $tag) (global.get $T_CATCH))
    (then
      ;; Catch installs a handler only when the action is evaluated.
      (return (call $ap
        (call $ap
          (call $ap (call $prim (global.get $T_CATCHR))
            (call $ap (local.get $x) (local.get $z))) (local.get $y))
        (local.get $z)))))
  (if (i32.eq (local.get $tag) (global.get $T_IO_GETARGREF))
    (then (return (call $pair (i32.load (i32.const 0x15c)) (local.get $x)))))
  (if (i32.or (i32.eq (local.get $tag) (global.get $T_IO_PRINT))
        (i32.eq (local.get $tag) (global.get $T_IO_SERIALIZE)))
    (then
      (local.set $p (call $serialize (local.get $y)
        (i32.eq (local.get $tag) (global.get $T_IO_SERIALIZE))))
      (if (i32.eqz (local.get $p)) (then (return (i32.const 0))))
      (local.set $q (call $bval (local.get $p)))
      (drop (call $bfile_write (i32.load offset=4 (local.get $q))
        (i32.load (local.get $q)) (call $pval (local.get $x))))
      (return (call $pair (call $prim (global.get $T_I)) (local.get $z)))))
  (if (i32.eq (local.get $tag) (global.get $T_IO_PP))
    (then
      ;; Debug printing preserves the graph's current state without forcing it.
      (local.set $p (call $serialize (local.get $x) (i32.const 0)))
      (if (i32.eqz (local.get $p)) (then (return (i32.const 0))))
      (local.set $q (call $bval (local.get $p)))
      (drop (call $bfile_write (i32.load offset=4 (local.get $q))
        (i32.load (local.get $q)) (global.get $wasi_stderr)))
      (return (call $pair (call $prim (global.get $T_I)) (local.get $y)))))
  (if (i32.eq (local.get $tag) (global.get $T_IO_DESERIALIZE))
    (then
      (call $bfile_read_record (call $pval (local.get $x)))
      (local.set $arity)
      (local.set $p)
      (if (i32.eqz (local.get $p))
        (then (return (call $raise_rts (i32.const 9)))))
      (local.set $q (call $parse_prefix (local.get $p) (local.get $arity)))
      (call $blob_free (local.get $p))
      (if (i32.eqz (local.get $q)) (then (return (i32.const 0))))
      (return (call $pair (local.get $q) (local.get $y)))))
  (if (i32.eq (local.get $tag) (global.get $T_IO_GC))
    (then
      ;; Request collection at the next outer safe point. The result graph
      ;; is published before that point. Collection shortens indirection links.
      ;; The red argument is reserved for additional combinator rewrites.
      (i32.store (i32.const 0x3048) (i32.const 1))
      (return (call $pair (call $prim (global.get $T_I)) (local.get $y)))))
  (if (i32.eq (local.get $tag) (global.get $T_IO_STATS))
    (then
      (local.set $p (call $pair
        (call $int (i32.load (i32.const 0x120)))
        (call $int (i32.wrap_i64 (global.get $reduction_count)))))
      (return (call $pair (local.get $p) (local.get $x)))))
  (if (i32.eq (local.get $tag) (global.get $T_IO_GETMASKINGSTATE))
    (then (return (call $pair (call $int (i32.load (i32.const 0x3040))) (local.get $x)))))
  (if (i32.eq (local.get $tag) (global.get $T_IO_SETMASKINGSTATE))
    (then
      (i32.store (i32.const 0x3040) (call $ival (local.get $x)))
      (return (call $pair (call $prim (global.get $T_I)) (local.get $y)))))

  ;; Dynamic C symbol lookup is outside the standalone target. Generated JS
  ;; and WASM imports use the fixed foreign table through IO_CCALL below.
  (if (i32.eq (local.get $tag) (global.get $T_DYNSYM))
    (then (call $fail (i32.const 37))))
  (if (i32.eq (local.get $tag) (global.get $T_IO_CCALL))
    (then
      (local.set $id (i32.load offset=4 (local.get $head)))
      (local.set $arity (call $service_arity (local.get $id)))
      (i32.store (i32.const 0x2f4) (local.get $id))
      (if (i32.ge_u (local.get $id) (i32.const 65536))
        (then
          (local.set $p (call $foreign_invoke
            (i32.sub (local.get $id) (i32.const 65536)) (local.get $args)))
          (if (i32.eqz (local.get $p)) (then (return (i32.const 0))))
          (return (call $pair (local.get $p)
            (call $value_arg (local.get $args) (local.get $arity))))))
      (local.set $p (call $service_memory (local.get $id) (local.get $args)))
      (if (i32.eqz (local.get $p))
        (then (local.set $p (call $service_io (local.get $id) (local.get $args)))))
      (if (i32.eqz (local.get $p))
        (then (local.set $p (call $service_math (local.get $id) (local.get $args)))))
      (if (i32.eqz (local.get $p))
        (then (local.set $p (call $service_digest (local.get $id) (local.get $args)))))
      (if (i32.eqz (local.get $p))
        (then
          (if (i32.load (i32.const 0x3004)) (then (return (i32.const 0))))
          (call $fail (i32.const 33))))
      (return (call $pair (local.get $p)
        (call $value_arg (local.get $args) (local.get $arity))))))
  (i32.const 0))

;; Primitive groups return zero when they do not own a tag. A pending exception
;; also returns zero, so check its root before trying the next group.
(func $primitive (param $tag i32) (param $args i32) (param $head i32) (result i32)
  (local $result i32)
  (local.set $result (call $numeric (local.get $tag)
    (call $value_arg (local.get $args) (i32.const 0))
    (call $value_arg (local.get $args) (i32.const 1))))
  (if (i32.or (local.get $result) (i32.load (i32.const 0x3004)))
    (then (return (local.get $result))))
  (local.set $result (call $primitive_bytes (local.get $tag) (local.get $args)))
  (if (i32.or (local.get $result) (i32.load (i32.const 0x3004)))
    (then (return (local.get $result))))
  (local.set $result (call $primitive_arrays (local.get $tag) (local.get $args)))
  (if (i32.or (local.get $result) (i32.load (i32.const 0x3004)))
    (then (return (local.get $result))))
  (local.set $result (call $primitive_state (local.get $tag) (local.get $args)))
  (if (i32.or (local.get $result)
        (i32.or (i32.load (i32.const 0x3004))
          (i32.eq (i32.load (i32.const 0x309c)) (i32.const 2))))
    (then (return (local.get $result))))
  (call $primitive_control (local.get $tag) (local.get $args) (local.get $head)))
