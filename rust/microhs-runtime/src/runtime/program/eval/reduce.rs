//! General fallback reducer used when the stack fast path delegates.
use super::*;

impl Program {
    pub(in crate::runtime) fn fill_eval_spine(
        &mut self,
        mut node: NodeId,
        spine: &mut EvalSpine,
    ) -> Result<NodeId, EvalError> {
        spine.clear();
        while let Some((fun, arg)) = self.cell(node).app_fields() {
            spine.push_desc(arg, node);
            node = self.resolve_profiled(fun)?;
        }
        Ok(node)
    }

    pub(in crate::runtime) fn apply_eval_spine_rewrite(
        &mut self,
        root: NodeId,
        spine: &EvalSpine,
        used: usize,
        mut node: NodeId,
    ) -> NodeId {
        let len = spine.len();
        debug_assert!(used <= len);
        if used == 0 && len == 0 {
            if node != root {
                self.set_cell_at(root.index(), Cell::indir_trusted(node));
            }
            return node;
        }
        if used > 0 {
            let redex = spine.app(used - 1);
            if node != redex {
                self.set_app_cell_at(redex.index(), Cell::indir_trusted(node));
            }
        }
        for head_idx in used..len {
            let app = spine.app(head_idx);
            let arg = spine.arg(head_idx);
            self.set_app_cell_at(app.index(), Cell::app_trusted(node, arg));
            node = app;
        }
        node
    }

    pub(in crate::runtime) fn apply_eval_spine_app(
        &mut self,
        root: NodeId,
        spine: &EvalSpine,
        used: usize,
        fun: NodeId,
        arg: NodeId,
    ) -> NodeId {
        let len = spine.len();
        debug_assert!(used <= len);
        let mut node = if used == 0 {
            self.app(fun, arg)
        } else {
            let redex = spine.app(used - 1);
            self.set_app_cell_at(redex.index(), Cell::app_trusted(fun, arg));
            redex
        };
        if used == 0 && len == 0 {
            if node != root {
                self.set_cell_at(root.index(), Cell::indir_trusted(node));
            }
            return node;
        }
        for head_idx in used..len {
            let app = spine.app(head_idx);
            let arg = spine.arg(head_idx);
            self.set_app_cell_at(app.index(), Cell::app_trusted(node, arg));
            node = app;
        }
        node
    }

    pub(in crate::runtime) fn eval_loop_result(
        &mut self,
        profile_head: ProfileHead,
        node: NodeId,
        reductions: usize,
    ) -> EvalLoopStep {
        self.profile_reduction(profile_head, reductions);
        EvalLoopStep { node, reductions }
    }

    // Keep hot reducer state explicit instead of constructing a one-off argument bundle.
    #[allow(clippy::too_many_arguments)]
    pub(in crate::runtime) fn eval_loop_app_result(
        &mut self,
        profile_head: ProfileHead,
        root: NodeId,
        spine: &EvalSpine,
        used: usize,
        fun: NodeId,
        arg: NodeId,
        reductions: usize,
    ) -> EvalLoopStep {
        let node = self.apply_eval_spine_app(root, spine, used, fun, arg);
        self.eval_loop_result(profile_head, node, reductions)
    }

    // Nested shortcut checks benchmark faster in this reducer hot path.
    #[allow(clippy::collapsible_if)]
    pub(in crate::runtime) fn eval_loop_step(
        &mut self,
        root: NodeId,
        budget: usize,
        spine: &mut EvalSpine,
        scratch_args: &mut Vec<NodeId>,
        _scratch_apps: &mut Vec<NodeId>,
    ) -> Result<Option<EvalLoopStep>, EvalError> {
        if self.profiling_enabled() {
            self.profile_fallback_eval_loop_step();
        }
        let head = self.fill_eval_spine(root, spine)?;
        let args_len = spine.len();
        let profile_head = if self.profiling_enabled() {
            self.profile_step(head, args_len)
        } else {
            ProfileHead::none()
        };

        macro_rules! arg {
            ($idx:expr) => {
                spine.arg($idx)
            };
        }
        macro_rules! app_step {
            ($used:expr, $fun:expr, $arg:expr) => {
                return Ok(Some(self.eval_loop_app_result(
                    profile_head,
                    root,
                    spine,
                    $used,
                    $fun,
                    $arg,
                    1,
                )));
            };
        }
        macro_rules! rewrite_step {
            ($used:expr, $node:expr, $reductions:expr) => {{
                let node = self.apply_eval_spine_rewrite(root, spine, $used, $node);
                return Ok(Some(self.eval_loop_result(profile_head, node, $reductions)));
            }};
        }
        let head_dispatch = match self.cell(head).prim() {
            Some(Prim::Known(known)) => EvalHead::Known(known),
            Some(Prim::Runtime(runtime)) => {
                let name = runtime.name();
                let action = runtime.strict_action(args_len);
                EvalHead::Other {
                    action,
                    fallback_name: Some(name),
                }
            }
            None => match self.cold_node(head) {
                Some(Node::Ffi(name)) => EvalHead::Ffi(name.to_string()),
                Some(Node::JsCall(call)) => EvalHead::JsCall {
                    tags: call.tags.clone(),
                    body: call.body.clone(),
                },
                Some(Node::JsWrap { tags }) => EvalHead::JsWrap {
                    tags: tags.to_string(),
                },
                _ => EvalHead::Whnf,
            },
        };

        let (known, fallback_name) = match head_dispatch {
            EvalHead::Ffi(name) => {
                std::hint::cold_path();
                if self.profiling_enabled() {
                    self.profile_arg_materialization(args_len);
                }
                spine.write_args_head_order(scratch_args);
                let Some((used, node)) = self.ffi_call(&name, scratch_args.as_slice())? else {
                    return Ok(None);
                };
                rewrite_step!(used, node, 1);
            }
            EvalHead::JsCall { tags, body } => {
                std::hint::cold_path();
                if self.profiling_enabled() {
                    self.profile_arg_materialization(args_len);
                }
                spine.write_args_head_order(scratch_args);
                let Some((used, node)) = self.js_call(&tags, &body, scratch_args.as_slice())?
                else {
                    return Ok(None);
                };
                rewrite_step!(used, node, 1);
            }
            EvalHead::JsWrap { tags } => {
                std::hint::cold_path();
                if self.profiling_enabled() {
                    self.profile_arg_materialization(args_len);
                }
                spine.write_args_head_order(scratch_args);
                let Some((used, node)) = self.js_wrap(&tags, scratch_args.as_slice())? else {
                    return Ok(None);
                };
                rewrite_step!(used, node, 1);
            }
            EvalHead::Known(known) => (Some(known), None),
            EvalHead::Other { fallback_name, .. } => (None, fallback_name),
            EvalHead::Whnf => return Ok(None),
        };
        use KnownPrim::*;

        if known == Some(U) && args_len >= 2 {
            if let Some(node) = self.selector_pair_field(arg!(0), arg!(1))? {
                self.profile_shortcut("selector_pair_field", 1);
                rewrite_step!(2, node, 1);
            }
        }

        let multi_threaded = self.live_thread_count > 1;

        if known == Some(IoThen) && args_len >= 3 && budget >= 2 && !multi_threaded {
            if let Some(reductions) = self.ignored_io_action_reductions(arg!(0), budget - 1)? {
                self.profile_shortcut("io_then_ignored_action", 1);
                let world = self
                    .run_ignored_io_action(arg!(0), arg!(2))?
                    .expect("preflighted ignored IO action should execute");
                let node = self.app(arg!(1), world);
                rewrite_step!(3, node, reductions + 1);
            }
            let k = self.prim("K");
            let then = self.app(k, arg!(1));
            let action = self.app(arg!(0), arg!(2));
            let node = self.app(action, then);
            rewrite_step!(3, node, 2);
        }

        if known == Some(IoBind) && args_len >= 3 {
            if let Some(result) = self.io_return_action_result(arg!(0))? {
                self.profile_shortcut("io_bind_return_action", 1);
                let next = self.app(arg!(1), result);
                let node = self.app(next, arg!(2));
                rewrite_step!(3, node, 2);
            }
        }

        let rewrite = match known {
            Some(I | Ord | Chr) if args_len >= 1 => Some((1, arg!(0))),
            Some(K) if args_len >= 2 => Some((2, arg!(0))),
            Some(A) if args_len >= 2 => Some((2, arg!(1))),
            Some(U) if args_len >= 2 => {
                app_step!(2, arg!(1), arg!(0));
            }
            Some(IoPerformIo) if args_len >= 1 => {
                let world = self.world();
                let k = self.prim("K");
                let action = self.app(arg!(0), world);
                let n = self.app(action, k);
                Some((1, n))
            }
            Some(IoAtomic) if args_len >= 2 => {
                let k = self.prim("K");
                let action = self.app(arg!(0), arg!(1));
                let result = self.app(action, k);
                let pair = self.prim("P");
                let result_pair = self.app(pair, result);
                let n = self.app(result_pair, arg!(1));
                Some((2, n))
            }
            Some(IoBind) if args_len >= 3 => {
                let action = self.app(arg!(0), arg!(2));
                let n = self.app(action, arg!(1));
                Some((3, n))
            }
            Some(IoThen) if args_len >= 2 => {
                let bind = self.prim("IO.>>=");
                let bind_action = self.app(bind, arg!(0));
                let k = self.prim("K");
                let then = self.app(k, arg!(1));
                let n = self.app(bind_action, then);
                Some((2, n))
            }
            Some(IoReturn) if args_len >= 3 => {
                let kx = self.app(arg!(2), arg!(0));
                let n = self.app(kx, arg!(1));
                Some((3, n))
            }
            Some(IoLazyBind) if args_len >= 3 => {
                let world_result = self.app(arg!(0), arg!(2));
                let fst = self.fst();
                let snd = self.snd();
                let result = self.app(fst, world_result);
                let world = self.app(snd, world_result);
                let next = self.app(arg!(1), result);
                let n = self.app(next, world);
                Some((3, n))
            }
            Some(IoStrict) if args_len >= 2 => {
                let n = self.app(arg!(0), arg!(1));
                Some((2, n))
            }
            Some(IoGc) if args_len >= 2 => {
                // Request a full collection at the next top-level step boundary (where
                // the GC roots are valid). This is what lets weak pointers whose keys
                // have gone out of scope die and their finalizers run.
                self.force_gc = true;
                let unit = self.prim("I");
                Some((2, self.pair(unit, arg!(1))))
            }
            Some(IoStats) if args_len >= 1 => {
                let alloc = self.int(i64::try_from(self.nodes.len()).unwrap_or(i64::MAX));
                let reductions = self.int(i64::try_from(self.reductions).unwrap_or(i64::MAX));
                let stats = self.pair(alloc, reductions);
                Some((1, self.pair(stats, arg!(0))))
            }
            Some(IoPp) if args_len >= 2 => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    let rendered = self.render(arg!(0));
                    eprintln!("{rendered}");
                }
                let unit = self.prim("I");
                Some((2, self.pair(unit, arg!(1))))
            }
            Some(IoPrint) if args_len >= 3 => {
                let ptr = self.eval_pointer_value(arg!(0))?;
                let value = self.reduce_node_whnf(arg!(1), FORCE_REDUCTION_LIMIT)?;
                let printed = match self.print_program(value) {
                    Ok(printed) => printed,
                    Err(_) => return Err(self.rts_exception(RTS_EXN_SERIALIZE)),
                };
                self.write_bfile_bytes(ptr, &printed)?;
                let unit = self.prim("I");
                Some((3, self.pair(unit, arg!(2))))
            }
            Some(IoSerialize) if args_len >= 3 => {
                let ptr = self.eval_pointer_value(arg!(0))?;
                let value = self.reduce_node_whnf(arg!(1), FORCE_REDUCTION_LIMIT)?;
                let serialized = match self.serialize_program(value) {
                    Ok(serialized) => serialized,
                    Err(_) => return Err(self.rts_exception(RTS_EXN_SERIALIZE)),
                };
                self.write_bfile_bytes(ptr, &serialized)?;
                let unit = self.prim("I");
                Some((3, self.pair(unit, arg!(2))))
            }
            Some(IoDeserialize) if args_len >= 2 => {
                let ptr = self.eval_pointer_value(arg!(0))?;
                let value = match self.deserialize_bfile(ptr) {
                    Ok(value) => value,
                    Err(_) => return Err(self.rts_exception(RTS_EXN_DESERIALIZE)),
                };
                Some((2, self.pair(value, arg!(1))))
            }
            Some(IoGetArgRef) if args_len >= 1 => {
                let arg_array = self.arg_ref_array();
                Some((1, self.pair(arg_array, arg!(0))))
            }
            Some(IoThid) if args_len >= 1 => {
                let id = self
                    .threads
                    .get(self.current_thread)
                    .and_then(Option::as_ref)
                    .map(|thread| thread.id)
                    .unwrap_or(1);
                let thread = self.push_node(Node::ThreadId(id));
                Some((1, self.pair(thread, arg!(0))))
            }
            Some(IoYield) if args_len >= 1 => {
                self.check_pending_async_exception(false)?;
                self.run_pending_weak_finalizers()?;
                let unit = self.prim("I");
                Some((1, self.pair(unit, arg!(0))))
            }
            Some(IoFork) if args_len >= 2 => {
                // forkIO action: spawn a runnable child reducing `action world`, and
                // return its ThreadId to the current thread. Preemption is by slice, so
                // the child runs when the current thread's slice expires.
                let action = arg!(0);
                let world = self.world();
                let child_root = self.app(action, world);
                let id = self.next_thread_id;
                self.next_thread_id += 1;
                let slot = self.threads.len();
                self.threads.push(Some(ThreadControl {
                    id,
                    root: child_root,
                    delivered_value: None,
                    pending_exception: None,
                    delay_ready: false,
                    masking_state: self.masking_state,
                }));
                self.thread_ids.push(id);
                self.thread_states.push(ThreadState::Runnable);
                self.live_thread_count += 1;
                self.run_queue.push_back(slot);
                // Leave the (previously single-thread, unbounded) slice so the scheduler
                // switches to preemptive slicing now that a second thread exists.
                self.reschedule_now = true;
                let thread_id = self.push_node(Node::ThreadId(id));
                Some((2, self.pair(thread_id, arg!(1))))
            }
            Some(IoGetMaskingState) if args_len >= 1 => {
                let state = self.int(self.masking_state);
                Some((1, self.pair(state, arg!(0))))
            }
            Some(IoSetMaskingState) if args_len >= 2 => {
                self.masking_state = self.eval_int(arg!(0))?;
                let masking_state = self.masking_state;
                if let Some(thread) = self.current_thread_mut() {
                    thread.masking_state = masking_state;
                }
                let unit = self.prim("I");
                Some((2, self.pair(unit, arg!(1))))
            }
            Some(Dynsym) if args_len >= 1 => {
                let name = self.eval_ffi_name(arg!(0))?;
                Some((1, self.push_node(Node::ffi(name))))
            }
            Some(IoThreadStatus) if args_len >= 2 => {
                let thread = self.eval_thread_id(arg!(0))?;
                let status = self
                    .thread_slot_for_id(thread)
                    .and_then(|slot| self.thread_states.get(slot))
                    .copied()
                    .unwrap_or(ThreadState::Finished);
                let status = self.int(match status {
                    ThreadState::Runnable => 0,
                    ThreadState::BlockedMVar => 1,
                    ThreadState::BlockedOther => 2,
                    ThreadState::Finished => 3,
                });
                Some((2, self.pair(status, arg!(1))))
            }
            Some(IoNewMVar) if args_len >= 1 => {
                let mvar = self.push_node(Node::MVar(None));
                self.register_mvar(mvar);
                Some((1, self.pair(mvar, arg!(0))))
            }
            Some(IoTakeMVar) if args_len >= 2 => {
                self.check_pending_async_exception(true)?;
                let mvar = self.eval_mvar_id(arg!(0))?;
                let value = match self.take_mvar(mvar, true) {
                    Ok(Some(value)) => value,
                    Ok(None) => return Err(EvalError::InvalidMVar),
                    Err(EvalError::Blocked(reason)) => {
                        return Err(self.block_current_thread_at(reason, root));
                    }
                    Err(err) => return Err(err),
                };
                Some((2, self.pair(value, arg!(1))))
            }
            Some(IoReadMVar) if args_len >= 2 => {
                self.check_pending_async_exception(true)?;
                let mvar = self.eval_mvar_id(arg!(0))?;
                let value = match self.read_mvar(mvar, true) {
                    Ok(Some(value)) => value,
                    Ok(None) => return Err(EvalError::InvalidMVar),
                    Err(EvalError::Blocked(reason)) => {
                        return Err(self.block_current_thread_at(reason, root));
                    }
                    Err(err) => return Err(err),
                };
                Some((2, self.pair(value, arg!(1))))
            }
            Some(IoPutMVar) if args_len >= 3 => {
                self.check_pending_async_exception(true)?;
                let mvar = self.eval_mvar_id(arg!(0))?;
                if let Err(err) = self.put_mvar(mvar, arg!(1), true) {
                    return Err(match err {
                        EvalError::Blocked(reason) => self.block_current_thread_at(reason, root),
                        err => err,
                    });
                }
                let unit = self.prim("I");
                Some((3, self.pair(unit, arg!(2))))
            }
            Some(IoTryTakeMVar) if args_len >= 2 => {
                let mvar = self.eval_mvar_id(arg!(0))?;
                let value = match self.take_mvar(mvar, false)? {
                    Some(value) => self.just(value),
                    None => self.nothing(),
                };
                Some((2, self.pair(value, arg!(1))))
            }
            Some(IoTryReadMVar) if args_len >= 2 => {
                let mvar = self.eval_mvar_id(arg!(0))?;
                let value = match self.read_mvar(mvar, false)? {
                    Some(value) => self.just(value),
                    None => self.nothing(),
                };
                Some((2, self.pair(value, arg!(1))))
            }
            Some(IoTryPutMVar) if args_len >= 3 => {
                let mvar = self.eval_mvar_id(arg!(0))?;
                let value = if self.put_mvar(mvar, arg!(1), false)? {
                    self.prim("A")
                } else {
                    self.prim("K")
                };
                Some((3, self.pair(value, arg!(2))))
            }
            Some(IoThreadDelay) if args_len >= 2 => {
                self.check_pending_async_exception(true)?;
                let delay_ready = self
                    .current_thread_mut()
                    .map(|thread| std::mem::take(&mut thread.delay_ready))
                    .unwrap_or(false);
                if delay_ready {
                    let unit = self.prim("I");
                    Some((2, self.pair(unit, arg!(1))))
                } else {
                    let usecs = self.eval_int(arg!(0))?;
                    let usecs = u128::try_from(usecs).map_err(|_| EvalError::Overflow)?;
                    let wake = self.scheduler_now_micros().saturating_add(usecs);
                    return Err(self.block_current_thread_at(BlockReason::Delay(wake), root));
                }
            }
            Some(IoThrowTo) if args_len >= 3 => {
                self.check_pending_async_exception(true)?;
                let thread = self.eval_thread_id(arg!(0))?;
                self.throw_to_thread(thread, arg!(1))?;
                let unit = self.prim("I");
                Some((3, self.pair(unit, arg!(2))))
            }
            Some(Catch) if args_len >= 3 => {
                let action = self.app(arg!(0), arg!(2));
                Some((3, self.catch_result(action, arg!(1), arg!(2))?))
            }
            Some(CatchR) if args_len >= 3 => {
                Some((3, self.catch_result(arg!(0), arg!(1), arg!(2))?))
            }
            Some(Raise) if args_len >= 1 => return Err(EvalError::Raised(arg!(0))),
            Some(Rnf) if args_len >= 2 => {
                let noerr = self.eval_int(arg!(0))? != 0;
                self.rnf(noerr, arg!(1))?;
                Some((2, self.prim("I")))
            }
            Some(Seq) if args_len >= 2 => Some((2, arg!(1))),
            Some(IsInt) if args_len >= 1 => {
                let root = self.resolve(arg!(0))?;
                let n = self.cell_int_value(root).unwrap_or(-1);
                Some((1, self.int(n)))
            }
            Some(Thnum) if args_len >= 1 => {
                let thread = self.eval_thread_id(arg!(0))?;
                Some((1, self.int(thread)))
            }
            Some(S) if args_len >= 3 => {
                let x = arg!(2);
                let left = self.app(arg!(0), x);
                let right = self.app(arg!(1), x);
                app_step!(3, left, right);
            }
            Some(SPrime) if args_len >= 4 => {
                let yw = self.app(arg!(1), arg!(3));
                let zw = self.app(arg!(2), arg!(3));
                let left = self.app(arg!(0), yw);
                app_step!(4, left, zw);
            }
            Some(B) if args_len >= 3 => {
                let yz = self.app(arg!(1), arg!(2));
                app_step!(3, arg!(0), yz);
            }
            Some(BPrime) if args_len >= 4 => {
                let zw = self.app(arg!(2), arg!(3));
                let xy = self.app(arg!(0), arg!(1));
                app_step!(4, xy, zw);
            }
            Some(BPrime) if args_len >= 2 => {
                let xy = self.app(arg!(0), arg!(1));
                let b = self.prim("B");
                app_step!(2, b, xy);
            }
            Some(Z) if args_len >= 3 => {
                app_step!(3, arg!(0), arg!(1));
            }
            Some(Z) if args_len >= 2 => {
                let xy = self.app(arg!(0), arg!(1));
                let k = self.prim("K");
                app_step!(2, k, xy);
            }
            Some(J) if args_len >= 3 => {
                app_step!(3, arg!(2), arg!(0));
            }
            Some(L) if args_len >= 3 => {
                app_step!(3, arg!(1), arg!(0));
            }
            Some(KK) if args_len >= 3 => Some((3, arg!(1))),
            Some(KA) if args_len >= 3 => Some((3, arg!(2))),
            Some(C) if args_len >= 3 => {
                let xz = self.app(arg!(0), arg!(2));
                app_step!(3, xz, arg!(1));
            }
            Some(CPrime) if args_len >= 4 => {
                let yw = self.app(arg!(1), arg!(3));
                let xyw = self.app(arg!(0), yw);
                app_step!(4, xyw, arg!(2));
            }
            Some(P) if args_len >= 3 => {
                let zx = self.app(arg!(2), arg!(0));
                app_step!(3, zx, arg!(1));
            }
            Some(R) if args_len >= 3 => {
                let yz = self.app(arg!(1), arg!(2));
                app_step!(3, yz, arg!(0));
            }
            Some(R) if args_len >= 2 => {
                let c = self.prim("C");
                let cy = self.app(c, arg!(1));
                app_step!(2, cy, arg!(0));
            }
            Some(O) if args_len >= 4 => {
                let wx = self.app(arg!(3), arg!(0));
                app_step!(4, wx, arg!(1));
            }
            Some(K2) if args_len >= 3 => Some((3, arg!(0))),
            Some(K2) if args_len >= 2 => {
                let k = self.prim("K");
                app_step!(2, k, arg!(0));
            }
            Some(K3) if args_len >= 4 => Some((4, arg!(0))),
            Some(K3) if args_len >= 2 => {
                let k2 = self.prim("K2");
                app_step!(2, k2, arg!(0));
            }
            Some(K4) if args_len >= 5 => Some((5, arg!(0))),
            Some(K4) if args_len >= 2 => {
                let k3 = self.prim("K3");
                app_step!(2, k3, arg!(0));
            }
            Some(CPrimeB) if args_len >= 4 => {
                let yw = self.app(arg!(1), arg!(3));
                let xz = self.app(arg!(0), arg!(2));
                app_step!(4, xz, yw);
            }
            Some(CPrimeB) if args_len >= 3 => {
                let xz = self.app(arg!(0), arg!(2));
                let b = self.prim("B");
                let bxz = self.app(b, xz);
                app_step!(3, bxz, arg!(1));
            }
            Some(Y) if args_len >= 1 => {
                app_step!(1, arg!(0), spine.app(0));
            }
            Some(Tag(tag)) if args_len >= 2 => {
                let tag = self.int(i64::from(tag));
                let ytag = self.app(arg!(1), tag);
                app_step!(2, ytag, arg!(0));
            }
            Some(Tuple(fields)) if args_len > usize::from(fields) => {
                let fields = usize::from(fields);
                if budget >= 2 {
                    let selector = arg!(fields);
                    let available_extra = args_len - fields - 1;
                    if let Some(extra_used) =
                        self.tuple_first_field_selector_extra(selector, fields, available_extra)?
                    {
                        self.profile_shortcut("tuple_first_field_selector", 1);
                        rewrite_step!(fields + 1 + extra_used, arg!(0), 2);
                    }
                }
                let mut n = arg!(fields);
                for idx in 0..fields - 1 {
                    n = self.app(n, arg!(idx));
                }
                app_step!(fields + 1, n, arg!(fields - 1));
            }
            _ if args_len >= 2 && fallback_name.is_some() => {
                let name = fallback_name.expect("fallback name checked above");
                let materialized_args = args_len.min(FALLBACK_PRIM_ARG_PREFIX);
                if self.profiling_enabled() {
                    self.profile_arg_materialization(materialized_args);
                }
                spine.write_args_head_order_prefix(scratch_args, FALLBACK_PRIM_ARG_PREFIX);
                self.fallback_runtime_prim_rewrite(name, scratch_args.as_slice(), args_len)?
            }
            _ if args_len >= 1 && fallback_name.is_some() => {
                let name = fallback_name.expect("fallback name checked above");
                let materialized_args = args_len.min(FALLBACK_PRIM_ARG_PREFIX);
                if self.profiling_enabled() {
                    self.profile_arg_materialization(materialized_args);
                }
                spine.write_args_head_order_prefix(scratch_args, FALLBACK_PRIM_ARG_PREFIX);
                self.fallback_runtime_prim_rewrite(name, scratch_args.as_slice(), args_len)?
            }
            _ => None,
        };

        let Some((mut used, mut node)) = rewrite else {
            if let Some(name) = fallback_name {
                if args_len != 0 && !is_supported_runtime_prim_name(name) {
                    return Err(EvalError::UnknownPrim(name.to_owned()));
                }
            }
            return Ok(None);
        };
        let mut reductions = 1;
        if matches!(known, Some(I | Ord | Chr)) {
            let mut alias_shortcuts = 0;
            while reductions < budget && used < args_len && self.is_identity_alias_node(node)? {
                node = arg!(used);
                used += 1;
                reductions += 1;
                alias_shortcuts += 1;
            }
            self.profile_shortcut("identity_alias_chain", alias_shortcuts);
        }
        let node = self.apply_eval_spine_rewrite(root, spine, used, node);
        Ok(Some(self.eval_loop_result(profile_head, node, reductions)))
    }
}
