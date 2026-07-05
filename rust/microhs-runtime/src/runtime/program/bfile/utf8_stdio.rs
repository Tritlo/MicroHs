use super::*;

impl Program {
    pub(in crate::runtime) fn get_utf8_bfile_byte(
        &mut self,
        ptr: i64,
        inner: i64,
    ) -> Result<i64, EvalError> {
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

    pub(in crate::runtime) fn refill_utf8_ascii(
        &mut self,
        ptr: i64,
        inner: i64,
    ) -> Result<(), EvalError> {
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

    pub(in crate::runtime) fn can_refill_utf8_ascii(
        &self,
        ptr: i64,
        inner: i64,
    ) -> Result<bool, EvalError> {
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

    pub(in crate::runtime) fn put_utf8_bfile_byte(
        &mut self,
        inner: i64,
        byte: i64,
    ) -> Result<(), EvalError> {
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

    pub(in crate::runtime) fn write_io_handle_bytes(
        &self,
        handle: StdHandle,
        bytes: &[u8],
    ) -> Result<(), EvalError> {
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

    pub(in crate::runtime) fn flush_io_handle(&self, handle: StdHandle) -> Result<(), EvalError> {
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

    pub(in crate::runtime) fn read_stdin_byte(&self) -> Result<i64, EvalError> {
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
