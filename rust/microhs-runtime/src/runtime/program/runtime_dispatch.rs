impl Program {
    fn node_trace_summary(&self, id: NodeId) -> String {
        let Some(cell) = self.nodes.get(id.index()).copied() else {
            return format!("{id:?}:<missing>");
        };
        let node = cell.to_node(&self.cold_nodes);
        match node {
            Node::Int(n) => format!("{id:?}:Int({n})"),
            Node::Int64(n) => format!("{id:?}:Int64({n})"),
            Node::Ptr(ptr) => format!("{id:?}:Ptr({ptr})"),
            Node::RawFunPtr(ptr) => format!("{id:?}:RawFunPtr({ptr})"),
            Node::ThreadId(n) => format!("{id:?}:ThreadId({n})"),
            Node::Prim(name) => format!("{id:?}:Prim({})", name.name()),
            Node::Ffi(name) => format!("{id:?}:Ffi({name})"),
            Node::Bytes(bytes) => format!("{id:?}:Bytes(len={})", bytes.len()),
            Node::BytesView(view) => format!(
                "{id:?}:BytesView(base={:?}, offset={}, len={})",
                view.base, view.offset, view.len
            ),
            Node::MutableBytes(bytes) => {
                format!(
                    "{id:?}:MutableBytes(size={}, capacity={})",
                    bytes.size, bytes.capacity
                )
            }
            Node::ForeignPtr(ptr) => format!(
                "{id:?}:ForeignPtr(ptr={}, offset={}, bytes={})",
                ptr.ptr,
                ptr.offset,
                ptr.bytes.as_ref().map_or(0, Vec::len)
            ),
            Node::App(fun, arg) => format!("{id:?}:App({fun:?},{arg:?})"),
            Node::Indir(target) => format!("{id:?}:Indir({target:?})"),
            Node::Free(next) => format!("{id:?}:Free({next:?})"),
            Node::BigInt(bytes) => format!("{id:?}:BigInt(len={})", bytes.len()),
            Node::Array(items) => format!("{id:?}:Array(len={})", items.len()),
            Node::Float64(n) => format!("{id:?}:Float64({n})"),
            Node::Float32(n) => format!("{id:?}:Float32({n})"),
            Node::Weak(_) => format!("{id:?}:Weak"),
            Node::MVar(_) => format!("{id:?}:MVar"),
            Node::JsCall(call) => format!("{id:?}:JsCall(tags={})", call.tags),
            Node::JsWrap { tags } => format!("{id:?}:JsWrap(tags={tags})"),
            Node::FunPtr(name) => format!("{id:?}:FunPtr({name})"),
            Node::Tick(bytes) => format!("{id:?}:Tick(len={})", bytes.len()),
        }
    }

    fn trace_invalid_op_error(
        &self,
        domain: &str,
        name: &str,
        args: &[NodeId],
        err: EvalError,
    ) -> EvalError {
        if matches!(err, EvalError::InvalidByteString)
            && std::env::var_os("MHS_TRACE_INVALID_BYTES").is_some()
        {
            eprintln!(
                "invalid bytes context: domain={domain} name={name} reductions={}",
                self.reductions
            );
            for (idx, arg) in args.iter().take(8).enumerate() {
                eprintln!("  arg{idx}: {}", self.node_trace_summary(*arg));
            }
            if args.len() > 8 {
                eprintln!("  ... {} more args", args.len() - 8);
            }
        }
        err
    }

    fn pointer_conversion(
        &mut self,
        name: &str,
        args: &[NodeId],
    ) -> Result<Option<(usize, NodeId)>, EvalError> {
        let value = match name {
            "toInt" => Node::Int(self.eval_pointer_value(args[0])?),
            "toPtr" => Node::Ptr(self.eval_pointer_value(args[0])?),
            "toFunPtr" => Node::RawFunPtr(self.eval_pointer_value(args[0])?),
            _ => return Ok(None),
        };
        Ok(Some((1, self.push_value_node(value))))
    }

    fn foreign_ptr_op(
        &mut self,
        name: &str,
        args: &[NodeId],
    ) -> Result<Option<(usize, NodeId)>, EvalError> {
        match self.foreign_ptr_op_inner(name, args) {
            Ok(result) => Ok(result),
            Err(err) => Err(self.trace_invalid_op_error("foreign_ptr_op", name, args, err)),
        }
    }

    fn foreign_ptr_op_inner(
        &mut self,
        name: &str,
        args: &[NodeId],
    ) -> Result<Option<(usize, NodeId)>, EvalError> {
        let rewrite = match name {
            "fp+" => {
                let foreign_ptr = self.eval_foreign_ptr_id(args[0])?;
                let offset = int_to_usize(self.eval_int(args[1])?)?;
                Some((2, self.offset_foreign_ptr(foreign_ptr, offset)?))
            }
            "fp2bs" => {
                let foreign_ptr = self.eval_foreign_ptr_id(args[0])?;
                let len = int_to_usize(self.eval_int(args[1])?)?;
                Some((2, self.foreign_ptr_to_bytes(foreign_ptr, len)?))
            }
            "fpnew" if args.len() >= 2 => {
                let ptr = self.eval_pointer_value(args[0])?;
                let node = self.foreign_ptr_node(None, 0, ptr);
                let foreign_ptr = self.push_node(node);
                Some((2, self.pair(foreign_ptr, args[1])))
            }
            "fpfin" if args.len() >= 3 => {
                let foreign_ptr = self.eval_foreign_ptr_id(args[1])?;
                self.set_foreign_ptr_finalizer(foreign_ptr, args[0])?;
                let unit = self.prim("I");
                Some((3, self.pair(unit, args[2])))
            }
            "fpfin" => {
                let foreign_ptr = self.eval_foreign_ptr_id(args[1])?;
                self.set_foreign_ptr_finalizer(foreign_ptr, args[0])?;
                Some((2, self.prim("I")))
            }
            _ => None,
        };
        Ok(rewrite)
    }

    fn foreign_ptr_unop(
        &mut self,
        name: &str,
        args: &[NodeId],
    ) -> Result<Option<(usize, NodeId)>, EvalError> {
        match self.foreign_ptr_unop_inner(name, args) {
            Ok(result) => Ok(result),
            Err(err) => Err(self.trace_invalid_op_error("foreign_ptr_unop", name, args, err)),
        }
    }

    fn foreign_ptr_unop_inner(
        &mut self,
        name: &str,
        args: &[NodeId],
    ) -> Result<Option<(usize, NodeId)>, EvalError> {
        let node = match name {
            "bs2fp" => {
                let bytes_id = self.eval_bytes_id(args[0])?;
                let bytes = self.bytes(bytes_id)?.to_vec();
                let ptr = self.pointer_for_node(bytes_id, 0)?;
                let node = self.foreign_ptr_node(Some(bytes), 0, ptr);
                self.push_node(node)
            }
            "fp2p" => {
                let foreign_ptr = self.eval_foreign_ptr_id(args[0])?;
                let ptr = self.foreign_ptr_value(foreign_ptr)?;
                self.push_node(Node::Ptr(ptr))
            }
            "fpnew" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let node = self.foreign_ptr_node(None, 0, ptr);
                self.push_node(node)
            }
            _ => return Ok(None),
        };
        Ok(Some((1, node)))
    }

    fn array_op(
        &mut self,
        name: &str,
        args: &[NodeId],
    ) -> Result<Option<(usize, NodeId)>, EvalError> {
        let rewrite = match name {
            "A.alloc" if args.len() >= 3 => {
                let len = int_to_usize(self.eval_int(args[0])?)?;
                let array = self.push_node(Node::array(vec![args[1]; len]));
                Some((3, self.pair(array, args[2])))
            }
            "A.alloc" => {
                let len = int_to_usize(self.eval_int(args[0])?)?;
                Some((2, self.push_node(Node::array(vec![args[1]; len]))))
            }
            "A.read" if args.len() >= 3 => {
                let array = self.eval_array_id(args[0])?;
                let index = int_to_usize(self.eval_int(args[1])?)?;
                let item = *self
                    .array(array)?
                    .get(index)
                    .ok_or(EvalError::InvalidArray)?;
                Some((3, self.pair(item, args[2])))
            }
            "A.read" if args.len() >= 2 => {
                let array = self.eval_array_id(args[0])?;
                let index = int_to_usize(self.eval_int(args[1])?)?;
                let item = *self
                    .array(array)?
                    .get(index)
                    .ok_or(EvalError::InvalidArray)?;
                Some((2, item))
            }
            "A.write" if args.len() >= 4 => {
                let array = self.eval_array_id(args[0])?;
                let index = int_to_usize(self.eval_int(args[1])?)?;
                let items = self.array_mut(array)?;
                let slot = items.get_mut(index).ok_or(EvalError::InvalidArray)?;
                *slot = args[2];
                let unit = self.prim("I");
                Some((4, self.pair(unit, args[3])))
            }
            "A.write" if args.len() >= 3 => {
                let array = self.eval_array_id(args[0])?;
                let index = int_to_usize(self.eval_int(args[1])?)?;
                let items = self.array_mut(array)?;
                let slot = items.get_mut(index).ok_or(EvalError::InvalidArray)?;
                *slot = args[2];
                Some((3, self.prim("I")))
            }
            "A.trunc" if args.len() >= 3 => {
                let array = self.eval_array_id(args[0])?;
                let len = int_to_usize(self.eval_int(args[1])?)?;
                let items = self.array_mut(array)?;
                if len >= items.len() {
                    return Err(EvalError::InvalidArray);
                }
                items.truncate(len);
                let unit = self.prim("I");
                Some((3, self.pair(unit, args[2])))
            }
            "A.trunc" if args.len() >= 2 => {
                let array = self.eval_array_id(args[0])?;
                let len = int_to_usize(self.eval_int(args[1])?)?;
                let items = self.array_mut(array)?;
                if len >= items.len() {
                    return Err(EvalError::InvalidArray);
                }
                items.truncate(len);
                Some((2, self.prim("I")))
            }
            "A.==" => {
                let x = self.eval_array_id(args[0])?;
                let y = self.eval_array_id(args[1])?;
                Some((2, self.prim(if x == y { "A" } else { "K" })))
            }
            _ => None,
        };
        Ok(rewrite)
    }

    fn array_unop(
        &mut self,
        name: &str,
        args: &[NodeId],
    ) -> Result<Option<(usize, NodeId)>, EvalError> {
        let node = match name {
            "A.copy" if args.len() >= 2 => {
                let array = self.eval_array_id(args[0])?;
                let items = self.array(array)?.to_vec();
                let copy = self.push_node(Node::array(items));
                self.pair(copy, args[1])
            }
            "A.copy" => {
                let array = self.eval_array_id(args[0])?;
                let items = self.array(array)?.to_vec();
                self.push_node(Node::array(items))
            }
            "A.size" if args.len() >= 2 => {
                let array = self.eval_array_id(args[0])?;
                let len =
                    i64::try_from(self.array(array)?.len()).map_err(|_| EvalError::Overflow)?;
                let size = self.int(len);
                self.pair(size, args[1])
            }
            "A.size" => {
                let array = self.eval_array_id(args[0])?;
                let len =
                    i64::try_from(self.array(array)?.len()).map_err(|_| EvalError::Overflow)?;
                self.int(len)
            }
            _ => return Ok(None),
        };
        let used = if matches!(name, "A.copy" | "A.size") && args.len() >= 2 {
            2
        } else {
            1
        };
        Ok(Some((used, node)))
    }

    fn stable_ptr_op(
        &mut self,
        name: &str,
        args: &[NodeId],
    ) -> Result<Option<(usize, NodeId)>, EvalError> {
        let rewrite = match name {
            "SPnew" if args.len() >= 2 => {
                let handle = self.new_stable_ptr(args[0])?;
                Some((2, self.pair(handle, args[1])))
            }
            "SPderef" if args.len() >= 2 => {
                let handle = self.stable_ptr_handle(args[0])?;
                let value = self.deref_stable_ptr(handle)?;
                Some((2, self.pair(value, args[1])))
            }
            "SPfree" if args.len() >= 2 => {
                let handle = self.stable_ptr_handle(args[0])?;
                self.free_stable_ptr(handle)?;
                let unit = self.prim("I");
                Some((2, self.pair(unit, args[1])))
            }
            _ => None,
        };
        Ok(rewrite)
    }

    fn stable_ptr_unop(
        &mut self,
        name: &str,
        args: &[NodeId],
    ) -> Result<Option<(usize, NodeId)>, EvalError> {
        let node = match name {
            "SPnew" => self.new_stable_ptr(args[0])?,
            "SPderef" => {
                let handle = self.stable_ptr_handle(args[0])?;
                self.deref_stable_ptr(handle)?
            }
            "SPfree" => {
                let handle = self.stable_ptr_handle(args[0])?;
                self.free_stable_ptr(handle)?;
                self.prim("I")
            }
            _ => return Ok(None),
        };
        Ok(Some((1, node)))
    }

    fn weak_ptr_op(
        &mut self,
        name: &str,
        args: &[NodeId],
    ) -> Result<Option<(usize, NodeId)>, EvalError> {
        let rewrite = match name {
            "Wknewfin" if args.len() >= 4 => {
                let weak = self.new_weak_ptr(args[0], args[1], Some(args[2]));
                Some((4, self.pair(weak, args[3])))
            }
            "Wknewfin" if args.len() >= 3 => {
                let weak = self.new_weak_ptr(args[0], args[1], Some(args[2]));
                Some((3, weak))
            }
            "Wknew" if args.len() >= 3 => {
                let weak = self.new_weak_ptr(args[0], args[1], None);
                Some((3, self.pair(weak, args[2])))
            }
            "Wknew" if args.len() >= 2 => {
                let weak = self.new_weak_ptr(args[0], args[1], None);
                Some((2, weak))
            }
            "Wkderef" if args.len() >= 2 => {
                let value = self.deref_weak_ptr(args[0])?;
                Some((2, self.pair(value, args[1])))
            }
            "Wkfinal" if args.len() >= 2 => {
                self.finalize_weak_ptr(args[0])?;
                let unit = self.prim("I");
                Some((2, self.pair(unit, args[1])))
            }
            _ => None,
        };
        Ok(rewrite)
    }

    fn weak_ptr_unop(
        &mut self,
        name: &str,
        args: &[NodeId],
    ) -> Result<Option<(usize, NodeId)>, EvalError> {
        let node = match name {
            "Wkderef" => self.deref_weak_ptr(args[0])?,
            "Wkfinal" => {
                self.finalize_weak_ptr(args[0])?;
                self.prim("I")
            }
            _ => return Ok(None),
        };
        Ok(Some((1, node)))
    }

    fn bytes_op(
        &mut self,
        name: &str,
        args: &[NodeId],
    ) -> Result<Option<(usize, NodeId)>, EvalError> {
        match self.bytes_op_inner(name, args) {
            Ok(result) => Ok(result),
            Err(err) => Err(self.trace_invalid_op_error("bytes_op", name, args, err)),
        }
    }

    fn bytes_op_inner(
        &mut self,
        name: &str,
        args: &[NodeId],
    ) -> Result<Option<(usize, NodeId)>, EvalError> {
        macro_rules! invalid_bytes {
            ($($arg:tt)*) => {{
                if std::env::var_os("MHS_TRACE_INVALID_BYTES").is_some() {
                    eprintln!(
                        "invalid bytes op {name}: reductions={}",
                        self.reductions,
                    );
                    eprintln!($($arg)*);
                }
                EvalError::InvalidByteString
            }};
        }
        if let Some(op) = BytesBinOp::from_prim(name) {
            let x = self.eval_bytes_id(args[0])?;
            let y = self.eval_bytes_id(args[1])?;
            let node = self.bytes_bin_result_node(op, x, y)?;
            return Ok(Some((2, node)));
        }

        let rewrite = match name {
            "packCString" if args.len() >= 2 => {
                let ptr = self.eval_pointer_value(args[0])?;
                let bytes = self.read_c_string(ptr)?;
                let bytes = self.push_node(Node::bytes(bytes));
                Some((2, self.pair(bytes, args[1])))
            }
            "packCStringLen" if args.len() >= 3 => {
                let ptr = self.eval_pointer_value(args[0])?;
                let len = int_to_usize(self.eval_int(args[1])?)?;
                let bytes = self.read_pointer_bytes(ptr, len)?;
                let bytes = self.push_node(Node::bytes(bytes));
                Some((3, self.pair(bytes, args[2])))
            }
            "packCStringLen" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let len = int_to_usize(self.eval_int(args[1])?)?;
                let bytes = self.read_pointer_bytes(ptr, len)?;
                Some((2, self.push_node(Node::bytes(bytes))))
            }
            "bsgrab" if args.len() >= 2 => {
                let ptr = self.eval_pointer_value(args[0])?;
                let bytes = self.read_c_string(ptr)?;
                let bytes = self.push_node(Node::bytes(bytes));
                Some((2, self.pair(bytes, args[1])))
            }
            "bsgrablen" if args.len() >= 3 => {
                let ptr = self.eval_pointer_value(args[0])?;
                let len = int_to_usize(self.eval_int(args[1])?)?;
                let bytes = self.read_pointer_bytes(ptr, len)?;
                let bytes = self.push_node(Node::bytes(bytes));
                Some((3, self.pair(bytes, args[2])))
            }
            "bsgrablen" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let len = int_to_usize(self.eval_int(args[1])?)?;
                let bytes = self.read_pointer_bytes(ptr, len)?;
                Some((2, self.push_node(Node::bytes(bytes))))
            }
            "bsnew" if args.len() >= 3 => {
                let size = int_to_usize(self.eval_int(args[0])?)?;
                let capacity = int_to_usize(self.eval_int(args[1])?)?;
                let bytes = self.new_mutable_bytes(size, capacity)?;
                Some((3, self.pair(bytes, args[2])))
            }
            "bsnew" => {
                let size = int_to_usize(self.eval_int(args[0])?)?;
                let capacity = int_to_usize(self.eval_int(args[1])?)?;
                Some((2, self.new_mutable_bytes(size, capacity)?))
            }
            "bsread" if args.len() >= 3 => {
                let bytes = self.eval_bytes_id(args[0])?;
                let index = int_to_usize(self.eval_int(args[1])?)?;
                let (len, storage_len) = self.byte_prim_lengths(bytes)?;
                if index >= storage_len {
                    return Err(invalid_bytes!(
                        "bsread bytes={bytes:?} index={index} len={len} storage_len={storage_len}"
                    ));
                }
                let byte = self.read_byte_unchecked_prim(bytes, index)?;
                let byte = self.int(byte as i64);
                Some((3, self.pair(byte, args[2])))
            }
            "bsread" => {
                let bytes = self.eval_bytes_id(args[0])?;
                let index = int_to_usize(self.eval_int(args[1])?)?;
                let (len, storage_len) = self.byte_prim_lengths(bytes)?;
                if index >= storage_len {
                    return Err(invalid_bytes!(
                        "bsread bytes={bytes:?} index={index} len={len} storage_len={storage_len}"
                    ));
                }
                let byte = self.read_byte_unchecked_prim(bytes, index)?;
                Some((2, self.int(byte as i64)))
            }
            "bswrite" if args.len() >= 4 => {
                let bytes = self.eval_bytes_id(args[0])?;
                let index = int_to_usize(self.eval_int(args[1])?)?;
                let byte = self.eval_int(args[2])? as u8;
                let (len, storage_len) = self.byte_prim_lengths(bytes)?;
                if index >= storage_len {
                    return Err(invalid_bytes!(
                        "bswrite bytes={bytes:?} index={index} len={len} storage_len={storage_len}"
                    ));
                }
                self.write_byte_unchecked_prim(bytes, index, byte)?;
                let unit = self.prim("I");
                Some((4, self.pair(unit, args[3])))
            }
            "bswrite" if args.len() >= 3 => {
                let bytes = self.eval_bytes_id(args[0])?;
                let index = int_to_usize(self.eval_int(args[1])?)?;
                let byte = self.eval_int(args[2])? as u8;
                let (len, storage_len) = self.byte_prim_lengths(bytes)?;
                if index >= storage_len {
                    return Err(invalid_bytes!(
                        "bswrite bytes={bytes:?} index={index} len={len} storage_len={storage_len}"
                    ));
                }
                self.write_byte_unchecked_prim(bytes, index, byte)?;
                Some((3, self.prim("I")))
            }
            "bsfreeze" if args.len() >= 2 => {
                let bytes = self.eval_bytes_id(args[0])?;
                let bytes = self.freeze_bytes(bytes)?;
                Some((2, self.pair(bytes, args[1])))
            }
            "bsappbyte" if args.len() >= 3 => {
                let bytes = self.eval_bytes_id(args[0])?;
                let byte = self.eval_int(args[1])? as u8;
                self.append_byte(bytes, byte)?;
                let unit = self.prim("I");
                Some((3, self.pair(unit, args[2])))
            }
            "bsappbyte" => {
                let bytes = self.eval_bytes_id(args[0])?;
                let byte = self.eval_int(args[1])? as u8;
                self.append_byte(bytes, byte)?;
                Some((2, self.prim("I")))
            }
            "bsappchar" if args.len() >= 3 => {
                let bytes = self.eval_bytes_id(args[0])?;
                let encoded = modified_utf8(self.eval_int(args[1])?)?;
                self.append_bytes(bytes, &encoded)?;
                let unit = self.prim("I");
                Some((3, self.pair(unit, args[2])))
            }
            "bsappchar" => {
                let bytes = self.eval_bytes_id(args[0])?;
                let encoded = modified_utf8(self.eval_int(args[1])?)?;
                self.append_bytes(bytes, &encoded)?;
                Some((2, self.prim("I")))
            }
            "bsreplicate" => {
                let len = int_to_usize(self.eval_int(args[0])?)?;
                let byte = self.eval_int(args[1])? as u8;
                Some((2, self.push_node(Node::bytes(vec![byte; len]))))
            }
            "bsindex" => {
                let bytes_id = self.eval_bytes_id(args[0])?;
                let index = int_to_usize(self.eval_int(args[1])?)?;
                let bytes = self.bytes(bytes_id)?;
                let len = bytes.len();
                if index >= len {
                    return Err(invalid_bytes!("bsindex index={index} len={len}"));
                }
                let byte = bytes[index];
                Some((2, self.int(byte as i64)))
            }
            "bssubstr" if args.len() >= 3 => {
                let bytes_id = self.eval_bytes_id(args[0])?;
                let offset = int_to_usize(self.eval_int(args[1])?)?;
                let len = int_to_usize(self.eval_int(args[2])?)?;
                let bytes = self.bytes(bytes_id)?;
                let bytes_len = bytes.len();
                offset
                    .checked_add(len)
                    .filter(|end| *end <= bytes.len())
                    .ok_or_else(|| {
                        invalid_bytes!("bssubstr offset={offset} len={len} bytes_len={bytes_len}")
                    })?;
                let node = self.byte_slice_node(bytes_id, offset, len)?;
                Some((3, self.push_node(node)))
            }
            _ => None,
        };
        Ok(rewrite)
    }

    fn bytes_unop(
        &mut self,
        name: &str,
        args: &[NodeId],
    ) -> Result<Option<(usize, NodeId)>, EvalError> {
        match self.bytes_unop_inner(name, args) {
            Ok(result) => Ok(result),
            Err(err) => Err(self.trace_invalid_op_error("bytes_unop", name, args, err)),
        }
    }

    fn bytes_unop_inner(
        &mut self,
        name: &str,
        args: &[NodeId],
    ) -> Result<Option<(usize, NodeId)>, EvalError> {
        let node = match name {
            "packCString" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let bytes = self.read_c_string(ptr)?;
                self.push_node(Node::bytes(bytes))
            }
            "bsgrab" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let bytes = self.read_c_string(ptr)?;
                self.push_node(Node::bytes(bytes))
            }
            "bslength" => {
                let bytes_id = self.eval_bytes_id(args[0])?;
                let len =
                    i64::try_from(self.bytes(bytes_id)?.len()).map_err(|_| EvalError::Overflow)?;
                self.int(len)
            }
            "headUTF8" => {
                let bytes_id = self.eval_bytes_id(args[0])?;
                let (codepoint, _) = head_utf8(self.bytes(bytes_id)?)?;
                self.int(codepoint as i64)
            }
            "tailUTF8" => {
                let bytes_id = self.eval_bytes_id(args[0])?;
                let bytes = self.bytes(bytes_id)?;
                let (_, offset) = head_utf8(bytes)?;
                let len = bytes.len() - offset;
                let node = self.byte_slice_node(bytes_id, offset, len)?;
                self.push_node(node)
            }
            "bsunpack" => {
                let bytes = self.eval_bytes(args[0])?;
                let values = bytes.into_iter().map(i64::from);
                self.int_list(values)
            }
            "fromUTF8" => {
                let bytes = self.eval_bytes(args[0])?;
                let values = decode_utf8_string_bytes(&bytes)?;
                self.int_list(values.into_iter().map(i64::from))
            }
            "bsfreeze" => {
                let bytes = self.eval_bytes_id(args[0])?;
                self.freeze_bytes(bytes)?
            }
            _ => return Ok(None),
        };
        Ok(Some((1, node)))
    }

    fn ffi_call(
        &mut self,
        name: &str,
        args: &[NodeId],
    ) -> Result<Option<(usize, NodeId)>, EvalError> {
        match self.ffi_call_inner(name, args) {
            Ok(result) => Ok(result),
            Err(err) => Err(self.trace_invalid_op_error("ffi", name, args, err)),
        }
    }

    fn ffi_call_inner(
        &mut self,
        name: &str,
        args: &[NodeId],
    ) -> Result<Option<(usize, NodeId)>, EvalError> {
        if !args.is_empty() {
            if let Some(result) = self.zero_arity_ffi_result(name)? {
                let result = self.push_value_node(result);
                return Ok(Some((1, self.pair(result, args[0]))));
            }
        }
        if args.len() >= 2 && is_unary_math_ffi_candidate(name) {
            if let Some(result) = self.unary_math_ffi_result(name, args[0])? {
                let result = self.push_value_node(result);
                return Ok(Some((2, self.pair(result, args[1]))));
            }
        }
        let arity = ffi_arity(name).ok_or_else(|| EvalError::UnknownFfi(name.to_owned()))?;
        if args.len() < arity + 1 {
            return Ok(None);
        }

        let result = match name {
            name if errno_constant(name).is_some() => {
                Node::Int(errno_constant(name).expect("checked errno constant"))
            }
            name if host_constant(name).is_some() => {
                Node::Int(host_constant(name).expect("checked host constant"))
            }
            "GETRAW" => Node::Int(-1),
            "GETTIMEMICRO" => Node::Int(current_time_micro()),
            "islinux" => Node::Int(i64::from(cfg!(target_os = "linux"))),
            "ismacos" => Node::Int(i64::from(cfg!(target_os = "macos"))),
            "iswindows" => Node::Int(i64::from(cfg!(target_os = "windows"))),
            "sizeof_char" => Node::Int(size_of_i64::<std::os::raw::c_char>()),
            "sizeof_short" => Node::Int(size_of_i64::<std::os::raw::c_short>()),
            "sizeof_int" => Node::Int(size_of_i64::<std::os::raw::c_int>()),
            "sizeof_long" => Node::Int(size_of_i64::<std::os::raw::c_long>()),
            "sizeof_llong" => Node::Int(size_of_i64::<std::os::raw::c_longlong>()),
            "sizeof_size_t" => Node::Int(size_of_i64::<usize>()),
            "want_gmp" => Node::Int(0),
            "want_imath" => Node::Int(1),
            "js_debug" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let bytes = self.read_c_string(ptr)?;
                host_js_debug(&bytes)?;
                Node::prim("I")
            }
            "js_eval_run" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let bytes = self.read_c_string(ptr)?;
                host_js_eval_run(&bytes)?;
                Node::prim("I")
            }
            "js_eval_call" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let bytes = self.read_c_string(ptr)?;
                let result = host_js_eval_call(&bytes)?;
                Node::Ptr(self.alloc_c_string_bytes(&result)?)
            }
            "js_set_haskellCallback" => {
                let callback = self.eval_int(args[0])?;
                host_js_set_haskell_callback(callback as i32)?;
                Node::prim("I")
            }
            "new_mpz" => self.new_mpz_node()?,
            "mpz_init_set_si" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int(args[1])?;
                self.write_mpz_value(ptr, MpzValue::from_i64(value))?;
                Node::prim("I")
            }
            "mpz_init_set_ui" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int(args[1])? as u64;
                self.write_mpz_value(ptr, MpzValue::from_u64(value))?;
                Node::prim("I")
            }
            "mpz_init_set_si64" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int64(args[1])?;
                self.write_mpz_value(ptr, MpzValue::from_i64(value))?;
                Node::prim("I")
            }
            "mpz_init_set_ui64" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int64(args[1])? as u64;
                self.write_mpz_value(ptr, MpzValue::from_u64(value))?;
                Node::prim("I")
            }
            "mpz_get_si" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int(self.mpz_value(ptr)?.to_i64_wrapping())
            }
            "mpz_get_si64" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int64(self.mpz_value(ptr)?.to_i64_wrapping())
            }
            "mpz_get_f" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Float32(self.mpz_value(ptr)?.to_f64() as f32)
            }
            "mpz_get_d" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Float64(self.mpz_value(ptr)?.to_f64())
            }
            "mpz_abs" => {
                let dst = self.eval_pointer_value(args[0])?;
                let src = self.eval_pointer_value(args[1])?;
                let mut value = self.mpz_value(src)?;
                value.negative = false;
                self.write_mpz_value(dst, value)?;
                Node::prim("I")
            }
            "mpz_neg" => {
                let dst = self.eval_pointer_value(args[0])?;
                let src = self.eval_pointer_value(args[1])?;
                let mut value = self.mpz_value(src)?;
                if !value.is_zero() {
                    value.negative = !value.negative;
                }
                self.write_mpz_value(dst, value)?;
                Node::prim("I")
            }
            "mpz_add" | "mpz_sub" | "mpz_mul" | "mpz_and" | "mpz_ior" | "mpz_xor" => {
                let dst = self.eval_pointer_value(args[0])?;
                let left_ptr = self.eval_pointer_value(args[1])?;
                let right_ptr = self.eval_pointer_value(args[2])?;
                let left = self.mpz_value(left_ptr)?;
                let right = self.mpz_value(right_ptr)?;
                let value = match name {
                    "mpz_add" => left.add(&right),
                    "mpz_sub" => left.sub(&right),
                    "mpz_mul" => left.mul(&right),
                    "mpz_and" => left.bitand(&right),
                    "mpz_ior" => left.bitor(&right),
                    "mpz_xor" => left.bitxor(&right),
                    _ => unreachable!("checked mpz binary op"),
                };
                self.write_mpz_value(dst, value)?;
                Node::prim("I")
            }
            "mpz_cmp" => {
                let left_ptr = self.eval_pointer_value(args[0])?;
                let right_ptr = self.eval_pointer_value(args[1])?;
                let left = self.mpz_value(left_ptr)?;
                let right = self.mpz_value(right_ptr)?;
                Node::Int(match left.cmp(&right) {
                    Ordering::Less => -1,
                    Ordering::Equal => 0,
                    Ordering::Greater => 1,
                })
            }
            "mpz_mul_2exp" => {
                let dst = self.eval_pointer_value(args[0])?;
                let src = self.eval_pointer_value(args[1])?;
                let shift = int_to_usize(self.eval_int(args[2])?)?;
                self.write_mpz_value(dst, self.mpz_value(src)?.shl_bits(shift))?;
                Node::prim("I")
            }
            "mpz_fdiv_q_2exp" => {
                let dst = self.eval_pointer_value(args[0])?;
                let src = self.eval_pointer_value(args[1])?;
                let shift = int_to_usize(self.eval_int(args[2])?)?;
                self.write_mpz_value(dst, self.mpz_value(src)?.fdiv_q_2exp(shift))?;
                Node::prim("I")
            }
            "mpz_tdiv_qr" => {
                let q_ptr = self.eval_pointer_value(args[0])?;
                let r_ptr = self.eval_pointer_value(args[1])?;
                let left_ptr = self.eval_pointer_value(args[2])?;
                let right_ptr = self.eval_pointer_value(args[3])?;
                let left = self.mpz_value(left_ptr)?;
                let right = self.mpz_value(right_ptr)?;
                let (quot, rem) = left.tdiv_qr(&right)?;
                self.write_mpz_value(q_ptr, quot)?;
                self.write_mpz_value(r_ptr, rem)?;
                Node::prim("I")
            }
            "mpz_popcount" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int(self.mpz_value(ptr)?.signed_popcount()?)
            }
            "mpz_tstbit" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let bit = int_to_usize(self.eval_int(args[1])?)?;
                Node::Int(i64::from(self.mpz_value(ptr)?.test_bit_signed(bit)))
            }
            "mpz_log2" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int(self.mpz_value(ptr)?.log2()?)
            }
            "&closeb" => Node::fun_ptr("closeb"),
            "&free" => Node::fun_ptr("free"),
            "&errno" | "errno" => Node::Ptr(self.errno_ptr()?),
            "malloc" => {
                let size = int_to_usize(self.eval_int(args[0])?)?;
                Node::Ptr(self.alloc_memory(size)?)
            }
            "calloc" => {
                let count = int_to_usize(self.eval_int(args[0])?)?;
                let size = int_to_usize(self.eval_int(args[1])?)?;
                Node::Ptr(self.calloc_memory(count, size)?)
            }
            "realloc" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let size = int_to_usize(self.eval_int(args[1])?)?;
                Node::Ptr(self.realloc_memory(ptr, size)?)
            }
            "free" => {
                let ptr = self.eval_pointer_value(args[0])?;
                self.free_memory(ptr)?;
                Node::prim("I")
            }
            "memcpy" | "memmove" => {
                let dst = self.eval_pointer_value(args[0])?;
                let src = self.eval_pointer_value(args[1])?;
                let len = int_to_usize(self.eval_int(args[2])?)?;
                let bytes = self.read_pointer_bytes(src, len)?;
                self.write_pointer_bytes(dst, &bytes)?;
                Node::prim("I")
            }
            "strcpy" => {
                let dst = self.eval_pointer_value(args[0])?;
                let src = self.eval_pointer_value(args[1])?;
                let mut bytes = self.read_c_string(src)?;
                bytes.push(0);
                self.write_pointer_bytes(dst, &bytes)?;
                Node::prim("I")
            }
            "strlen" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let len = self.c_string_len(ptr)?;
                Node::Int(i64::try_from(len).map_err(|_| EvalError::Overflow)?)
            }
            "putchar" => {
                let byte = self.eval_int(args[0])?;
                self.write_io_handle_bytes(StdHandle::Stdout, &[byte as u8])?;
                Node::prim("I")
            }
            "md5String" => {
                let input = self.eval_pointer_value(args[0])?;
                let result = self.eval_pointer_value(args[1])?;
                let bytes = self.read_c_string(input)?;
                self.write_pointer_bytes(result, &md5_bytes(&bytes))?;
                Node::prim("I")
            }
            "md5Array" => {
                let input = self.eval_pointer_value(args[0])?;
                let result = self.eval_pointer_value(args[1])?;
                let len = int_to_usize(self.eval_int(args[2])?)?;
                let bytes = self.read_pointer_bytes(input, len)?;
                self.write_pointer_bytes(result, &md5_bytes(&bytes))?;
                Node::prim("I")
            }
            "md5BFILE" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let result = self.eval_pointer_value(args[1])?;
                let mut ctx = Md5Context::new();
                loop {
                    let bytes = self.read_bfile_bytes(ptr, 1024)?;
                    if bytes.is_empty() {
                        break;
                    }
                    ctx.update(&bytes);
                }
                self.write_pointer_bytes(result, &ctx.finalize())?;
                Node::prim("I")
            }
            "getenv" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let name = self.read_c_string(ptr)?;
                let ptr = if let Some(mut bytes) = getenv_bytes(&name) {
                    bytes.push(0);
                    let ptr = self.alloc_memory(bytes.len())?;
                    self.write_pointer_bytes(ptr, &bytes)?;
                    ptr
                } else {
                    0
                };
                Node::Ptr(ptr)
            }
            "setenv" => {
                let name_ptr = self.eval_pointer_value(args[0])?;
                let value_ptr = self.eval_pointer_value(args[1])?;
                let overwrite = self.eval_int(args[2])?;
                let name = self.read_c_string(name_ptr)?;
                let value = self.read_c_string(value_ptr)?;
                self.host_int_node(setenv_bytes(&name, &value, overwrite))?
            }
            "unsetenv" => {
                let name_ptr = self.eval_pointer_value(args[0])?;
                let name = self.read_c_string(name_ptr)?;
                self.host_int_node(unsetenv_bytes(&name))?
            }
            "environ" => Node::Ptr(self.alloc_environ()?),
            "strerror_r" => {
                let errno = int_to_i32(self.eval_int(args[0])?)?;
                let ptr = self.eval_pointer_value(args[1])?;
                let size = int_to_usize(self.eval_int(args[2])?)?;
                Node::Int(self.write_strerror(errno, ptr, size)?)
            }
            "remove" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let path = self.read_c_string(ptr)?;
                self.host_int_node(remove_path_bytes(&path))?
            }
            "system" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let command = if ptr == 0 {
                    None
                } else {
                    Some(self.read_c_string(ptr)?)
                };
                Node::Int(system_command_bytes(command.as_deref()))
            }
            "chdir" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let path = self.read_c_string(ptr)?;
                self.host_int_node(chdir_path_bytes(&path))?
            }
            "mkdir" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let path = self.read_c_string(ptr)?;
                let mode = self.eval_int(args[1])?;
                self.host_int_node(mkdir_path_bytes(&path, mode))?
            }
            "getcwd" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let size = int_to_usize(self.eval_int(args[1])?)?;
                match current_dir_bytes() {
                    Ok(mut bytes) => {
                        bytes.push(0);
                        if bytes.len() <= size {
                            self.write_pointer_bytes(ptr, &bytes)?;
                            Node::Ptr(ptr)
                        } else {
                            self.set_errno_value(errno_i32("ERANGE"))?;
                            Node::Ptr(0)
                        }
                    }
                    Err(errno) => {
                        self.set_errno_value(errno)?;
                        Node::Ptr(0)
                    }
                }
            }
            "get_permissions" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let path = self.read_c_string(ptr)?;
                self.host_int_node(get_permissions_path_bytes(&path))?
            }
            "set_permissions" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let path = self.read_c_string(ptr)?;
                let permissions = self.eval_int(args[1])?;
                self.host_int_node(set_permissions_path_bytes(&path, permissions))?
            }
            "get_executable_path" => {
                let path = self
                    .executable_path
                    .clone()
                    .map(Ok)
                    .unwrap_or_else(executable_path_bytes);
                let ptr = match path {
                    Ok(mut bytes) => {
                        bytes.push(0);
                        let ptr = self.alloc_memory(bytes.len())?;
                        self.write_pointer_bytes(ptr, &bytes)?;
                        ptr
                    }
                    Err(errno) => {
                        self.set_errno_value(errno)?;
                        0
                    }
                };
                Node::Ptr(ptr)
            }
            "tmpname" => {
                let pre_ptr = self.eval_pointer_value(args[0])?;
                let suf_ptr = self.eval_pointer_value(args[1])?;
                let pre = self.read_c_string(pre_ptr)?;
                let suf = self.read_c_string(suf_ptr)?;
                match tmpname_bytes(&pre, &suf) {
                    Ok(mut bytes) => {
                        bytes.push(0);
                        let ptr = self.alloc_memory(bytes.len())?;
                        self.write_pointer_bytes(ptr, &bytes)?;
                        Node::Ptr(ptr)
                    }
                    Err(errno) => {
                        self.set_errno_value(errno)?;
                        Node::Ptr(0)
                    }
                }
            }
            "opendir" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let path = self.read_c_string(ptr)?;
                match dir_entries_path_bytes(&path) {
                    Ok(entries) => Node::Ptr(self.alloc_dir(entries)?),
                    Err(errno) => {
                        self.set_errno_value(errno)?;
                        Node::Ptr(0)
                    }
                }
            }
            "readdir" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Ptr(self.read_dir_entry(ptr)?)
            }
            "closedir" => {
                let ptr = self.eval_pointer_value(args[0])?;
                if self.close_dir(ptr).is_ok() {
                    Node::Int(0)
                } else {
                    self.set_errno_value(errno_i32("EBADF"))?;
                    Node::Int(-1)
                }
            }
            "c_d_name" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Ptr(ptr)
            }
            "fopen" => {
                let path_ptr = self.eval_pointer_value(args[0])?;
                let mode_ptr = self.eval_pointer_value(args[1])?;
                let path = self.read_c_string(path_ptr)?;
                let mode = self.read_c_string(mode_ptr)?;
                match native_fopen_bfile(&path, &mode) {
                    Ok(bfile) => Node::Ptr(self.alloc_bfile(bfile)?),
                    Err(errno) => {
                        self.set_errno_value(errno)?;
                        Node::Ptr(0)
                    }
                }
            }
            "open" => {
                let path_ptr = self.eval_pointer_value(args[0])?;
                let flags = int_to_i32(self.eval_int(args[1])?)?;
                let mode = self.eval_int(args[2])?;
                let path = self.read_c_string(path_ptr)?;
                self.host_int_node(open_fd_path_bytes(&path, flags, mode))?
            }
            "add_FILE" => {
                let ptr = self.eval_pointer_value(args[0])?;
                if ptr != 0 && handle_from_ptr(ptr).is_none() {
                    self.bfile(ptr)?;
                }
                Node::Ptr(ptr)
            }
            "add_fd" => {
                let fd = int_to_i32(self.eval_int(args[0])?)?;
                match native_fd_bfile(fd) {
                    Ok(bfile) => Node::Ptr(self.alloc_bfile(bfile)?),
                    Err(errno) => {
                        self.set_errno_value(errno)?;
                        Node::Ptr(0)
                    }
                }
            }
            "add_utf8" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Ptr(self.add_utf8_bfile(ptr)?)
            }
            "add_crlf" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Ptr(self.add_crlf_bfile(ptr)?)
            }
            "add_rle_decompressor" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Ptr(self.add_rle_bfile(ptr, true)?)
            }
            "add_rle_compressor" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Ptr(self.add_rle_bfile(ptr, false)?)
            }
            "add_base64_decoder" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Ptr(self.add_base64_bfile(ptr, true)?)
            }
            "add_base64_encoder" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Ptr(self.add_base64_bfile(ptr, false)?)
            }
            "add_lz77_decompressor" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Ptr(self.add_lz77_bfile(ptr, true)?)
            }
            "add_lz77_compressor" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Ptr(self.add_lz77_bfile(ptr, false)?)
            }
            "add_bwt_decompressor" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Ptr(self.add_bwt_bfile(ptr, true)?)
            }
            "add_bwt_compressor" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Ptr(self.add_bwt_bfile(ptr, false)?)
            }
            "add_lzma_decompressor" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Ptr(self.add_lzma_bfile(ptr, true)?)
            }
            "add_lzma_compressor" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Ptr(self.add_lzma_bfile(ptr, false)?)
            }
            "add_buf" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let size = self.eval_int(args[1])?;
                Node::Ptr(self.add_buf_bfile(ptr, size)?)
            }
            "openb_wr_mem" => Node::Ptr(self.alloc_bfile(BFile {
                kind: BFileKind::Memory {
                    bytes: Vec::new(),
                    pos: 0,
                },
                readable: false,
                writable: true,
            })?),
            "openb_rd_mem" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let len = int_to_usize(self.eval_int(args[1])?)?;
                let kind = self.memory_read_bfile_kind(ptr, len)?;
                Node::Ptr(self.alloc_bfile(BFile {
                    kind,
                    readable: true,
                    writable: false,
                })?)
            }
            "get_mem" => {
                let bfile_ptr = self.eval_pointer_value(args[0])?;
                let bufp = self.eval_pointer_value(args[1])?;
                let lenp = self.eval_pointer_value(args[2])?;
                let buffer = self.bfile_output_bytes(bfile_ptr)?;
                let len = i64::try_from(buffer.len()).map_err(|_| EvalError::Overflow)?;
                let ptr = self.alloc_memory(buffer.len())?;
                self.write_pointer_bytes(ptr, &buffer)?;
                self.poke_signed(bufp, 8, ptr)?;
                self.poke_signed(lenp, 8, len)?;
                Node::prim("I")
            }
            "getcpu" => {
                let sec_ptr = self.eval_pointer_value(args[0])?;
                let nsec_ptr = self.eval_pointer_value(args[1])?;
                let (sec, nsec) = cpu_time();
                self.poke_unsigned(sec_ptr, size_of::<std::os::raw::c_ulong>(), sec)?;
                self.poke_unsigned(nsec_ptr, size_of::<std::os::raw::c_ulong>(), nsec)?;
                Node::prim("I")
            }
            "gettimeofday" => {
                let timeval_ptr = self.eval_pointer_value(args[0])?;
                let timezone_ptr = self.eval_pointer_value(args[1])?;
                self.gettimeofday_node(timeval_ptr, timezone_ptr)?
            }
            "accept" => {
                let fd = int_to_i32(self.eval_int(args[0])?)?;
                let addr_ptr = self.eval_pointer_value(args[1])?;
                let len_ptr = self.eval_pointer_value(args[2])?;
                self.accept_socket_node(fd, addr_ptr, len_ptr)?
            }
            "bind" => {
                let fd = int_to_i32(self.eval_int(args[0])?)?;
                let addr_ptr = self.eval_pointer_value(args[1])?;
                let len = int_to_usize(self.eval_int(args[2])?)?;
                let addr = self.read_pointer_bytes(addr_ptr, len)?;
                self.host_int_node(bind_socket(fd, &addr))?
            }
            "close" => {
                let fd = int_to_i32(self.eval_int(args[0])?)?;
                self.host_int_node(close_fd(fd))?
            }
            "connect" => {
                let fd = int_to_i32(self.eval_int(args[0])?)?;
                let addr_ptr = self.eval_pointer_value(args[1])?;
                let len = int_to_usize(self.eval_int(args[2])?)?;
                let addr = self.read_pointer_bytes(addr_ptr, len)?;
                self.host_int_node(connect_socket(fd, &addr))?
            }
            "fcntl" => {
                let fd = int_to_i32(self.eval_int(args[0])?)?;
                let cmd = int_to_i32(self.eval_int(args[1])?)?;
                let arg = int_to_i32(self.eval_int(args[2])?)?;
                self.host_int_node(fcntl_fd(fd, cmd, arg))?
            }
            "getsockopt" => {
                let fd = int_to_i32(self.eval_int(args[0])?)?;
                let level = int_to_i32(self.eval_int(args[1])?)?;
                let optname = int_to_i32(self.eval_int(args[2])?)?;
                let optval_ptr = self.eval_pointer_value(args[3])?;
                let optlen_ptr = self.eval_pointer_value(args[4])?;
                self.getsockopt_node(fd, level, optname, optval_ptr, optlen_ptr)?
            }
            "listen" => {
                let fd = int_to_i32(self.eval_int(args[0])?)?;
                let backlog = int_to_i32(self.eval_int(args[1])?)?;
                self.host_int_node(listen_socket(fd, backlog))?
            }
            "recv" => {
                let fd = int_to_i32(self.eval_int(args[0])?)?;
                let buf_ptr = self.eval_pointer_value(args[1])?;
                let len = int_to_usize(self.eval_int(args[2])?)?;
                let flags = int_to_i32(self.eval_int(args[3])?)?;
                self.recv_socket_node(fd, buf_ptr, len, flags)?
            }
            "send" => {
                let fd = int_to_i32(self.eval_int(args[0])?)?;
                let buf_ptr = self.eval_pointer_value(args[1])?;
                let len = int_to_usize(self.eval_int(args[2])?)?;
                let flags = int_to_i32(self.eval_int(args[3])?)?;
                self.send_socket_node(fd, buf_ptr, len, flags)?
            }
            "setsockopt" => {
                let fd = int_to_i32(self.eval_int(args[0])?)?;
                let level = int_to_i32(self.eval_int(args[1])?)?;
                let optname = int_to_i32(self.eval_int(args[2])?)?;
                let optval_ptr = self.eval_pointer_value(args[3])?;
                let optlen = int_to_usize(self.eval_int(args[4])?)?;
                let optval = self.read_pointer_bytes(optval_ptr, optlen)?;
                self.host_int_node(setsockopt_socket(fd, level, optname, &optval))?
            }
            "socket" => {
                let domain = int_to_i32(self.eval_int(args[0])?)?;
                let typ = int_to_i32(self.eval_int(args[1])?)?;
                let protocol = int_to_i32(self.eval_int(args[2])?)?;
                self.host_int_node(socket_fd(domain, typ, protocol))?
            }
            "closeb" => {
                let ptr = self.eval_pointer_value(args[0])?;
                self.close_bfile(ptr)?;
                Node::prim("I")
            }
            "flushb" => {
                let ptr = self.eval_pointer_value(args[0])?;
                self.flush_bfile(ptr)?;
                Node::prim("I")
            }
            "getb" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int(self.get_bfile_byte(ptr)?)
            }
            "putb" => {
                let byte = self.eval_int(args[0])?;
                let ptr = self.eval_pointer_value(args[1])?;
                self.put_bfile_byte(ptr, byte)?;
                Node::prim("I")
            }
            "ungetb" => {
                let byte = self.eval_int(args[0])?;
                let ptr = self.eval_pointer_value(args[1])?;
                self.unget_bfile_byte(ptr, byte)?;
                Node::prim("I")
            }
            "readb" => {
                let dst = self.eval_pointer_value(args[0])?;
                let len = int_to_usize(self.eval_int(args[1])?)?;
                let ptr = self.eval_pointer_value(args[2])?;
                Node::Int(
                    i64::try_from(self.read_bfile(ptr, dst, len)?)
                        .map_err(|_| EvalError::Overflow)?,
                )
            }
            "writeb" => {
                let src = self.eval_pointer_value(args[0])?;
                let len = int_to_usize(self.eval_int(args[1])?)?;
                let ptr = self.eval_pointer_value(args[2])?;
                Node::Int(
                    i64::try_from(self.write_bfile(ptr, src, len)?)
                        .map_err(|_| EvalError::Overflow)?,
                )
            }
            "lz77c" => {
                let src = self.eval_pointer_value(args[0])?;
                let len = int_to_usize(self.eval_int(args[1])?)?;
                let out_ptr = self.eval_pointer_value(args[2])?;
                let bytes = self.read_pointer_bytes(src, len)?;
                let compressed = lz77_compress(&bytes)?;
                let compressed_ptr = self.alloc_memory(compressed.len())?;
                self.write_pointer_bytes(compressed_ptr, &compressed)?;
                self.poke_signed(out_ptr, 8, compressed_ptr)?;
                Node::Int(i64::try_from(compressed.len()).map_err(|_| EvalError::Overflow)?)
            }
            "peekPtr" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Ptr(self.peek_signed(ptr, 8)?)
            }
            "pokePtr" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_pointer_value(args[1])?;
                self.poke_signed(ptr, 8, value)?;
                Node::prim("I")
            }
            "peekWord" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let result = self.int(self.peek_unsigned(ptr, 8)? as i64);
                return Ok(Some((2, self.pair(result, args[1]))));
            }
            "pokeWord" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int(args[1])?;
                self.poke_unsigned(ptr, 8, value as u64)?;
                return Ok(Some((3, self.unit_pair(args[2]))));
            }
            "peek_uint8" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int(self.peek_unsigned(ptr, 1)? as i64)
            }
            "poke_uint8" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int(args[1])?;
                self.poke_unsigned(ptr, 1, value as u64)?;
                Node::prim("I")
            }
            "peek_uint16" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int(self.peek_unsigned(ptr, 2)? as i64)
            }
            "poke_uint16" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int(args[1])?;
                self.poke_unsigned(ptr, 2, value as u64)?;
                Node::prim("I")
            }
            "peek_uint32" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int(self.peek_unsigned(ptr, 4)? as i64)
            }
            "poke_uint32" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int(args[1])?;
                self.poke_unsigned(ptr, 4, value as u64)?;
                Node::prim("I")
            }
            "peek_uint64" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let result = self.push_node(Node::Int64(self.peek_unsigned(ptr, 8)? as i64));
                return Ok(Some((2, self.pair(result, args[1]))));
            }
            "poke_uint64" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int64(args[1])?;
                self.poke_unsigned(ptr, 8, value as u64)?;
                return Ok(Some((3, self.unit_pair(args[2]))));
            }
            "peek_int8" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int(self.peek_signed(ptr, 1)?)
            }
            "poke_int8" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int(args[1])?;
                self.poke_signed(ptr, 1, value)?;
                Node::prim("I")
            }
            "peek_int16" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int(self.peek_signed(ptr, 2)?)
            }
            "poke_int16" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int(args[1])?;
                self.poke_signed(ptr, 2, value)?;
                Node::prim("I")
            }
            "peek_int32" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int(self.peek_signed(ptr, 4)?)
            }
            "poke_int32" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int(args[1])?;
                self.poke_signed(ptr, 4, value)?;
                Node::prim("I")
            }
            "peek_int64" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int64(self.peek_signed(ptr, 8)?)
            }
            "poke_int64" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int64(args[1])?;
                self.poke_signed(ptr, 8, value)?;
                Node::prim("I")
            }
            "peek_char" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int(self.peek_c_char(ptr)?)
            }
            "poke_char" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int(args[1])?;
                self.poke_c_char(ptr, value)?;
                Node::prim("I")
            }
            "peek_schar" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int(self.peek_signed(ptr, size_of::<std::os::raw::c_schar>())?)
            }
            "poke_schar" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int(args[1])?;
                self.poke_signed(ptr, size_of::<std::os::raw::c_schar>(), value)?;
                Node::prim("I")
            }
            "peek_uchar" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int(self.peek_unsigned(ptr, size_of::<std::os::raw::c_uchar>())? as i64)
            }
            "poke_uchar" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int(args[1])?;
                self.poke_unsigned(ptr, size_of::<std::os::raw::c_uchar>(), value as u64)?;
                Node::prim("I")
            }
            "peek_short" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int(self.peek_signed(ptr, size_of::<std::os::raw::c_short>())?)
            }
            "poke_short" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int(args[1])?;
                self.poke_signed(ptr, size_of::<std::os::raw::c_short>(), value)?;
                Node::prim("I")
            }
            "peek_ushort" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int(self.peek_unsigned(ptr, size_of::<std::os::raw::c_ushort>())? as i64)
            }
            "poke_ushort" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int(args[1])?;
                self.poke_unsigned(ptr, size_of::<std::os::raw::c_ushort>(), value as u64)?;
                Node::prim("I")
            }
            "peek_int" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int(self.peek_signed(ptr, size_of::<std::os::raw::c_int>())?)
            }
            "poke_int" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int(args[1])?;
                self.poke_signed(ptr, size_of::<std::os::raw::c_int>(), value)?;
                Node::prim("I")
            }
            "peek_uint" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int(self.peek_unsigned(ptr, size_of::<std::os::raw::c_uint>())? as i64)
            }
            "poke_uint" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int(args[1])?;
                self.poke_unsigned(ptr, size_of::<std::os::raw::c_uint>(), value as u64)?;
                Node::prim("I")
            }
            "peek_long" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int(self.peek_signed(ptr, size_of::<std::os::raw::c_long>())?)
            }
            "poke_long" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int(args[1])?;
                self.poke_signed(ptr, size_of::<std::os::raw::c_long>(), value)?;
                Node::prim("I")
            }
            "peek_ulong" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int(self.peek_unsigned(ptr, size_of::<std::os::raw::c_ulong>())? as i64)
            }
            "poke_ulong" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int(args[1])?;
                self.poke_unsigned(ptr, size_of::<std::os::raw::c_ulong>(), value as u64)?;
                Node::prim("I")
            }
            "peek_llong" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int(self.peek_signed(ptr, size_of::<std::os::raw::c_longlong>())?)
            }
            "poke_llong" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int(args[1])?;
                self.poke_signed(ptr, size_of::<std::os::raw::c_longlong>(), value)?;
                Node::prim("I")
            }
            "peek_ullong" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int(self.peek_unsigned(ptr, size_of::<std::os::raw::c_ulonglong>())? as i64)
            }
            "poke_ullong" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int(args[1])?;
                self.poke_unsigned(ptr, size_of::<std::os::raw::c_ulonglong>(), value as u64)?;
                Node::prim("I")
            }
            "peek_size_t" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Int(self.peek_unsigned(ptr, size_of::<usize>())? as i64)
            }
            "poke_size_t" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_int(args[1])?;
                self.poke_unsigned(ptr, size_of::<usize>(), value as u64)?;
                Node::prim("I")
            }
            "peek_flt32" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Float32(f32::from_ne_bytes(self.peek_array(ptr)?))
            }
            "poke_flt32" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_float32(args[1])?;
                self.write_pointer_bytes(ptr, &value.to_ne_bytes())?;
                Node::prim("I")
            }
            "peek_flt64" => {
                let ptr = self.eval_pointer_value(args[0])?;
                Node::Float64(f64::from_ne_bytes(self.peek_array(ptr)?))
            }
            "poke_flt64" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let value = self.eval_float64(args[1])?;
                self.write_pointer_bytes(ptr, &value.to_ne_bytes())?;
                Node::prim("I")
            }
            "acos" => Node::Float64(self.eval_float64(args[0])?.acos()),
            "asin" => Node::Float64(self.eval_float64(args[0])?.asin()),
            "atan" => Node::Float64(self.eval_float64(args[0])?.atan()),
            "cos" => Node::Float64(self.eval_float64(args[0])?.cos()),
            "exp" => Node::Float64(self.eval_float64(args[0])?.exp()),
            "log" => Node::Float64(self.eval_float64(args[0])?.ln()),
            "sin" => Node::Float64(self.eval_float64(args[0])?.sin()),
            "sqrt" => Node::Float64(self.eval_float64(args[0])?.sqrt()),
            "tan" => Node::Float64(self.eval_float64(args[0])?.tan()),
            "atan2" => {
                let x = self.eval_float64(args[0])?;
                let y = self.eval_float64(args[1])?;
                Node::Float64(x.atan2(y))
            }
            "pow" => {
                let x = self.eval_float64(args[0])?;
                let y = self.eval_float64(args[1])?;
                Node::Float64(x.powf(y))
            }
            "scalbn" => {
                let x = self.eval_float64(args[0])?;
                let n = int_to_i32(self.eval_int(args[1])?)?;
                Node::Float64(x * 2.0f64.powi(n))
            }
            "acosf" => Node::Float32(self.eval_float32(args[0])?.acos()),
            "asinf" => Node::Float32(self.eval_float32(args[0])?.asin()),
            "atanf" => Node::Float32(self.eval_float32(args[0])?.atan()),
            "cosf" => Node::Float32(self.eval_float32(args[0])?.cos()),
            "expf" => Node::Float32(self.eval_float32(args[0])?.exp()),
            "logf" => Node::Float32(self.eval_float32(args[0])?.ln()),
            "sinf" => Node::Float32(self.eval_float32(args[0])?.sin()),
            "sqrtf" => Node::Float32(self.eval_float32(args[0])?.sqrt()),
            "tanf" => Node::Float32(self.eval_float32(args[0])?.tan()),
            "atan2f" => {
                let x = self.eval_float32(args[0])?;
                let y = self.eval_float32(args[1])?;
                Node::Float32(x.atan2(y))
            }
            "powf" => {
                let x = self.eval_float32(args[0])?;
                let y = self.eval_float32(args[1])?;
                Node::Float32(x.powf(y))
            }
            "scalbnf" => {
                let x = self.eval_float32(args[0])?;
                let n = int_to_i32(self.eval_int(args[1])?)?;
                Node::Float32(x * 2.0f32.powi(n))
            }
            _ => unreachable!("checked FFI symbol"),
        };
        let result = self.push_value_node(result);
        Ok(Some((arity + 1, self.pair(result, args[arity]))))
    }

    fn zero_arity_ffi_result(&mut self, name: &str) -> Result<Option<Node>, EvalError> {
        if let Some(value) = errno_constant(name) {
            return Ok(Some(Node::Int(value)));
        }
        if let Some(value) = host_constant(name) {
            return Ok(Some(Node::Int(value)));
        }
        let result = match name {
            "GETRAW" => Node::Int(-1),
            "GETTIMEMICRO" => Node::Int(current_time_micro()),
            "islinux" => Node::Int(i64::from(cfg!(target_os = "linux"))),
            "ismacos" => Node::Int(i64::from(cfg!(target_os = "macos"))),
            "iswindows" => Node::Int(i64::from(cfg!(target_os = "windows"))),
            "sizeof_char" => Node::Int(size_of_i64::<std::os::raw::c_char>()),
            "sizeof_short" => Node::Int(size_of_i64::<std::os::raw::c_short>()),
            "sizeof_int" => Node::Int(size_of_i64::<std::os::raw::c_int>()),
            "sizeof_long" => Node::Int(size_of_i64::<std::os::raw::c_long>()),
            "sizeof_llong" => Node::Int(size_of_i64::<std::os::raw::c_longlong>()),
            "sizeof_size_t" => Node::Int(size_of_i64::<usize>()),
            "want_gmp" => Node::Int(0),
            "want_imath" => Node::Int(1),
            "&closeb" => Node::fun_ptr("closeb"),
            "&free" => Node::fun_ptr("free"),
            "&errno" | "errno" => Node::Ptr(self.errno_ptr()?),
            _ => return Ok(None),
        };
        Ok(Some(result))
    }

    fn unary_math_ffi_result(
        &mut self,
        name: &str,
        arg: NodeId,
    ) -> Result<Option<Node>, EvalError> {
        let result = match name {
            "acos" => Node::Float64(self.eval_float64(arg)?.acos()),
            "asin" => Node::Float64(self.eval_float64(arg)?.asin()),
            "atan" => Node::Float64(self.eval_float64(arg)?.atan()),
            "cos" => Node::Float64(self.eval_float64(arg)?.cos()),
            "exp" => Node::Float64(self.eval_float64(arg)?.exp()),
            "log" => Node::Float64(self.eval_float64(arg)?.ln()),
            "sin" => Node::Float64(self.eval_float64(arg)?.sin()),
            "sqrt" => Node::Float64(self.eval_float64(arg)?.sqrt()),
            "tan" => Node::Float64(self.eval_float64(arg)?.tan()),
            "acosf" => Node::Float32(self.eval_float32(arg)?.acos()),
            "asinf" => Node::Float32(self.eval_float32(arg)?.asin()),
            "atanf" => Node::Float32(self.eval_float32(arg)?.atan()),
            "cosf" => Node::Float32(self.eval_float32(arg)?.cos()),
            "expf" => Node::Float32(self.eval_float32(arg)?.exp()),
            "logf" => Node::Float32(self.eval_float32(arg)?.ln()),
            "sinf" => Node::Float32(self.eval_float32(arg)?.sin()),
            "sqrtf" => Node::Float32(self.eval_float32(arg)?.sqrt()),
            "tanf" => Node::Float32(self.eval_float32(arg)?.tan()),
            _ => return Ok(None),
        };
        Ok(Some(result))
    }

    fn js_call(
        &mut self,
        tags: &str,
        body: &[u8],
        args: &[NodeId],
    ) -> Result<Option<(usize, NodeId)>, EvalError> {
        let tags = tags.as_bytes();
        validate_js_tags(tags)?;
        let arity = tags.len() - 1;
        if args.len() < arity + 1 {
            return Ok(None);
        }
        let mut js_args = Vec::with_capacity(arity);
        for (idx, tag) in tags[1..].iter().copied().enumerate() {
            let arg = match tag {
                b'D' => JsArg::Double(self.eval_float64(args[idx])?),
                b'F' => JsArg::Double(f64::from(self.eval_float32(args[idx])?)),
                b'B' => JsArg::Int(i32::from(self.eval_bool(args[idx])?)),
                b'P' => JsArg::UInt(self.eval_pointer_value(args[idx])? as u32),
                b'J' => JsArg::Object(self.eval_js_object_handle(args[idx])?),
                b'S' => JsArg::String(self.eval_bytes(args[idx])?),
                b'U' => JsArg::UInt(self.eval_int(args[idx])? as u32),
                b'I' => JsArg::Int(self.eval_int(args[idx])? as i32),
                _ => return Err(EvalError::InvalidByteString),
            };
            js_args.push(arg);
        }
        let result = match tags[0] {
            b'V' => {
                host_js_call_void(body, arity, &js_args)?;
                Node::prim("I")
            }
            b'D' => Node::Float64(host_js_call_double(body, arity, &js_args)?),
            b'F' => Node::Float32(host_js_call_double(body, arity, &js_args)? as f32),
            b'P' => Node::Ptr(i64::from(host_js_call_ptr(body, arity, &js_args)?)),
            b'B' => Node::prim(if host_js_call_bool(body, arity, &js_args)? {
                "A"
            } else {
                "K"
            }),
            b'S' => Node::bytes(host_js_call_string(body, arity, &js_args)?),
            b'I' => Node::Int(i64::from(host_js_call_int(body, arity, &js_args)?)),
            b'U' => Node::Int(i64::from(host_js_call_uint(body, arity, &js_args)?)),
            b'J' => self.js_object_node(host_js_call_object(body, arity, &js_args)?),
            _ => return Err(EvalError::InvalidByteString),
        };
        let result = self.push_node(result);
        Ok(Some((arity + 1, self.pair(result, args[arity]))))
    }

    fn js_wrap(
        &mut self,
        tags: &str,
        args: &[NodeId],
    ) -> Result<Option<(usize, NodeId)>, EvalError> {
        validate_js_tags(tags.as_bytes())?;
        if args.len() < 2 {
            return Ok(None);
        }
        let program_handle = self.js_program_handle.ok_or(EvalError::UnsupportedJsFfi)?;
        let wrapper_index = self.register_js_wrapper_tags(tags)?;
        let stable_ptr = self.new_stable_ptr_handle(args[0])?;
        let object = match host_js_make_wrapper(program_handle, stable_ptr, wrapper_index) {
            Ok(object) => object,
            Err(err) => {
                let _ = self.free_stable_ptr(usize::try_from(stable_ptr).unwrap_or(usize::MAX));
                return Err(err);
            }
        };
        let result_node = self.js_object_node(object);
        let result = self.push_node(result_node);
        Ok(Some((2, self.pair(result, args[1]))))
    }

    fn register_js_wrapper_tags(&mut self, tags: &str) -> Result<u32, EvalError> {
        let index = u32::try_from(self.js_wrapper_tags.len()).map_err(|_| EvalError::Overflow)?;
        self.js_wrapper_tags.push(tags.to_owned());
        Ok(index)
    }

    fn js_value_node(&mut self, tag: u8, value: &JsValue) -> Result<NodeId, EvalError> {
        let node = match (tag, value) {
            (b'I', JsValue::Int(value)) => Node::Int(i64::from(*value)),
            (b'U', JsValue::UInt(value)) => Node::Int(i64::from(*value)),
            (b'D', JsValue::Double(value)) => Node::Float64(*value),
            (b'F', JsValue::Float(value)) => Node::Float32(*value),
            (b'B', JsValue::Bool(value)) => return Ok(self.prim(if *value { "A" } else { "K" })),
            (b'P', JsValue::Pointer(value)) => Node::Ptr(i64::from(*value)),
            (b'J', JsValue::Object(value)) => self.js_object_node(*value),
            (b'S', JsValue::Bytes(value)) => Node::bytes(value.clone()),
            _ => return Err(EvalError::InvalidByteString),
        };
        Ok(self.push_value_node(node))
    }

    fn js_value_from_node(&mut self, tag: u8, id: NodeId) -> Result<JsValue, EvalError> {
        match tag {
            b'V' => {
                let _ = self.reduce_node_whnf(id, FORCE_REDUCTION_LIMIT)?;
                Ok(JsValue::Unit)
            }
            b'I' => Ok(JsValue::Int(int_to_i32(self.eval_int(id)?)?)),
            b'U' => Ok(JsValue::UInt(
                u32::try_from(self.eval_int(id)?).map_err(|_| EvalError::Overflow)?,
            )),
            b'D' => Ok(JsValue::Double(self.eval_float64(id)?)),
            b'F' => Ok(JsValue::Float(self.eval_float32(id)?)),
            b'B' => Ok(JsValue::Bool(self.eval_bool(id)?)),
            b'P' => Ok(JsValue::Pointer(
                u32::try_from(self.eval_pointer_value(id)?).map_err(|_| EvalError::Overflow)?,
            )),
            b'J' => Ok(JsValue::Object(self.eval_js_object_handle(id)?)),
            b'S' => Ok(JsValue::Bytes(self.eval_bytes(id)?)),
            _ => Err(EvalError::InvalidByteString),
        }
    }

}
