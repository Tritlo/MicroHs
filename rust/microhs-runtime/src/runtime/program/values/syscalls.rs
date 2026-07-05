//! Host syscall result helpers that need guest memory marshaling.
use super::*;

impl Program {
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
                self.poke_unsigned(len_ptr, size_of::<libc::socklen_t>(), u64::from(len))?;
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
                self.poke_unsigned(optlen_ptr, size_of::<libc::socklen_t>(), u64::from(len))?;
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
}
