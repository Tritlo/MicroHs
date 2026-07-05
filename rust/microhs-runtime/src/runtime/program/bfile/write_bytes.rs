//! Byte-at-a-time BFILE write filters and encoders.
use super::*;

impl Program {
    pub(in crate::runtime) fn unget_bfile_byte(
        &mut self,
        ptr: i64,
        byte: i64,
    ) -> Result<(), EvalError> {
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

    pub(in crate::runtime) fn put_bfile_byte(
        &mut self,
        ptr: i64,
        byte: i64,
    ) -> Result<(), EvalError> {
        if let Some(handle) = handle_from_ptr(ptr) {
            // The std streams are UTF-8 text handles (matching C's add_utf8-wrapped
            // stdio), so a byte written here is a codepoint that must be UTF-8 encoded.
            let mut buf = [0u8; 4];
            let len = Self::encode_modified_utf8(byte, &mut buf)?;
            return self.write_io_handle_bytes(handle, &buf[..len]);
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

    pub(in crate::runtime) fn put_crlf_bfile_byte(
        &mut self,
        inner: i64,
        byte: i64,
    ) -> Result<(), EvalError> {
        if byte == i64::from(b'\n') {
            self.put_bfile_byte(inner, i64::from(b'\r'))?;
        }
        self.put_bfile_byte(inner, byte)
    }

    pub(in crate::runtime) fn put_rle_bfile_byte(
        &mut self,
        ptr: i64,
        byte: i64,
    ) -> Result<(), EvalError> {
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

    pub(in crate::runtime) fn put_base64_bfile_byte(
        &mut self,
        ptr: i64,
        byte: i64,
    ) -> Result<(), EvalError> {
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

    pub(in crate::runtime) fn put_buf_bfile_byte(
        &mut self,
        ptr: i64,
        byte: i64,
    ) -> Result<(), EvalError> {
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
}
