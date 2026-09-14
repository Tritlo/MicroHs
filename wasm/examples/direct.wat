(module
  (global $value (mut i32) (i32.const 0))

  (func (export "add_i32") (export "add_u32")
    (param i32 i32) (result i32)
    (i32.add (local.get 0) (local.get 1)))

  (func (export "add_i64") (export "add_u64")
    (param i64 i64) (result i64)
    (i64.add (local.get 0) (local.get 1)))

  (func (export "add_f32") (param f32 f32) (result f32)
    (f32.add (local.get 0) (local.get 1)))

  (func (export "add_f64") (param f64 f64) (result f64)
    (f64.add (local.get 0) (local.get 1)))

  (func (export "set_value") (param i32)
    (global.set $value (local.get 0)))

  (func (export "get_value") (result i32)
    (global.get $value)))
