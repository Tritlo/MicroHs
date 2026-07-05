//! Block-oriented BFILE reads and writes.
use super::*;

impl Program {
    pub(in crate::runtime) fn read_bfile(
        &mut self,
        ptr: i64,
        dst: i64,
        len: usize,
    ) -> Result<usize, EvalError> {
        let bytes = self.read_bfile_bytes(ptr, len)?;
        self.write_pointer_bytes(dst, &bytes)?;
        Ok(bytes.len())
    }

    pub(in crate::runtime) fn read_bfile_bytes(
        &mut self,
        ptr: i64,
        len: usize,
    ) -> Result<Vec<u8>, EvalError> {
        if let Some(handle) = handle_from_ptr(ptr) {
            if handle != StdHandle::Stdin {
                return Err(EvalError::InvalidHandle);
            }
            #[cfg(any(not(target_arch = "wasm32"), target_os = "wasi"))]
            {
                use std::io::Read as _;

                let mut bytes = vec![0; len];
                let read = std::io::stdin()
                    .lock()
                    .read(&mut bytes)
                    .map_err(|_| EvalError::InvalidHandle)?;
                bytes.truncate(read);
                return Ok(bytes);
            }
            #[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
            {
                return Ok(Vec::new());
            }
        }
        let (uses_getb_fallback, uses_memory_view) = {
            let bfile = self.bfile(ptr)?;
            if !bfile.readable {
                return Err(EvalError::InvalidHandle);
            }
            (
                matches!(
                    &bfile.kind,
                    BFileKind::Utf8 { .. }
                        | BFileKind::Crlf { .. }
                        | BFileKind::Rle { .. }
                        | BFileKind::Base64 { .. }
                        | BFileKind::Buf { .. }
                ),
                matches!(&bfile.kind, BFileKind::ReadOnlyMemoryView { .. }),
            )
        };
        if uses_getb_fallback {
            let mut bytes = Vec::with_capacity(len);
            for _ in 0..len {
                let byte = self.get_bfile_byte(ptr)?;
                if byte < 0 {
                    break;
                }
                bytes.push(byte as u8);
            }
            return Ok(bytes);
        }
        if uses_memory_view {
            return self.read_only_memory_view_bytes(ptr, len);
        }
        let bfile = self.bfile_mut(ptr)?;
        match &mut bfile.kind {
            BFileKind::Memory { bytes, pos } => {
                let end = pos
                    .checked_add(len)
                    .map(|end| end.min(bytes.len()))
                    .ok_or(EvalError::Overflow)?;
                let out = bytes[*pos..end].to_vec();
                *pos = end;
                Ok(out)
            }
            BFileKind::ReadOnlyMemoryView { .. } => unreachable!("handled above"),
            #[cfg(any(not(target_arch = "wasm32"), target_os = "wasi"))]
            BFileKind::NativeFile { file, ungot } => {
                use std::io::Read as _;

                #[cfg(target_os = "wasi")]
                wasi_trace_host("readb_native", &format!("len={len}"));
                let mut bytes = vec![0; len];
                let mut read = 0;
                while read < len {
                    let Some(byte) = ungot.pop() else {
                        break;
                    };
                    bytes[read] = byte;
                    read += 1;
                }
                if read < len {
                    read += file
                        .borrow_mut()
                        .read(&mut bytes[read..])
                        .map_err(|_| EvalError::InvalidHandle)?;
                }
                bytes.truncate(read);
                Ok(bytes)
            }
            #[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
            BFileKind::BrowserFile { handle, ungot } => {
                let mut bytes = vec![0; len];
                let mut read = 0;
                while read < len {
                    let Some(byte) = ungot.pop() else {
                        break;
                    };
                    bytes[read] = byte;
                    read += 1;
                }
                if read < len {
                    let host_read = unsafe {
                        mhs_host_file_read(*handle, bytes[read..].as_mut_ptr(), len - read)
                    };
                    if host_read < 0 {
                        return Err(EvalError::InvalidHandle);
                    }
                    read += usize::try_from(host_read).map_err(|_| EvalError::Overflow)?;
                }
                bytes.truncate(read);
                Ok(bytes)
            }
            BFileKind::Utf8 { .. }
            | BFileKind::Crlf { .. }
            | BFileKind::Rle { .. }
            | BFileKind::Base64 { .. }
            | BFileKind::Lz77 { read: false, .. }
            | BFileKind::Bwt { read: false, .. }
            | BFileKind::Lzma { read: false, .. }
            | BFileKind::Buf { .. } => unreachable!("handled above"),
            BFileKind::Lz77 {
                read, buffer, pos, ..
            } => {
                if !*read {
                    return Err(EvalError::InvalidHandle);
                }
                let end = pos
                    .checked_add(len)
                    .map(|end| end.min(buffer.len()))
                    .ok_or(EvalError::Overflow)?;
                let out = buffer[*pos..end].to_vec();
                *pos = end;
                Ok(out)
            }
            BFileKind::Bwt {
                read, buffer, pos, ..
            } => {
                if !*read {
                    return Err(EvalError::InvalidHandle);
                }
                let end = pos
                    .checked_add(len)
                    .map(|end| end.min(buffer.len()))
                    .ok_or(EvalError::Overflow)?;
                let out = buffer[*pos..end].to_vec();
                *pos = end;
                Ok(out)
            }
            BFileKind::Lzma {
                read, buffer, pos, ..
            } => {
                if !*read {
                    return Err(EvalError::InvalidHandle);
                }
                let end = pos
                    .checked_add(len)
                    .map(|end| end.min(buffer.len()))
                    .ok_or(EvalError::Overflow)?;
                let out = buffer[*pos..end].to_vec();
                *pos = end;
                Ok(out)
            }
        }
    }

    pub(in crate::runtime) fn write_bfile(
        &mut self,
        ptr: i64,
        src: i64,
        len: usize,
    ) -> Result<usize, EvalError> {
        let bytes = self.read_pointer_bytes(src, len)?;
        self.write_bfile_bytes(ptr, &bytes)
    }

    pub(in crate::runtime) fn write_bfile_bytes(
        &mut self,
        ptr: i64,
        bytes: &[u8],
    ) -> Result<usize, EvalError> {
        if let Some(handle) = handle_from_ptr(ptr) {
            self.write_io_handle_bytes(handle, bytes)?;
            return Ok(bytes.len());
        }
        let uses_putb_fallback = {
            let bfile = self.bfile(ptr)?;
            if !bfile.writable {
                return Err(EvalError::InvalidHandle);
            }
            matches!(
                &bfile.kind,
                BFileKind::Utf8 { .. }
                    | BFileKind::Crlf { .. }
                    | BFileKind::Rle { .. }
                    | BFileKind::Base64 { .. }
                    | BFileKind::Buf { .. }
            )
        };
        if uses_putb_fallback {
            for byte in bytes {
                self.put_bfile_byte(ptr, i64::from(*byte))?;
            }
            return Ok(bytes.len());
        }
        let bfile = self.bfile_mut(ptr)?;
        match &mut bfile.kind {
            BFileKind::Memory { bytes: buffer, pos } => {
                let end = pos.checked_add(bytes.len()).ok_or(EvalError::Overflow)?;
                if end > buffer.len() {
                    buffer.resize(end, 0);
                }
                buffer[*pos..end].copy_from_slice(&bytes);
                *pos = end;
            }
            BFileKind::ReadOnlyMemoryView { .. } => {
                unreachable!("read-only handle is not writable")
            }
            #[cfg(any(not(target_arch = "wasm32"), target_os = "wasi"))]
            BFileKind::NativeFile { file, .. } => {
                use std::io::Write as _;

                #[cfg(target_os = "wasi")]
                wasi_trace_host("writeb_native", &format!("len={}", bytes.len()));
                file.borrow_mut()
                    .write_all(bytes)
                    .map_err(|_| EvalError::InvalidHandle)?;
            }
            #[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
            BFileKind::BrowserFile { handle, .. } => {
                let written = unsafe { mhs_host_file_write(*handle, bytes.as_ptr(), bytes.len()) };
                if written < 0 || usize::try_from(written).ok() != Some(bytes.len()) {
                    return Err(EvalError::InvalidHandle);
                }
            }
            BFileKind::Utf8 { .. }
            | BFileKind::Crlf { .. }
            | BFileKind::Rle { .. }
            | BFileKind::Base64 { .. }
            | BFileKind::Lz77 { read: true, .. }
            | BFileKind::Bwt { read: true, .. }
            | BFileKind::Lzma { read: true, .. }
            | BFileKind::Buf { .. } => unreachable!("handled above"),
            BFileKind::Lz77 {
                read, buffer, pos, ..
            } => {
                if *read {
                    return Err(EvalError::InvalidHandle);
                }
                let end = pos.checked_add(bytes.len()).ok_or(EvalError::Overflow)?;
                if end > buffer.len() {
                    buffer.resize(end, 0);
                }
                buffer[*pos..end].copy_from_slice(bytes);
                *pos = end;
            }
            BFileKind::Bwt {
                read, buffer, pos, ..
            } => {
                if *read {
                    return Err(EvalError::InvalidHandle);
                }
                let end = pos.checked_add(bytes.len()).ok_or(EvalError::Overflow)?;
                if end > buffer.len() {
                    buffer.resize(end, 0);
                }
                buffer[*pos..end].copy_from_slice(bytes);
                *pos = end;
            }
            BFileKind::Lzma {
                read, buffer, pos, ..
            } => {
                if *read {
                    return Err(EvalError::InvalidHandle);
                }
                let end = pos.checked_add(bytes.len()).ok_or(EvalError::Overflow)?;
                if end > buffer.len() {
                    buffer.resize(end, 0);
                }
                buffer[*pos..end].copy_from_slice(bytes);
                *pos = end;
            }
        }
        Ok(bytes.len())
    }

    pub(in crate::runtime) fn bfile_output_bytes(&self, ptr: i64) -> Result<Vec<u8>, EvalError> {
        let bfile = self.bfile(ptr)?;
        if !bfile.writable {
            return Err(EvalError::InvalidHandle);
        }
        match &bfile.kind {
            BFileKind::Memory { bytes, pos } => Ok(bytes[..*pos].to_vec()),
            BFileKind::ReadOnlyMemoryView { .. } => Err(EvalError::InvalidHandle),
            #[cfg(any(not(target_arch = "wasm32"), target_os = "wasi"))]
            BFileKind::NativeFile { .. } => Err(EvalError::InvalidHandle),
            #[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
            BFileKind::BrowserFile { .. } => Err(EvalError::InvalidHandle),
            BFileKind::Utf8 { .. } => Err(EvalError::InvalidHandle),
            BFileKind::Crlf { .. } => Err(EvalError::InvalidHandle),
            BFileKind::Rle { .. } => Err(EvalError::InvalidHandle),
            BFileKind::Base64 { .. } => Err(EvalError::InvalidHandle),
            BFileKind::Lz77 { .. } => Err(EvalError::InvalidHandle),
            BFileKind::Bwt { .. } => Err(EvalError::InvalidHandle),
            BFileKind::Lzma { .. } => Err(EvalError::InvalidHandle),
            BFileKind::Buf { .. } => Err(EvalError::InvalidHandle),
        }
    }

    #[cold]
    #[inline(never)]
    pub(in crate::runtime) fn deserialize_bfile(&mut self, ptr: i64) -> Result<NodeId, EvalError> {
        let mut input = Vec::new();
        let mut last_error = None;
        loop {
            let byte = self.read_bfile_bytes(ptr, 1)?;
            if byte.is_empty() {
                return Err(last_error
                    .map(Self::deserialize_parse_error)
                    .unwrap_or(EvalError::InvalidByteString));
            }
            input.push(byte[0]);
            match crate::parse::parse_program(&input) {
                Ok(parsed) => return self.append_parsed_program(parsed),
                Err(err) => last_error = Some(err),
            }
        }
    }

    pub(in crate::runtime) fn deserialize_parse_error(
        error: crate::parse::ParseError,
    ) -> EvalError {
        match error {
            crate::parse::ParseError::UnknownPrim(name) => EvalError::UnknownPrim(name),
            _ => EvalError::InvalidByteString,
        }
    }

    #[cold]
    #[inline(never)]
    pub(in crate::runtime) fn append_parsed_program(
        &mut self,
        parsed: Program,
    ) -> Result<NodeId, EvalError> {
        let root = parsed.root();
        let node_count = parsed.node_count();
        let mut remap = Vec::with_capacity(node_count);
        for index in 0..node_count {
            let node = parsed.node_for_debug(NodeId::from_index(index));
            let id = match node {
                Node::App(_, _)
                | Node::Indir(_)
                | Node::BytesView(_)
                | Node::MVar(Some(_))
                | Node::Weak(_)
                | Node::Array(_) => self.push_node(Node::Indir(None)),
                Node::Free(_) => return Err(EvalError::InvalidByteString),
                node => self.push_value_node(node),
            };
            remap.push(id);
        }

        for index in 0..node_count {
            let node = parsed.node_for_debug(NodeId::from_index(index));
            let target = remap[index];
            let node = match node {
                Node::App(fun, arg) => Node::App(
                    Self::remap_parsed_id(&remap, fun)?,
                    Self::remap_parsed_id(&remap, arg)?,
                ),
                Node::Indir(target) => Node::Indir(
                    target
                        .map(|id| Self::remap_parsed_id(&remap, id))
                        .transpose()?,
                ),
                Node::BytesView(view) => Node::bytes_view(
                    Self::remap_parsed_id(&remap, view.base)?,
                    view.offset,
                    view.len,
                ),
                Node::MVar(Some(value)) => Node::MVar(Some(Self::remap_parsed_id(&remap, value)?)),
                Node::Weak(weak) => {
                    let weak = *weak;
                    Node::Weak(Box::new(WeakNode {
                        key: weak
                            .key
                            .map(|id| Self::remap_parsed_id(&remap, id))
                            .transpose()?,
                        value: weak
                            .value
                            .map(|id| Self::remap_parsed_id(&remap, id))
                            .transpose()?,
                        finalizer: weak
                            .finalizer
                            .map(|id| Self::remap_parsed_id(&remap, id))
                            .transpose()?,
                    }))
                }
                Node::Array(items) => Node::array(
                    items
                        .iter()
                        .map(|id| Self::remap_parsed_id(&remap, *id))
                        .collect::<Result<Vec<_>, _>>()?,
                ),
                Node::Free(_) => return Err(EvalError::InvalidByteString),
                _ => continue,
            };
            let is_weak = matches!(&node, Node::Weak(_));
            self.set_node_at(target.index(), node);
            if is_weak {
                self.weak_nodes.push(target);
            }
        }

        Self::remap_parsed_id(&remap, root)
    }

    pub(in crate::runtime) fn remap_parsed_id(
        remap: &[NodeId],
        id: NodeId,
    ) -> Result<NodeId, EvalError> {
        remap
            .get(id.index())
            .copied()
            .ok_or(EvalError::InvalidByteString)
    }
}
