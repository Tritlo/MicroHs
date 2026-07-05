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
}
