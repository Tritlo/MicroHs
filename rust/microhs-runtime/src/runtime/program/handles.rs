//! Stable pointer, weak pointer, foreign pointer, and MVar helpers.
use super::*;

impl Program {
    pub(in crate::runtime) fn new_stable_ptr_handle(
        &mut self,
        value: NodeId,
    ) -> Result<i64, EvalError> {
        let mut slot = self.stable_ptr_first_free.max(1);
        while self.stable_ptrs.get(slot).is_some_and(Option::is_some) {
            slot += 1;
        }
        if slot == self.stable_ptrs.len() {
            self.stable_ptrs.push(Some(value));
        } else {
            self.stable_ptrs[slot] = Some(value);
        }
        self.stable_ptr_first_free = slot + 1;
        while self
            .stable_ptrs
            .get(self.stable_ptr_first_free)
            .is_some_and(Option::is_some)
        {
            self.stable_ptr_first_free += 1;
        }
        i64::try_from(slot).map_err(|_| EvalError::Overflow)
    }

    pub(in crate::runtime) fn new_stable_ptr(
        &mut self,
        value: NodeId,
    ) -> Result<NodeId, EvalError> {
        let handle = self.new_stable_ptr_handle(value)?;
        Ok(self.int(handle))
    }

    pub(in crate::runtime) fn stable_ptr_handle(&mut self, id: NodeId) -> Result<usize, EvalError> {
        usize::try_from(self.eval_int(id)?).map_err(|_| EvalError::InvalidStablePtr)
    }

    pub(in crate::runtime) fn deref_stable_ptr(&self, handle: usize) -> Result<NodeId, EvalError> {
        self.stable_ptrs
            .get(handle)
            .and_then(|value| *value)
            .ok_or(EvalError::InvalidStablePtr)
    }

    pub(in crate::runtime) fn free_stable_ptr(&mut self, handle: usize) -> Result<(), EvalError> {
        let slot = self
            .stable_ptrs
            .get_mut(handle)
            .ok_or(EvalError::InvalidStablePtr)?;
        if slot.is_none() {
            return Err(EvalError::InvalidStablePtr);
        }
        *slot = None;
        if handle != 0 && handle < self.stable_ptr_first_free {
            self.stable_ptr_first_free = handle;
        }
        Ok(())
    }

    pub(in crate::runtime) fn new_foreign_finalizer(&mut self, arg: i64) -> usize {
        let state = ForeignFinalizerState {
            arg,
            finalizer: None,
        };
        if let Some(index) = self.foreign_finalizer_free.pop() {
            self.foreign_finalizers[index] = Some(state);
            index
        } else {
            let index = self.foreign_finalizers.len();
            self.foreign_finalizers.push(Some(state));
            index
        }
    }

    pub(in crate::runtime) fn foreign_ptr_node(
        &mut self,
        bytes: Option<Vec<u8>>,
        offset: usize,
        ptr: i64,
    ) -> Node {
        let finalizer = Some(self.new_foreign_finalizer(ptr));
        Node::ForeignPtr(Box::new(ForeignPtrNode {
            bytes: bytes.map(Rc::from),
            offset,
            ptr,
            finalizer,
        }))
    }

    pub(in crate::runtime) fn offset_foreign_ptr(
        &mut self,
        id: NodeId,
        by: usize,
    ) -> Result<NodeId, EvalError> {
        let (bytes, offset, ptr, finalizer) = match self.cold_node(id) {
            Some(Node::ForeignPtr(foreign_ptr)) => (
                foreign_ptr.bytes.clone(),
                foreign_ptr.offset,
                foreign_ptr.ptr,
                foreign_ptr.finalizer,
            ),
            _ => return Err(EvalError::ExpectedForeignPtr(id)),
        };
        let offset = offset.checked_add(by).ok_or(EvalError::Overflow)?;
        let ptr = ptr
            .checked_add(i64::try_from(by).map_err(|_| EvalError::Overflow)?)
            .ok_or(EvalError::Overflow)?;
        Ok(self.push_node(Node::ForeignPtr(Box::new(ForeignPtrNode {
            bytes,
            offset,
            ptr,
            finalizer,
        }))))
    }

    pub(in crate::runtime) fn foreign_ptr_to_bytes(
        &mut self,
        id: NodeId,
        len: usize,
    ) -> Result<NodeId, EvalError> {
        let (ptr, offset, backing) = match self.cold_node(id) {
            Some(Node::ForeignPtr(foreign_ptr)) => (
                foreign_ptr.ptr,
                foreign_ptr.offset,
                foreign_ptr.bytes.clone(),
            ),
            _ => return Err(EvalError::ExpectedForeignPtr(id)),
        };
        let bytes = match self.read_pointer_bytes(ptr, len) {
            Ok(bytes) => bytes,
            Err(_) if backing.is_some() => {
                let bytes = backing.as_ref().expect("checked above");
                let end = offset
                    .checked_add(len)
                    .filter(|end| *end <= bytes.len())
                    .ok_or(EvalError::InvalidByteString)?;
                bytes[offset..end].to_vec()
            }
            Err(err) => return Err(err),
        };
        Ok(self.push_node(Node::bytes(bytes)))
    }

    pub(in crate::runtime) fn foreign_ptr_value(&self, id: NodeId) -> Result<i64, EvalError> {
        match self.cold_node(id) {
            Some(Node::ForeignPtr(foreign_ptr)) => Ok(foreign_ptr.ptr),
            _ => match self.cell(id).prim() {
                Some(prim) => std_handle_ptr(prim.name()).ok_or(EvalError::ExpectedForeignPtr(id)),
                _ => Err(EvalError::ExpectedForeignPtr(id)),
            },
        }
    }

    pub(in crate::runtime) fn eval_js_object_handle(
        &mut self,
        id: NodeId,
    ) -> Result<u32, EvalError> {
        let foreign_ptr = self.eval_foreign_ptr_id(id)?;
        u32::try_from(self.foreign_ptr_value(foreign_ptr)?).map_err(|_| EvalError::Overflow)
    }

    pub(in crate::runtime) fn js_object_node(&mut self, handle: u32) -> Node {
        self.foreign_ptr_node(None, 0, i64::from(handle))
    }

    pub(in crate::runtime) fn set_foreign_ptr_finalizer(
        &mut self,
        id: NodeId,
        finalizer: NodeId,
    ) -> Result<(), EvalError> {
        let finalizer = self.eval_foreign_finalizer(finalizer)?;
        let (group, ptr) = match self.cold_node(id) {
            Some(Node::ForeignPtr(foreign_ptr)) => (foreign_ptr.finalizer, foreign_ptr.ptr),
            _ => return Err(EvalError::ExpectedForeignPtr(id)),
        };
        let group = match group {
            Some(group) => group,
            None => {
                let group = self.new_foreign_finalizer(ptr);
                match self.cold_node_mut(id) {
                    Some(Node::ForeignPtr(foreign_ptr)) => {
                        foreign_ptr.finalizer = Some(group);
                    }
                    _ => return Err(EvalError::ExpectedForeignPtr(id)),
                }
                group
            }
        };
        let state = self
            .foreign_finalizers
            .get_mut(group)
            .and_then(Option::as_mut)
            .ok_or(EvalError::ExpectedForeignPtr(id))?;
        state.finalizer = Some(finalizer);
        Ok(())
    }

    pub(in crate::runtime) fn eval_foreign_finalizer(
        &mut self,
        id: NodeId,
    ) -> Result<ForeignFinalizer, EvalError> {
        let root = self.reduce_node_whnf(id, FORCE_REDUCTION_LIMIT)?;
        let root = self.resolve(root)?;
        if let Some(value) = self.cell_raw_fun_ptr_value(root) {
            return if value == 0 {
                Ok(ForeignFinalizer::RawZero)
            } else {
                Err(EvalError::UnsupportedForeignFinalizer(format!(
                    "FunPtr#{value}"
                )))
            };
        }
        match self.cold_node(root) {
            Some(Node::FunPtr(name)) => match name.as_str() {
                "0" => Ok(ForeignFinalizer::RawZero),
                "free" => Ok(ForeignFinalizer::Free),
                "closeb" => Ok(ForeignFinalizer::CloseB),
                _ => Err(EvalError::UnsupportedForeignFinalizer(name.to_string())),
            },
            Some(Node::Ffi(name)) => match name.as_str() {
                "&free" => Ok(ForeignFinalizer::Free),
                "&closeb" => Ok(ForeignFinalizer::CloseB),
                _ => Err(EvalError::UnsupportedForeignFinalizer(name.to_string())),
            },
            _ => Err(EvalError::UnsupportedForeignFinalizer(
                self.node_trace_summary(root),
            )),
        }
    }

    fn node_trace_summary(&self, id: NodeId) -> String {
        let Some(cell) = self.nodes.get(id.index()).copied() else {
            return format!("{id:?}:<missing>");
        };
        let node = cell.to_node(&self.cold_nodes);
        match node {
            Node::Int(n) => format!("{id:?}:Int({n})"),
            Node::Int64(n) => format!("{id:?}:Int64({n})"),
            Node::Ptr(ptr) => format!("{id:?}:Ptr({ptr})"),
            Node::RawFunPtr(ptr) => format!("{id:?}:RawFunPtr({ptr})"),
            Node::ThreadId(n) => format!("{id:?}:ThreadId({n})"),
            Node::Prim(name) => format!("{id:?}:Prim({})", name.name()),
            Node::Ffi(name) => format!("{id:?}:Ffi({name})"),
            Node::Bytes(bytes) => format!("{id:?}:Bytes(len={})", bytes.len()),
            Node::BytesView(view) => format!(
                "{id:?}:BytesView(base={:?}, offset={}, len={})",
                view.base, view.offset, view.len
            ),
            Node::MutableBytes(bytes) => {
                format!(
                    "{id:?}:MutableBytes(size={}, capacity={})",
                    bytes.size, bytes.capacity
                )
            }
            Node::ForeignPtr(ptr) => format!(
                "{id:?}:ForeignPtr(ptr={}, offset={}, bytes={})",
                ptr.ptr,
                ptr.offset,
                ptr.bytes.as_ref().map_or(0, |bytes| bytes.len())
            ),
            Node::App(fun, arg) => format!("{id:?}:App({fun:?},{arg:?})"),
            Node::Indir(target) => format!("{id:?}:Indir({target:?})"),
            Node::Free(next) => format!("{id:?}:Free({next:?})"),
            Node::BigInt(bytes) => format!("{id:?}:BigInt(len={})", bytes.len()),
            Node::Array(items) => format!("{id:?}:Array(len={})", items.len()),
            Node::Float64(n) => format!("{id:?}:Float64({n})"),
            Node::Float32(n) => format!("{id:?}:Float32({n})"),
            Node::Weak(_) => format!("{id:?}:Weak"),
            Node::MVar(_) => format!("{id:?}:MVar"),
            Node::JsCall(call) => format!("{id:?}:JsCall(tags={})", call.tags),
            Node::JsWrap { tags } => format!("{id:?}:JsWrap(tags={tags})"),
            Node::FunPtr(name) => format!("{id:?}:FunPtr({name})"),
            Node::Tick(bytes) => format!("{id:?}:Tick(len={})", bytes.len()),
        }
    }

    pub(in crate::runtime) fn new_weak_ptr(
        &mut self,
        key: NodeId,
        value: NodeId,
        finalizer: Option<NodeId>,
    ) -> NodeId {
        let finalizer = finalizer.map(|finalizer| {
            let world = self.world();
            self.app(finalizer, world)
        });
        let weak = self.push_node(Node::Weak(Box::new(WeakNode {
            key: Some(key),
            value: Some(value),
            finalizer,
        })));
        self.weak_nodes.push(weak);
        weak
    }

    pub(in crate::runtime) fn eval_weak_id(&mut self, id: NodeId) -> Result<NodeId, EvalError> {
        self.eval_whnf_value(
            id,
            |program, root| match program.cold_node(root) {
                Some(Node::Weak(_)) => Some(root),
                _ => None,
            },
            EvalError::ExpectedWeak,
        )
    }

    pub(in crate::runtime) fn deref_weak_ptr(&mut self, id: NodeId) -> Result<NodeId, EvalError> {
        let id = self.eval_weak_id(id)?;
        let value = match self.cold_node(id) {
            Some(Node::Weak(weak)) => weak.value,
            _ => unreachable!(),
        };
        Ok(match value {
            Some(value) => self.just(value),
            None => self.nothing(),
        })
    }

    pub(in crate::runtime) fn finalize_weak_ptr(&mut self, id: NodeId) -> Result<(), EvalError> {
        let id = self.eval_weak_id(id)?;
        let finalizer = match self.cold_node_mut(id) {
            Some(Node::Weak(weak)) => weak.finalizer.take(),
            _ => unreachable!(),
        };
        if let Some(finalizer) = finalizer {
            self.reduce_node_whnf(finalizer, FORCE_REDUCTION_LIMIT)?;
        }
        Ok(())
    }

    pub(in crate::runtime) fn eval_mvar_id(&mut self, id: NodeId) -> Result<NodeId, EvalError> {
        self.eval_whnf_value(
            id,
            |program, root| match program.cold_node(root) {
                Some(Node::MVar(_)) => Some(root),
                _ => None,
            },
            EvalError::ExpectedMVar,
        )
    }

    pub(in crate::runtime) fn scheduler_now_micros(&self) -> u128 {
        self.scheduler_epoch.elapsed().as_nanos() / 1_000
    }

    pub(in crate::runtime) fn current_thread_mut(&mut self) -> Option<&mut ThreadControl> {
        self.threads
            .get_mut(self.current_thread)
            .and_then(Option::as_mut)
    }

    pub(in crate::runtime) fn check_pending_async_exception(
        &mut self,
        interruptible_blocking_point: bool,
    ) -> Result<(), EvalError> {
        if self.pending_async_count == 0 {
            return Ok(());
        }
        let mask = self.masking_state;
        if mask == MASK_UNINTERRUPTIBLE
            || (!interruptible_blocking_point && mask == MASK_INTERRUPTIBLE)
        {
            return Ok(());
        }
        if let Some(exn) = self
            .threads
            .get_mut(self.current_thread)
            .and_then(Option::as_mut)
            .and_then(|thread| thread.pending_exception.take())
        {
            self.pending_async_count -= 1;
            return Err(EvalError::Raised(exn));
        }
        Ok(())
    }

    pub(in crate::runtime) fn block_current_thread_at(
        &mut self,
        reason: BlockReason,
        restart_root: NodeId,
    ) -> EvalError {
        self.save_current_thread_state(self.current_thread, restart_root);
        EvalError::Blocked(reason)
    }

    pub(in crate::runtime) fn register_mvar(&mut self, id: NodeId) {
        self.mvar_waiters.entry(id).or_default();
    }

    pub(in crate::runtime) fn read_mvar(
        &mut self,
        id: NodeId,
        blocking: bool,
    ) -> Result<Option<NodeId>, EvalError> {
        if let Some(value) = self
            .threads
            .get_mut(self.current_thread)
            .and_then(Option::as_mut)
            .and_then(|thread| thread.delivered_value.take())
        {
            return Ok(Some(value));
        }
        match self.cold_node(id) {
            Some(Node::MVar(Some(value))) => Ok(Some(*value)),
            Some(Node::MVar(None)) if blocking => {
                Err(EvalError::Blocked(BlockReason::ReadMVar(id)))
            }
            Some(Node::MVar(None)) => Ok(None),
            _ => Err(EvalError::ExpectedMVar(id)),
        }
    }

    pub(in crate::runtime) fn take_mvar(
        &mut self,
        id: NodeId,
        blocking: bool,
    ) -> Result<Option<NodeId>, EvalError> {
        let value = match self.cold_node_mut(id) {
            Some(Node::MVar(value @ Some(_))) => value.take(),
            Some(Node::MVar(None)) if blocking => {
                return Err(EvalError::Blocked(BlockReason::TakeMVar(id)));
            }
            Some(Node::MVar(None)) => return Ok(None),
            _ => return Err(EvalError::ExpectedMVar(id)),
        };
        self.wake_one_takeput(id);
        Ok(value)
    }

    pub(in crate::runtime) fn put_mvar(
        &mut self,
        id: NodeId,
        new_value: NodeId,
        blocking: bool,
    ) -> Result<bool, EvalError> {
        match self.cold_node_mut(id) {
            Some(Node::MVar(value)) if value.is_none() => {
                *value = Some(new_value);
                self.wake_one_takeput(id);
                self.wake_all_readers(id, new_value);
                Ok(true)
            }
            Some(Node::MVar(_)) if blocking => Err(EvalError::Blocked(BlockReason::PutMVar(id))),
            Some(Node::MVar(_)) => Ok(false),
            _ => Err(EvalError::ExpectedMVar(id)),
        }
    }

    pub(in crate::runtime) fn wake_one_takeput(&mut self, id: NodeId) {
        loop {
            let Some(slot) = self
                .mvar_waiters
                .get_mut(&id)
                .and_then(|queues| queues.takeput.pop_front())
            else {
                return;
            };
            if self.make_runnable(slot) {
                return;
            }
        }
    }

    pub(in crate::runtime) fn wake_all_readers(&mut self, id: NodeId, value: NodeId) {
        let readers = self
            .mvar_waiters
            .get_mut(&id)
            .map(|queues| queues.read.drain(..).collect::<Vec<_>>())
            .unwrap_or_default();
        for slot in readers {
            if let Some(thread) = self.threads.get_mut(slot).and_then(Option::as_mut) {
                thread.delivered_value = Some(value);
            }
            self.make_runnable(slot);
        }
    }

    pub(in crate::runtime) fn make_runnable(&mut self, slot: usize) -> bool {
        if self.threads.get(slot).and_then(Option::as_ref).is_none() {
            return false;
        }
        if let Some(state) = self.thread_states.get_mut(slot) {
            *state = ThreadState::Runnable;
        }
        self.delay_wakeups.remove(&slot);
        if slot != self.current_thread && !self.run_queue.contains(&slot) {
            self.run_queue.push_back(slot);
        }
        true
    }

    pub(in crate::runtime) fn thread_slot_for_id(&self, id: i64) -> Option<usize> {
        self.thread_ids
            .iter()
            .position(|thread_id| *thread_id == id)
    }

    pub(in crate::runtime) fn throw_to_thread(
        &mut self,
        id: i64,
        exn: NodeId,
    ) -> Result<(), EvalError> {
        let Some(slot) = self.thread_slot_for_id(id) else {
            return Ok(());
        };
        let Some(thread) = self.threads.get_mut(slot).and_then(Option::as_mut) else {
            return Ok(());
        };
        let was_empty = thread.pending_exception.is_none();
        thread.pending_exception = Some(exn);
        let should_unpark = thread.masking_state != MASK_UNINTERRUPTIBLE;
        if was_empty {
            self.pending_async_count += 1;
        }
        if should_unpark {
            self.unpark_thread(slot);
        }
        Ok(())
    }

    pub(in crate::runtime) fn unpark_thread(&mut self, slot: usize) {
        for queues in self.mvar_waiters.values_mut() {
            queues.takeput.retain(|waiter| *waiter != slot);
            queues.read.retain(|waiter| *waiter != slot);
        }
        if let Some(thread) = self.threads.get_mut(slot).and_then(Option::as_mut) {
            thread.delivered_value = None;
        }
        self.delay_wakeups.remove(&slot);
        self.make_runnable(slot);
    }

    pub(in crate::runtime) fn int_list(&mut self, values: impl IntoIterator<Item = i64>) -> NodeId {
        let values: Vec<_> = values.into_iter().collect();
        let mut list = self.prim("K");
        for value in values.into_iter().rev() {
            let cons = self.prim("O");
            let value = self.int(value);
            let head = self.app(cons, value);
            list = self.app(head, list);
        }
        list
    }

    pub(in crate::runtime) fn arg_ref_array(&mut self) -> NodeId {
        if let Some(array) = self.arg_ref_array {
            return array;
        }
        let mut list = self.prim("K");
        for arg in self.program_args.clone().into_iter().rev() {
            let string = self.int_list(arg.into_iter().map(i64::from));
            let cons = self.prim("O");
            let cons_string = self.app(cons, string);
            list = self.app(cons_string, list);
        }
        let array = self.push_node(Node::array(vec![list]));
        self.arg_ref_array = Some(array);
        array
    }

    pub(in crate::runtime) fn reduce_node_whnf(
        &mut self,
        root: NodeId,
        limit: usize,
    ) -> Result<NodeId, EvalError> {
        self.reduce_whnf_from(root, limit, false)
            .map(|(root, _)| root)
    }

    pub(in crate::runtime) fn rnf(&mut self, noerr: bool, root: NodeId) -> Result<(), EvalError> {
        let mut seen = HashSet::new();
        let mut stack = vec![root];
        while let Some(root) = stack.pop() {
            let root = self.resolve(root)?;
            if !seen.insert(root) {
                continue;
            }
            let root = match self.reduce_node_whnf(root, FORCE_REDUCTION_LIMIT) {
                Ok(root) => self.resolve(root)?,
                Err(EvalError::Raised(_)) if noerr => continue,
                Err(err) => return Err(err),
            };
            if let Some((fun, arg)) = self.cell(root).app_fields() {
                stack.push(arg);
                stack.push(fun);
            }
        }
        Ok(())
    }

    pub fn render(&self, root: NodeId) -> String {
        let mut out = String::new();
        self.render_into(root, 0, &mut out);
        out
    }

    pub(in crate::runtime) fn render_into(&self, id: NodeId, depth: usize, out: &mut String) {
        if depth > 80 {
            out.push_str("...");
            return;
        }
        let Ok(id) = self.resolve(id) else {
            out.push_str("<dangling>");
            return;
        };
        let node = self.node_for_debug(id);
        match node {
            Node::App(fun, arg) => {
                out.push('(');
                self.render_into(fun, depth + 1, out);
                out.push(' ');
                self.render_into(arg, depth + 1, out);
                out.push(')');
            }
            Node::Indir(_) => out.push_str("<indir>"),
            Node::Free(_) => out.push_str("<free>"),
            Node::Prim(name) => out.push_str(name.name()),
            Node::Int(n) => out.push_str(&n.to_string()),
            Node::Int64(n) => {
                out.push_str(&n.to_string());
                out.push_str("i64");
            }
            Node::Float64(n) => out.push_str(&format_float(n)),
            Node::Float32(n) => {
                out.push_str(&format_float(f64::from(n)));
                out.push('f');
            }
            Node::ThreadId(n) => {
                out.push_str("ThreadId#");
                out.push_str(&n.to_string());
            }
            Node::Ptr(n) => {
                out.push_str("Ptr#");
                out.push_str(&n.to_string());
            }
            Node::RawFunPtr(n) => {
                out.push_str("FunPtr#");
                out.push_str(&n.to_string());
            }
            Node::ForeignPtr(foreign_ptr) => {
                if let Some(mpz) = self.mpz_decimal_bytes_for_ptr(foreign_ptr.ptr) {
                    out.push('%');
                    render_bytes(mpz, out);
                } else {
                    out.push_str("ForeignPtr#");
                    out.push_str(&foreign_ptr.ptr.to_string());
                }
            }
            Node::Weak(_) => {
                out.push_str("Weak#");
                out.push_str(&id.0.to_string());
            }
            Node::MVar(_) => {
                out.push_str("MVar#");
                out.push_str(&id.0.to_string());
            }
            Node::BigInt(bytes) => {
                out.push('%');
                render_bytes(&bytes, out);
            }
            Node::Bytes(bytes) => render_bytes(&bytes, out),
            Node::BytesView(_) => {
                if let Ok(bytes) = self.bytes(id) {
                    render_bytes(bytes, out);
                } else {
                    out.push_str("<bytes-view>");
                }
            }
            Node::MutableBytes(bytes) => render_bytes(bytes.visible(), out),
            Node::Array(items) => {
                out.push('[');
                for (idx, item) in items.iter().enumerate() {
                    if idx != 0 {
                        out.push_str(", ");
                    }
                    self.render_into(*item, depth + 1, out);
                }
                out.push(']');
            }
            Node::Ffi(name) => {
                out.push('^');
                out.push_str(&name);
            }
            Node::JsCall(call) => {
                out.push('~');
                out.push_str(&call.tags);
                out.push(' ');
                render_bytes(&call.body, out);
            }
            Node::JsWrap { tags } => {
                out.push('`');
                out.push_str(&tags);
            }
            Node::FunPtr(name) => {
                out.push(';');
                out.push_str(&name);
            }
            Node::Tick(name) => {
                out.push('!');
                render_bytes(&name, out);
            }
        }
    }
}
