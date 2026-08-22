//! JavaScript FFI runtime primitive dispatch.
use super::*;

impl Program {
    #[cold]
    pub(in crate::runtime) fn js_call(
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
                b'B' => JsArg::Bool(self.eval_bool(args[idx])?),
                b'P' => JsArg::Pointer(self.eval_pointer_value(args[idx])?),
                b'J' => JsArg::Object(self.eval_js_object_handle(args[idx])?),
                b'S' => JsArg::String(self.eval_bytes(args[idx])?),
                b'U' => JsArg::UInt(
                    u32::try_from(self.eval_int(args[idx])?).map_err(|_| EvalError::Overflow)?,
                ),
                b'I' => JsArg::Int(int_to_i32(self.eval_int(args[idx])?)?),
                _ => return Err(EvalError::InvalidByteString),
            };
            js_args.push(arg);
        }
        let program_handle = self.js_program_handle.ok_or(EvalError::UnsupportedJsFfi)?;
        let result = match tags[0] {
            b'V' => {
                host_js_call_void(program_handle, body, arity, &js_args)?;
                Node::prim("I")
            }
            b'D' => Node::Float64(host_js_call_double(program_handle, body, arity, &js_args)?),
            b'F' => {
                Node::Float32(host_js_call_double(program_handle, body, arity, &js_args)? as f32)
            }
            b'P' => Node::Ptr(host_js_call_ptr(program_handle, body, arity, &js_args)?),
            b'B' => Node::prim(
                if host_js_call_bool(program_handle, body, arity, &js_args)? {
                    "A"
                } else {
                    "K"
                },
            ),
            b'S' => Node::bytes(host_js_call_string(program_handle, body, arity, &js_args)?),
            b'I' => Node::Int(i64::from(host_js_call_int(
                program_handle,
                body,
                arity,
                &js_args,
            )?)),
            b'U' => Node::Int(i64::from(host_js_call_uint(
                program_handle,
                body,
                arity,
                &js_args,
            )?)),
            b'J' => {
                self.js_object_node(host_js_call_object(program_handle, body, arity, &js_args)?)
            }
            _ => return Err(EvalError::InvalidByteString),
        };
        let result = self.push_node(result);
        Ok(Some((arity + 1, self.pair(result, args[arity]))))
    }

    #[cold]
    pub(in crate::runtime) fn js_wrap(
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

    pub(in crate::runtime) fn register_js_wrapper_tags(
        &mut self,
        tags: &str,
    ) -> Result<u32, EvalError> {
        let index = u32::try_from(self.js_wrapper_tags.len()).map_err(|_| EvalError::Overflow)?;
        self.js_wrapper_tags.push(tags.to_owned());
        Ok(index)
    }

    #[cold]
    pub(crate) fn register_js_exports(
        &mut self,
        exports: Vec<JsExportDecl>,
    ) -> Result<(), EvalError> {
        std::hint::cold_path();
        for export in exports {
            validate_js_tags(export.tags.as_bytes())?;
            let stable_ptr = usize::try_from(self.new_stable_ptr_handle(export.closure)?)
                .map_err(|_| EvalError::Overflow)?;
            let wrapper_index = self.register_js_wrapper_tags(&export.tags)?;
            self.js_exports.push(JsExport {
                name: export.name,
                stable_ptr,
                wrapper_index,
                is_io: export.is_io,
            });
        }
        Ok(())
    }

    #[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
    pub(in crate::runtime) fn js_value_node(
        &mut self,
        tag: u8,
        value: &JsValue,
    ) -> Result<NodeId, EvalError> {
        let node = match (tag, value) {
            (b'I', JsValue::Int(value)) => Node::Int(i64::from(*value)),
            (b'U', JsValue::UInt(value)) => Node::Int(i64::from(*value)),
            (b'D', JsValue::Double(value)) => Node::Float64(*value),
            (b'F', JsValue::Float(value)) => Node::Float32(*value),
            (b'B', JsValue::Bool(value)) => return Ok(self.prim(if *value { "A" } else { "K" })),
            (b'P', JsValue::Pointer(value)) => Node::Ptr(*value),
            (b'J', JsValue::Object(value)) => self.js_object_node(*value),
            (b'S', JsValue::Bytes(value)) => Node::bytes(value.clone()),
            _ => return Err(EvalError::InvalidByteString),
        };
        Ok(self.push_value_node(node))
    }

    #[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
    pub(in crate::runtime) fn js_value_from_node(
        &mut self,
        tag: u8,
        id: NodeId,
    ) -> Result<JsValue, EvalError> {
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
            b'P' => Ok(JsValue::Pointer(self.eval_pointer_value(id)?)),
            b'J' => Ok(JsValue::Object(self.eval_js_object_handle(id)?)),
            b'S' => Ok(JsValue::Bytes(self.eval_bytes(id)?)),
            _ => Err(EvalError::InvalidByteString),
        }
    }
}
