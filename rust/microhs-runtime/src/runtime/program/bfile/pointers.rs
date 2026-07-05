//! Pointer decoding and memory access for node, allocation, and BFILE pointers.
use super::*;

impl Program {
    pub(in crate::runtime) fn pointer_for_node(
        &mut self,
        id: NodeId,
        offset: usize,
    ) -> Result<i64, EvalError> {
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

    pub(in crate::runtime) fn pointer_for_allocation(
        &self,
        slot: usize,
        offset: usize,
    ) -> Result<i64, EvalError> {
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

    pub(in crate::runtime) fn pointer_for_bfile(&self, slot: usize) -> Result<i64, EvalError> {
        let slot = i64::try_from(slot).map_err(|_| EvalError::Overflow)?;
        BFILE_PTR_BASE
            .checked_add(
                slot.checked_mul(BFILE_PTR_STRIDE)
                    .ok_or(EvalError::Overflow)?,
            )
            .filter(|ptr| *ptr < DIR_PTR_BASE)
            .ok_or(EvalError::Overflow)
    }

    pub(in crate::runtime) fn pointer_for_dir(&self, slot: usize) -> Result<i64, EvalError> {
        let slot = i64::try_from(slot).map_err(|_| EvalError::Overflow)?;
        DIR_PTR_BASE
            .checked_add(
                slot.checked_mul(DIR_PTR_STRIDE)
                    .ok_or(EvalError::Overflow)?,
            )
            .filter(|ptr| *ptr < ALLOCATION_PTR_BASE)
            .ok_or(EvalError::Overflow)
    }

    pub(in crate::runtime) fn decode_pointer(&self, ptr: i64) -> Result<(usize, usize), EvalError> {
        if ptr <= 0 {
            return Err(EvalError::InvalidByteString);
        }
        let slot_word = usize::try_from(ptr >> 32).map_err(|_| EvalError::InvalidByteString)?;
        if slot_word == 0 {
            return Err(EvalError::InvalidByteString);
        }
        let offset =
            usize::try_from(ptr & 0xffff_ffff).map_err(|_| EvalError::InvalidByteString)?;
        let slot = slot_word - 1;
        let block = self
            .node_pointers
            .get(slot)
            .copied()
            .ok_or(EvalError::InvalidByteString)?;
        Ok((block.index(), offset))
    }

    pub(in crate::runtime) fn decode_allocation_pointer(
        &self,
        ptr: i64,
    ) -> Result<(usize, usize), EvalError> {
        if !(ALLOCATION_PTR_BASE..0).contains(&ptr) {
            return Err(EvalError::InvalidByteString);
        }
        let raw = ptr
            .checked_sub(ALLOCATION_PTR_BASE)
            .ok_or(EvalError::Overflow)?;
        let slot = usize::try_from(raw / ALLOCATION_PTR_STRIDE)
            .map_err(|_| EvalError::InvalidByteString)?;
        let offset = usize::try_from(raw % ALLOCATION_PTR_STRIDE)
            .map_err(|_| EvalError::InvalidByteString)?;
        let bytes = self
            .allocations
            .get(slot)
            .and_then(Option::as_ref)
            .ok_or(EvalError::InvalidByteString)?;
        if offset > bytes.len() {
            return Err(EvalError::InvalidByteString);
        }
        Ok((slot, offset))
    }

    pub(in crate::runtime) fn decode_bfile_pointer(&self, ptr: i64) -> Result<usize, EvalError> {
        if !(BFILE_PTR_BASE..DIR_PTR_BASE).contains(&ptr) {
            return Err(EvalError::InvalidHandle);
        }
        let raw = ptr.checked_sub(BFILE_PTR_BASE).ok_or(EvalError::Overflow)?;
        if raw % BFILE_PTR_STRIDE != 0 {
            return Err(EvalError::InvalidHandle);
        }
        usize::try_from(raw / BFILE_PTR_STRIDE).map_err(|_| EvalError::InvalidHandle)
    }

    pub(in crate::runtime) fn decode_dir_pointer(&self, ptr: i64) -> Result<usize, EvalError> {
        if !(DIR_PTR_BASE..ALLOCATION_PTR_BASE).contains(&ptr) {
            return Err(EvalError::InvalidHandle);
        }
        let raw = ptr.checked_sub(DIR_PTR_BASE).ok_or(EvalError::Overflow)?;
        if raw % DIR_PTR_STRIDE != 0 {
            return Err(EvalError::InvalidHandle);
        }
        usize::try_from(raw / DIR_PTR_STRIDE).map_err(|_| EvalError::InvalidHandle)
    }

    pub(in crate::runtime) fn allocation_bytes(
        &self,
        ptr: i64,
    ) -> Result<Option<&[u8]>, EvalError> {
        if !(ALLOCATION_PTR_BASE..0).contains(&ptr) {
            return Ok(None);
        }
        let (slot, offset) = self.decode_allocation_pointer(ptr)?;
        let bytes = self
            .allocations
            .get(slot)
            .and_then(Option::as_ref)
            .ok_or(EvalError::InvalidByteString)?;
        Ok(Some(&bytes[offset..]))
    }

    pub(in crate::runtime) fn pointer_bytes(&self, ptr: i64) -> Result<&[u8], EvalError> {
        if let Some(bytes) = self.allocation_bytes(ptr)? {
            return Ok(bytes);
        }
        let (base, offset) = self.decode_pointer(ptr)?;
        if base >= self.nodes.len() {
            return Err(EvalError::InvalidByteString);
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
                return bytes.get(offset..).ok_or(EvalError::InvalidByteString);
            }
            _ => return Err(EvalError::InvalidByteString),
        };
        if offset > bytes.len() {
            return Err(EvalError::InvalidByteString);
        }
        Ok(&bytes[offset..])
    }

    pub(in crate::runtime) fn write_pointer_bytes(
        &mut self,
        ptr: i64,
        bytes: &[u8],
    ) -> Result<(), EvalError> {
        if (ALLOCATION_PTR_BASE..0).contains(&ptr) {
            let (slot, offset) = self.decode_allocation_pointer(ptr)?;
            let write_len = bytes.len();
            let available = self.allocations[slot]
                .as_ref()
                .expect("checked allocation slot")
                .len()
                .saturating_sub(offset);
            if available < write_len {
                return Err(EvalError::InvalidByteString);
            }
            let dst = self.allocations[slot]
                .as_mut()
                .expect("checked allocation slot");
            dst[offset..offset + write_len].copy_from_slice(bytes);
            return Ok(());
        }
        let (base, offset) = self.decode_pointer(ptr)?;
        if base >= self.nodes.len() {
            return Err(EvalError::InvalidByteString);
        }
        let write_len = bytes.len();
        let node_id = NodeId::from_index(base);
        if matches!(self.cold_node(node_id), Some(Node::BytesView(_))) {
            let current = self.bytes(node_id)?;
            let available = current.len().saturating_sub(offset);
            if offset > current.len() || available < write_len {
                return Err(EvalError::InvalidByteString);
            }
            let mut owned = current.to_vec();
            owned[offset..offset + write_len].copy_from_slice(bytes);
            self.set_node_at(node_id.index(), Node::bytes(owned));
            return Ok(());
        }
        let is_mutable_bytes = match self.cold_node(node_id) {
            Some(Node::Bytes(dst)) => {
                let available = dst.len().saturating_sub(offset);
                if offset > dst.len() || available < write_len {
                    return Err(EvalError::InvalidByteString);
                }
                false
            }
            Some(Node::MutableBytes(dst)) => {
                let storage_len = dst.bytes.len();
                let available = storage_len.saturating_sub(offset);
                if offset > storage_len || available < write_len {
                    return Err(EvalError::InvalidByteString);
                }
                true
            }
            _ => return Err(EvalError::InvalidByteString),
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

    pub(in crate::runtime) fn peek_array<const N: usize>(
        &self,
        ptr: i64,
    ) -> Result<[u8; N], EvalError> {
        self.read_pointer_bytes(ptr, N)?
            .try_into()
            .map_err(|_| EvalError::InvalidByteString)
    }

    pub(in crate::runtime) fn peek_unsigned(
        &self,
        ptr: i64,
        size: usize,
    ) -> Result<u64, EvalError> {
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

    pub(in crate::runtime) fn poke_unsigned(
        &mut self,
        ptr: i64,
        size: usize,
        value: u64,
    ) -> Result<(), EvalError> {
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

    pub(in crate::runtime) fn peek_signed(&self, ptr: i64, size: usize) -> Result<i64, EvalError> {
        let unsigned = self.peek_unsigned(ptr, size)?;
        let shift = (8 - size) * 8;
        Ok(((unsigned << shift) as i64) >> shift)
    }

    pub(in crate::runtime) fn poke_signed(
        &mut self,
        ptr: i64,
        size: usize,
        value: i64,
    ) -> Result<(), EvalError> {
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

    pub(in crate::runtime) fn peek_c_char(&self, ptr: i64) -> Result<i64, EvalError> {
        if std::os::raw::c_char::MIN < 0 {
            self.peek_signed(ptr, size_of::<std::os::raw::c_char>())
        } else {
            Ok(self.peek_unsigned(ptr, size_of::<std::os::raw::c_char>())? as i64)
        }
    }

    pub(in crate::runtime) fn poke_c_char(
        &mut self,
        ptr: i64,
        value: i64,
    ) -> Result<(), EvalError> {
        if std::os::raw::c_char::MIN < 0 {
            self.poke_signed(ptr, size_of::<std::os::raw::c_char>(), value)
        } else {
            self.poke_unsigned(ptr, size_of::<std::os::raw::c_char>(), value as u64)
        }
    }

    pub(in crate::runtime) fn read_pointer_bytes(
        &self,
        ptr: i64,
        len: usize,
    ) -> Result<Vec<u8>, EvalError> {
        let bytes = self.pointer_bytes(ptr)?;
        let bytes = bytes.get(..len).ok_or(EvalError::InvalidByteString)?;
        Ok(bytes.to_vec())
    }

    pub(in crate::runtime) fn read_c_string(&self, ptr: i64) -> Result<Vec<u8>, EvalError> {
        let bytes = self.pointer_bytes(ptr)?;
        let len = c_string_len(bytes);
        Ok(bytes[..len].to_vec())
    }

    pub(in crate::runtime) fn c_string_len(&self, ptr: i64) -> Result<usize, EvalError> {
        Ok(c_string_len(self.pointer_bytes(ptr)?))
    }

    pub(in crate::runtime) fn bfile(&self, ptr: i64) -> Result<&BFile, EvalError> {
        let slot = self.decode_bfile_pointer(ptr)?;
        self.bfiles
            .get(slot)
            .and_then(Option::as_ref)
            .ok_or(EvalError::InvalidHandle)
    }

    pub(in crate::runtime) fn bfile_mut(&mut self, ptr: i64) -> Result<&mut BFile, EvalError> {
        let slot = self.decode_bfile_pointer(ptr)?;
        self.bfiles
            .get_mut(slot)
            .and_then(Option::as_mut)
            .ok_or(EvalError::InvalidHandle)
    }

    pub(in crate::runtime) fn read_only_memory_view_state(
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

    pub(in crate::runtime) fn get_read_only_memory_view_byte(
        &mut self,
        ptr: i64,
    ) -> Result<i64, EvalError> {
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

    pub(in crate::runtime) fn read_only_memory_view_bytes(
        &mut self,
        ptr: i64,
        len: usize,
    ) -> Result<Vec<u8>, EvalError> {
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

    pub(in crate::runtime) fn unget_read_only_memory_view_byte(
        &mut self,
        ptr: i64,
        byte: i64,
    ) -> Result<(), EvalError> {
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
}
