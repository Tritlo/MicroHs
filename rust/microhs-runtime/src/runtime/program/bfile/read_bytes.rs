//! Byte-at-a-time BFILE read filters and decoders.
use super::*;

impl Program {
    pub(in crate::runtime) fn get_bfile_byte(&mut self, ptr: i64) -> Result<i64, EvalError> {
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

    pub(in crate::runtime) fn get_crlf_bfile_byte(&mut self, inner: i64) -> Result<i64, EvalError> {
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

    pub(in crate::runtime) fn get_rle_bfile_byte(&mut self, ptr: i64) -> Result<i64, EvalError> {
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

    pub(in crate::runtime) fn get_rle_rep(
        &mut self,
        inner: i64,
    ) -> Result<Option<usize>, EvalError> {
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

    pub(in crate::runtime) fn get_base64_bfile_byte(&mut self, ptr: i64) -> Result<i64, EvalError> {
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

    pub(in crate::runtime) fn get_base64_quartet(
        &mut self,
        inner: i64,
    ) -> Result<Option<[i32; 4]>, EvalError> {
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

    pub(in crate::runtime) fn get_buf_bfile_byte(&mut self, ptr: i64) -> Result<i64, EvalError> {
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
}
