//! C-style allocation table, errno, and host integer result helpers.
use super::*;

impl Program {
    pub(in crate::runtime) fn alloc_memory(&mut self, size: usize) -> Result<i64, EvalError> {
        let bytes = vec![0; size];
        let mut slot = self.allocation_first_free;
        while self.allocations.get(slot).is_some_and(Option::is_some) {
            slot += 1;
        }
        if slot == self.allocations.len() {
            self.allocations.push(Some(bytes));
        } else {
            self.allocations[slot] = Some(bytes);
        }
        self.allocation_first_free = slot + 1;
        while self
            .allocations
            .get(self.allocation_first_free)
            .is_some_and(Option::is_some)
        {
            self.allocation_first_free += 1;
        }
        self.pointer_for_allocation(slot, 0)
    }

    pub(in crate::runtime) fn alloc_c_string_bytes(
        &mut self,
        bytes: &[u8],
    ) -> Result<i64, EvalError> {
        let len = bytes.len().checked_add(1).ok_or(EvalError::Overflow)?;
        let ptr = self.alloc_memory(len)?;
        self.write_pointer_bytes(ptr, bytes)?;
        self.write_pointer_bytes(
            ptr.checked_add(i64::try_from(bytes.len()).map_err(|_| EvalError::Overflow)?)
                .ok_or(EvalError::Overflow)?,
            &[0],
        )?;
        Ok(ptr)
    }

    pub(in crate::runtime) fn errno_ptr(&mut self) -> Result<i64, EvalError> {
        if let Some(ptr) = self.errno_ptr {
            return Ok(ptr);
        }
        let ptr = self.alloc_memory(size_of::<std::os::raw::c_int>())?;
        self.errno_ptr = Some(ptr);
        self.write_errno_cell(ptr)?;
        Ok(ptr)
    }

    pub(in crate::runtime) fn set_errno_value(&mut self, value: i32) -> Result<(), EvalError> {
        self.errno_value = value;
        if let Some(ptr) = self.errno_ptr {
            self.write_errno_cell(ptr)?;
        }
        Ok(())
    }

    pub(in crate::runtime) fn write_errno_cell(&mut self, ptr: i64) -> Result<(), EvalError> {
        let value = self.errno_value as std::os::raw::c_int;
        self.write_pointer_bytes(ptr, &value.to_ne_bytes())
    }

    pub(in crate::runtime) fn host_int_node(
        &mut self,
        result: HostIntResult,
    ) -> Result<Node, EvalError> {
        if let Some(errno) = result.errno {
            self.set_errno_value(errno)?;
        }
        Ok(Node::Int(result.value))
    }

    #[cfg_attr(not(all(unix, not(target_arch = "wasm32"))), allow(dead_code))]
    pub(in crate::runtime) fn syscall_result_node(
        &mut self,
        value: i64,
    ) -> Result<Node, EvalError> {
        if value < 0 {
            self.set_errno_value(last_errno())?;
        }
        Ok(Node::Int(value))
    }

    pub(in crate::runtime) fn calloc_memory(
        &mut self,
        count: usize,
        size: usize,
    ) -> Result<i64, EvalError> {
        let len = count.checked_mul(size).ok_or(EvalError::Overflow)?;
        self.alloc_memory(len)
    }

    pub(in crate::runtime) fn realloc_memory(
        &mut self,
        ptr: i64,
        size: usize,
    ) -> Result<i64, EvalError> {
        if ptr == 0 {
            return self.alloc_memory(size);
        }
        let (slot, offset) = self.decode_allocation_pointer(ptr)?;
        if offset != 0 {
            return Err(EvalError::InvalidByteString);
        }
        let bytes = self
            .allocations
            .get_mut(slot)
            .and_then(Option::as_mut)
            .ok_or(EvalError::InvalidByteString)?;
        bytes.resize(size, 0);
        self.pointer_for_allocation(slot, 0)
    }

    pub(in crate::runtime) fn free_memory(&mut self, ptr: i64) -> Result<(), EvalError> {
        if ptr == 0 {
            return Ok(());
        }
        let (slot_index, offset) = self.decode_allocation_pointer(ptr)?;
        if offset != 0 {
            return Err(EvalError::InvalidByteString);
        }
        let slot = self
            .allocations
            .get_mut(slot_index)
            .ok_or(EvalError::InvalidByteString)?;
        if slot.is_none() {
            return Err(EvalError::InvalidByteString);
        }
        *slot = None;
        if slot_index < self.allocation_first_free {
            self.allocation_first_free = slot_index;
        }
        Ok(())
    }
}
