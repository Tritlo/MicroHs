;; Default import registry for a command with no program-specific foreign calls.
;; A generated PREFIX.wat fragment replaces this file when --ffi PREFIX is used.
(func $foreign_lookup (param i32 i32) (result i32) (i32.const -1))
(func $foreign_arity (param i32) (result i32) (i32.const -1))
(func $foreign_name (param i32) (result i32) (i32.const 0))
(func $foreign_invoke (param i32 i32) (result i32)
  (call $fail (i32.const 90)) (unreachable))
