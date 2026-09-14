;; Mutable array primitives. The evaluator has forced the required arguments.
;; An array descriptor contains its length followed by graph references.
;; The collector scans every descriptor word. Truncation clears removed roots.
;; The primitive functions allocate without collection and do not evaluate.

;; Allocate and initialize an array. Its fill value remains lazy.
(func $array_new (param $length i32) (param $fill i32) (result i32)
  (local $descriptor i32) (local $index i32) (local $node i32)
  (if (i32.gt_u (local.get $length) (i32.const 0x0fffffff))
    (then (call $fail (i32.const 40)) (unreachable)))
  (local.set $descriptor (call $blob_alloc
    (i32.shl (i32.add (local.get $length) (i32.const 1)) (i32.const 2)) (i32.const -1)))
  (i32.store (local.get $descriptor) (local.get $length))
  (block $done
    (loop $fill_elements
      (br_if $done (i32.eq (local.get $index) (local.get $length)))
      (i32.store offset=4 (i32.add (local.get $descriptor)
          (i32.shl (local.get $index) (i32.const 2))) (local.get $fill))
      (local.set $index (i32.add (local.get $index) (i32.const 1)))
      (br $fill_elements)))
  (local.set $node (call $node (global.get $T_ARR)))
  (i32.store offset=4 (local.get $node) (local.get $descriptor))
  (local.get $node))

;; Return the result graph, or zero when this is not an array primitive.
(func $primitive_arrays (param $tag i32) (param $args i32) (result i32)
  (local $a i32) (local $descriptor i32) (local $index i32)
  (local $length i32) (local $result i32)
  (local.set $a (call $value_arg (local.get $args) (i32.const 0)))
  (if (i32.eq (local.get $tag) (global.get $T_ARR_ALLOC))
    (then
      (local.set $result (call $array_new (call $ival (local.get $a))
        (call $value_arg (local.get $args) (i32.const 1))))
      (return (call $pair (local.get $result) (call $value_arg (local.get $args) (i32.const 2))))))
  (if (i32.or (i32.lt_u (local.get $tag) (global.get $T_ARR_COPY))
        (i32.gt_u (local.get $tag) (global.get $T_ARR_EQ)))
    (then (return (i32.const 0))))
  (local.set $descriptor (call $aval (local.get $a)))
  (local.set $length (i32.load (local.get $descriptor)))
  (if (i32.eq (local.get $tag) (global.get $T_ARR_EQ))
    (then
      (return (call $boolean (i32.eq (local.get $descriptor)
        (call $aval (call $value_arg (local.get $args) (i32.const 1))))))))
  (if (i32.eq (local.get $tag) (global.get $T_ARR_COPY))
    (then
      (local.set $result (call $array_new (local.get $length) (i32.const 0)))
      (memory.copy (i32.add (i32.load offset=4 (local.get $result)) (i32.const 4))
        (i32.add (local.get $descriptor) (i32.const 4))
        (i32.shl (local.get $length) (i32.const 2)))
      (return (call $pair (local.get $result) (call $value_arg (local.get $args) (i32.const 1))))))
  (if (i32.eq (local.get $tag) (global.get $T_ARR_SIZE))
    (then
      (return (call $pair (call $int (local.get $length))
        (call $value_arg (local.get $args) (i32.const 1))))))
  (local.set $index (call $ival (call $value_arg (local.get $args) (i32.const 1))))
  ;; The existing format rejects an equal-length truncation as well as an
  ;; out-of-range read or write. The unsigned check also rejects negatives.
  (if (i32.ge_u (local.get $index) (local.get $length))
    (then (call $fail (i32.const 41)) (unreachable)))
  (if (i32.eq (local.get $tag) (global.get $T_ARR_READ))
    (then
      (return (call $pair
        (i32.load offset=4 (i32.add (local.get $descriptor)
          (i32.shl (local.get $index) (i32.const 2))))
        (call $value_arg (local.get $args) (i32.const 2))))))
  (if (i32.eq (local.get $tag) (global.get $T_ARR_WRITE))
    (then
      (i32.store offset=4 (i32.add (local.get $descriptor)
          (i32.shl (local.get $index) (i32.const 2)))
        (call $value_arg (local.get $args) (i32.const 2)))
      (return (call $pair (call $prim (global.get $T_I))
        (call $value_arg (local.get $args) (i32.const 3))))))
  (memory.fill (i32.add (local.get $descriptor)
      (i32.shl (i32.add (local.get $index) (i32.const 1)) (i32.const 2)))
    (i32.const 0) (i32.shl (i32.sub (local.get $length) (local.get $index)) (i32.const 2)))
  (i32.store (local.get $descriptor) (local.get $index))
  (call $pair (call $prim (global.get $T_I))
    (call $value_arg (local.get $args) (i32.const 2))))
