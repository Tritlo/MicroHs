//! Public Program API for reduction, profiling, and runtime configuration.
use super::*;

#[cfg(feature = "embedded")]
const EMBEDDED_POLL_INTERVAL: usize = 4 * 1024 * 1024;

impl Program {
    pub(in crate::runtime) fn resolve(&self, mut id: NodeId) -> Result<NodeId, EvalError> {
        loop {
            let Some(cell) = self.nodes.get(id.index()).copied() else {
                return Err(EvalError::DanglingIndirection(id));
            };
            match cell.tag_bits() {
                tag if tag == CellTag::Indir.bits() => match cell.option_id_word1() {
                    Some(next) => id = next,
                    None => return Err(EvalError::DanglingIndirection(id)),
                },
                tag if tag == CellTag::Free.bits() => {
                    return Err(EvalError::DanglingIndirection(id));
                }
                _ => return Ok(id),
            }
        }
    }

    #[inline]
    pub(in crate::runtime) fn resolve_whnf_trusted(&self, mut id: NodeId) -> NodeId {
        loop {
            let cell = self.cell_trusted(id);
            match cell.tag_bits() {
                tag if tag == CellTag::Indir.bits() => {
                    let target = cell
                        .indir_target_trusted()
                        .expect("trusted indirection must have a target");
                    debug_assert_ne!(pack_option_id(Some(target)), CELL_NONE_ID);
                    id = target;
                }
                tag => {
                    debug_assert_ne!(tag, CellTag::Free.bits());
                    return id;
                }
            }
        }
    }

    pub(in crate::runtime) fn resolve_profiled(
        &mut self,
        mut id: NodeId,
    ) -> Result<NodeId, EvalError> {
        if !self.profiling_enabled() {
            return self.resolve(id);
        }

        let mut depth = 0;
        loop {
            let Some(cell) = self.nodes.get(id.index()).copied() else {
                return Err(EvalError::DanglingIndirection(id));
            };
            match cell.tag_bits() {
                tag if tag == CellTag::Indir.bits() => match cell.option_id_word1() {
                    Some(next) => {
                        id = next;
                        depth += 1;
                    }
                    None => return Err(EvalError::DanglingIndirection(id)),
                },
                tag if tag == CellTag::Free.bits() => {
                    return Err(EvalError::DanglingIndirection(id));
                }
                _ => {
                    self.profile_resolve_chain(depth);
                    return Ok(id);
                }
            }
        }
    }

    pub fn reduce_whnf(&mut self, limit: usize) -> Result<(NodeId, usize), EvalError> {
        let (root, steps) = self.reduce_whnf_from(self.root, limit, true)?;
        self.root = root;
        Ok((root, steps))
    }

    pub fn reduce_main(&mut self, limit: usize) -> Result<(NodeId, usize), EvalError> {
        let world = self.world();
        let main_root = self.app(self.root, world);
        // `main` is thread slot 0 (id 1). Track it in `self.root` so the GC — which
        // always marks `self.root` — follows the running program rather than pinning
        // the original `main` template (weak pointers depend on that liveness).
        self.threads = vec![Some(ThreadControl {
            id: 1,
            root: main_root,
            delivered_value: None,
            pending_exception: None,
            delay_ready: false,
            masking_state: MASK_UNMASKED,
        })];
        self.thread_ids = vec![1];
        self.thread_states = vec![ThreadState::Runnable];
        self.live_thread_count = 1;
        self.pending_async_count = 0;
        self.next_thread_id = 2;
        self.run_queue = std::collections::VecDeque::from([0usize]);
        self.mvar_waiters.clear();
        self.delay_wakeups.clear();
        self.scheduler_epoch = Instant::now();
        self.current_thread = 0;
        self.preserve_thread_root_once = false;
        self.root = main_root;
        let start = self.reductions;
        #[cfg(feature = "embedded")]
        let mut next_poll = start.saturating_add(EMBEDDED_POLL_INTERVAL);
        // Cooperative round-robin scheduler. Each thread is a graph reduced in slices:
        // on StepLimit the eval stack is discarded but the graph keeps every completed
        // reduction (redexes rewritten to indirections) and the thread root advances to
        // the continuation, so re-reducing resumes from the frontier without replaying
        // side effects. With one thread this is exactly the transparent single-thread
        // driver (byte-identical self-host).
        loop {
            #[cfg(feature = "embedded")]
            if self.reductions >= next_poll {
                std::hint::cold_path();
                if embedded_poll_cancelled(self.reductions) {
                    return Err(EvalError::Cancelled);
                }
                while self.reductions >= next_poll {
                    let advanced = next_poll.saturating_add(EMBEDDED_POLL_INTERVAL);
                    if advanced == next_poll {
                        break;
                    }
                    next_poll = advanced;
                }
            }
            self.wake_due_delays();
            let Some(tid) = self.run_queue.pop_front() else {
                self.wait_for_runnable_thread()?;
                continue;
            };
            let Some(root) = self
                .threads
                .get(tid)
                .and_then(|t| t.as_ref())
                .map(|t| t.root)
            else {
                continue; // reaped slot left in the queue
            };
            self.current_thread = tid;
            if let Some(thread) = self.threads[tid].as_ref() {
                self.masking_state = thread.masking_state;
            }
            self.root = root;
            let used = self.reductions - start;
            let remaining = limit.saturating_sub(used);
            if remaining == 0 {
                return Err(EvalError::StepLimit { limit });
            }
            // A lone runnable thread runs unbounded — full single-thread speed with no
            // slicing overhead (the self-host stays byte-identical and fast). Once a
            // second thread exists, slice preemptively so threads interleave.
            let alive = self.live_thread_count;
            let slice = if alive > 1 {
                REDUCTION_SLICE.min(remaining)
            } else {
                remaining
            };
            #[cfg(feature = "embedded")]
            let slice = slice.min(next_poll.saturating_sub(self.reductions).max(1));
            match self.reduce_node_whnf(root, slice) {
                Ok(final_root) => {
                    self.save_current_thread_state(tid, final_root);
                    if tid == 0 {
                        // main finished: the program is done; other threads are dropped.
                        self.root = final_root;
                        self.flush_open_bfiles()?;
                        return Ok((final_root, self.reductions - start));
                    }
                    self.finish_thread(tid, final_root);
                }
                Err(EvalError::StepLimit { .. }) => {
                    if !std::mem::take(&mut self.preserve_thread_root_once) {
                        self.save_current_thread_state(tid, root);
                    }
                    if std::mem::take(&mut self.reschedule_now) {
                        // Yielded right after a fork: keep running this thread next so it
                        // makes progress before the new child (preserves output order).
                        self.run_queue.push_front(tid);
                    } else {
                        self.run_queue.push_back(tid); // slice expired; resume later
                    }
                }
                Err(EvalError::Blocked(reason)) => {
                    self.park_thread(tid, reason);
                }
                Err(err) => {
                    self.save_current_thread_state(tid, root);
                    if tid == 0 {
                        let _ = self.flush_open_bfiles();
                        return Err(err);
                    }
                    if let EvalError::Raised(exn) = err {
                        self.print_child_exception(exn)?;
                    }
                    // A child died with an uncaught exception; reap it. (Refined when
                    // the throwTo tests land.)
                    self.finish_thread(tid, root);
                }
            }
        }
    }

    pub(in crate::runtime) fn save_current_thread_state(&mut self, tid: usize, root: NodeId) {
        if let Some(thread) = self.threads.get_mut(tid).and_then(Option::as_mut) {
            thread.root = root;
            thread.masking_state = self.masking_state;
        }
    }

    pub(in crate::runtime) fn finish_thread(&mut self, tid: usize, root: NodeId) {
        self.save_current_thread_state(tid, root);
        self.live_thread_count -= 1;
        if self
            .threads
            .get(tid)
            .and_then(Option::as_ref)
            .and_then(|thread| thread.pending_exception)
            .is_some()
        {
            self.pending_async_count -= 1;
        }
        if let Some(state) = self.thread_states.get_mut(tid) {
            *state = ThreadState::Finished;
        }
        self.delay_wakeups.remove(&tid);
        for queues in self.mvar_waiters.values_mut() {
            queues.takeput.retain(|slot| *slot != tid);
            queues.read.retain(|slot| *slot != tid);
        }
        self.threads[tid] = None;
    }

    pub(in crate::runtime) fn park_thread(&mut self, tid: usize, reason: BlockReason) {
        match reason {
            BlockReason::TakeMVar(mvar) | BlockReason::PutMVar(mvar) => {
                self.mvar_waiters
                    .entry(mvar)
                    .or_default()
                    .takeput
                    .push_back(tid);
                if let Some(state) = self.thread_states.get_mut(tid) {
                    *state = ThreadState::BlockedMVar;
                }
            }
            BlockReason::ReadMVar(mvar) => {
                self.mvar_waiters
                    .entry(mvar)
                    .or_default()
                    .read
                    .push_back(tid);
                if let Some(state) = self.thread_states.get_mut(tid) {
                    *state = ThreadState::BlockedMVar;
                }
            }
            BlockReason::Delay(wake) => {
                self.delay_wakeups.insert(tid, wake);
                if let Some(state) = self.thread_states.get_mut(tid) {
                    *state = ThreadState::BlockedOther;
                }
            }
        }
    }

    pub(in crate::runtime) fn wake_due_delays(&mut self) {
        let now = self.scheduler_now_micros();
        let due = self
            .delay_wakeups
            .iter()
            .filter_map(|(slot, wake)| (*wake <= now).then_some(*slot))
            .collect::<Vec<_>>();
        for slot in due {
            self.delay_wakeups.remove(&slot);
            if let Some(thread) = self.threads.get_mut(slot).and_then(Option::as_mut) {
                thread.delay_ready = true;
            }
            self.make_runnable(slot);
        }
    }

    pub(in crate::runtime) fn wait_for_runnable_thread(&mut self) -> Result<(), EvalError> {
        if self.delay_wakeups.is_empty() {
            return Err(EvalError::Deadlock);
        }
        loop {
            self.wake_due_delays();
            if !self.run_queue.is_empty() {
                return Ok(());
            }
            let Some(next_wake) = self.delay_wakeups.values().copied().min() else {
                return Err(EvalError::Deadlock);
            };
            let now = self.scheduler_now_micros();
            if next_wake > now {
                let sleep_micros = ((next_wake - now) / 4).max(50);
                let sleep_micros = u64::try_from(sleep_micros).unwrap_or(u64::MAX);
                std::thread::sleep(std::time::Duration::from_micros(sleep_micros));
            }
        }
    }

    pub(in crate::runtime) fn print_child_exception(
        &mut self,
        exn: NodeId,
    ) -> Result<(), EvalError> {
        let message = self.uncaught_exception_message_bytes(exn)?;
        let mut line = b"Uncaught child exception: ".to_vec();
        line.extend_from_slice(&message);
        line.push(b'\n');
        self.write_io_handle_bytes(StdHandle::Stdout, &line)?;
        Ok(())
    }

    pub fn reduction_count(&self) -> usize {
        self.reductions
    }

    pub fn uncaught_exception_message_bytes(&mut self, exn: NodeId) -> Result<Vec<u8>, EvalError> {
        let exn = self.resolve(exn)?;
        if let Some(code) = self.cell_int_value(exn) {
            return Ok(rts_exception_message(code).to_vec());
        }

        let u = self.prim("U");
        let k2 = self.prim("K2");
        let true_ = self.prim("A");
        let k2_true = self.app(k2, true_);
        let inner = self.app(u, k2_true);
        let show_exn = self.app(u, inner);
        let displayed = self.app(show_exn, exn);
        self.eval_string_bytes(displayed)
    }

    #[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
    fn apply_js_wrapper(
        &mut self,
        tags: &str,
        stable_ptr: usize,
        args: &[JsValue],
        limit: usize,
    ) -> Result<JsValue, EvalError> {
        let tags = tags.as_bytes();
        validate_js_tags(tags)?;
        if args.len() != tags.len() - 1 {
            return Err(EvalError::InvalidArray);
        }

        let mut root = self.deref_stable_ptr(stable_ptr)?;
        for (tag, arg) in tags[1..].iter().copied().zip(args) {
            let arg = self.js_value_node(tag, arg)?;
            root = self.app(root, arg);
        }
        let perform_io = self.prim("IO.performIO");
        let root = self.app(perform_io, root);
        let root = self.reduce_node_whnf(root, limit)?;
        self.js_value_from_node(tags[0], root)
    }

    #[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
    pub(crate) fn apply_js_wrapper_index(
        &mut self,
        wrapper_index: u32,
        stable_ptr: usize,
        args: &[JsValue],
        limit: usize,
    ) -> Result<JsValue, EvalError> {
        let tags = self.js_wrapper_tags(wrapper_index)?.to_owned();
        self.apply_js_wrapper(&tags, stable_ptr, args, limit)
    }

    #[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
    pub(crate) fn js_wrapper_tags(&self, wrapper_index: u32) -> Result<&str, EvalError> {
        self.js_wrapper_tags
            .get(usize::try_from(wrapper_index).map_err(|_| EvalError::Overflow)?)
            .map(String::as_str)
            .ok_or(EvalError::InvalidArray)
    }
}

#[cfg(feature = "embedded")]
#[inline(never)]
fn embedded_poll_cancelled(steps_so_far: usize) -> bool {
    std::hint::cold_path();
    host_poll(u64::try_from(steps_so_far).unwrap_or(u64::MAX))
}

#[cfg(feature = "embedded")]
std::cfg_select! {
    all(target_arch = "wasm32", not(target_os = "wasi")) => {
        fn host_poll(steps_so_far: u64) -> bool {
            unsafe { mhs_host_poll(steps_so_far) != 0 }
        }

        #[link(wasm_import_module = "env")]
        unsafe extern "C" {
            fn mhs_host_poll(steps_so_far: u64) -> i32;
        }
    }
    _ => {
        fn host_poll(_steps_so_far: u64) -> bool {
            false
        }
    }
}
