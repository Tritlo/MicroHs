impl Program {
    fn alloc_bfile(&mut self, bfile: BFile) -> Result<i64, EvalError> {
        let slot = if let Some(slot) = self.bfiles.iter().position(Option::is_none) {
            self.bfiles[slot] = Some(bfile);
            slot
        } else {
            self.bfiles.push(Some(bfile));
            self.bfiles.len() - 1
        };
        self.pointer_for_bfile(slot)
    }

    fn read_only_memory_view(&self, ptr: i64, len: usize) -> Result<Option<BFileKind>, EvalError> {
        if len <= READ_ONLY_MEMORY_VIEW_MIN_LEN {
            return Ok(None);
        }
        if ptr <= 0 {
            return Ok(None);
        }
        let Ok((base, offset)) = self.decode_pointer(ptr) else {
            return Ok(None);
        };
        let base = NodeId::from_index(base);
        match self.cold_node(base) {
            Some(Node::Bytes(_) | Node::BytesView(_)) => {
                let bytes = self.bytes(base)?;
                let end = offset.checked_add(len).ok_or(EvalError::Overflow)?;
                if end > bytes.len() {
                    return Err(trace_invalid_bytes!(
                        self,
                        "read pointer too short ptr={ptr} len={len} available={}",
                        bytes.len().saturating_sub(offset)
                    ));
                }
                Ok(Some(BFileKind::ReadOnlyMemoryView {
                    base,
                    offset,
                    len,
                    pos: 0,
                }))
            }
            _ => Ok(None),
        }
    }

    fn memory_read_bfile_kind(&self, ptr: i64, len: usize) -> Result<BFileKind, EvalError> {
        if let Some(kind) = self.read_only_memory_view(ptr, len)? {
            return Ok(kind);
        }
        let bytes = self.read_pointer_bytes(ptr, len)?;
        Ok(BFileKind::Memory { bytes, pos: 0 })
    }

    fn alloc_dir(&mut self, entries: Vec<Vec<u8>>) -> Result<i64, EvalError> {
        let dir = DirHandle { entries, pos: 0 };
        let slot = if let Some(slot) = self.dirs.iter().position(Option::is_none) {
            self.dirs[slot] = Some(dir);
            slot
        } else {
            self.dirs.push(Some(dir));
            self.dirs.len() - 1
        };
        self.pointer_for_dir(slot)
    }

    fn alloc_environ(&mut self) -> Result<i64, EvalError> {
        let vars = environ_bytes();
        let mut pointers = Vec::with_capacity(
            (vars.len() + 1)
                .checked_mul(size_of::<i64>())
                .ok_or(EvalError::Overflow)?,
        );
        for mut var in vars {
            var.push(0);
            let ptr = self.alloc_memory(var.len())?;
            self.write_pointer_bytes(ptr, &var)?;
            pointers.extend_from_slice(&ptr.to_ne_bytes());
        }
        pointers.extend_from_slice(&0_i64.to_ne_bytes());
        let ptr = self.alloc_memory(pointers.len())?;
        self.write_pointer_bytes(ptr, &pointers)?;
        Ok(ptr)
    }

    fn bfile_permissions(&self, ptr: i64) -> Result<(bool, bool), EvalError> {
        if let Some(handle) = handle_from_ptr(ptr) {
            return Ok(match handle {
                StdHandle::Stdin => (true, false),
                StdHandle::Stdout | StdHandle::Stderr => (false, true),
            });
        }
        let bfile = self.bfile(ptr)?;
        Ok((bfile.readable, bfile.writable))
    }

    fn add_utf8_bfile(&mut self, ptr: i64) -> Result<i64, EvalError> {
        let (readable, writable) = self.bfile_permissions(ptr)?;
        self.alloc_bfile(BFile {
            kind: BFileKind::Utf8 {
                inner: ptr,
                unget: None,
                pending: Vec::new(),
                pending_pos: 0,
            },
            readable,
            writable,
        })
    }

    fn add_crlf_bfile(&mut self, ptr: i64) -> Result<i64, EvalError> {
        let (readable, writable) = self.bfile_permissions(ptr)?;
        self.alloc_bfile(BFile {
            kind: BFileKind::Crlf { inner: ptr },
            readable,
            writable,
        })
    }

    fn add_rle_bfile(&mut self, ptr: i64, read: bool) -> Result<i64, EvalError> {
        let (inner_readable, inner_writable) = self.bfile_permissions(ptr)?;
        self.alloc_bfile(BFile {
            kind: BFileKind::Rle {
                inner: ptr,
                read,
                count: 0,
                byte: -1,
                unget: None,
            },
            readable: read && inner_readable,
            writable: !read && inner_writable,
        })
    }

    fn add_base64_bfile(&mut self, ptr: i64, read: bool) -> Result<i64, EvalError> {
        let (inner_readable, inner_writable) = self.bfile_permissions(ptr)?;
        self.alloc_bfile(BFile {
            kind: BFileKind::Base64 {
                inner: ptr,
                read,
                encbuf: [0; 3],
                encpos: 0,
                linelen: if read { 0 } else { 76 },
                outcol: 0,
                unget: None,
                outbuf: [0; 3],
                outpos: 0,
                outlen: 0,
            },
            readable: read && inner_readable,
            writable: !read && inner_writable,
        })
    }

    fn add_lz77_bfile(&mut self, ptr: i64, read: bool) -> Result<i64, EvalError> {
        let (inner_readable, inner_writable) = self.bfile_permissions(ptr)?;
        if read {
            if !inner_readable {
                return Err(EvalError::InvalidHandle);
            }
            let magic = self.read_bfile_bytes(ptr, 3)?;
            if magic != b"LZ1" {
                return Err(EvalError::InvalidByteString);
            }
            let len = self.read_bfile_u32_le(ptr)?;
            let compressed = self.read_bfile_bytes(ptr, len)?;
            if compressed.len() != len {
                return Err(EvalError::InvalidByteString);
            }
            let buffer = lz77_decompress(&compressed)?;
            self.alloc_bfile(BFile {
                kind: BFileKind::Lz77 {
                    inner: ptr,
                    read: true,
                    buffer,
                    pos: 0,
                    numflush: 0,
                },
                readable: true,
                writable: false,
            })
        } else {
            if !inner_writable {
                return Err(EvalError::InvalidHandle);
            }
            self.alloc_bfile(BFile {
                kind: BFileKind::Lz77 {
                    inner: ptr,
                    read: false,
                    buffer: Vec::with_capacity(25_000),
                    pos: 0,
                    numflush: 0,
                },
                readable: false,
                writable: true,
            })
        }
    }

    fn read_bfile_u32_le(&mut self, ptr: i64) -> Result<usize, EvalError> {
        let bytes = self.read_bfile_bytes(ptr, 4)?;
        let bytes: [u8; 4] = bytes.try_into().map_err(|_| EvalError::InvalidByteString)?;
        usize::try_from(u32::from_le_bytes(bytes)).map_err(|_| EvalError::Overflow)
    }

    fn add_bwt_bfile(&mut self, ptr: i64, read: bool) -> Result<i64, EvalError> {
        let (inner_readable, inner_writable) = self.bfile_permissions(ptr)?;
        if read {
            if !inner_readable {
                return Err(EvalError::InvalidHandle);
            }
            let magic = self.read_bfile_bytes(ptr, 3)?;
            if magic != b"BW1" {
                return Err(EvalError::InvalidByteString);
            }
            let len = self.read_bfile_u32_le(ptr)?;
            let zero = self.read_bfile_u32_le(ptr)?;
            let last = self.read_bfile_bytes(ptr, len)?;
            if last.len() != len {
                return Err(EvalError::InvalidByteString);
            }
            let buffer = bwt_decode(&last, zero)?;
            self.alloc_bfile(BFile {
                kind: BFileKind::Bwt {
                    inner: ptr,
                    read: true,
                    buffer,
                    pos: 0,
                    numflush: 0,
                },
                readable: true,
                writable: false,
            })
        } else {
            if !inner_writable {
                return Err(EvalError::InvalidHandle);
            }
            self.alloc_bfile(BFile {
                kind: BFileKind::Bwt {
                    inner: ptr,
                    read: false,
                    buffer: Vec::with_capacity(25_000),
                    pos: 0,
                    numflush: 0,
                },
                readable: false,
                writable: true,
            })
        }
    }

    fn add_lzma_bfile(&mut self, ptr: i64, read: bool) -> Result<i64, EvalError> {
        let (inner_readable, inner_writable) = self.bfile_permissions(ptr)?;
        if read {
            if !inner_readable {
                return Err(EvalError::InvalidHandle);
            }
            let magic = self.read_bfile_bytes(ptr, 3)?;
            if magic != b"LZ2" {
                return Err(EvalError::InvalidByteString);
            }
            let len = self.read_bfile_u32_le(ptr)?;
            let compressed = self.read_bfile_bytes(ptr, len)?;
            if compressed.len() != len {
                return Err(EvalError::InvalidByteString);
            }
            let buffer = lzma_decompress_payload(&compressed)?;
            self.alloc_bfile(BFile {
                kind: BFileKind::Lzma {
                    inner: ptr,
                    read: true,
                    buffer,
                    pos: 0,
                    numflush: 0,
                },
                readable: true,
                writable: false,
            })
        } else {
            if !inner_writable {
                return Err(EvalError::InvalidHandle);
            }
            self.alloc_bfile(BFile {
                kind: BFileKind::Lzma {
                    inner: ptr,
                    read: false,
                    buffer: Vec::with_capacity(25_000),
                    pos: 0,
                    numflush: 0,
                },
                readable: false,
                writable: true,
            })
        }
    }

    fn add_buf_bfile(&mut self, ptr: i64, bufsize: i64) -> Result<i64, EvalError> {
        let (readable, writable) = self.bfile_permissions(ptr)?;
        let linebuf = bufsize < 0;
        let size = if linebuf {
            bufsize.checked_neg().ok_or(EvalError::Overflow)?
        } else {
            bufsize
        };
        let size = int_to_usize(size)?;
        self.alloc_bfile(BFile {
            kind: BFileKind::Buf {
                inner: ptr,
                unget: None,
                buffer: vec![0; size],
                cur: 0,
                pos: 0,
                linebuf,
                read: false,
            },
            readable,
            writable,
        })
    }

    fn pointer_for_node(&mut self, id: NodeId, offset: usize) -> Result<i64, EvalError> {
        let offset = i64::try_from(offset).map_err(|_| EvalError::Overflow)?;
        if offset >= NODE_PTR_STRIDE {
            return Err(EvalError::Overflow);
        }
        let slot = if let Some(slot) = self.node_pointer_slots.get(&id) {
            *slot
        } else {
            let slot = self.node_pointers.len();
            self.node_pointers.push(id);
            self.node_pointer_slots.insert(id, slot);
            slot
        };
        let slot_word = slot
            .checked_add(1)
            .and_then(|slot| i64::try_from(slot).ok())
            .ok_or(EvalError::Overflow)?;
        if slot_word >= (1_i64 << 31) {
            return Err(EvalError::Overflow);
        }
        slot_word
            .checked_mul(NODE_PTR_STRIDE)
            .and_then(|base| base.checked_add(offset))
            .ok_or(EvalError::Overflow)
    }

    fn pointer_for_allocation(&self, slot: usize, offset: usize) -> Result<i64, EvalError> {
        let slot = i64::try_from(slot).map_err(|_| EvalError::Overflow)?;
        let offset = i64::try_from(offset).map_err(|_| EvalError::Overflow)?;
        if offset >= ALLOCATION_PTR_STRIDE {
            return Err(EvalError::Overflow);
        }
        let ptr = ALLOCATION_PTR_BASE
            .checked_add(
                slot.checked_mul(ALLOCATION_PTR_STRIDE)
                    .and_then(|raw| raw.checked_add(offset))
                    .ok_or(EvalError::Overflow)?,
            )
            .ok_or(EvalError::Overflow)?;
        if ptr >= 0 {
            return Err(EvalError::Overflow);
        }
        Ok(ptr)
    }

    fn pointer_for_bfile(&self, slot: usize) -> Result<i64, EvalError> {
        let slot = i64::try_from(slot).map_err(|_| EvalError::Overflow)?;
        BFILE_PTR_BASE
            .checked_add(
                slot.checked_mul(BFILE_PTR_STRIDE)
                    .ok_or(EvalError::Overflow)?,
            )
            .filter(|ptr| *ptr < DIR_PTR_BASE)
            .ok_or(EvalError::Overflow)
    }

    fn pointer_for_dir(&self, slot: usize) -> Result<i64, EvalError> {
        let slot = i64::try_from(slot).map_err(|_| EvalError::Overflow)?;
        DIR_PTR_BASE
            .checked_add(
                slot.checked_mul(DIR_PTR_STRIDE)
                    .ok_or(EvalError::Overflow)?,
            )
            .filter(|ptr| *ptr < ALLOCATION_PTR_BASE)
            .ok_or(EvalError::Overflow)
    }

    fn decode_pointer(&self, ptr: i64) -> Result<(usize, usize), EvalError> {
        if ptr <= 0 {
            return Err(trace_invalid_bytes!(self, "decode_pointer ptr={ptr}"));
        }
        let slot_word = usize::try_from(ptr >> 32)
            .map_err(|_| trace_invalid_bytes!(self, "decode_pointer block ptr={ptr}"))?;
        if slot_word == 0 {
            return Err(trace_invalid_bytes!(
                self,
                "decode_pointer zero slot ptr={ptr}"
            ));
        }
        let offset = usize::try_from(ptr & 0xffff_ffff)
            .map_err(|_| trace_invalid_bytes!(self, "decode_pointer offset ptr={ptr}"))?;
        let slot = slot_word - 1;
        let block = self.node_pointers.get(slot).copied().ok_or_else(|| {
            trace_invalid_bytes!(
                self,
                "decode_pointer missing node slot ptr={ptr} slot={slot} slots={}",
                self.node_pointers.len()
            )
        })?;
        Ok((block.index(), offset))
    }

    fn decode_allocation_pointer(&self, ptr: i64) -> Result<(usize, usize), EvalError> {
        if ptr < ALLOCATION_PTR_BASE || ptr >= 0 {
            return Err(trace_invalid_bytes!(
                self,
                "decode_allocation_pointer ptr={ptr}"
            ));
        }
        let raw = ptr
            .checked_sub(ALLOCATION_PTR_BASE)
            .ok_or(EvalError::Overflow)?;
        let slot = usize::try_from(raw / ALLOCATION_PTR_STRIDE)
            .map_err(|_| trace_invalid_bytes!(self, "allocation slot ptr={ptr} raw={raw}"))?;
        let offset = usize::try_from(raw % ALLOCATION_PTR_STRIDE)
            .map_err(|_| trace_invalid_bytes!(self, "allocation offset ptr={ptr} raw={raw}"))?;
        let bytes = self
            .allocations
            .get(slot)
            .and_then(Option::as_ref)
            .ok_or_else(|| {
                trace_invalid_bytes!(self, "allocation missing ptr={ptr} slot={slot}")
            })?;
        if offset > bytes.len() {
            return Err(trace_invalid_bytes!(
                self,
                "allocation offset out of range ptr={ptr} slot={slot} offset={offset} len={}",
                bytes.len()
            ));
        }
        Ok((slot, offset))
    }

    fn decode_bfile_pointer(&self, ptr: i64) -> Result<usize, EvalError> {
        if !(BFILE_PTR_BASE..DIR_PTR_BASE).contains(&ptr) {
            return Err(EvalError::InvalidHandle);
        }
        let raw = ptr.checked_sub(BFILE_PTR_BASE).ok_or(EvalError::Overflow)?;
        if raw % BFILE_PTR_STRIDE != 0 {
            return Err(EvalError::InvalidHandle);
        }
        usize::try_from(raw / BFILE_PTR_STRIDE).map_err(|_| EvalError::InvalidHandle)
    }

    fn decode_dir_pointer(&self, ptr: i64) -> Result<usize, EvalError> {
        if !(DIR_PTR_BASE..ALLOCATION_PTR_BASE).contains(&ptr) {
            return Err(EvalError::InvalidHandle);
        }
        let raw = ptr.checked_sub(DIR_PTR_BASE).ok_or(EvalError::Overflow)?;
        if raw % DIR_PTR_STRIDE != 0 {
            return Err(EvalError::InvalidHandle);
        }
        usize::try_from(raw / DIR_PTR_STRIDE).map_err(|_| EvalError::InvalidHandle)
    }

    fn allocation_bytes(&self, ptr: i64) -> Result<Option<&[u8]>, EvalError> {
        if ptr < ALLOCATION_PTR_BASE || ptr >= 0 {
            return Ok(None);
        }
        let (slot, offset) = self.decode_allocation_pointer(ptr)?;
        let bytes = self
            .allocations
            .get(slot)
            .and_then(Option::as_ref)
            .ok_or_else(|| {
                trace_invalid_bytes!(self, "allocation read missing ptr={ptr} slot={slot}")
            })?;
        Ok(Some(&bytes[offset..]))
    }

    fn pointer_bytes(&self, ptr: i64) -> Result<&[u8], EvalError> {
        if let Some(bytes) = self.allocation_bytes(ptr)? {
            return Ok(bytes);
        }
        let (base, offset) = self.decode_pointer(ptr)?;
        if base >= self.nodes.len() {
            return Err(trace_invalid_bytes!(
                self,
                "pointer base missing ptr={ptr} base={base} offset={offset} nodes={}",
                self.nodes.len()
            ));
        };
        let node_id = NodeId::from_index(base);
        let bytes = match self.cold_node(node_id) {
            Some(Node::Bytes(bytes)) => bytes.as_slice(),
            Some(Node::BytesView(_)) => self.bytes(node_id)?,
            Some(Node::MutableBytes(bytes)) => bytes.bytes.as_slice(),
            Some(Node::ForeignPtr(foreign_ptr)) if foreign_ptr.bytes.is_some() => {
                let bytes = foreign_ptr.bytes.as_ref().expect("checked above");
                let offset = foreign_ptr
                    .offset
                    .checked_add(offset)
                    .ok_or(EvalError::Overflow)?;
                return bytes.get(offset..).ok_or_else(|| {
                    trace_invalid_bytes!(
                        self,
                        "foreign pointer offset out of range ptr={ptr} base={base} offset={offset} len={}",
                        bytes.len()
                    )
                });
            }
            _ => {
                return Err(trace_invalid_bytes!(
                    self,
                    "pointer base not bytes ptr={ptr} base={base} offset={offset} kind={}",
                    self.profile_head_key(NodeId::from_index(base))
                ));
            }
        };
        if offset > bytes.len() {
            return Err(trace_invalid_bytes!(
                self,
                "pointer offset out of range ptr={ptr} base={base} offset={offset} len={}",
                bytes.len()
            ));
        }
        Ok(&bytes[offset..])
    }

    fn write_pointer_bytes(&mut self, ptr: i64, bytes: &[u8]) -> Result<(), EvalError> {
        if ptr >= ALLOCATION_PTR_BASE && ptr < 0 {
            let (slot, offset) = self.decode_allocation_pointer(ptr)?;
            let write_len = bytes.len();
            let available = self.allocations[slot]
                .as_ref()
                .expect("checked allocation slot")
                .len()
                .saturating_sub(offset);
            if available < write_len {
                return Err(trace_invalid_bytes!(
                    self,
                    "allocation write too short ptr={ptr} slot={slot} offset={offset} write_len={write_len} available={available}"
                ));
            }
            let dst = self.allocations[slot]
                .as_mut()
                .expect("checked allocation slot");
            dst[offset..offset + write_len].copy_from_slice(bytes);
            return Ok(());
        }
        let (base, offset) = self.decode_pointer(ptr)?;
        if base >= self.nodes.len() {
            return Err(trace_invalid_bytes!(
                self,
                "write pointer base missing ptr={ptr} base={base} offset={offset} nodes={}",
                self.nodes.len()
            ));
        }
        let write_len = bytes.len();
        let node_id = NodeId::from_index(base);
        if matches!(self.cold_node(node_id), Some(Node::BytesView(_))) {
            let current = self.bytes(node_id)?;
            let available = current.len().saturating_sub(offset);
            if offset > current.len() || available < write_len {
                return Err(trace_invalid_bytes!(
                    self,
                    "write bytes view out of range ptr={ptr} base={base} offset={offset} write_len={write_len} len={}",
                    current.len()
                ));
            }
            let mut owned = current.to_vec();
            owned[offset..offset + write_len].copy_from_slice(bytes);
            self.set_node_at(node_id.index(), Node::bytes(owned));
            return Ok(());
        }
        let kind = self.profile_head_key(NodeId::from_index(base));
        let is_mutable_bytes = match self.cold_node(node_id) {
            Some(Node::Bytes(dst)) => {
                let available = dst.len().saturating_sub(offset);
                if offset > dst.len() || available < write_len {
                    return Err(trace_invalid_bytes!(
                        self,
                        "write bytes out of range ptr={ptr} base={base} offset={offset} write_len={write_len} len={}",
                        dst.len()
                    ));
                }
                false
            }
            Some(Node::MutableBytes(dst)) => {
                let storage_len = dst.bytes.len();
                let available = storage_len.saturating_sub(offset);
                if offset > storage_len || available < write_len {
                    return Err(trace_invalid_bytes!(
                        self,
                        "write mutable bytes out of range ptr={ptr} base={base} offset={offset} write_len={write_len} storage_len={storage_len}"
                    ));
                }
                true
            }
            _ => {
                return Err(trace_invalid_bytes!(
                    self,
                    "write pointer base not bytes ptr={ptr} base={base} offset={offset} kind={}",
                    kind
                ));
            }
        };
        let dst = match self.cold_node_mut(node_id) {
            Some(Node::Bytes(dst)) if !is_mutable_bytes => &mut dst[offset..offset + write_len],
            Some(Node::MutableBytes(dst)) if is_mutable_bytes => {
                &mut dst.bytes[offset..offset + write_len]
            }
            _ => unreachable!("validated pointer target changed during write"),
        };
        dst.copy_from_slice(bytes);
        Ok(())
    }

    fn peek_array<const N: usize>(&self, ptr: i64) -> Result<[u8; N], EvalError> {
        self.read_pointer_bytes(ptr, N)?
            .try_into()
            .map_err(|_| EvalError::InvalidByteString)
    }

    fn peek_unsigned(&self, ptr: i64, size: usize) -> Result<u64, EvalError> {
        if size == 0 || size > 8 {
            return Err(EvalError::Overflow);
        }
        let bytes = self.read_pointer_bytes(ptr, size)?;
        let mut wide = [0; 8];
        if cfg!(target_endian = "little") {
            wide[..size].copy_from_slice(&bytes);
        } else {
            wide[8 - size..].copy_from_slice(&bytes);
        }
        Ok(u64::from_ne_bytes(wide))
    }

    fn poke_unsigned(&mut self, ptr: i64, size: usize, value: u64) -> Result<(), EvalError> {
        if size == 0 || size > 8 {
            return Err(EvalError::Overflow);
        }
        let wide = value.to_ne_bytes();
        let bytes = if cfg!(target_endian = "little") {
            &wide[..size]
        } else {
            &wide[8 - size..]
        };
        self.write_pointer_bytes(ptr, bytes)
    }

    fn peek_signed(&self, ptr: i64, size: usize) -> Result<i64, EvalError> {
        let unsigned = self.peek_unsigned(ptr, size)?;
        let shift = (8 - size) * 8;
        Ok(((unsigned << shift) as i64) >> shift)
    }

    fn poke_signed(&mut self, ptr: i64, size: usize, value: i64) -> Result<(), EvalError> {
        if size == 0 || size > 8 {
            return Err(EvalError::Overflow);
        }
        let wide = value.to_ne_bytes();
        let bytes = if cfg!(target_endian = "little") {
            &wide[..size]
        } else {
            &wide[8 - size..]
        };
        self.write_pointer_bytes(ptr, bytes)
    }

    fn peek_c_char(&self, ptr: i64) -> Result<i64, EvalError> {
        if std::os::raw::c_char::MIN < 0 {
            self.peek_signed(ptr, size_of::<std::os::raw::c_char>())
        } else {
            Ok(self.peek_unsigned(ptr, size_of::<std::os::raw::c_char>())? as i64)
        }
    }

    fn poke_c_char(&mut self, ptr: i64, value: i64) -> Result<(), EvalError> {
        if std::os::raw::c_char::MIN < 0 {
            self.poke_signed(ptr, size_of::<std::os::raw::c_char>(), value)
        } else {
            self.poke_unsigned(ptr, size_of::<std::os::raw::c_char>(), value as u64)
        }
    }

    fn read_pointer_bytes(&self, ptr: i64, len: usize) -> Result<Vec<u8>, EvalError> {
        let bytes = self.pointer_bytes(ptr)?;
        let bytes = bytes.get(..len).ok_or_else(|| {
            trace_invalid_bytes!(
                self,
                "read pointer too short ptr={ptr} len={len} available={}",
                bytes.len()
            )
        })?;
        Ok(bytes.to_vec())
    }

    fn read_c_string(&self, ptr: i64) -> Result<Vec<u8>, EvalError> {
        let bytes = self.pointer_bytes(ptr)?;
        let len = c_string_len(bytes);
        Ok(bytes[..len].to_vec())
    }

    fn c_string_len(&self, ptr: i64) -> Result<usize, EvalError> {
        Ok(c_string_len(self.pointer_bytes(ptr)?))
    }

    fn bfile(&self, ptr: i64) -> Result<&BFile, EvalError> {
        let slot = self.decode_bfile_pointer(ptr)?;
        self.bfiles
            .get(slot)
            .and_then(Option::as_ref)
            .ok_or(EvalError::InvalidHandle)
    }

    fn bfile_mut(&mut self, ptr: i64) -> Result<&mut BFile, EvalError> {
        let slot = self.decode_bfile_pointer(ptr)?;
        self.bfiles
            .get_mut(slot)
            .and_then(Option::as_mut)
            .ok_or(EvalError::InvalidHandle)
    }

    fn read_only_memory_view_state(
        &self,
        ptr: i64,
    ) -> Result<(NodeId, usize, usize, usize), EvalError> {
        let bfile = self.bfile(ptr)?;
        if !bfile.readable {
            return Err(EvalError::InvalidHandle);
        }
        match &bfile.kind {
            BFileKind::ReadOnlyMemoryView {
                base,
                offset,
                len,
                pos,
            } => Ok((*base, *offset, *len, *pos)),
            _ => Err(EvalError::InvalidHandle),
        }
    }

    fn get_read_only_memory_view_byte(&mut self, ptr: i64) -> Result<i64, EvalError> {
        let (base, offset, len, pos) = self.read_only_memory_view_state(ptr)?;
        if pos >= len {
            return Ok(-1);
        }
        let index = offset.checked_add(pos).ok_or(EvalError::Overflow)?;
        let byte = self
            .bytes(base)?
            .get(index)
            .copied()
            .ok_or(EvalError::InvalidByteString)?;
        let bfile = self.bfile_mut(ptr)?;
        match &mut bfile.kind {
            BFileKind::ReadOnlyMemoryView { pos, .. } => {
                *pos += 1;
                Ok(i64::from(byte))
            }
            _ => Err(EvalError::InvalidHandle),
        }
    }

    fn read_only_memory_view_bytes(&mut self, ptr: i64, len: usize) -> Result<Vec<u8>, EvalError> {
        let (base, offset, view_len, current_pos) = self.read_only_memory_view_state(ptr)?;
        let read_len = len.min(view_len.saturating_sub(current_pos));
        let start = offset.checked_add(current_pos).ok_or(EvalError::Overflow)?;
        let end = start.checked_add(read_len).ok_or(EvalError::Overflow)?;
        let bytes = self
            .bytes(base)?
            .get(start..end)
            .ok_or(EvalError::InvalidByteString)?
            .to_vec();
        let bfile = self.bfile_mut(ptr)?;
        match &mut bfile.kind {
            BFileKind::ReadOnlyMemoryView { pos, .. } => {
                *pos = current_pos
                    .checked_add(read_len)
                    .ok_or(EvalError::Overflow)?;
                Ok(bytes)
            }
            _ => Err(EvalError::InvalidHandle),
        }
    }

    fn unget_read_only_memory_view_byte(&mut self, ptr: i64, byte: i64) -> Result<(), EvalError> {
        let (base, offset, _len, pos) = self.read_only_memory_view_state(ptr)?;
        if pos == 0 {
            return Err(EvalError::InvalidHandle);
        }
        let index = offset.checked_add(pos - 1).ok_or(EvalError::Overflow)?;
        let expected = self
            .bytes(base)?
            .get(index)
            .copied()
            .ok_or(EvalError::InvalidByteString)?;
        if expected != byte as u8 {
            return Err(EvalError::InvalidHandle);
        }
        let bfile = self.bfile_mut(ptr)?;
        match &mut bfile.kind {
            BFileKind::ReadOnlyMemoryView { pos, .. } => {
                *pos -= 1;
                Ok(())
            }
            _ => Err(EvalError::InvalidHandle),
        }
    }

    fn close_bfile(&mut self, ptr: i64) -> Result<(), EvalError> {
        if let Some(handle) = handle_from_ptr(ptr) {
            return self.flush_io_handle(handle);
        }
        let slot = self.decode_bfile_pointer(ptr)?;
        let flush_self = {
            let bfile = self
                .bfiles
                .get(slot)
                .and_then(Option::as_ref)
                .ok_or(EvalError::InvalidHandle)?;
            matches!(
                &bfile.kind,
                BFileKind::Buf { read: false, .. }
                    | BFileKind::Rle { read: false, .. }
                    | BFileKind::Base64 { read: false, .. }
                    | BFileKind::Lz77 { read: false, .. }
                    | BFileKind::Bwt { read: false, .. }
                    | BFileKind::Lzma { read: false, .. }
            )
        };
        if flush_self {
            self.flush_bfile(ptr)?;
        }
        let close_extra = {
            let bfile = self
                .bfiles
                .get_mut(slot)
                .and_then(Option::as_mut)
                .ok_or(EvalError::InvalidHandle)?;
            match &mut bfile.kind {
                BFileKind::Base64 {
                    inner,
                    read,
                    linelen,
                    outcol,
                    ..
                } if !*read && *linelen != 0 && *outcol != 0 => {
                    *outcol = 0;
                    Some((*inner, vec![b'\n']))
                }
                _ => None,
            }
        };
        if let Some((inner, bytes)) = close_extra {
            let written = self.write_bfile_bytes(inner, &bytes)?;
            if written != bytes.len() {
                return Err(EvalError::InvalidHandle);
            }
        }
        let close_inner = {
            let bfile = self
                .bfiles
                .get(slot)
                .and_then(Option::as_ref)
                .ok_or(EvalError::InvalidHandle)?;
            match &bfile.kind {
                BFileKind::Utf8 { inner, .. }
                | BFileKind::Crlf { inner }
                | BFileKind::Rle { inner, .. }
                | BFileKind::Base64 { inner, .. }
                | BFileKind::Lz77 { inner, .. }
                | BFileKind::Bwt { inner, .. }
                | BFileKind::Lzma { inner, .. }
                | BFileKind::Buf { inner, .. } => Some(*inner),
                _ => None,
            }
        };
        if let Some(inner) = close_inner {
            self.close_bfile(inner)?;
        }
        let slot = self.bfiles.get_mut(slot).ok_or(EvalError::InvalidHandle)?;
        let _bfile = slot.as_ref().ok_or(EvalError::InvalidHandle)?;
        #[cfg(any(not(target_arch = "wasm32"), target_os = "wasi"))]
        if let BFileKind::NativeFile { file, .. } = &_bfile.kind {
            use std::io::Write as _;

            if _bfile.writable {
                file.borrow_mut()
                    .flush()
                    .map_err(|_| EvalError::InvalidHandle)?;
            }
        }
        #[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
        if let BFileKind::BrowserFile { handle, .. } = &_bfile.kind {
            if _bfile.writable {
                let rc = unsafe { mhs_host_file_flush(*handle) };
                if rc < 0 {
                    return Err(EvalError::InvalidHandle);
                }
            }
            let rc = unsafe { mhs_host_file_close(*handle) };
            if rc < 0 {
                return Err(EvalError::InvalidHandle);
            }
        }
        *slot = None;
        Ok(())
    }

    fn read_dir_entry(&mut self, ptr: i64) -> Result<i64, EvalError> {
        let slot = self.decode_dir_pointer(ptr)?;
        let name = {
            let dir = self
                .dirs
                .get_mut(slot)
                .and_then(Option::as_mut)
                .ok_or(EvalError::InvalidHandle)?;
            let Some(name) = dir.entries.get(dir.pos) else {
                return Ok(0);
            };
            dir.pos += 1;
            name.clone()
        };
        let mut bytes = name;
        bytes.push(0);
        let ptr = self.alloc_memory(bytes.len())?;
        self.write_pointer_bytes(ptr, &bytes)?;
        Ok(ptr)
    }

    fn close_dir(&mut self, ptr: i64) -> Result<(), EvalError> {
        let slot = self.decode_dir_pointer(ptr)?;
        let slot = self.dirs.get_mut(slot).ok_or(EvalError::InvalidHandle)?;
        if slot.is_none() {
            return Err(EvalError::InvalidHandle);
        }
        *slot = None;
        Ok(())
    }

    fn flush_bfile(&mut self, ptr: i64) -> Result<(), EvalError> {
        if let Some(handle) = handle_from_ptr(ptr) {
            return self.flush_io_handle(handle);
        }
        let flush_inner = {
            let bfile = self.bfile_mut(ptr)?;
            match &mut bfile.kind {
                BFileKind::Utf8 { inner, .. } => Some((*inner, Vec::new())),
                BFileKind::Crlf { inner } => Some((*inner, Vec::new())),
                BFileKind::Rle {
                    inner,
                    read,
                    count,
                    byte,
                    ..
                } => {
                    if *read {
                        None
                    } else {
                        let bytes = rle_pending_bytes(*count, *byte)?;
                        *count = 0;
                        Some((*inner, bytes))
                    }
                }
                BFileKind::Base64 {
                    inner,
                    read,
                    encbuf,
                    encpos,
                    linelen,
                    outcol,
                    ..
                } => {
                    if *read {
                        None
                    } else {
                        let bytes = base64_pending_bytes(encbuf, encpos, *linelen, outcol)?;
                        *encpos = 0;
                        Some((*inner, bytes))
                    }
                }
                BFileKind::Lz77 {
                    inner,
                    read,
                    buffer,
                    pos,
                    numflush,
                } => {
                    if *read {
                        None
                    } else if *numflush > 0 && *pos == 0 {
                        *numflush = numflush.checked_add(1).ok_or(EvalError::Overflow)?;
                        None
                    } else {
                        *numflush = numflush.checked_add(1).ok_or(EvalError::Overflow)?;
                        let compressed = lz77_compress(&buffer[..*pos])?;
                        let mut bytes = Vec::with_capacity(7 + compressed.len());
                        bytes.extend_from_slice(b"LZ1");
                        bytes.extend_from_slice(&u32_le_bytes(compressed.len())?);
                        bytes.extend_from_slice(&compressed);
                        buffer.clear();
                        *pos = 0;
                        Some((*inner, bytes))
                    }
                }
                BFileKind::Bwt {
                    inner,
                    read,
                    buffer,
                    pos,
                    numflush,
                } => {
                    if *read {
                        None
                    } else if *numflush > 0 && *pos == 0 {
                        *numflush = numflush.checked_add(1).ok_or(EvalError::Overflow)?;
                        None
                    } else {
                        *numflush = numflush.checked_add(1).ok_or(EvalError::Overflow)?;
                        let (zero, last) = bwt_encode(&buffer[..*pos])?;
                        let mut bytes = Vec::with_capacity(11 + last.len());
                        bytes.extend_from_slice(b"BW1");
                        bytes.extend_from_slice(&u32_le_bytes(*pos)?);
                        bytes.extend_from_slice(&u32_le_bytes(zero)?);
                        bytes.extend_from_slice(&last);
                        buffer.clear();
                        *pos = 0;
                        Some((*inner, bytes))
                    }
                }
                BFileKind::Lzma {
                    inner,
                    read,
                    buffer,
                    pos,
                    numflush,
                } => {
                    if *read {
                        None
                    } else if *numflush > 0 && *pos == 0 {
                        *numflush = numflush.checked_add(1).ok_or(EvalError::Overflow)?;
                        None
                    } else {
                        *numflush = numflush.checked_add(1).ok_or(EvalError::Overflow)?;
                        let compressed = lzma_compress_payload(&buffer[..*pos])?;
                        let mut bytes = Vec::with_capacity(7 + compressed.len());
                        bytes.extend_from_slice(b"LZ2");
                        bytes.extend_from_slice(&u32_le_bytes(compressed.len())?);
                        bytes.extend_from_slice(&compressed);
                        buffer.clear();
                        *pos = 0;
                        Some((*inner, bytes))
                    }
                }
                BFileKind::Buf {
                    inner,
                    buffer,
                    pos,
                    read,
                    ..
                } => {
                    let bytes = if !*read && *pos > 0 {
                        let bytes = buffer[..*pos].to_vec();
                        *pos = 0;
                        bytes
                    } else {
                        Vec::new()
                    };
                    Some((*inner, bytes))
                }
                _ => None,
            }
        };
        if let Some((inner, bytes)) = flush_inner {
            if !bytes.is_empty() {
                let written = self.write_bfile_bytes(inner, &bytes)?;
                if written != bytes.len() {
                    return Err(EvalError::InvalidHandle);
                }
            }
            return self.flush_bfile(inner);
        }
        let _bfile = self.bfile(ptr)?;
        #[cfg(any(not(target_arch = "wasm32"), target_os = "wasi"))]
        if let BFileKind::NativeFile { file, .. } = &_bfile.kind {
            use std::io::Write as _;

            if _bfile.writable {
                file.borrow_mut()
                    .flush()
                    .map_err(|_| EvalError::InvalidHandle)?;
            }
        }
        #[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
        if let BFileKind::BrowserFile { handle, .. } = &_bfile.kind {
            if _bfile.writable {
                let rc = unsafe { mhs_host_file_flush(*handle) };
                if rc < 0 {
                    return Err(EvalError::InvalidHandle);
                }
            }
        }
        Ok(())
    }

    fn get_bfile_byte(&mut self, ptr: i64) -> Result<i64, EvalError> {
        if handle_from_ptr(ptr) == Some(StdHandle::Stdin) {
            return self.read_stdin_byte();
        }
        enum SpecialBFileRead {
            ReadOnlyMemoryView,
            Utf8(i64, i64),
            Crlf(i64),
            Rle,
            Base64,
            Buf,
        }
        let special = {
            let bfile = self.bfile_mut(ptr)?;
            if !bfile.readable {
                return Err(EvalError::InvalidHandle);
            }
            match &mut bfile.kind {
                BFileKind::ReadOnlyMemoryView { .. } => Some(SpecialBFileRead::ReadOnlyMemoryView),
                BFileKind::Utf8 {
                    inner,
                    unget,
                    pending,
                    pending_pos,
                } => {
                    if let Some(byte) = unget.take() {
                        return Ok(byte);
                    }
                    if *pending_pos < pending.len() {
                        let byte = pending[*pending_pos];
                        *pending_pos += 1;
                        if *pending_pos == pending.len() {
                            pending.clear();
                            *pending_pos = 0;
                        }
                        return Ok(i64::from(byte));
                    }
                    pending.clear();
                    *pending_pos = 0;
                    Some(SpecialBFileRead::Utf8(ptr, *inner))
                }
                BFileKind::Crlf { inner } => Some(SpecialBFileRead::Crlf(*inner)),
                BFileKind::Rle { read, .. } if *read => Some(SpecialBFileRead::Rle),
                BFileKind::Base64 { read, .. } if *read => Some(SpecialBFileRead::Base64),
                BFileKind::Buf { .. } => Some(SpecialBFileRead::Buf),
                _ => None,
            }
        };
        match special {
            Some(SpecialBFileRead::ReadOnlyMemoryView) => {
                return self.get_read_only_memory_view_byte(ptr);
            }
            Some(SpecialBFileRead::Utf8(ptr, inner)) => {
                return self.get_utf8_bfile_byte(ptr, inner);
            }
            Some(SpecialBFileRead::Crlf(inner)) => return self.get_crlf_bfile_byte(inner),
            Some(SpecialBFileRead::Rle) => return self.get_rle_bfile_byte(ptr),
            Some(SpecialBFileRead::Base64) => return self.get_base64_bfile_byte(ptr),
            Some(SpecialBFileRead::Buf) => return self.get_buf_bfile_byte(ptr),
            None => {}
        }
        let bfile = self.bfile_mut(ptr)?;
        match &mut bfile.kind {
            BFileKind::Memory { bytes, pos } => {
                if *pos >= bytes.len() {
                    return Ok(-1);
                }
                let byte = bytes[*pos];
                *pos += 1;
                Ok(i64::from(byte))
            }
            BFileKind::ReadOnlyMemoryView { .. } => unreachable!("handled above"),
            #[cfg(any(not(target_arch = "wasm32"), target_os = "wasi"))]
            BFileKind::NativeFile { file, ungot } => {
                if let Some(byte) = ungot.pop() {
                    return Ok(i64::from(byte));
                }
                use std::io::Read as _;

                #[cfg(target_os = "wasi")]
                wasi_trace_every("getb_native", 8192);
                let mut byte = [0];
                match file.borrow_mut().read(&mut byte) {
                    Ok(0) => Ok(-1),
                    Ok(_) => Ok(i64::from(byte[0])),
                    Err(_) => Err(EvalError::InvalidHandle),
                }
            }
            #[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
            BFileKind::BrowserFile { handle, ungot } => {
                if let Some(byte) = ungot.pop() {
                    return Ok(i64::from(byte));
                }
                let mut byte = [0];
                let read = unsafe { mhs_host_file_read(*handle, byte.as_mut_ptr(), 1) };
                if read < 0 {
                    Err(EvalError::InvalidHandle)
                } else if read == 0 {
                    Ok(-1)
                } else {
                    Ok(i64::from(byte[0]))
                }
            }
            BFileKind::Utf8 { .. } => unreachable!("handled above"),
            BFileKind::Crlf { .. } => unreachable!("handled above"),
            BFileKind::Rle { .. } => unreachable!("handled above"),
            BFileKind::Base64 { .. } => unreachable!("handled above"),
            BFileKind::Lz77 {
                read, buffer, pos, ..
            } => {
                if !*read {
                    return Err(EvalError::InvalidHandle);
                }
                if *pos >= buffer.len() {
                    return Ok(-1);
                }
                let byte = buffer[*pos];
                *pos += 1;
                Ok(i64::from(byte))
            }
            BFileKind::Bwt {
                read, buffer, pos, ..
            } => {
                if !*read {
                    return Err(EvalError::InvalidHandle);
                }
                if *pos >= buffer.len() {
                    return Ok(-1);
                }
                let byte = buffer[*pos];
                *pos += 1;
                Ok(i64::from(byte))
            }
            BFileKind::Lzma {
                read, buffer, pos, ..
            } => {
                if !*read {
                    return Err(EvalError::InvalidHandle);
                }
                if *pos >= buffer.len() {
                    return Ok(-1);
                }
                let byte = buffer[*pos];
                *pos += 1;
                Ok(i64::from(byte))
            }
            BFileKind::Buf { .. } => unreachable!("handled above"),
        }
    }

    fn get_crlf_bfile_byte(&mut self, inner: i64) -> Result<i64, EvalError> {
        let byte = self.get_bfile_byte(inner)?;
        if byte != i64::from(b'\r') {
            return Ok(byte);
        }
        let next = self.get_bfile_byte(inner)?;
        if next == i64::from(b'\n') {
            return Ok(next);
        }
        if next >= 0 {
            self.unget_bfile_byte(inner, next)?;
        }
        Ok(byte)
    }

    fn get_rle_bfile_byte(&mut self, ptr: i64) -> Result<i64, EvalError> {
        let inner = {
            let bfile = self.bfile_mut(ptr)?;
            if !bfile.readable {
                return Err(EvalError::InvalidHandle);
            }
            match &mut bfile.kind {
                BFileKind::Rle {
                    inner,
                    read,
                    count,
                    byte,
                    unget,
                } => {
                    if !*read {
                        return Err(EvalError::InvalidHandle);
                    }
                    if let Some(byte) = unget.take() {
                        return Ok(byte);
                    }
                    if *count > 0 {
                        *count -= 1;
                        return Ok(*byte);
                    }
                    *inner
                }
                _ => return Err(EvalError::InvalidHandle),
            }
        };

        let Some(rep) = self.get_rle_rep(inner)? else {
            return Ok(-1);
        };
        if rep == 1 {
            let byte = self.get_bfile_byte(inner)?;
            if byte < 0 {
                return Ok(-1);
            }
            return Ok(byte | 0x80);
        }

        let byte = self.get_bfile_byte(inner)?;
        if byte < 0 {
            return Ok(-1);
        }
        let bfile = self.bfile_mut(ptr)?;
        match &mut bfile.kind {
            BFileKind::Rle {
                count, byte: out, ..
            } => {
                *count = rep;
                *out = byte;
                Ok(byte)
            }
            _ => Err(EvalError::InvalidHandle),
        }
    }

    fn get_rle_rep(&mut self, inner: i64) -> Result<Option<usize>, EvalError> {
        let mut n = 0usize;
        loop {
            let byte = self.get_bfile_byte(inner)?;
            if byte < 0 {
                return Ok(None);
            }
            if byte < 128 {
                self.unget_bfile_byte(inner, byte)?;
                return Ok(Some(n));
            }
            let digit = usize::try_from(byte - 128).map_err(|_| EvalError::Overflow)?;
            n = n
                .checked_mul(128)
                .and_then(|n| n.checked_add(digit))
                .ok_or(EvalError::Overflow)?;
        }
    }

    fn get_base64_bfile_byte(&mut self, ptr: i64) -> Result<i64, EvalError> {
        let inner = {
            let bfile = self.bfile_mut(ptr)?;
            if !bfile.readable {
                return Err(EvalError::InvalidHandle);
            }
            match &mut bfile.kind {
                BFileKind::Base64 {
                    inner,
                    read,
                    unget,
                    outbuf,
                    outpos,
                    outlen,
                    ..
                } => {
                    if !*read {
                        return Err(EvalError::InvalidHandle);
                    }
                    if let Some(byte) = unget.take() {
                        return Ok(byte);
                    }
                    if *outpos < *outlen {
                        let byte = outbuf[*outpos];
                        *outpos += 1;
                        return Ok(i64::from(byte));
                    }
                    *inner
                }
                _ => return Err(EvalError::InvalidHandle),
            }
        };

        let Some(v) = self.get_base64_quartet(inner)? else {
            return Ok(-1);
        };
        if v[0] < 0 || v[1] < 0 {
            return Err(EvalError::InvalidByteString);
        }
        let mut outbuf = [0; 3];
        let outlen;
        let mut triple = ((v[0] as u32) << 18) | ((v[1] as u32) << 12);
        if v[2] == -3 {
            outbuf[0] = ((triple >> 16) & 0xff) as u8;
            outlen = 1;
        } else {
            if v[2] < 0 {
                return Err(EvalError::InvalidByteString);
            }
            triple |= (v[2] as u32) << 6;
            if v[3] == -3 {
                outbuf[0] = ((triple >> 16) & 0xff) as u8;
                outbuf[1] = ((triple >> 8) & 0xff) as u8;
                outlen = 2;
            } else {
                if v[3] < 0 {
                    return Err(EvalError::InvalidByteString);
                }
                triple |= v[3] as u32;
                outbuf[0] = ((triple >> 16) & 0xff) as u8;
                outbuf[1] = ((triple >> 8) & 0xff) as u8;
                outbuf[2] = (triple & 0xff) as u8;
                outlen = 3;
            }
        }

        let bfile = self.bfile_mut(ptr)?;
        match &mut bfile.kind {
            BFileKind::Base64 {
                outbuf: buffer,
                outpos,
                outlen: len,
                ..
            } => {
                *buffer = outbuf;
                *outpos = 1;
                *len = outlen;
                Ok(i64::from(outbuf[0]))
            }
            _ => Err(EvalError::InvalidHandle),
        }
    }

    fn get_base64_quartet(&mut self, inner: i64) -> Result<Option<[i32; 4]>, EvalError> {
        let mut v = [0; 4];
        let mut got = 0;
        loop {
            let byte = self.get_bfile_byte(inner)?;
            if byte < 0 {
                if got == 0 {
                    return Ok(None);
                }
                return Err(EvalError::InvalidByteString);
            }
            match base64_decode_value(byte as u8) {
                Base64Input::Whitespace => continue,
                Base64Input::Invalid => return Err(EvalError::InvalidByteString),
                Base64Input::Value(value) => {
                    v[got] = value;
                    got += 1;
                    if got == 4 {
                        return Ok(Some(v));
                    }
                }
            }
        }
    }

    fn get_buf_bfile_byte(&mut self, ptr: i64) -> Result<i64, EvalError> {
        loop {
            enum BufReadAction {
                Direct(i64),
                Refill { inner: i64, size: usize },
            }
            let action = {
                let bfile = self.bfile_mut(ptr)?;
                if !bfile.readable {
                    return Err(EvalError::InvalidHandle);
                }
                match &mut bfile.kind {
                    BFileKind::Buf {
                        inner,
                        unget,
                        buffer,
                        cur,
                        pos,
                        read,
                        ..
                    } => {
                        *read = true;
                        if let Some(byte) = unget.take() {
                            return Ok(byte);
                        }
                        if buffer.is_empty() {
                            BufReadAction::Direct(*inner)
                        } else if *pos >= *cur {
                            BufReadAction::Refill {
                                inner: *inner,
                                size: buffer.len(),
                            }
                        } else {
                            let byte = buffer[*pos];
                            *pos += 1;
                            return Ok(i64::from(byte));
                        }
                    }
                    _ => return Err(EvalError::InvalidHandle),
                }
            };
            match action {
                BufReadAction::Direct(inner) => return self.get_bfile_byte(inner),
                BufReadAction::Refill { inner, size } => {
                    let bytes = self.read_bfile_bytes(inner, size)?;
                    if bytes.is_empty() {
                        return Ok(-1);
                    }
                    let bfile = self.bfile_mut(ptr)?;
                    match &mut bfile.kind {
                        BFileKind::Buf {
                            buffer, cur, pos, ..
                        } => {
                            buffer[..bytes.len()].copy_from_slice(&bytes);
                            *cur = bytes.len();
                            *pos = 0;
                        }
                        _ => return Err(EvalError::InvalidHandle),
                    }
                }
            }
        }
    }

    fn unget_bfile_byte(&mut self, ptr: i64, byte: i64) -> Result<(), EvalError> {
        enum SpecialBFileUnget {
            Crlf(i64),
            ReadOnlyMemoryView,
        }
        let special = {
            let bfile = self.bfile_mut(ptr)?;
            if !bfile.readable {
                return Err(EvalError::InvalidHandle);
            }
            match &bfile.kind {
                BFileKind::Crlf { inner } => Some(SpecialBFileUnget::Crlf(*inner)),
                BFileKind::ReadOnlyMemoryView { .. } => Some(SpecialBFileUnget::ReadOnlyMemoryView),
                _ => None,
            }
        };
        match special {
            Some(SpecialBFileUnget::Crlf(inner)) => return self.unget_bfile_byte(inner, byte),
            Some(SpecialBFileUnget::ReadOnlyMemoryView) => {
                return self.unget_read_only_memory_view_byte(ptr, byte);
            }
            None => {}
        }
        let bfile = self.bfile_mut(ptr)?;
        match &mut bfile.kind {
            BFileKind::Memory { bytes, pos } => {
                if *pos == 0 {
                    return Err(EvalError::InvalidHandle);
                }
                let byte = byte as u8;
                if bytes[*pos - 1] != byte {
                    return Err(EvalError::InvalidHandle);
                }
                *pos -= 1;
                Ok(())
            }
            BFileKind::ReadOnlyMemoryView { .. } => unreachable!("handled above"),
            #[cfg(any(not(target_arch = "wasm32"), target_os = "wasi"))]
            BFileKind::NativeFile { ungot, .. } => {
                ungot.push(byte as u8);
                Ok(())
            }
            #[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
            BFileKind::BrowserFile { ungot, .. } => {
                ungot.push(byte as u8);
                Ok(())
            }
            BFileKind::Utf8 { unget, .. } => {
                if unget.is_some() {
                    return Err(EvalError::InvalidHandle);
                }
                *unget = Some(byte);
                Ok(())
            }
            BFileKind::Crlf { .. } => unreachable!("handled above"),
            BFileKind::Rle { unget, .. } => {
                if unget.is_some() {
                    return Err(EvalError::InvalidHandle);
                }
                *unget = Some(byte);
                Ok(())
            }
            BFileKind::Base64 { unget, .. } => {
                if unget.is_some() {
                    return Err(EvalError::InvalidHandle);
                }
                *unget = Some(byte);
                Ok(())
            }
            BFileKind::Lz77 { read, pos, .. } => {
                if !*read || *pos == 0 {
                    return Err(EvalError::InvalidHandle);
                }
                *pos -= 1;
                Ok(())
            }
            BFileKind::Bwt { read, pos, .. } => {
                if !*read || *pos == 0 {
                    return Err(EvalError::InvalidHandle);
                }
                *pos -= 1;
                Ok(())
            }
            BFileKind::Lzma { read, pos, .. } => {
                if !*read || *pos == 0 {
                    return Err(EvalError::InvalidHandle);
                }
                *pos -= 1;
                Ok(())
            }
            BFileKind::Buf { unget, .. } => {
                if unget.is_some() {
                    return Err(EvalError::InvalidHandle);
                }
                *unget = Some(byte);
                Ok(())
            }
        }
    }

    fn put_bfile_byte(&mut self, ptr: i64, byte: i64) -> Result<(), EvalError> {
        if let Some(handle) = handle_from_ptr(ptr) {
            return self.write_io_handle_bytes(handle, &[byte as u8]);
        }
        enum SpecialBFileWrite {
            Utf8(i64),
            Crlf(i64),
            Rle,
            Base64,
            Buf,
        }
        let special = {
            let bfile = self.bfile_mut(ptr)?;
            if !bfile.writable {
                return Err(EvalError::InvalidHandle);
            }
            match &bfile.kind {
                BFileKind::Utf8 { inner, .. } => Some(SpecialBFileWrite::Utf8(*inner)),
                BFileKind::Crlf { inner } => Some(SpecialBFileWrite::Crlf(*inner)),
                BFileKind::Rle { read, .. } if !*read => Some(SpecialBFileWrite::Rle),
                BFileKind::Base64 { read, .. } if !*read => Some(SpecialBFileWrite::Base64),
                BFileKind::Buf { .. } => Some(SpecialBFileWrite::Buf),
                _ => None,
            }
        };
        match special {
            Some(SpecialBFileWrite::Utf8(inner)) => return self.put_utf8_bfile_byte(inner, byte),
            Some(SpecialBFileWrite::Crlf(inner)) => return self.put_crlf_bfile_byte(inner, byte),
            Some(SpecialBFileWrite::Rle) => return self.put_rle_bfile_byte(ptr, byte),
            Some(SpecialBFileWrite::Base64) => return self.put_base64_bfile_byte(ptr, byte),
            Some(SpecialBFileWrite::Buf) => return self.put_buf_bfile_byte(ptr, byte),
            None => {}
        }
        let bfile = self.bfile_mut(ptr)?;
        match &mut bfile.kind {
            BFileKind::Memory { bytes, pos } => {
                if *pos == bytes.len() {
                    bytes.push(byte as u8);
                } else if *pos < bytes.len() {
                    bytes[*pos] = byte as u8;
                } else {
                    return Err(EvalError::InvalidHandle);
                }
                *pos += 1;
                Ok(())
            }
            BFileKind::ReadOnlyMemoryView { .. } => {
                unreachable!("read-only handle is not writable")
            }
            #[cfg(any(not(target_arch = "wasm32"), target_os = "wasi"))]
            BFileKind::NativeFile { file, .. } => {
                use std::io::Write as _;

                #[cfg(target_os = "wasi")]
                wasi_trace_every("putb_native", 8192);
                file.borrow_mut()
                    .write_all(&[byte as u8])
                    .map_err(|_| EvalError::InvalidHandle)
            }
            #[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
            BFileKind::BrowserFile { handle, .. } => {
                let byte = [byte as u8];
                let written = unsafe { mhs_host_file_write(*handle, byte.as_ptr(), 1) };
                if written == 1 {
                    Ok(())
                } else {
                    Err(EvalError::InvalidHandle)
                }
            }
            BFileKind::Utf8 { .. } => unreachable!("handled above"),
            BFileKind::Crlf { .. } => unreachable!("handled above"),
            BFileKind::Rle { .. } => unreachable!("handled above"),
            BFileKind::Base64 { .. } => unreachable!("handled above"),
            BFileKind::Lz77 {
                read, buffer, pos, ..
            } => {
                if *read {
                    return Err(EvalError::InvalidHandle);
                }
                if *pos == buffer.len() {
                    buffer.push(byte as u8);
                } else if *pos < buffer.len() {
                    buffer[*pos] = byte as u8;
                } else {
                    return Err(EvalError::InvalidHandle);
                }
                *pos += 1;
                Ok(())
            }
            BFileKind::Bwt {
                read, buffer, pos, ..
            } => {
                if *read {
                    return Err(EvalError::InvalidHandle);
                }
                if *pos == buffer.len() {
                    buffer.push(byte as u8);
                } else if *pos < buffer.len() {
                    buffer[*pos] = byte as u8;
                } else {
                    return Err(EvalError::InvalidHandle);
                }
                *pos += 1;
                Ok(())
            }
            BFileKind::Lzma {
                read, buffer, pos, ..
            } => {
                if *read {
                    return Err(EvalError::InvalidHandle);
                }
                if *pos == buffer.len() {
                    buffer.push(byte as u8);
                } else if *pos < buffer.len() {
                    buffer[*pos] = byte as u8;
                } else {
                    return Err(EvalError::InvalidHandle);
                }
                *pos += 1;
                Ok(())
            }
            BFileKind::Buf { .. } => unreachable!("handled above"),
        }
    }

    fn put_crlf_bfile_byte(&mut self, inner: i64, byte: i64) -> Result<(), EvalError> {
        if byte == i64::from(b'\n') {
            self.put_bfile_byte(inner, i64::from(b'\r'))?;
        }
        self.put_bfile_byte(inner, byte)
    }

    fn put_rle_bfile_byte(&mut self, ptr: i64, byte: i64) -> Result<(), EvalError> {
        if byte < 0 {
            return Err(EvalError::InvalidByteString);
        }
        let (inner, pending) = {
            let bfile = self.bfile_mut(ptr)?;
            if !bfile.writable {
                return Err(EvalError::InvalidHandle);
            }
            match &mut bfile.kind {
                BFileKind::Rle {
                    inner,
                    read,
                    count,
                    byte: current,
                    ..
                } => {
                    if *read {
                        return Err(EvalError::InvalidHandle);
                    }
                    if (byte & 0x80) != 0 {
                        let pending = rle_pending_bytes(*count, *current)?;
                        *count = 0;
                        *current = -1;
                        (*inner, Some((pending, vec![0x81, (byte as u8) & 0x7f])))
                    } else if byte == *current {
                        *count = count.checked_add(1).ok_or(EvalError::Overflow)?;
                        (*inner, None)
                    } else {
                        let pending = rle_pending_bytes(*count, *current)?;
                        *count = 1;
                        *current = byte;
                        (*inner, Some((pending, Vec::new())))
                    }
                }
                _ => return Err(EvalError::InvalidHandle),
            }
        };
        if let Some((pending, suffix)) = pending {
            let pending_written = self.write_bfile_bytes(inner, &pending)?;
            if pending_written != pending.len() {
                return Err(EvalError::InvalidHandle);
            }
            let suffix_written = self.write_bfile_bytes(inner, &suffix)?;
            if suffix_written != suffix.len() {
                return Err(EvalError::InvalidHandle);
            }
        }
        Ok(())
    }

    fn put_base64_bfile_byte(&mut self, ptr: i64, byte: i64) -> Result<(), EvalError> {
        if byte < 0 {
            return Err(EvalError::InvalidByteString);
        }
        let action = {
            let bfile = self.bfile_mut(ptr)?;
            if !bfile.writable {
                return Err(EvalError::InvalidHandle);
            }
            match &mut bfile.kind {
                BFileKind::Base64 {
                    inner,
                    read,
                    encbuf,
                    encpos,
                    linelen,
                    outcol,
                    ..
                } => {
                    if *read {
                        return Err(EvalError::InvalidHandle);
                    }
                    encbuf[*encpos] = byte as u8;
                    *encpos += 1;
                    if *encpos == 3 {
                        let bytes = base64_full_quad_bytes(encbuf, *linelen, outcol);
                        *encpos = 0;
                        Some((*inner, bytes))
                    } else {
                        None
                    }
                }
                _ => return Err(EvalError::InvalidHandle),
            }
        };
        if let Some((inner, bytes)) = action {
            let written = self.write_bfile_bytes(inner, &bytes)?;
            if written != bytes.len() {
                return Err(EvalError::InvalidHandle);
            }
        }
        Ok(())
    }

    fn put_buf_bfile_byte(&mut self, ptr: i64, byte: i64) -> Result<(), EvalError> {
        if byte < 0 {
            return Err(EvalError::InvalidByteString);
        }
        let byte = byte as u8;
        loop {
            enum BufWriteAction {
                Direct(i64),
                Flush { inner: i64, bytes: Vec<u8> },
                Done,
            }
            let action = {
                let bfile = self.bfile_mut(ptr)?;
                if !bfile.writable {
                    return Err(EvalError::InvalidHandle);
                }
                match &mut bfile.kind {
                    BFileKind::Buf {
                        inner,
                        buffer,
                        pos,
                        linebuf,
                        ..
                    } => {
                        if buffer.is_empty() {
                            BufWriteAction::Direct(*inner)
                        } else if *pos >= buffer.len() {
                            let bytes = buffer.clone();
                            *pos = 0;
                            BufWriteAction::Flush {
                                inner: *inner,
                                bytes,
                            }
                        } else {
                            buffer[*pos] = byte;
                            *pos += 1;
                            if *linebuf && byte == b'\n' {
                                let bytes = buffer[..*pos].to_vec();
                                *pos = 0;
                                BufWriteAction::Flush {
                                    inner: *inner,
                                    bytes,
                                }
                            } else {
                                BufWriteAction::Done
                            }
                        }
                    }
                    _ => return Err(EvalError::InvalidHandle),
                }
            };
            match action {
                BufWriteAction::Direct(inner) => {
                    return self.put_bfile_byte(inner, i64::from(byte));
                }
                BufWriteAction::Flush { inner, bytes } => {
                    let written = self.write_bfile_bytes(inner, &bytes)?;
                    if written != bytes.len() {
                        return Err(EvalError::InvalidHandle);
                    }
                }
                BufWriteAction::Done => return Ok(()),
            }
        }
    }

    fn read_bfile(&mut self, ptr: i64, dst: i64, len: usize) -> Result<usize, EvalError> {
        let bytes = self.read_bfile_bytes(ptr, len)?;
        self.write_pointer_bytes(dst, &bytes)?;
        Ok(bytes.len())
    }

    fn read_bfile_bytes(&mut self, ptr: i64, len: usize) -> Result<Vec<u8>, EvalError> {
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

    fn write_bfile(&mut self, ptr: i64, src: i64, len: usize) -> Result<usize, EvalError> {
        let bytes = self.read_pointer_bytes(src, len)?;
        self.write_bfile_bytes(ptr, &bytes)
    }

    fn write_bfile_bytes(&mut self, ptr: i64, bytes: &[u8]) -> Result<usize, EvalError> {
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

    fn bfile_output_bytes(&self, ptr: i64) -> Result<Vec<u8>, EvalError> {
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
    fn deserialize_bfile(&mut self, ptr: i64) -> Result<NodeId, EvalError> {
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

    fn deserialize_parse_error(error: crate::parse::ParseError) -> EvalError {
        match error {
            crate::parse::ParseError::UnknownPrim(name) => EvalError::UnknownPrim(name),
            _ => EvalError::InvalidByteString,
        }
    }

    #[cold]
    #[inline(never)]
    fn append_parsed_program(&mut self, parsed: Program) -> Result<NodeId, EvalError> {
        let root = parsed.root();
        let nodes = parsed.nodes();
        let mut remap = Vec::with_capacity(nodes.len());
        for node in &nodes {
            let id = match node {
                Node::App(_, _)
                | Node::Indir(_)
                | Node::BytesView(_)
                | Node::MVar(Some(_))
                | Node::Weak(_)
                | Node::Array(_) => self.push_node(Node::Indir(None)),
                Node::Free(_) => return Err(EvalError::InvalidByteString),
                node => self.push_value_node(node.clone()),
            };
            remap.push(id);
        }

        for (index, node) in nodes.into_iter().enumerate() {
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

    fn remap_parsed_id(remap: &[NodeId], id: NodeId) -> Result<NodeId, EvalError> {
        remap
            .get(id.index())
            .copied()
            .ok_or(EvalError::InvalidByteString)
    }

    fn get_utf8_bfile_byte(&mut self, ptr: i64, inner: i64) -> Result<i64, EvalError> {
        let c1 = self.get_bfile_byte(inner)?;
        if c1 < 0 {
            return Ok(-1);
        }
        if (c1 & 0x80) == 0 {
            self.refill_utf8_ascii(ptr, inner)?;
            return Ok(c1);
        }
        let c2 = self.get_bfile_byte(inner)?;
        if c2 < 0 {
            return Ok(-1);
        }
        if (c1 & 0xe0) == 0xc0 {
            let c = ((c1 & 0x1f) << 6) | (c2 & 0x3f);
            if 0 < c && c < 0x80 {
                return Err(EvalError::InvalidByteString);
            }
            return Ok(c);
        }
        let c3 = self.get_bfile_byte(inner)?;
        if c3 < 0 {
            return Ok(-1);
        }
        if (c1 & 0xf0) == 0xe0 {
            let c = ((c1 & 0x0f) << 12) | ((c2 & 0x3f) << 6) | (c3 & 0x3f);
            if c < 0x800 {
                return Err(EvalError::InvalidByteString);
            }
            return Ok(c);
        }
        let c4 = self.get_bfile_byte(inner)?;
        if c4 < 0 {
            return Ok(-1);
        }
        if (c1 & 0xf8) == 0xf0 {
            let c = ((c1 & 0x07) << 18) | ((c2 & 0x3f) << 12) | ((c3 & 0x3f) << 6) | (c4 & 0x3f);
            if c < 0x10000 {
                return Err(EvalError::InvalidByteString);
            }
            return Ok(c);
        }
        Err(EvalError::InvalidByteString)
    }

    fn refill_utf8_ascii(&mut self, ptr: i64, inner: i64) -> Result<(), EvalError> {
        if !self.can_refill_utf8_ascii(ptr, inner)? {
            return Ok(());
        }
        let bytes = self.read_bfile_bytes(inner, UTF8_ASCII_REFILL)?;
        if bytes.is_empty() {
            return Ok(());
        }
        let ascii_len = bytes
            .iter()
            .position(|byte| (byte & 0x80) != 0)
            .unwrap_or(bytes.len());
        for byte in bytes[ascii_len..].iter().rev() {
            self.unget_bfile_byte(inner, i64::from(*byte))?;
        }
        if ascii_len == 0 {
            return Ok(());
        }
        let bfile = self.bfile_mut(ptr)?;
        match &mut bfile.kind {
            BFileKind::Utf8 {
                pending,
                pending_pos,
                ..
            } => {
                pending.clear();
                pending.extend_from_slice(&bytes[..ascii_len]);
                *pending_pos = 0;
                Ok(())
            }
            _ => Err(EvalError::InvalidHandle),
        }
    }

    fn can_refill_utf8_ascii(&self, ptr: i64, inner: i64) -> Result<bool, EvalError> {
        let outer = self.bfile(ptr)?;
        let BFileKind::Utf8 {
            unget,
            pending,
            pending_pos,
            ..
        } = &outer.kind
        else {
            return Err(EvalError::InvalidHandle);
        };
        if unget.is_some() || *pending_pos < pending.len() {
            return Ok(false);
        }
        if handle_from_ptr(inner).is_some() {
            return Ok(false);
        }

        let inner = self.bfile(inner)?;
        match &inner.kind {
            BFileKind::Memory { .. } | BFileKind::ReadOnlyMemoryView { .. } => Ok(true),
            #[cfg(any(not(target_arch = "wasm32"), target_os = "wasi"))]
            BFileKind::NativeFile { .. } => Ok(true),
            #[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
            BFileKind::BrowserFile { .. } => Ok(true),
            _ => Ok(false),
        }
    }

    fn put_utf8_bfile_byte(&mut self, inner: i64, byte: i64) -> Result<(), EvalError> {
        if byte < 0 {
            return Err(EvalError::InvalidByteString);
        }
        if 0 < byte && byte < 0x80 {
            self.put_bfile_byte(inner, byte)?;
        } else if byte < 0x800 {
            self.put_bfile_byte(inner, (byte >> 6) | 0xc0)?;
            self.put_bfile_byte(inner, (byte & 0x3f) | 0x80)?;
        } else if byte < 0x10000 {
            self.put_bfile_byte(inner, (byte >> 12) | 0xe0)?;
            self.put_bfile_byte(inner, ((byte >> 6) & 0x3f) | 0x80)?;
            self.put_bfile_byte(inner, (byte & 0x3f) | 0x80)?;
        } else if byte < 0x110000 {
            self.put_bfile_byte(inner, (byte >> 18) | 0xf0)?;
            self.put_bfile_byte(inner, ((byte >> 12) & 0x3f) | 0x80)?;
            self.put_bfile_byte(inner, ((byte >> 6) & 0x3f) | 0x80)?;
            self.put_bfile_byte(inner, (byte & 0x3f) | 0x80)?;
        } else {
            return Err(EvalError::InvalidByteString);
        }
        Ok(())
    }

    fn write_io_handle_bytes(&self, handle: StdHandle, bytes: &[u8]) -> Result<(), EvalError> {
        if handle == StdHandle::Stdin {
            return Err(EvalError::InvalidHandle);
        }
        #[cfg(any(not(target_arch = "wasm32"), target_os = "wasi"))]
        {
            use std::io::Write as _;

            match handle {
                StdHandle::Stdout => {
                    let mut stdout = std::io::stdout().lock();
                    stdout
                        .write_all(bytes)
                        .map_err(|_| EvalError::InvalidHandle)?;
                }
                StdHandle::Stderr => {
                    let mut stderr = std::io::stderr().lock();
                    stderr
                        .write_all(bytes)
                        .map_err(|_| EvalError::InvalidHandle)?;
                }
                StdHandle::Stdin => unreachable!("checked above"),
            }
        }
        #[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
        {
            let _ = bytes;
        }
        Ok(())
    }

    fn flush_io_handle(&self, handle: StdHandle) -> Result<(), EvalError> {
        if handle == StdHandle::Stdin {
            return Ok(());
        }
        #[cfg(any(not(target_arch = "wasm32"), target_os = "wasi"))]
        {
            use std::io::Write as _;

            match handle {
                StdHandle::Stdout => std::io::stdout()
                    .lock()
                    .flush()
                    .map_err(|_| EvalError::InvalidHandle)?,
                StdHandle::Stderr => std::io::stderr()
                    .lock()
                    .flush()
                    .map_err(|_| EvalError::InvalidHandle)?,
                StdHandle::Stdin => unreachable!("checked above"),
            }
        }
        Ok(())
    }

    fn read_stdin_byte(&self) -> Result<i64, EvalError> {
        #[cfg(any(not(target_arch = "wasm32"), target_os = "wasi"))]
        {
            use std::io::Read as _;

            let mut byte = [0];
            match std::io::stdin().lock().read(&mut byte) {
                Ok(0) => Ok(-1),
                Ok(_) => Ok(i64::from(byte[0])),
                Err(_) => Err(EvalError::InvalidHandle),
            }
        }
        #[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
        {
            Ok(-1)
        }
    }

}
