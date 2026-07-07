//! BFILE allocation and standard handle setup.
use super::*;

impl Program {
    pub(in crate::runtime) fn alloc_bfile(&mut self, bfile: BFile) -> Result<i64, EvalError> {
        let mut slot = self.bfile_first_free;
        while self.bfiles.get(slot).is_some_and(Option::is_some) {
            slot += 1;
        }
        if slot == self.bfiles.len() {
            self.bfiles.push(Some(bfile));
        } else {
            self.bfiles[slot] = Some(bfile);
        }
        self.bfile_first_free = slot + 1;
        while self
            .bfiles
            .get(self.bfile_first_free)
            .is_some_and(Option::is_some)
        {
            self.bfile_first_free += 1;
        }
        self.pointer_for_bfile(slot)
    }

    pub(in crate::runtime) fn read_only_memory_view(
        &self,
        ptr: i64,
        len: usize,
    ) -> Result<Option<BFileKind>, EvalError> {
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
                    return Err(EvalError::InvalidByteString);
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

    pub(in crate::runtime) fn memory_read_bfile_kind(
        &self,
        ptr: i64,
        len: usize,
    ) -> Result<BFileKind, EvalError> {
        if let Some(kind) = self.read_only_memory_view(ptr, len)? {
            return Ok(kind);
        }
        let bytes = self.read_pointer_bytes(ptr, len)?;
        Ok(BFileKind::Memory { bytes, pos: 0 })
    }

    pub(in crate::runtime) fn alloc_dir(
        &mut self,
        entries: Vec<Vec<u8>>,
    ) -> Result<i64, EvalError> {
        let dir = DirHandle { entries, pos: 0 };
        let mut slot = self.dir_first_free;
        while self.dirs.get(slot).is_some_and(Option::is_some) {
            slot += 1;
        }
        if slot == self.dirs.len() {
            self.dirs.push(Some(dir));
        } else {
            self.dirs[slot] = Some(dir);
        }
        self.dir_first_free = slot + 1;
        while self
            .dirs
            .get(self.dir_first_free)
            .is_some_and(Option::is_some)
        {
            self.dir_first_free += 1;
        }
        self.pointer_for_dir(slot)
    }

    pub(in crate::runtime) fn alloc_environ(&mut self) -> Result<i64, EvalError> {
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

    pub(in crate::runtime) fn bfile_permissions(
        &self,
        ptr: i64,
    ) -> Result<(bool, bool), EvalError> {
        if let Some(handle) = handle_from_ptr(ptr) {
            return Ok(match handle {
                StdHandle::Stdin => (true, false),
                StdHandle::Stdout | StdHandle::Stderr => (false, true),
            });
        }
        let bfile = self.bfile(ptr)?;
        Ok((bfile.readable, bfile.writable))
    }

    pub(in crate::runtime) fn add_utf8_bfile(&mut self, ptr: i64) -> Result<i64, EvalError> {
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

    pub(in crate::runtime) fn add_crlf_bfile(&mut self, ptr: i64) -> Result<i64, EvalError> {
        let (readable, writable) = self.bfile_permissions(ptr)?;
        self.alloc_bfile(BFile {
            kind: BFileKind::Crlf { inner: ptr },
            readable,
            writable,
        })
    }

    pub(in crate::runtime) fn add_rle_bfile(
        &mut self,
        ptr: i64,
        read: bool,
    ) -> Result<i64, EvalError> {
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

    pub(in crate::runtime) fn add_base64_bfile(
        &mut self,
        ptr: i64,
        read: bool,
    ) -> Result<i64, EvalError> {
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

    pub(in crate::runtime) fn add_lz77_bfile(
        &mut self,
        ptr: i64,
        read: bool,
    ) -> Result<i64, EvalError> {
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

    pub(in crate::runtime) fn read_bfile_u32_le(&mut self, ptr: i64) -> Result<usize, EvalError> {
        let bytes = self.read_bfile_bytes(ptr, 4)?;
        let bytes: [u8; 4] = bytes.try_into().map_err(|_| EvalError::InvalidByteString)?;
        usize::try_from(u32::from_le_bytes(bytes)).map_err(|_| EvalError::Overflow)
    }

    pub(in crate::runtime) fn add_bwt_bfile(
        &mut self,
        ptr: i64,
        read: bool,
    ) -> Result<i64, EvalError> {
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

    pub(in crate::runtime) fn add_lzma_bfile(
        &mut self,
        ptr: i64,
        read: bool,
    ) -> Result<i64, EvalError> {
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

    pub(in crate::runtime) fn add_buf_bfile(
        &mut self,
        ptr: i64,
        bufsize: i64,
    ) -> Result<i64, EvalError> {
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
}
