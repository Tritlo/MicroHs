//! Fallback runtime-primitive dispatch for the general reducer.
use super::*;

impl Program {
    pub(in crate::runtime) fn fallback_runtime_prim_rewrite(
        &mut self,
        name: &str,
        args: &[NodeId],
        args_len: usize,
    ) -> Result<Option<(usize, NodeId)>, EvalError> {
        macro_rules! dispatch_helper {
            ($key:literal, $expr:expr) => {{
                self.profile_primitive_dispatch_probe($key);
                let result = $expr?;
                if result.is_some() {
                    self.profile_primitive_dispatch_hit($key);
                }
                result
            }};
        }

        if args_len >= 2 {
            return Ok(
                dispatch_helper!("fallback_array_op", self.array_op(name, args))
                    .or(dispatch_helper!(
                        "fallback_foreign_ptr_op",
                        self.foreign_ptr_op(name, args)
                    ))
                    .or(dispatch_helper!(
                        "fallback_stable_ptr_op",
                        self.stable_ptr_op(name, args)
                    ))
                    .or(dispatch_helper!(
                        "fallback_weak_ptr_op",
                        self.weak_ptr_op(name, args)
                    ))
                    .or(dispatch_helper!(
                        "fallback_bytes_op",
                        self.bytes_op(name, args)
                    ))
                    .or(dispatch_helper!(
                        "fallback_float64_binop",
                        self.float64_binop(name, args)
                    ))
                    .or(dispatch_helper!(
                        "fallback_float32_binop",
                        self.float32_binop(name, args)
                    ))
                    .or(dispatch_helper!(
                        "fallback_int64_binop",
                        self.int64_binop(name, args)
                    ))
                    .or(dispatch_helper!(
                        "fallback_int_binop",
                        self.int_binop(name, args)
                    ))
                    .or(dispatch_helper!(
                        "fallback_array_unop",
                        self.array_unop(name, args)
                    ))
                    .or(dispatch_helper!(
                        "fallback_bytes_unop",
                        self.bytes_unop(name, args)
                    ))
                    .or(dispatch_helper!(
                        "fallback_float64_unop",
                        self.float64_unop(name, args)
                    ))
                    .or(dispatch_helper!(
                        "fallback_float32_unop",
                        self.float32_unop(name, args)
                    ))
                    .or(dispatch_helper!(
                        "fallback_pointer_conversion",
                        self.pointer_conversion(name, args)
                    ))
                    .or(dispatch_helper!(
                        "fallback_float_conversion",
                        self.float_conversion(name, args)
                    ))
                    .or(dispatch_helper!(
                        "fallback_int64_unop",
                        self.int64_unop(name, args)
                    ))
                    .or(dispatch_helper!(
                        "fallback_int_conversion",
                        self.int_conversion(name, args)
                    ))
                    .or(dispatch_helper!(
                        "fallback_int_unop",
                        self.int_unop(name, args)
                    )),
            );
        }

        if args_len >= 1 {
            return Ok(
                dispatch_helper!("fallback_array_unop", self.array_unop(name, args))
                    .or(dispatch_helper!(
                        "fallback_foreign_ptr_unop",
                        self.foreign_ptr_unop(name, args)
                    ))
                    .or(dispatch_helper!(
                        "fallback_stable_ptr_unop",
                        self.stable_ptr_unop(name, args)
                    ))
                    .or(dispatch_helper!(
                        "fallback_weak_ptr_unop",
                        self.weak_ptr_unop(name, args)
                    ))
                    .or(dispatch_helper!(
                        "fallback_bytes_unop",
                        self.bytes_unop(name, args)
                    ))
                    .or(dispatch_helper!(
                        "fallback_float64_unop",
                        self.float64_unop(name, args)
                    ))
                    .or(dispatch_helper!(
                        "fallback_float32_unop",
                        self.float32_unop(name, args)
                    ))
                    .or(dispatch_helper!(
                        "fallback_pointer_conversion",
                        self.pointer_conversion(name, args)
                    ))
                    .or(dispatch_helper!(
                        "fallback_float_conversion",
                        self.float_conversion(name, args)
                    ))
                    .or(dispatch_helper!(
                        "fallback_int64_unop",
                        self.int64_unop(name, args)
                    ))
                    .or(dispatch_helper!(
                        "fallback_int_conversion",
                        self.int_conversion(name, args)
                    ))
                    .or(dispatch_helper!(
                        "fallback_int_unop",
                        self.int_unop(name, args)
                    )),
            );
        }

        Ok(None)
    }
}
