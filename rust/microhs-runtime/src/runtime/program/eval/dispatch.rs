//! Strict primitive frame completion and eval-time primitive helpers.
use super::*;

impl Program {
    pub(in crate::runtime) fn bytes_bin_result_node(
        &mut self,
        op: BytesBinOp,
        x: NodeId,
        y: NodeId,
    ) -> Result<NodeId, EvalError> {
        let node = match op {
            BytesBinOp::Append => {
                let mut bytes = self.bytes(x)?.to_vec();
                bytes.extend(self.bytes(y)?);
                self.push_node(Node::bytes(bytes))
            }
            BytesBinOp::AppendDot => {
                let mut bytes = self.bytes(x)?.to_vec();
                bytes.push(b'.');
                bytes.extend(self.bytes(y)?);
                self.push_node(Node::bytes(bytes))
            }
            BytesBinOp::Eq
            | BytesBinOp::Ne
            | BytesBinOp::Lt
            | BytesBinOp::Le
            | BytesBinOp::Gt
            | BytesBinOp::Ge
            | BytesBinOp::Cmp => {
                let cmp = self.bytes(x)?.cmp(self.bytes(y)?);
                match op {
                    BytesBinOp::Eq => self.prim(if cmp == Ordering::Equal { "A" } else { "K" }),
                    BytesBinOp::Ne => self.prim(if cmp != Ordering::Equal { "A" } else { "K" }),
                    BytesBinOp::Lt => self.prim(if cmp == Ordering::Less { "A" } else { "K" }),
                    BytesBinOp::Le => self.prim(if cmp != Ordering::Greater { "A" } else { "K" }),
                    BytesBinOp::Gt => self.prim(if cmp == Ordering::Greater { "A" } else { "K" }),
                    BytesBinOp::Ge => self.prim(if cmp != Ordering::Less { "A" } else { "K" }),
                    BytesBinOp::Cmp => self.ordering(cmp),
                    BytesBinOp::Append | BytesBinOp::AppendDot => unreachable!(),
                }
            }
        };
        Ok(node)
    }

    pub(in crate::runtime) fn int_binop(
        &mut self,
        name: &str,
        args: &[NodeId],
    ) -> Result<Option<(usize, NodeId)>, EvalError> {
        let Some(op) = IntBinOp::from_prim(name) else {
            return Ok(None);
        };
        let x = self.eval_int(args[0])?;
        let y = self.eval_int(args[1])?;
        let result = op
            .apply(x, y)
            .map_err(|err| self.arithmetic_eval_error(err))?;
        let node = match result {
            IntResult::Int(n) => self.int(n),
            IntResult::Bool(b) => self.prim(if b { "A" } else { "K" }),
            IntResult::Ordering(ord) => self.ordering(ord),
        };
        Ok(Some((2, node)))
    }

    pub(in crate::runtime) fn int_unop(
        &mut self,
        name: &str,
        args: &[NodeId],
    ) -> Result<Option<(usize, NodeId)>, EvalError> {
        let Some(op) = IntUnOp::from_prim(name) else {
            return Ok(None);
        };
        let x = self.eval_int(args[0])?;
        let n = op.apply(x).map_err(|err| self.arithmetic_eval_error(err))?;
        let node = self.int(n);
        Ok(Some((1, node)))
    }

    pub(in crate::runtime) fn int64_binop(
        &mut self,
        name: &str,
        args: &[NodeId],
    ) -> Result<Option<(usize, NodeId)>, EvalError> {
        let Some(op) = Int64BinOp::from_prim(name) else {
            return Ok(None);
        };
        let x = self.eval_int64(args[0])?;
        let y = if op.rhs_is_shift() {
            self.eval_int(args[1])?
        } else {
            self.eval_int64(args[1])?
        };
        let result = op
            .apply(x, y)
            .map_err(|err| self.arithmetic_eval_error(err))?;
        let node = match result {
            Int64Result::Int64(n) => self.push_node(Node::Int64(n)),
            Int64Result::Bool(b) => self.prim(if b { "A" } else { "K" }),
            Int64Result::Ordering(ord) => self.ordering(ord),
        };
        Ok(Some((2, node)))
    }

    pub(in crate::runtime) fn int64_unop(
        &mut self,
        name: &str,
        args: &[NodeId],
    ) -> Result<Option<(usize, NodeId)>, EvalError> {
        let Some(op) = Int64UnOp::from_prim(name) else {
            return Ok(None);
        };
        let x = self.eval_int64(args[0])?;
        let result = op.apply(x).map_err(|err| self.arithmetic_eval_error(err))?;
        let node = match result {
            Int64UnResult::Int64(n) => self.push_node(Node::Int64(n)),
            Int64UnResult::Int(n) => self.int(n),
        };
        Ok(Some((1, node)))
    }

    pub(in crate::runtime) fn int_conversion(
        &mut self,
        name: &str,
        args: &[NodeId],
    ) -> Result<Option<(usize, NodeId)>, EvalError> {
        let node = match name {
            "itoI" | "utoU" => {
                let n = self.eval_int(args[0])?;
                self.push_node(Node::Int64(n))
            }
            "Itoi" | "Utou" => {
                let n = self.eval_int64(args[0])?;
                self.int(n)
            }
            _ => return Ok(None),
        };
        Ok(Some((1, node)))
    }

    pub(in crate::runtime) fn float64_binop(
        &mut self,
        name: &str,
        args: &[NodeId],
    ) -> Result<Option<(usize, NodeId)>, EvalError> {
        let Some(op) = Float64BinOp::from_prim(name) else {
            return Ok(None);
        };
        let x = self.eval_float64(args[0])?;
        let y = self.eval_float64(args[1])?;
        let node = match op.apply(x, y) {
            Float64Result::Float(n) => self.push_node(Node::Float64(n)),
            Float64Result::Bool(b) => self.prim(if b { "A" } else { "K" }),
        };
        Ok(Some((2, node)))
    }

    pub(in crate::runtime) fn float64_unop(
        &mut self,
        name: &str,
        args: &[NodeId],
    ) -> Result<Option<(usize, NodeId)>, EvalError> {
        let Some(op) = Float64UnOp::from_prim(name) else {
            return Ok(None);
        };
        let x = self.eval_float64(args[0])?;
        let node = self.push_node(Node::Float64(op.apply(x)));
        Ok(Some((1, node)))
    }

    pub(in crate::runtime) fn float32_binop(
        &mut self,
        name: &str,
        args: &[NodeId],
    ) -> Result<Option<(usize, NodeId)>, EvalError> {
        let Some(op) = Float32BinOp::from_prim(name) else {
            return Ok(None);
        };
        let x = self.eval_float32(args[0])?;
        let y = self.eval_float32(args[1])?;
        let node = match op.apply(x, y) {
            Float32Result::Float(n) => self.push_node(Node::Float32(n)),
            Float32Result::Bool(b) => self.prim(if b { "A" } else { "K" }),
        };
        Ok(Some((2, node)))
    }

    pub(in crate::runtime) fn float32_unop(
        &mut self,
        name: &str,
        args: &[NodeId],
    ) -> Result<Option<(usize, NodeId)>, EvalError> {
        let Some(op) = Float32UnOp::from_prim(name) else {
            return Ok(None);
        };
        let x = self.eval_float32(args[0])?;
        let node = self.push_node(Node::Float32(op.apply(x)));
        Ok(Some((1, node)))
    }

    pub(in crate::runtime) fn float_conversion(
        &mut self,
        name: &str,
        args: &[NodeId],
    ) -> Result<Option<(usize, NodeId)>, EvalError> {
        let node = match name {
            "itod" => {
                let n = self.eval_int(args[0])?;
                self.push_node(Node::Float64(n as f64))
            }
            "utod" => {
                let n = self.eval_int(args[0])?;
                self.push_node(Node::Float64((n as u64) as f64))
            }
            "Itod" => {
                let n = self.eval_int64(args[0])?;
                self.push_node(Node::Float64(n as f64))
            }
            "dtoi" => {
                let n = self.eval_float64(args[0])?;
                self.int(n as i64)
            }
            "itof" => {
                let n = self.eval_int(args[0])?;
                self.push_node(Node::Float32(n as f32))
            }
            "utof" => {
                let n = self.eval_int(args[0])?;
                self.push_node(Node::Float32((n as u64) as f32))
            }
            "Itof" => {
                let n = self.eval_int64(args[0])?;
                self.push_node(Node::Float32(n as f32))
            }
            "ftoi" => {
                let n = self.eval_float32(args[0])?;
                self.int(n as i64)
            }
            "dtof" => {
                let n = self.eval_float64(args[0])?;
                self.push_node(Node::Float32(n as f32))
            }
            "ftod" => {
                let n = self.eval_float32(args[0])?;
                self.push_node(Node::Float64(n as f64))
            }
            "toDbl" => {
                let n = self.eval_int64(args[0])?;
                self.push_node(Node::Float64(f64::from_bits(n as u64)))
            }
            "fromDbl" => {
                let n = self.eval_float64(args[0])?;
                self.push_node(Node::Int64(n.to_bits() as i64))
            }
            "toFlt" => {
                let n = self.eval_int(args[0])?;
                self.push_node(Node::Float32(f32::from_bits(n as u32)))
            }
            "fromFlt" => {
                let n = self.eval_float32(args[0])?;
                // Raw float bits are an unsigned 32-bit word; zero-extend to i64 so the
                // sign bit (e.g. -0.0 = 0x8000_0000) survives the Word comparison in
                // isNegZeroFloat32. Sign-extending here breaks negative-zero detection.
                self.int(i64::from(n.to_bits()))
            }
            _ => return Ok(None),
        };
        Ok(Some((1, node)))
    }

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
