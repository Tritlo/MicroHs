use super::*;

impl Program {
    pub fn resolve(&self, mut id: NodeId) -> Result<NodeId, EvalError> {
        loop {
            let Some(cell) = self.nodes.get(id.index()).copied() else {
                return Err(EvalError::DanglingIndirection(id));
            };
            match cell.tag_bits() {
                tag if tag == CellTag::Indir.bits() => match cell.option_id_word1() {
                    Some(next) => id = next,
                    None => return Err(EvalError::DanglingIndirection(id)),
                },
                tag if tag == CellTag::Free.bits() => {
                    return Err(EvalError::DanglingIndirection(id));
                }
                _ => return Ok(id),
            }
        }
    }

    #[inline]
    pub(in crate::runtime) fn resolve_whnf_trusted(&self, mut id: NodeId) -> NodeId {
        loop {
            let cell = self.cell_trusted(id);
            match cell.tag_bits() {
                tag if tag == CellTag::Indir.bits() => {
                    debug_assert_ne!(cell.word1, CELL_NONE_ID);
                    id = NodeId(cell.word1 as u32);
                }
                tag => {
                    debug_assert_ne!(tag, CellTag::Free.bits());
                    return id;
                }
            }
        }
    }

    pub(in crate::runtime) fn resolve_profiled(
        &mut self,
        mut id: NodeId,
    ) -> Result<NodeId, EvalError> {
        if self.profile.is_none() {
            return self.resolve(id);
        }

        let mut depth = 0;
        loop {
            let Some(cell) = self.nodes.get(id.index()).copied() else {
                return Err(EvalError::DanglingIndirection(id));
            };
            match cell.tag_bits() {
                tag if tag == CellTag::Indir.bits() => match cell.option_id_word1() {
                    Some(next) => {
                        id = next;
                        depth += 1;
                    }
                    None => return Err(EvalError::DanglingIndirection(id)),
                },
                tag if tag == CellTag::Free.bits() => {
                    return Err(EvalError::DanglingIndirection(id));
                }
                _ => {
                    self.profile_resolve_chain(depth);
                    return Ok(id);
                }
            }
        }
    }

    pub fn reduce_whnf(&mut self, limit: usize) -> Result<(NodeId, usize), EvalError> {
        let (root, steps) = self.reduce_whnf_from(self.root, limit, true, false)?;
        self.root = root;
        Ok((root, steps))
    }

    pub fn reduce_main(&mut self, limit: usize) -> Result<(NodeId, usize), EvalError> {
        let world = self.world();
        let root = self.app(self.root, world);
        let reductions = self.reductions;
        let root = self.reduce_node_whnf(root, limit)?;
        Ok((root, self.reductions - reductions))
    }

    pub fn reduction_count(&self) -> usize {
        self.reductions
    }

    pub fn uncaught_exception_message_bytes(&mut self, exn: NodeId) -> Result<Vec<u8>, EvalError> {
        let exn = self.resolve(exn)?;
        if let Some(code) = self.cell(exn).int_value() {
            return Ok(rts_exception_message(code).to_vec());
        }

        let u = self.prim("U");
        let k2 = self.prim("K2");
        let true_ = self.prim("A");
        let k2_true = self.app(k2, true_);
        let inner = self.app(u, k2_true);
        let show_exn = self.app(u, inner);
        let displayed = self.app(show_exn, exn);
        self.eval_string_bytes(displayed)
    }

    pub fn apply_stable_ptr_pointer(
        &mut self,
        stable_ptr: usize,
        arg: i64,
        limit: usize,
    ) -> Result<i64, EvalError> {
        let fun = self.deref_stable_ptr(stable_ptr)?;
        let arg = self.push_node(Node::Ptr(arg));
        let action = self.app(fun, arg);
        let perform_io = self.prim("IO.performIO");
        let root = self.app(perform_io, action);
        let root = self.reduce_node_whnf(root, limit)?;
        self.eval_pointer_value(root)
    }

    pub fn apply_js_wrapper(
        &mut self,
        tags: &str,
        stable_ptr: usize,
        args: &[JsValue],
        limit: usize,
    ) -> Result<JsValue, EvalError> {
        let tags = tags.as_bytes();
        validate_js_tags(tags)?;
        if args.len() != tags.len() - 1 {
            return Err(EvalError::InvalidArray);
        }

        let mut root = self.deref_stable_ptr(stable_ptr)?;
        for (tag, arg) in tags[1..].iter().copied().zip(args) {
            let arg = self.js_value_node(tag, arg)?;
            root = self.app(root, arg);
        }
        let perform_io = self.prim("IO.performIO");
        let root = self.app(perform_io, root);
        let root = self.reduce_node_whnf(root, limit)?;
        self.js_value_from_node(tags[0], root)
    }

    pub fn apply_js_wrapper_index(
        &mut self,
        wrapper_index: u32,
        stable_ptr: usize,
        args: &[JsValue],
        limit: usize,
    ) -> Result<JsValue, EvalError> {
        let tags = self.js_wrapper_tags(wrapper_index)?.to_owned();
        self.apply_js_wrapper(&tags, stable_ptr, args, limit)
    }

    pub fn js_wrapper_tags(&self, wrapper_index: u32) -> Result<&str, EvalError> {
        self.js_wrapper_tags
            .get(usize::try_from(wrapper_index).map_err(|_| EvalError::Overflow)?)
            .map(String::as_str)
            .ok_or(EvalError::InvalidArray)
    }
}
