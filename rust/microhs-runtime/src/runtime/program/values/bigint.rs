//! Big-integer foreign-pointer helpers for the imath-backed GMP surface.
use super::*;

impl Program {
    pub(in crate::runtime) fn new_mpz_node(&mut self) -> Result<Node, EvalError> {
        let bigint = self.push_node(Node::bigint(b"0".to_vec()));
        let ptr = self.pointer_for_node(bigint, 0)?;
        Ok(self.foreign_ptr_node(None, 0, ptr))
    }

    pub(in crate::runtime) fn mpz_node_id(&self, ptr: i64) -> Result<NodeId, EvalError> {
        let (slot, offset) = self.decode_pointer(ptr)?;
        if offset != 0 {
            return Err(EvalError::ExpectedForeignPtr(NodeId::from_index(slot)));
        }
        let id = NodeId::from_index(slot);
        match self.cold_node(id) {
            Some(Node::BigInt(_)) => Ok(id),
            _ => Err(EvalError::ExpectedForeignPtr(id)),
        }
    }

    pub(in crate::runtime) fn mpz_decimal_bytes_for_ptr(&self, ptr: i64) -> Option<&[u8]> {
        let (slot, offset) = self.decode_pointer(ptr).ok()?;
        if offset != 0 {
            return None;
        }
        let id = NodeId::from_index(slot);
        match self.cold_node(id)? {
            Node::BigInt(bytes) => Some(bytes.as_slice()),
            _ => None,
        }
    }

    pub(in crate::runtime) fn mpz_value(&self, ptr: i64) -> Result<MpzValue, EvalError> {
        let id = self.mpz_node_id(ptr)?;
        let Some(Node::BigInt(bytes)) = self.cold_node(id) else {
            return Err(EvalError::ExpectedForeignPtr(id));
        };
        MpzValue::parse_decimal(bytes).map_err(|_| EvalError::InvalidByteString)
    }

    pub(in crate::runtime) fn write_mpz_value(
        &mut self,
        ptr: i64,
        value: MpzValue,
    ) -> Result<(), EvalError> {
        let id = self.mpz_node_id(ptr)?;
        self.set_node_at(id.index(), Node::bigint(value.to_decimal_bytes()));
        Ok(())
    }
}
