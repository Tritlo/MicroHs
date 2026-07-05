//! Bytestring and mutable-bytes runtime primitive dispatch.
use super::*;

impl Program {
    pub(in crate::runtime) fn bytes_op(
        &mut self,
        name: &str,
        args: &[NodeId],
    ) -> Result<Option<(usize, NodeId)>, EvalError> {
        match self.bytes_op_inner(name, args) {
            Ok(result) => Ok(result),
            Err(err) => Err(self.trace_invalid_op_error("bytes_op", name, args, err)),
        }
    }

    pub(in crate::runtime) fn bytes_op_inner(
        &mut self,
        name: &str,
        args: &[NodeId],
    ) -> Result<Option<(usize, NodeId)>, EvalError> {
        macro_rules! invalid_bytes {
            ($($arg:tt)*) => {{
                if std::env::var_os("MHS_TRACE_INVALID_BYTES").is_some() {
                    eprintln!(
                        "invalid bytes op {name}: reductions={}",
                        self.reductions,
                    );
                    eprintln!($($arg)*);
                }
                EvalError::InvalidByteString
            }};
        }
        if let Some(op) = BytesBinOp::from_prim(name) {
            let x = self.eval_bytes_id(args[0])?;
            let y = self.eval_bytes_id(args[1])?;
            let node = self.bytes_bin_result_node(op, x, y)?;
            return Ok(Some((2, node)));
        }

        let rewrite = match name {
            "packCString" if args.len() >= 2 => {
                let ptr = self.eval_pointer_value(args[0])?;
                let bytes = self.read_c_string(ptr)?;
                let bytes = self.push_node(Node::bytes(bytes));
                Some((2, self.pair(bytes, args[1])))
            }
            "packCStringLen" if args.len() >= 3 => {
                let ptr = self.eval_pointer_value(args[0])?;
                let len = int_to_usize(self.eval_int(args[1])?)?;
                let bytes = self.read_pointer_bytes(ptr, len)?;
                let bytes = self.push_node(Node::bytes(bytes));
                Some((3, self.pair(bytes, args[2])))
            }
            "packCStringLen" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let len = int_to_usize(self.eval_int(args[1])?)?;
                let bytes = self.read_pointer_bytes(ptr, len)?;
                Some((2, self.push_node(Node::bytes(bytes))))
            }
            "bsgrab" if args.len() >= 2 => {
                let ptr = self.eval_pointer_value(args[0])?;
                let bytes = self.read_c_string(ptr)?;
                let bytes = self.push_node(Node::bytes(bytes));
                Some((2, self.pair(bytes, args[1])))
            }
            "bsgrablen" if args.len() >= 3 => {
                let ptr = self.eval_pointer_value(args[0])?;
                let len = int_to_usize(self.eval_int(args[1])?)?;
                let bytes = self.read_pointer_bytes(ptr, len)?;
                let bytes = self.push_node(Node::bytes(bytes));
                Some((3, self.pair(bytes, args[2])))
            }
            "bsgrablen" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let len = int_to_usize(self.eval_int(args[1])?)?;
                let bytes = self.read_pointer_bytes(ptr, len)?;
                Some((2, self.push_node(Node::bytes(bytes))))
            }
            "bsnew" if args.len() >= 3 => {
                let size = int_to_usize(self.eval_int(args[0])?)?;
                let capacity = int_to_usize(self.eval_int(args[1])?)?;
                let bytes = self.new_mutable_bytes(size, capacity)?;
                Some((3, self.pair(bytes, args[2])))
            }
            "bsnew" => {
                let size = int_to_usize(self.eval_int(args[0])?)?;
                let capacity = int_to_usize(self.eval_int(args[1])?)?;
                Some((2, self.new_mutable_bytes(size, capacity)?))
            }
            "bsread" if args.len() >= 3 => {
                let bytes = self.eval_bytes_id(args[0])?;
                let index = int_to_usize(self.eval_int(args[1])?)?;
                let (len, storage_len) = self.byte_prim_lengths(bytes)?;
                if index >= storage_len {
                    return Err(invalid_bytes!(
                        "bsread bytes={bytes:?} index={index} len={len} storage_len={storage_len}"
                    ));
                }
                let byte = self.read_byte_unchecked_prim(bytes, index)?;
                let byte = self.int(byte as i64);
                Some((3, self.pair(byte, args[2])))
            }
            "bsread" => {
                let bytes = self.eval_bytes_id(args[0])?;
                let index = int_to_usize(self.eval_int(args[1])?)?;
                let (len, storage_len) = self.byte_prim_lengths(bytes)?;
                if index >= storage_len {
                    return Err(invalid_bytes!(
                        "bsread bytes={bytes:?} index={index} len={len} storage_len={storage_len}"
                    ));
                }
                let byte = self.read_byte_unchecked_prim(bytes, index)?;
                Some((2, self.int(byte as i64)))
            }
            "bswrite" if args.len() >= 4 => {
                let bytes = self.eval_bytes_id(args[0])?;
                let index = int_to_usize(self.eval_int(args[1])?)?;
                let byte = self.eval_int(args[2])? as u8;
                let (len, storage_len) = self.byte_prim_lengths(bytes)?;
                if index >= storage_len {
                    return Err(invalid_bytes!(
                        "bswrite bytes={bytes:?} index={index} len={len} storage_len={storage_len}"
                    ));
                }
                self.write_byte_unchecked_prim(bytes, index, byte)?;
                let unit = self.prim("I");
                Some((4, self.pair(unit, args[3])))
            }
            "bswrite" if args.len() >= 3 => {
                let bytes = self.eval_bytes_id(args[0])?;
                let index = int_to_usize(self.eval_int(args[1])?)?;
                let byte = self.eval_int(args[2])? as u8;
                let (len, storage_len) = self.byte_prim_lengths(bytes)?;
                if index >= storage_len {
                    return Err(invalid_bytes!(
                        "bswrite bytes={bytes:?} index={index} len={len} storage_len={storage_len}"
                    ));
                }
                self.write_byte_unchecked_prim(bytes, index, byte)?;
                Some((3, self.prim("I")))
            }
            "bsfreeze" if args.len() >= 2 => {
                let bytes = self.eval_bytes_id(args[0])?;
                let bytes = self.freeze_bytes(bytes)?;
                Some((2, self.pair(bytes, args[1])))
            }
            "bsappbyte" if args.len() >= 3 => {
                let bytes = self.eval_bytes_id(args[0])?;
                let byte = self.eval_int(args[1])? as u8;
                self.append_byte(bytes, byte)?;
                let unit = self.prim("I");
                Some((3, self.pair(unit, args[2])))
            }
            "bsappbyte" => {
                let bytes = self.eval_bytes_id(args[0])?;
                let byte = self.eval_int(args[1])? as u8;
                self.append_byte(bytes, byte)?;
                Some((2, self.prim("I")))
            }
            "bsappchar" if args.len() >= 3 => {
                let bytes = self.eval_bytes_id(args[0])?;
                let encoded = modified_utf8(self.eval_int(args[1])?)?;
                self.append_bytes(bytes, &encoded)?;
                let unit = self.prim("I");
                Some((3, self.pair(unit, args[2])))
            }
            "bsappchar" => {
                let bytes = self.eval_bytes_id(args[0])?;
                let encoded = modified_utf8(self.eval_int(args[1])?)?;
                self.append_bytes(bytes, &encoded)?;
                Some((2, self.prim("I")))
            }
            "bsreplicate" => {
                let len = int_to_usize(self.eval_int(args[0])?)?;
                let byte = self.eval_int(args[1])? as u8;
                Some((2, self.push_node(Node::bytes(vec![byte; len]))))
            }
            "bsindex" => {
                let bytes_id = self.eval_bytes_id(args[0])?;
                let index = int_to_usize(self.eval_int(args[1])?)?;
                let bytes = self.bytes(bytes_id)?;
                let len = bytes.len();
                if index >= len {
                    return Err(invalid_bytes!("bsindex index={index} len={len}"));
                }
                let byte = bytes[index];
                Some((2, self.int(byte as i64)))
            }
            "bssubstr" if args.len() >= 3 => {
                let bytes_id = self.eval_bytes_id(args[0])?;
                let offset = int_to_usize(self.eval_int(args[1])?)?;
                let len = int_to_usize(self.eval_int(args[2])?)?;
                let bytes = self.bytes(bytes_id)?;
                let bytes_len = bytes.len();
                offset
                    .checked_add(len)
                    .filter(|end| *end <= bytes.len())
                    .ok_or_else(|| {
                        invalid_bytes!("bssubstr offset={offset} len={len} bytes_len={bytes_len}")
                    })?;
                let node = self.byte_slice_node(bytes_id, offset, len)?;
                Some((3, self.push_node(node)))
            }
            _ => None,
        };
        Ok(rewrite)
    }

    pub(in crate::runtime) fn bytes_unop(
        &mut self,
        name: &str,
        args: &[NodeId],
    ) -> Result<Option<(usize, NodeId)>, EvalError> {
        match self.bytes_unop_inner(name, args) {
            Ok(result) => Ok(result),
            Err(err) => Err(self.trace_invalid_op_error("bytes_unop", name, args, err)),
        }
    }

    pub(in crate::runtime) fn bytes_unop_inner(
        &mut self,
        name: &str,
        args: &[NodeId],
    ) -> Result<Option<(usize, NodeId)>, EvalError> {
        let node = match name {
            "packCString" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let bytes = self.read_c_string(ptr)?;
                self.push_node(Node::bytes(bytes))
            }
            "bsgrab" => {
                let ptr = self.eval_pointer_value(args[0])?;
                let bytes = self.read_c_string(ptr)?;
                self.push_node(Node::bytes(bytes))
            }
            "bslength" => {
                let bytes_id = self.eval_bytes_id(args[0])?;
                let len =
                    i64::try_from(self.bytes(bytes_id)?.len()).map_err(|_| EvalError::Overflow)?;
                self.int(len)
            }
            "headUTF8" => {
                let bytes_id = self.eval_bytes_id(args[0])?;
                let (codepoint, _) = head_utf8(self.bytes(bytes_id)?)?;
                self.int(codepoint as i64)
            }
            "tailUTF8" => {
                let bytes_id = self.eval_bytes_id(args[0])?;
                let bytes = self.bytes(bytes_id)?;
                let (_, offset) = head_utf8(bytes)?;
                let len = bytes.len() - offset;
                let node = self.byte_slice_node(bytes_id, offset, len)?;
                self.push_node(node)
            }
            "bsunpack" => {
                let bytes = self.eval_bytes(args[0])?;
                let values = bytes.into_iter().map(i64::from);
                self.int_list(values)
            }
            "fromUTF8" => {
                let bytes = self.eval_bytes(args[0])?;
                let values = decode_utf8_string_bytes(&bytes)?;
                self.int_list(values.into_iter().map(i64::from))
            }
            "bsfreeze" => {
                let bytes = self.eval_bytes_id(args[0])?;
                self.freeze_bytes(bytes)?
            }
            _ => return Ok(None),
        };
        Ok(Some((1, node)))
    }
}
