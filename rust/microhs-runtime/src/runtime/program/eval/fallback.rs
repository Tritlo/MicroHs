//! Fallback runtime-primitive dispatch for the general reducer.
use super::*;

impl Program {
    pub(in crate::runtime) fn fallback_runtime_prim_rewrite(
        &mut self,
        name: &str,
        args: &[NodeId],
        args_len: usize,
    ) -> Result<Option<(usize, NodeId)>, EvalError> {
        if args_len >= 2 {
            return Ok(self
                .array_op(name, args)?
                .or(self.foreign_ptr_op(name, args)?)
                .or(self.stable_ptr_op(name, args)?)
                .or(self.weak_ptr_op(name, args)?)
                .or(self.bytes_op(name, args)?)
                .or(self.float64_binop(name, args)?)
                .or(self.float32_binop(name, args)?)
                .or(self.int64_binop(name, args)?)
                .or(self.int_binop(name, args)?)
                .or(self.array_unop(name, args)?)
                .or(self.bytes_unop(name, args)?)
                .or(self.float64_unop(name, args)?)
                .or(self.float32_unop(name, args)?)
                .or(self.pointer_conversion(name, args)?)
                .or(self.float_conversion(name, args)?)
                .or(self.int64_unop(name, args)?)
                .or(self.int_conversion(name, args)?)
                .or(self.int_unop(name, args)?));
        }

        if args_len >= 1 {
            return Ok(self
                .array_unop(name, args)?
                .or(self.foreign_ptr_unop(name, args)?)
                .or(self.stable_ptr_unop(name, args)?)
                .or(self.weak_ptr_unop(name, args)?)
                .or(self.bytes_unop(name, args)?)
                .or(self.float64_unop(name, args)?)
                .or(self.float32_unop(name, args)?)
                .or(self.pointer_conversion(name, args)?)
                .or(self.float_conversion(name, args)?)
                .or(self.int64_unop(name, args)?)
                .or(self.int_conversion(name, args)?)
                .or(self.int_unop(name, args)?));
        }

        Ok(None)
    }
}
