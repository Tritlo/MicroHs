//! Value forcing and conversion helpers for runtime primitive implementations.
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
            self.trace_suspicious_pointer_value(root, value);
            return Ok(value);
        }
        let root = self.reduce_node_whnf(root, FORCE_REDUCTION_LIMIT)?;
        let root = self.resolve(root)?;
        let value = self
            .pointer_value_from_whnf(root)
            .ok_or(EvalError::ExpectedPointer(root))?;
        self.trace_suspicious_pointer_value(root, value);
        Ok(value)
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

    pub(in crate::runtime) fn trace_suspicious_pointer_value(&self, root: NodeId, ptr: i64) {
        if std::env::var_os("MHS_TRACE_INVALID_BYTES").is_none() {
            return;
        }
        if ptr >= 0 || handle_from_ptr(ptr).is_some() {
            return;
        }
        let suspicious = if ptr < BFILE_PTR_BASE {
            true
        } else if ptr < DIR_PTR_BASE {
            self.decode_bfile_pointer(ptr)
                .ok()
                .and_then(|slot| self.bfiles.get(slot))
                .and_then(Option::as_ref)
                .is_none()
        } else if ptr < ALLOCATION_PTR_BASE {
            self.decode_dir_pointer(ptr)
                .ok()
                .and_then(|slot| self.dirs.get(slot))
                .and_then(Option::as_ref)
                .is_none()
        } else {
            self.decode_allocation_pointer(ptr).is_err()
        };
        if suspicious {
            eprintln!(
                "suspicious pointer value: reductions={} ptr={ptr} root={}",
                self.reductions,
                self.node_trace_summary(root)
            );
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

    pub(in crate::runtime) fn byte_slice_node(
        &self,
        id: NodeId,
        offset: usize,
        len: usize,
    ) -> Result<Node, EvalError> {
        let bytes = self.bytes(id)?;
        let end = offset.checked_add(len).ok_or(EvalError::Overflow)?;
        if end > bytes.len() {
            return Err(EvalError::InvalidByteString);
        }
        match self.cold_node(id) {
            Some(Node::Bytes(_)) => Ok(Node::bytes_view(id, offset, len)),
            Some(Node::BytesView(view)) => {
                let offset = view.offset.checked_add(offset).ok_or(EvalError::Overflow)?;
                Ok(Node::bytes_view(view.base, offset, len))
            }
            Some(Node::MutableBytes(_)) => Ok(Node::bytes(bytes[offset..end].to_vec())),
            _ => Err(self.expected_bytes_error(id)),
        }
    }

    pub(in crate::runtime) fn materialize_bytes_view_for_write(
        &mut self,
        id: NodeId,
    ) -> Result<(), EvalError> {
        if matches!(self.cold_node(id), Some(Node::BytesView(_))) {
            let bytes = self.bytes(id)?.to_vec();
            self.set_node_at(id.index(), Node::bytes(bytes));
        }
        Ok(())
    }

    pub(in crate::runtime) fn new_mutable_bytes(
        &mut self,
        size: usize,
        capacity: usize,
    ) -> Result<NodeId, EvalError> {
        if size > capacity {
            return Err(EvalError::InvalidByteString);
        }
        let bytes = vec![0; capacity];
        Ok(
            self.push_node(Node::MutableBytes(Box::new(MutableBytesNode {
                bytes,
                size,
                capacity,
            }))),
        )
    }

    pub(in crate::runtime) fn freeze_bytes(&mut self, id: NodeId) -> Result<NodeId, EvalError> {
        let frozen = match self.cold_node(id) {
            Some(Node::Bytes(_)) => return Ok(id),
            Some(Node::BytesView(_)) => return Ok(id),
            Some(Node::MutableBytes(bytes)) => bytes.visible().to_vec(),
            _ => return Err(self.expected_bytes_error(id)),
        };
        self.set_node_at(id.index(), Node::bytes(frozen));
        Ok(id)
    }

    pub(in crate::runtime) fn append_byte(
        &mut self,
        id: NodeId,
        byte: u8,
    ) -> Result<(), EvalError> {
        self.materialize_bytes_view_for_write(id)?;
        match self.cold_node_mut(id) {
            Some(Node::Bytes(bytes)) => {
                bytes.push(byte);
                Ok(())
            }
            Some(Node::MutableBytes(bytes)) => {
                if bytes.size >= bytes.capacity {
                    bytes.capacity = bytes
                        .capacity
                        .checked_add(bytes.capacity / 2)
                        .and_then(|capacity| capacity.checked_add(2))
                        .ok_or(EvalError::Overflow)?;
                    if bytes.capacity < bytes.size {
                        return Err(EvalError::Overflow);
                    }
                    if bytes.capacity > bytes.bytes.capacity() {
                        bytes.bytes.reserve(bytes.capacity - bytes.bytes.capacity());
                    }
                    bytes.bytes.resize(bytes.capacity, 0);
                }
                bytes.bytes[bytes.size] = byte;
                bytes.size += 1;
                Ok(())
            }
            _ => Err(self.expected_bytes_error(id)),
        }
    }

    pub(in crate::runtime) fn append_bytes(
        &mut self,
        id: NodeId,
        bytes: &[u8],
    ) -> Result<(), EvalError> {
        self.materialize_bytes_view_for_write(id)?;
        match self.cold_node_mut(id) {
            Some(Node::Bytes(dst)) => {
                dst.extend_from_slice(bytes);
                Ok(())
            }
            Some(Node::MutableBytes(_)) => {
                for &byte in bytes {
                    self.append_byte(id, byte)?;
                }
                Ok(())
            }
            _ => Err(self.expected_bytes_error(id)),
        }
    }

    pub(in crate::runtime) fn read_byte_unchecked_prim(
        &self,
        id: NodeId,
        index: usize,
    ) -> Result<u8, EvalError> {
        match self.cold_node(id) {
            Some(Node::Bytes(bytes)) => bytes
                .get(index)
                .copied()
                .ok_or(EvalError::InvalidByteString),
            Some(Node::BytesView(_)) => self
                .bytes(id)?
                .get(index)
                .copied()
                .ok_or(EvalError::InvalidByteString),
            Some(Node::MutableBytes(bytes)) => bytes
                .bytes
                .get(index)
                .copied()
                .ok_or(EvalError::InvalidByteString),
            _ => Err(self.expected_bytes_error(id)),
        }
    }

    pub(in crate::runtime) fn byte_prim_lengths(
        &self,
        id: NodeId,
    ) -> Result<(usize, usize), EvalError> {
        match self.cold_node(id) {
            Some(Node::Bytes(bytes)) => Ok((bytes.len(), bytes.len())),
            Some(Node::BytesView(view)) => Ok((view.len, view.len)),
            Some(Node::MutableBytes(bytes)) => Ok((bytes.size, bytes.bytes.len())),
            _ => Err(self.expected_bytes_error(id)),
        }
    }

    pub(in crate::runtime) fn write_byte_unchecked_prim(
        &mut self,
        id: NodeId,
        index: usize,
        byte: u8,
    ) -> Result<(), EvalError> {
        self.materialize_bytes_view_for_write(id)?;
        match self.cold_node_mut(id) {
            Some(Node::Bytes(bytes)) => {
                let slot = bytes.get_mut(index).ok_or(EvalError::InvalidByteString)?;
                *slot = byte;
                Ok(())
            }
            Some(Node::MutableBytes(bytes)) => {
                let slot = bytes
                    .bytes
                    .get_mut(index)
                    .ok_or(EvalError::InvalidByteString)?;
                *slot = byte;
                Ok(())
            }
            _ => Err(self.expected_bytes_error(id)),
        }
    }

    pub(in crate::runtime) fn alloc_memory(&mut self, size: usize) -> Result<i64, EvalError> {
        let bytes = vec![0; size];
        let slot = if let Some(slot) = self.allocations.iter().position(Option::is_none) {
            self.allocations[slot] = Some(bytes);
            slot
        } else {
            self.allocations.push(Some(bytes));
            self.allocations.len() - 1
        };
        self.pointer_for_allocation(slot, 0)
    }

    pub(in crate::runtime) fn alloc_c_string_bytes(
        &mut self,
        bytes: &[u8],
    ) -> Result<i64, EvalError> {
        let len = bytes.len().checked_add(1).ok_or(EvalError::Overflow)?;
        let ptr = self.alloc_memory(len)?;
        self.write_pointer_bytes(ptr, bytes)?;
        self.write_pointer_bytes(
            ptr.checked_add(i64::try_from(bytes.len()).map_err(|_| EvalError::Overflow)?)
                .ok_or(EvalError::Overflow)?,
            &[0],
        )?;
        Ok(ptr)
    }

    pub(in crate::runtime) fn errno_ptr(&mut self) -> Result<i64, EvalError> {
        if let Some(ptr) = self.errno_ptr {
            return Ok(ptr);
        }
        let ptr = self.alloc_memory(size_of::<std::os::raw::c_int>())?;
        self.errno_ptr = Some(ptr);
        self.write_errno_cell(ptr)?;
        Ok(ptr)
    }

    pub(in crate::runtime) fn set_errno_value(&mut self, value: i32) -> Result<(), EvalError> {
        self.errno_value = value;
        if let Some(ptr) = self.errno_ptr {
            self.write_errno_cell(ptr)?;
        }
        Ok(())
    }

    pub(in crate::runtime) fn write_errno_cell(&mut self, ptr: i64) -> Result<(), EvalError> {
        let value = self.errno_value as std::os::raw::c_int;
        self.write_pointer_bytes(ptr, &value.to_ne_bytes())
    }

    pub(in crate::runtime) fn host_int_node(
        &mut self,
        result: HostIntResult,
    ) -> Result<Node, EvalError> {
        if let Some(errno) = result.errno {
            self.set_errno_value(errno)?;
        }
        Ok(Node::Int(result.value))
    }

    #[cfg_attr(not(all(unix, not(target_arch = "wasm32"))), allow(dead_code))]
    pub(in crate::runtime) fn syscall_result_node(
        &mut self,
        value: i64,
    ) -> Result<Node, EvalError> {
        if value < 0 {
            self.set_errno_value(last_errno())?;
        }
        Ok(Node::Int(value))
    }

    pub(in crate::runtime) fn gettimeofday_node(
        &mut self,
        timeval_ptr: i64,
        timezone_ptr: i64,
    ) -> Result<Node, EvalError> {
        #[cfg(all(unix, not(target_arch = "wasm32")))]
        {
            let _ = timezone_ptr;
            let mut tv = std::mem::MaybeUninit::<libc::timeval>::uninit();
            // SAFETY: libc writes the timeval on success. The timezone argument is obsolete.
            let rc = unsafe { libc::gettimeofday(tv.as_mut_ptr(), std::ptr::null_mut()) };
            if rc < 0 {
                return self.syscall_result_node(i64::from(rc));
            }
            if timeval_ptr != 0 {
                // SAFETY: gettimeofday succeeded, so tv is initialized.
                let tv = unsafe { tv.assume_init() };
                // SAFETY: tv is a plain C struct; copying its bytes matches the C FFI layout.
                let bytes = unsafe {
                    std::slice::from_raw_parts(
                        (&tv as *const libc::timeval).cast::<u8>(),
                        size_of::<libc::timeval>(),
                    )
                };
                self.write_pointer_bytes(timeval_ptr, bytes)?;
            }
            Ok(Node::Int(i64::from(rc)))
        }
        #[cfg(not(all(unix, not(target_arch = "wasm32"))))]
        {
            let _ = (timeval_ptr, timezone_ptr);
            self.host_int_node(HostIntResult::err(errno_i32("ENOSYS")))
        }
    }

    pub(in crate::runtime) fn accept_socket_node(
        &mut self,
        fd: i32,
        addr_ptr: i64,
        len_ptr: i64,
    ) -> Result<Node, EvalError> {
        #[cfg(all(unix, not(target_arch = "wasm32")))]
        {
            let mut len = 0 as libc::socklen_t;
            let mut buffer = Vec::new();
            let (addr_arg, len_arg) = if addr_ptr != 0 && len_ptr != 0 {
                let raw_len = self.peek_unsigned(len_ptr, size_of::<libc::socklen_t>())?;
                len = libc::socklen_t::try_from(raw_len).map_err(|_| EvalError::Overflow)?;
                buffer.resize(usize::try_from(len).map_err(|_| EvalError::Overflow)?, 0);
                (buffer.as_mut_ptr().cast(), &mut len as *mut _)
            } else {
                (std::ptr::null_mut(), std::ptr::null_mut())
            };
            // SAFETY: pointers either point to temporary buffers or are null, as accepted by libc.
            let rc = unsafe { libc::accept(fd, addr_arg, len_arg) };
            if rc >= 0 && addr_ptr != 0 && len_ptr != 0 {
                let written = usize::try_from(len)
                    .map_err(|_| EvalError::Overflow)?
                    .min(buffer.len());
                self.write_pointer_bytes(addr_ptr, &buffer[..written])?;
                self.poke_unsigned(
                    len_ptr,
                    size_of::<libc::socklen_t>(),
                    u64::try_from(len).map_err(|_| EvalError::Overflow)?,
                )?;
            }
            self.syscall_result_node(i64::from(rc))
        }
        #[cfg(not(all(unix, not(target_arch = "wasm32"))))]
        {
            let _ = (fd, addr_ptr, len_ptr);
            self.host_int_node(HostIntResult::err(errno_i32("ENOSYS")))
        }
    }

    pub(in crate::runtime) fn getsockopt_node(
        &mut self,
        fd: i32,
        level: i32,
        optname: i32,
        optval_ptr: i64,
        optlen_ptr: i64,
    ) -> Result<Node, EvalError> {
        #[cfg(all(unix, not(target_arch = "wasm32")))]
        {
            if optlen_ptr == 0 {
                return self.host_int_node(HostIntResult::err(errno_i32("EINVAL")));
            }
            let raw_len = self.peek_unsigned(optlen_ptr, size_of::<libc::socklen_t>())?;
            let mut len = libc::socklen_t::try_from(raw_len).map_err(|_| EvalError::Overflow)?;
            let mut buffer = vec![0; usize::try_from(len).map_err(|_| EvalError::Overflow)?];
            let optval = if optval_ptr == 0 {
                std::ptr::null_mut()
            } else {
                buffer.as_mut_ptr().cast()
            };
            // SAFETY: optval points to a temporary output buffer or is null; len points to stack storage.
            let rc = unsafe { libc::getsockopt(fd, level, optname, optval, &mut len) };
            if rc >= 0 {
                if optval_ptr != 0 {
                    let written = usize::try_from(len)
                        .map_err(|_| EvalError::Overflow)?
                        .min(buffer.len());
                    self.write_pointer_bytes(optval_ptr, &buffer[..written])?;
                }
                self.poke_unsigned(
                    optlen_ptr,
                    size_of::<libc::socklen_t>(),
                    u64::try_from(len).map_err(|_| EvalError::Overflow)?,
                )?;
            }
            self.syscall_result_node(i64::from(rc))
        }
        #[cfg(not(all(unix, not(target_arch = "wasm32"))))]
        {
            let _ = (fd, level, optname, optval_ptr, optlen_ptr);
            self.host_int_node(HostIntResult::err(errno_i32("ENOSYS")))
        }
    }

    pub(in crate::runtime) fn recv_socket_node(
        &mut self,
        fd: i32,
        buf_ptr: i64,
        len: usize,
        flags: i32,
    ) -> Result<Node, EvalError> {
        #[cfg(all(unix, not(target_arch = "wasm32")))]
        {
            let mut buffer = vec![0; len];
            let ptr = if len == 0 {
                std::ptr::null_mut()
            } else {
                buffer.as_mut_ptr().cast()
            };
            // SAFETY: ptr points to a temporary buffer large enough for len bytes, or is null for len 0.
            let rc = unsafe { libc::recv(fd, ptr, len, flags) };
            if rc >= 0 && len != 0 {
                let read = usize::try_from(rc).map_err(|_| EvalError::Overflow)?;
                self.write_pointer_bytes(buf_ptr, &buffer[..read])?;
            }
            self.syscall_result_node(rc as i64)
        }
        #[cfg(not(all(unix, not(target_arch = "wasm32"))))]
        {
            let _ = (fd, buf_ptr, len, flags);
            self.host_int_node(HostIntResult::err(errno_i32("ENOSYS")))
        }
    }

    pub(in crate::runtime) fn send_socket_node(
        &mut self,
        fd: i32,
        buf_ptr: i64,
        len: usize,
        flags: i32,
    ) -> Result<Node, EvalError> {
        #[cfg(all(unix, not(target_arch = "wasm32")))]
        {
            let buffer = if len == 0 {
                Vec::new()
            } else {
                self.read_pointer_bytes(buf_ptr, len)?
            };
            let ptr = if len == 0 {
                std::ptr::null()
            } else {
                buffer.as_ptr().cast()
            };
            // SAFETY: ptr points to len bytes copied from guest memory, or is null for len 0.
            let rc = unsafe { libc::send(fd, ptr, len, flags) };
            self.syscall_result_node(rc as i64)
        }
        #[cfg(not(all(unix, not(target_arch = "wasm32"))))]
        {
            let _ = (fd, buf_ptr, len, flags);
            self.host_int_node(HostIntResult::err(errno_i32("ENOSYS")))
        }
    }

    pub(in crate::runtime) fn new_mpz_node(&mut self) -> Result<Node, EvalError> {
        let bigint = self.push_node(Node::bigint(b"0".to_vec()));
        let ptr = self.pointer_for_node(bigint, 0)?;
        Ok(self.foreign_ptr_node(None, 0, ptr))
    }

    pub(in crate::runtime) fn mpz_node_id(&self, ptr: i64) -> Result<NodeId, EvalError> {
        let (slot, offset) = self.decode_pointer(ptr)?;
        if offset != 0 {
            return Err(EvalError::ExpectedForeignPtr(NodeId::from_index(slot)));
        }
        let id = NodeId::from_index(slot);
        match self.cold_node(id) {
            Some(Node::BigInt(_)) => Ok(id),
            _ => Err(EvalError::ExpectedForeignPtr(id)),
        }
    }

    pub(in crate::runtime) fn mpz_decimal_bytes_for_ptr(&self, ptr: i64) -> Option<&[u8]> {
        let (slot, offset) = self.decode_pointer(ptr).ok()?;
        if offset != 0 {
            return None;
        }
        let id = NodeId::from_index(slot);
        match self.cold_node(id)? {
            Node::BigInt(bytes) => Some(bytes.as_slice()),
            _ => None,
        }
    }

    pub(in crate::runtime) fn mpz_value(&self, ptr: i64) -> Result<MpzValue, EvalError> {
        let id = self.mpz_node_id(ptr)?;
        let Some(Node::BigInt(bytes)) = self.cold_node(id) else {
            return Err(EvalError::ExpectedForeignPtr(id));
        };
        MpzValue::parse_decimal(bytes).map_err(|_| EvalError::InvalidByteString)
    }

    pub(in crate::runtime) fn write_mpz_value(
        &mut self,
        ptr: i64,
        value: MpzValue,
    ) -> Result<(), EvalError> {
        let id = self.mpz_node_id(ptr)?;
        self.set_node_at(id.index(), Node::bigint(value.to_decimal_bytes()));
        Ok(())
    }

    pub(in crate::runtime) fn write_strerror(
        &mut self,
        errno: i32,
        ptr: i64,
        size: usize,
    ) -> Result<i64, EvalError> {
        if size == 0 {
            let erange = errno_i32("ERANGE");
            self.set_errno_value(erange)?;
            return Ok(i64::from(erange));
        }
        let bytes = strerror_bytes(errno);
        let truncated = bytes.len() + 1 > size;
        let copy_len = if truncated { size - 1 } else { bytes.len() };
        let mut out = Vec::with_capacity(copy_len + 1);
        out.extend_from_slice(&bytes[..copy_len]);
        out.push(0);
        self.write_pointer_bytes(ptr, &out)?;
        if truncated {
            let erange = errno_i32("ERANGE");
            self.set_errno_value(erange)?;
            Ok(i64::from(erange))
        } else {
            Ok(0)
        }
    }

    pub(in crate::runtime) fn calloc_memory(
        &mut self,
        count: usize,
        size: usize,
    ) -> Result<i64, EvalError> {
        let len = count.checked_mul(size).ok_or(EvalError::Overflow)?;
        self.alloc_memory(len)
    }

    pub(in crate::runtime) fn realloc_memory(
        &mut self,
        ptr: i64,
        size: usize,
    ) -> Result<i64, EvalError> {
        if ptr == 0 {
            return self.alloc_memory(size);
        }
        let (slot, offset) = self.decode_allocation_pointer(ptr)?;
        if offset != 0 {
            return Err(EvalError::InvalidByteString);
        }
        let bytes = self
            .allocations
            .get_mut(slot)
            .and_then(Option::as_mut)
            .ok_or(EvalError::InvalidByteString)?;
        bytes.resize(size, 0);
        self.pointer_for_allocation(slot, 0)
    }

    pub(in crate::runtime) fn free_memory(&mut self, ptr: i64) -> Result<(), EvalError> {
        if ptr == 0 {
            return Ok(());
        }
        let (slot, offset) = self.decode_allocation_pointer(ptr)?;
        if offset != 0 {
            return Err(EvalError::InvalidByteString);
        }
        let slot = self
            .allocations
            .get_mut(slot)
            .ok_or(EvalError::InvalidByteString)?;
        if slot.is_none() {
            return Err(EvalError::InvalidByteString);
        }
        *slot = None;
        Ok(())
    }
}
