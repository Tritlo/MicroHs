//! BFILE open, close, flush, seek, and mode lifecycle operations.
use super::*;

impl Program {
    pub(in crate::runtime) fn close_bfile(&mut self, ptr: i64) -> Result<(), EvalError> {
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
        let slot_index = slot;
        let slot = self
            .bfiles
            .get_mut(slot_index)
            .ok_or(EvalError::InvalidHandle)?;
        let _bfile = slot.as_ref().ok_or(EvalError::InvalidHandle)?;
        #[cfg(any(not(target_arch = "wasm32"), target_os = "wasi"))]
        if let BFileKind::NativeFile { file, .. } = &_bfile.kind {
            if _bfile.writable {
                file.flush().map_err(|_| EvalError::InvalidHandle)?;
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
        if slot_index < self.bfile_first_free {
            self.bfile_first_free = slot_index;
        }
        Ok(())
    }

    pub(in crate::runtime) fn read_dir_entry(&mut self, ptr: i64) -> Result<i64, EvalError> {
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

    pub(in crate::runtime) fn close_dir(&mut self, ptr: i64) -> Result<(), EvalError> {
        let slot_index = self.decode_dir_pointer(ptr)?;
        let slot = self
            .dirs
            .get_mut(slot_index)
            .ok_or(EvalError::InvalidHandle)?;
        if slot.is_none() {
            return Err(EvalError::InvalidHandle);
        }
        *slot = None;
        if slot_index < self.dir_first_free {
            self.dir_first_free = slot_index;
        }
        Ok(())
    }

    pub(in crate::runtime) fn flush_bfile(&mut self, ptr: i64) -> Result<(), EvalError> {
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
            if _bfile.writable {
                file.flush().map_err(|_| EvalError::InvalidHandle)?;
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

    pub(in crate::runtime) fn flush_open_bfiles(&mut self) -> Result<(), EvalError> {
        let ptrs = self
            .bfiles
            .iter()
            .enumerate()
            .filter_map(|(slot, bfile)| bfile.as_ref().filter(|bfile| bfile.writable).map(|_| slot))
            .map(|slot| self.pointer_for_bfile(slot))
            .collect::<Result<Vec<_>, _>>()?;
        for ptr in ptrs {
            if self.bfile(ptr).is_ok() {
                self.flush_bfile(ptr)?;
            }
        }
        Ok(())
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        let _ = self.flush_open_bfiles();
    }
}
