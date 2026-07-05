//! Forcing MicroHs values and projecting scalar/cold-node payloads.
use super::*;

impl Program {
    pub(in crate::runtime) fn eval_ffi_name(&mut self, id: NodeId) -> Result<String, EvalError> {
        let bytes = self.eval_string_bytes(id)?;
        String::from_utf8(bytes).map_err(|_| EvalError::InvalidByteString)
    }

    pub(in crate::runtime) fn eval_string_bytes(
        &mut self,
        id: NodeId,
    ) -> Result<Vec<u8>, EvalError> {
        let root = self.reduce_node_whnf(id, FORCE_REDUCTION_LIMIT)?;
        let root = self.resolve(root)?;
        let bytes = match self.cold_node(root) {
            Some(Node::Bytes(bytes)) => bytes.as_slice().to_vec(),
            Some(Node::BytesView(_)) => self.bytes(root)?.to_vec(),
            Some(Node::MutableBytes(bytes)) => bytes.visible().to_vec(),
            _ => self.eval_char_list(root)?,
        };
        Ok(bytes)
    }

    pub(in crate::runtime) fn eval_char_list(
        &mut self,
        mut id: NodeId,
    ) -> Result<Vec<u8>, EvalError> {
        let mut out = Vec::new();
        loop {
            let root = self.reduce_node_whnf(id, FORCE_REDUCTION_LIMIT)?;
            let root = self.resolve(root)?;
            if matches!(self.cell(root).prim(), Some(Prim::Known(KnownPrim::K))) {
                return Ok(out);
            }
            match self.cell(root).app_fields() {
                Some((fun, tail)) => {
                    let fun = self.resolve(fun)?;
                    let Some((cons, head)) = self.cell(fun).app_fields() else {
                        return Err(EvalError::InvalidByteString);
                    };
                    let cons = self.resolve(cons)?;
                    if matches!(self.cell(cons).prim(), Some(Prim::Known(KnownPrim::O))) {
                        out.extend(modified_utf8(self.eval_int(head)?)?);
                        id = tail;
                    } else {
                        return Err(EvalError::InvalidByteString);
                    }
                }
                _ => return Err(EvalError::InvalidByteString),
            }
        }
    }

    #[inline]
    pub(in crate::runtime) fn eval_whnf_value<T>(
        &mut self,
        id: NodeId,
        extract: impl Fn(&Self, NodeId) -> Option<T>,
        expected: impl Fn(NodeId) -> EvalError,
    ) -> Result<T, EvalError> {
        let root = self.resolve(id)?;
        if let Some(value) = extract(self, root) {
            return Ok(value);
        }
        let root = self.reduce_node_whnf(root, FORCE_REDUCTION_LIMIT)?;
        let root = self.resolve(root)?;
        extract(self, root).ok_or_else(|| expected(root))
    }

    pub(in crate::runtime) fn eval_int(&mut self, id: NodeId) -> Result<i64, EvalError> {
        self.eval_whnf_value(
            id,
            |program, root| program.cell_int_value(root),
            EvalError::ExpectedInt,
        )
    }

    pub(in crate::runtime) fn eval_int64(&mut self, id: NodeId) -> Result<i64, EvalError> {
        self.eval_whnf_value(
            id,
            |program, root| program.cell_int64_value(root),
            EvalError::ExpectedInt64,
        )
    }

    pub(in crate::runtime) fn eval_float64(&mut self, id: NodeId) -> Result<f64, EvalError> {
        self.eval_whnf_value(
            id,
            |program, root| program.cell_float64_value(root),
            EvalError::ExpectedFloat64,
        )
    }

    pub(in crate::runtime) fn eval_float32(&mut self, id: NodeId) -> Result<f32, EvalError> {
        self.eval_whnf_value(
            id,
            |program, root| program.cell_float32_value(root),
            EvalError::ExpectedFloat32,
        )
    }

    pub(in crate::runtime) fn eval_bool(&mut self, id: NodeId) -> Result<bool, EvalError> {
        self.eval_whnf_value(
            id,
            |program, root| match program.cell(root).prim() {
                Some(Prim::Known(KnownPrim::A)) => Some(true),
                Some(Prim::Known(KnownPrim::K)) => Some(false),
                _ => None,
            },
            EvalError::ExpectedInt,
        )
    }

    pub(in crate::runtime) fn eval_thread_id(&mut self, id: NodeId) -> Result<i64, EvalError> {
        self.eval_whnf_value(
            id,
            |program, root| program.cell_thread_id_value(root),
            EvalError::ExpectedThreadId,
        )
    }

    pub(in crate::runtime) fn eval_pointer_value(&mut self, id: NodeId) -> Result<i64, EvalError> {
        let root = self.resolve(id)?;
        if let Some(value) = self.pointer_value_from_whnf(root) {
            return Ok(value);
        }
        let root = self.reduce_node_whnf(root, FORCE_REDUCTION_LIMIT)?;
        let root = self.resolve(root)?;
        self.pointer_value_from_whnf(root)
            .ok_or(EvalError::ExpectedPointer(root))
    }

    pub(in crate::runtime) fn pointer_value_from_whnf(&self, root: NodeId) -> Option<i64> {
        let cell = self.cell(root);
        if let Some(value) = self.cell_int_value(root) {
            return Some(value);
        }
        if let Some(value) = self.cell_ptr_value(root) {
            return Some(value);
        }
        if let Some(value) = self.cell_raw_fun_ptr_value(root) {
            return Some(value);
        }
        if let Some(value) = self.cell_thread_id_value(root) {
            return Some(value);
        }
        match cell.prim() {
            Some(prim) => std_handle_ptr(prim.name()),
            _ => None,
        }
    }

    pub(in crate::runtime) fn expected_bytes_error(&self, id: NodeId) -> EvalError {
        EvalError::ExpectedBytes(id)
    }

    pub(in crate::runtime) fn eval_foreign_ptr_id(
        &mut self,
        id: NodeId,
    ) -> Result<NodeId, EvalError> {
        self.eval_whnf_value(
            id,
            |program, root| {
                if matches!(program.cold_node(root), Some(Node::ForeignPtr(_))) {
                    return Some(root);
                }
                match program.cell(root).prim() {
                    Some(prim) if std_handle(prim.name()).is_some() => Some(root),
                    _ => None,
                }
            },
            EvalError::ExpectedForeignPtr,
        )
    }

    pub(in crate::runtime) fn eval_bytes(&mut self, id: NodeId) -> Result<Vec<u8>, EvalError> {
        let id = self.eval_bytes_id(id)?;
        Ok(self.bytes(id)?.to_vec())
    }

    pub(in crate::runtime) fn eval_bytes_id(&mut self, id: NodeId) -> Result<NodeId, EvalError> {
        self.eval_whnf_value(
            id,
            |program, root| match program.cold_node(root) {
                Some(Node::Bytes(_) | Node::BytesView(_) | Node::MutableBytes(_)) => Some(root),
                _ => None,
            },
            EvalError::ExpectedBytes,
        )
    }

    pub(in crate::runtime) fn eval_array_id(&mut self, id: NodeId) -> Result<NodeId, EvalError> {
        self.eval_whnf_value(
            id,
            |program, root| match program.cold_node(root) {
                Some(Node::Array(_)) => Some(root),
                _ => None,
            },
            EvalError::ExpectedArray,
        )
    }

    pub(in crate::runtime) fn array(&self, id: NodeId) -> Result<&[NodeId], EvalError> {
        match self.cold_node(id) {
            Some(Node::Array(items)) => Ok(items.as_slice()),
            _ => Err(EvalError::ExpectedArray(id)),
        }
    }

    pub(in crate::runtime) fn array_mut(
        &mut self,
        id: NodeId,
    ) -> Result<&mut Vec<NodeId>, EvalError> {
        match self.cold_node_mut(id) {
            Some(Node::Array(items)) => Ok(items.as_mut()),
            _ => Err(EvalError::ExpectedArray(id)),
        }
    }

    pub(in crate::runtime) fn bytes(&self, id: NodeId) -> Result<&[u8], EvalError> {
        match self.cold_node(id) {
            Some(Node::Bytes(bytes)) => Ok(bytes.as_slice()),
            Some(Node::BytesView(view)) => {
                let base = self.bytes(view.base)?;
                let end = view
                    .offset
                    .checked_add(view.len)
                    .ok_or(EvalError::Overflow)?;
                base.get(view.offset..end)
                    .ok_or(EvalError::InvalidByteString)
            }
            Some(Node::MutableBytes(bytes)) => Ok(bytes.visible()),
            _ => Err(self.expected_bytes_error(id)),
        }
    }
}
