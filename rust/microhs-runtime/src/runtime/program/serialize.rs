impl Program {
    #[cold]
    #[inline(never)]
    pub fn serialize_program(&self, root: NodeId) -> Result<Vec<u8>, EvalError> {
        let mut labels = self.find_serialization_labels(root)?;
        let mut out = b"v8.4\n".to_vec();
        push_display(&mut out, labels.shared.len());
        out.push(b'\n');
        self.serialize_comb_into(root, &mut labels, &mut out)?;
        out.extend_from_slice(b"}\n");
        Ok(out)
    }

    #[cold]
    #[inline(never)]
    fn print_program(&self, root: NodeId) -> Result<Vec<u8>, EvalError> {
        let mut labels = self.find_serialization_labels(root)?;
        let mut out = Vec::new();
        self.print_comb_into(root, &mut labels, &mut out)?;
        out.push(b'\n');
        Ok(out)
    }

    fn find_serialization_labels(&self, root: NodeId) -> Result<SerializationLabels, EvalError> {
        let mut labels = SerializationLabels::default();
        let mut marked = HashSet::new();
        let mut work = vec![root];
        while let Some(id) = work.pop() {
            let id = self.resolve(id)?;
            let node = self.node_for_debug(id);
            if !serialization_shareable_node(&node) {
                continue;
            }
            if !marked.insert(id) {
                labels.shared.insert(id);
                continue;
            }
            match node {
                Node::App(fun, arg) => {
                    work.push(arg);
                    work.push(fun);
                }
                Node::Array(items) => {
                    work.extend(items.iter().copied());
                }
                _ => {}
            }
        }
        Ok(labels)
    }

    #[cold]
    #[inline(never)]
    fn print_comb_into(
        &self,
        root: NodeId,
        labels: &mut SerializationLabels,
        out: &mut Vec<u8>,
    ) -> Result<(), EvalError> {
        enum PrintTask {
            Node(NodeId),
            Byte(u8),
        }

        let mut work = vec![PrintTask::Node(root)];
        while let Some(task) = work.pop() {
            match task {
                PrintTask::Byte(byte) => out.push(byte),
                PrintTask::Node(id) => {
                    let id = self.resolve(id)?;
                    if labels.shared.contains(&id) {
                        if !labels.printed.insert(id) {
                            out.push(b'_');
                            push_display(out, id.index());
                            continue;
                        }
                        out.push(b':');
                        push_display(out, id.index());
                        out.push(b' ');
                    }

                    match self.node_for_debug(id) {
                        Node::App(fun, arg) => {
                            out.push(b'(');
                            work.push(PrintTask::Byte(b')'));
                            work.push(PrintTask::Node(arg));
                            work.push(PrintTask::Byte(b' '));
                            work.push(PrintTask::Node(fun));
                        }
                        Node::Indir(_) | Node::Free(_) => {
                            return Err(EvalError::DanglingIndirection(id));
                        }
                        Node::Prim(name) => {
                            out.extend_from_slice(name.name().as_bytes());
                        }
                        Node::Int(n) => {
                            out.push(b'#');
                            push_display(out, n);
                        }
                        Node::Int64(n) => {
                            out.extend_from_slice(b"##");
                            push_display(out, n);
                        }
                        Node::Float64(n) => {
                            out.push(b'&');
                            out.extend_from_slice(format_float(n).as_bytes());
                        }
                        Node::Float32(n) => {
                            out.extend_from_slice(b"&&");
                            out.extend_from_slice(format_float(f64::from(n)).as_bytes());
                        }
                        Node::ThreadId(n) => {
                            out.extend_from_slice(b"ThreadId#");
                            push_display(out, n);
                        }
                        Node::Ptr(ptr) => {
                            if ptr == 0 {
                                out.extend_from_slice(b"(toPtr #0)");
                            } else if let Some(handle) = handle_name_from_ptr(ptr) {
                                out.extend_from_slice(handle.as_bytes());
                            } else {
                                out.extend_from_slice(b"Ptr#");
                                push_display(out, ptr);
                            }
                        }
                        Node::RawFunPtr(ptr) => {
                            out.push(b';');
                            push_display(out, ptr);
                        }
                        Node::ForeignPtr(foreign_ptr) => {
                            if let Some(mpz) = self.mpz_decimal_bytes_for_ptr(foreign_ptr.ptr) {
                                out.push(b'%');
                                out.extend_from_slice(mpz);
                                out.push(b'"');
                            } else if let Some(bytes) = &foreign_ptr.bytes {
                                serialize_bytes_comb(bytes, out);
                            } else if let Some(handle) = handle_name_from_ptr(foreign_ptr.ptr) {
                                out.extend_from_slice(handle.as_bytes());
                            } else {
                                out.extend_from_slice(b"ForeignPtr#");
                                push_display(out, foreign_ptr.ptr);
                            }
                        }
                        Node::Weak(_) | Node::MVar(_) => {
                            return Err(EvalError::UnsupportedSerialization(id));
                        }
                        Node::BigInt(bytes) => {
                            serialize_bigint_decimal(&bytes, out);
                        }
                        Node::Bytes(bytes) => {
                            serialize_bytes_comb(&bytes, out);
                        }
                        Node::BytesView(_) => {
                            serialize_bytes_comb(self.bytes(id)?, out);
                        }
                        Node::MutableBytes(bytes) => {
                            serialize_bytes_comb(bytes.visible(), out);
                        }
                        Node::Array(items) => {
                            out.push(b'[');
                            push_display(out, items.len());
                            out.push(b']');
                            for item in items.iter().rev() {
                                work.push(PrintTask::Node(*item));
                                work.push(PrintTask::Byte(b' '));
                            }
                        }
                        Node::Ffi(name) => {
                            out.push(b'^');
                            out.extend_from_slice(name.as_bytes());
                        }
                        Node::JsCall(call) => {
                            out.push(b'~');
                            out.extend_from_slice(call.tags.as_bytes());
                            out.push(b' ');
                            serialize_bytes_quoted(&call.body, out);
                        }
                        Node::JsWrap { tags } => {
                            out.push(b'`');
                            out.extend_from_slice(tags.as_bytes());
                        }
                        Node::FunPtr(name) => {
                            out.push(b';');
                            out.extend_from_slice(name.as_bytes());
                        }
                        Node::Tick(name) => {
                            out.push(b'!');
                            serialize_bytes_quoted(&name, out);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    #[cold]
    #[inline(never)]
    fn serialize_comb_into(
        &self,
        root: NodeId,
        labels: &mut SerializationLabels,
        out: &mut Vec<u8>,
    ) -> Result<(), EvalError> {
        enum SerializeTask {
            Node(NodeId),
            App,
            Array(usize),
            Label(NodeId),
        }

        let mut work = vec![SerializeTask::Node(root)];
        while let Some(task) = work.pop() {
            match task {
                SerializeTask::Node(id) => {
                    let id = self.resolve(id)?;
                    let share = if labels.shared.contains(&id) {
                        if !labels.printed.insert(id) {
                            out.push(b'_');
                            push_display(out, id.index());
                            out.push(b' ');
                            continue;
                        }
                        true
                    } else {
                        false
                    };

                    match self.node_for_debug(id) {
                        Node::App(fun, arg) => {
                            if share {
                                work.push(SerializeTask::Label(id));
                            }
                            work.push(SerializeTask::App);
                            work.push(SerializeTask::Node(arg));
                            work.push(SerializeTask::Node(fun));
                        }
                        Node::Indir(_) | Node::Free(_) => {
                            return Err(EvalError::DanglingIndirection(id));
                        }
                        Node::Prim(name) => {
                            out.extend_from_slice(name.name().as_bytes());
                            out.push(b' ');
                            if share {
                                work.push(SerializeTask::Label(id));
                            }
                        }
                        Node::Int(n) => {
                            out.push(b'#');
                            push_display(out, n);
                            out.push(b' ');
                            if share {
                                work.push(SerializeTask::Label(id));
                            }
                        }
                        Node::Int64(n) => {
                            out.extend_from_slice(b"##");
                            push_display(out, n);
                            out.push(b' ');
                            if share {
                                work.push(SerializeTask::Label(id));
                            }
                        }
                        Node::Float64(n) => {
                            out.push(b'&');
                            out.extend_from_slice(format_float(n).as_bytes());
                            out.push(b' ');
                            if share {
                                work.push(SerializeTask::Label(id));
                            }
                        }
                        Node::Float32(n) => {
                            out.extend_from_slice(b"&&");
                            out.extend_from_slice(format_float(f64::from(n)).as_bytes());
                            out.push(b' ');
                            if share {
                                work.push(SerializeTask::Label(id));
                            }
                        }
                        Node::ThreadId(_) | Node::Weak(_) | Node::MVar(_) => {
                            return Err(EvalError::UnsupportedSerialization(id));
                        }
                        Node::Ptr(ptr) => {
                            serialize_ptr(ptr, out);
                            out.push(b' ');
                            if share {
                                work.push(SerializeTask::Label(id));
                            }
                        }
                        Node::RawFunPtr(ptr) => {
                            out.extend_from_slice(b"toFunPtr #");
                            push_display(out, ptr);
                            out.extend_from_slice(b" @ ");
                            if share {
                                work.push(SerializeTask::Label(id));
                            }
                        }
                        Node::ForeignPtr(foreign_ptr) => {
                            if let Some(mpz) = self.mpz_decimal_bytes_for_ptr(foreign_ptr.ptr) {
                                serialize_bigint_decimal(mpz, out);
                            } else if let Some(bytes) = &foreign_ptr.bytes {
                                if foreign_ptr.offset == 0 {
                                    out.extend_from_slice(b"bs2fp ");
                                    serialize_bytes_comb(bytes, out);
                                    out.extend_from_slice(b" @");
                                } else {
                                    out.extend_from_slice(b"fp+ bs2fp ");
                                    serialize_bytes_comb(bytes, out);
                                    out.extend_from_slice(b" @ #");
                                    push_display(out, foreign_ptr.offset);
                                    out.extend_from_slice(b" @");
                                }
                            } else {
                                out.extend_from_slice(b"fpnew ");
                                serialize_ptr(foreign_ptr.ptr, out);
                                out.extend_from_slice(b" @");
                            }
                            out.push(b' ');
                            if share {
                                work.push(SerializeTask::Label(id));
                            }
                        }
                        Node::BigInt(bytes) => {
                            serialize_bigint_decimal(&bytes, out);
                            out.push(b' ');
                            if share {
                                work.push(SerializeTask::Label(id));
                            }
                        }
                        Node::Bytes(bytes) => {
                            serialize_bytes_comb(&bytes, out);
                            out.push(b' ');
                            if share {
                                work.push(SerializeTask::Label(id));
                            }
                        }
                        Node::BytesView(_) => {
                            serialize_bytes_comb(self.bytes(id)?, out);
                            out.push(b' ');
                            if share {
                                work.push(SerializeTask::Label(id));
                            }
                        }
                        Node::MutableBytes(bytes) => {
                            serialize_bytes_comb(bytes.visible(), out);
                            out.push(b' ');
                            if share {
                                work.push(SerializeTask::Label(id));
                            }
                        }
                        Node::Array(items) => {
                            if share {
                                work.push(SerializeTask::Label(id));
                            }
                            work.push(SerializeTask::Array(items.len()));
                            for item in items.iter().rev() {
                                work.push(SerializeTask::Node(*item));
                            }
                        }
                        Node::Ffi(name) => {
                            out.push(b'^');
                            out.extend_from_slice(name.as_bytes());
                            out.push(b' ');
                            if share {
                                work.push(SerializeTask::Label(id));
                            }
                        }
                        Node::JsCall(call) => {
                            out.push(b'~');
                            out.extend_from_slice(call.tags.as_bytes());
                            out.push(b' ');
                            serialize_bytes_quoted(&call.body, out);
                            out.push(b' ');
                            if share {
                                work.push(SerializeTask::Label(id));
                            }
                        }
                        Node::JsWrap { tags } => {
                            out.push(b'`');
                            out.extend_from_slice(tags.as_bytes());
                            out.push(b' ');
                            if share {
                                work.push(SerializeTask::Label(id));
                            }
                        }
                        Node::FunPtr(name) => {
                            out.push(b';');
                            out.extend_from_slice(name.as_bytes());
                            out.push(b' ');
                            if share {
                                work.push(SerializeTask::Label(id));
                            }
                        }
                        Node::Tick(name) => {
                            out.push(b'!');
                            serialize_bytes_quoted(&name, out);
                            out.push(b' ');
                            if share {
                                work.push(SerializeTask::Label(id));
                            }
                        }
                    }
                }
                SerializeTask::App => out.push(b'@'),
                SerializeTask::Array(len) => {
                    out.push(b'[');
                    push_display(out, len);
                    out.push(b']');
                    out.push(b' ');
                }
                SerializeTask::Label(id) => {
                    out.push(b':');
                    push_display(out, id.index());
                    out.push(b' ');
                }
            }
        }
        Ok(())
    }
}
