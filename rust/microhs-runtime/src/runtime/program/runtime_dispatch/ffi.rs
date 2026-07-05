//! C FFI, filesystem, socket, bigint, and libc-like runtime primitive dispatch.
use super::*;

impl Program {
    pub(in crate::runtime) fn ffi_call(
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

    pub(in crate::runtime) fn zero_arity_ffi_result(
        &mut self,
        name: &str,
    ) -> Result<Option<Node>, EvalError> {
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

    pub(in crate::runtime) fn unary_math_ffi_result(
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
}
