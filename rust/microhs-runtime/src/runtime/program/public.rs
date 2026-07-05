//! Public Program API for reduction, profiling, and runtime configuration.
use super::*;

/// Reductions a thread runs before the scheduler may switch to another (C's `SLICE`).
const REDUCTION_SLICE: usize = 100_000;

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
        })];
        self.next_thread_id = 2;
        self.run_queue = std::collections::VecDeque::from([0usize]);
        self.current_thread = 0;
        self.root = main_root;
        let start = self.reductions;
        // Cooperative round-robin scheduler. Each thread is a graph reduced in slices:
        // on StepLimit the eval stack is discarded but the graph keeps every completed
        // reduction (redexes rewritten to indirections) and the thread root advances to
        // the continuation, so re-reducing resumes from the frontier without replaying
        // side effects. With one thread this is exactly the transparent single-thread
        // driver (byte-identical self-host).
        loop {
            let Some(tid) = self.run_queue.pop_front() else {
                return Err(EvalError::Deadlock);
            };
            let Some(root) = self.threads.get(tid).and_then(|t| t.as_ref()).map(|t| t.root) else {
                continue; // reaped slot left in the queue
            };
            self.current_thread = tid;
            self.root = root;
            let used = self.reductions - start;
            let remaining = limit.saturating_sub(used);
            if remaining == 0 {
                return Err(EvalError::StepLimit { limit });
            }
            let slice = REDUCTION_SLICE.min(remaining);
            match self.reduce_node_whnf(root, slice) {
                Ok(final_root) => {
                    if tid == 0 {
                        // main finished: the program is done; other threads are dropped.
                        self.root = final_root;
                        return Ok((final_root, self.reductions - start));
                    }
                    self.threads[tid] = None; // reap a finished child
                }
                Err(EvalError::StepLimit { .. }) => {
                    self.run_queue.push_back(tid); // slice expired; resume later
                }
                Err(err) => {
                    if tid == 0 {
                        return Err(err);
                    }
                    // A child died with an uncaught exception; reap it. (Refined when
                    // the throwTo tests land.)
                    self.threads[tid] = None;
                }
            }
        }
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
