;; Runtime services and generated JS/WASM imports occupy separate ID ranges.
;; Fixed services retain their serialized indices. Generated imports start at
;; 65536, which leaves room to extend the fixed table without renumbering them.
;; The selected foreign fragment supplies lookup, arity, name, and invocation.

(func $service_lookup (param $name i32) (param $length i32) (result i32)
  (local $id i32)
  (local.set $id (call $fixed_service_lookup (local.get $name) (local.get $length)))
  (if (i32.ne (local.get $id) (i32.const -1))
    (then (return (local.get $id))))
  (local.set $id (call $foreign_lookup (local.get $name) (local.get $length)))
  (if (result i32) (i32.eq (local.get $id) (i32.const -1))
    (then (i32.const -1))
    (else (i32.add (local.get $id) (i32.const 65536)))))

(func $service_arity (param $id i32) (result i32)
  (if (result i32) (i32.ge_u (local.get $id) (i32.const 65536))
    (then (call $foreign_arity (i32.sub (local.get $id) (i32.const 65536))))
    (else (call $fixed_service_arity (local.get $id)))))

(func $service_name (param $id i32) (result i32)
  (if (result i32) (i32.ge_u (local.get $id) (i32.const 65536))
    (then (call $foreign_name (i32.sub (local.get $id) (i32.const 65536))))
    (else (call $fixed_service_name (local.get $id)))))
