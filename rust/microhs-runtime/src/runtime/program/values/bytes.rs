//! Byte and mutable-byte node accessors and updates.
use super::*;

impl Program {
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
}
