//! General fallback reducer used when the stack fast path delegates.
use super::*;

impl Program {
    pub(in crate::runtime) fn eval_loop_result(
        &mut self,
        profile_head: ProfileHead,
        node: NodeId,
        reductions: usize,
    ) -> EvalLoopStep {
        if profile_head.is_some() {
            self.profile_reduction(profile_head, reductions);
        }
        EvalLoopStep { node, reductions }
    }

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

    pub(in crate::runtime) fn eval_loop_step(
        &mut self,
        root: NodeId,
        budget: usize,
        spine: &mut EvalSpine,
        scratch_args: &mut Vec<NodeId>,
        scratch_apps: &mut Vec<NodeId>,
        frame_stack: &mut EvalFrameStack,
        strict_markers: bool,
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
        macro_rules! strict_marker_step {
            ($used:expr, $variant:ident, $frame:ident, $kind:expr, $next:expr) => {{
                let redex = self.strict_redex_from_eval_spine(root, $used, spine, scratch_apps);
                if self.profiling_enabled() {
                    self.profile_eval_frame_push(stringify!($variant));
                }
                frame_stack.push(EvalFrame::$variant($frame {
                    redex,
                    profile_head,
                    kind: $kind,
                }));
                return Ok(Some(EvalLoopStep {
                    node: $next,
                    reductions: 0,
                }));
            }};
        }
        macro_rules! strict_int64_shift_marker_step {
            ($used:expr, $op:expr, $x:expr, $next:expr) => {{
                let redex = self.strict_redex_from_eval_spine(root, $used, spine, scratch_apps);
                if self.profiling_enabled() {
                    self.profile_eval_frame_push("Int64Shift");
                }
                frame_stack.push(EvalFrame::Int64Shift(Int64ShiftFrame {
                    redex,
                    profile_head,
                    op: $op,
                    x: $x,
                }));
                return Ok(Some(EvalLoopStep {
                    node: $next,
                    reductions: 0,
                }));
            }};
        }

        let head_dispatch = match self.cell(head).prim() {
            Some(Prim::Known(known)) => EvalHead::Known(known),
            Some(Prim::Runtime(runtime)) => {
                let name = runtime.name();
                let action = runtime.strict_action(args_len);
                let needs_fallback_name =
                    !strict_markers || matches!(action, StrictPrimitiveAction::None);
                EvalHead::Other {
                    action,
                    fallback_name: needs_fallback_name.then_some(name),
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

        let (known, strict_action, fallback_name) = match head_dispatch {
            EvalHead::Ffi(name) => {
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
                if self.profiling_enabled() {
                    self.profile_arg_materialization(args_len);
                }
                spine.write_args_head_order(scratch_args);
                let Some((used, node)) = self.js_wrap(&tags, scratch_args.as_slice())? else {
                    return Ok(None);
                };
                rewrite_step!(used, node, 1);
            }
            EvalHead::Known(known) => (Some(known), StrictPrimitiveAction::None, None),
            EvalHead::Other {
                action,
                fallback_name,
            } => (None, action, fallback_name),
            EvalHead::Whnf => return Ok(None),
        };
        use KnownPrim::*;

        if known == Some(U) && args_len >= 2 {
            if let Some(node) = self.selector_pair_field(arg!(0), arg!(1))? {
                self.profile_shortcut("selector_pair_field", 1);
                rewrite_step!(2, node, 1);
            }
        }

        if known == Some(IoThen) && args_len >= 3 && budget >= 2 {
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

        if strict_markers {
            match known {
                Some(IoStrict) if args_len >= 2 => {
                    strict_marker_step!(
                        2,
                        Whnf,
                        WhnfFrame,
                        WhnfFrameKind::IoStrict {
                            action: arg!(0),
                            value: arg!(1),
                        },
                        arg!(1)
                    );
                }
                Some(Seq) if args_len >= 2 => {
                    strict_marker_step!(
                        2,
                        Whnf,
                        WhnfFrame,
                        WhnfFrameKind::Seq { result: arg!(1) },
                        arg!(0)
                    );
                }
                Some(IsInt) if args_len >= 1 => {
                    strict_marker_step!(1, Whnf, WhnfFrame, WhnfFrameKind::IsInt, arg!(0));
                }
                _ => {}
            }
        }

        if strict_markers && known.is_none() {
            match strict_action {
                StrictPrimitiveAction::IntBin(op) => {
                    let x = arg!(0);
                    strict_marker_step!(
                        2,
                        Int,
                        IntFrame,
                        IntFrameKind::BinSecond { op, x },
                        arg!(1)
                    );
                }
                StrictPrimitiveAction::IntUn(op) => {
                    strict_marker_step!(1, Int, IntFrame, IntFrameKind::Un { op }, arg!(0));
                }
                StrictPrimitiveAction::Int64Bin(op) => {
                    if op.rhs_is_shift() {
                        strict_int64_shift_marker_step!(2, op, arg!(0), arg!(1));
                    } else if op.driver_marker_safe() {
                        strict_marker_step!(
                            2,
                            Int64,
                            Int64Frame,
                            Int64FrameKind::BinSecond { op, x: arg!(0) },
                            arg!(1)
                        );
                    }
                }
                StrictPrimitiveAction::Int64Un(op) => {
                    strict_marker_step!(1, Int64, Int64Frame, Int64FrameKind::Un { op }, arg!(0));
                }
                StrictPrimitiveAction::Float64Bin(op) => {
                    strict_marker_step!(
                        2,
                        Float64,
                        Float64Frame,
                        Float64FrameKind::BinSecond { op, x: arg!(0) },
                        arg!(1)
                    );
                }
                StrictPrimitiveAction::Float64Un(op) => {
                    strict_marker_step!(
                        1,
                        Float64,
                        Float64Frame,
                        Float64FrameKind::Un { op },
                        arg!(0)
                    );
                }
                StrictPrimitiveAction::Float32Bin(op) => {
                    strict_marker_step!(
                        2,
                        Float32,
                        Float32Frame,
                        Float32FrameKind::BinSecond { op, x: arg!(0) },
                        arg!(1)
                    );
                }
                StrictPrimitiveAction::Float32Un(op) => {
                    strict_marker_step!(
                        1,
                        Float32,
                        Float32Frame,
                        Float32FrameKind::Un { op },
                        arg!(0)
                    );
                }
                StrictPrimitiveAction::BytesBin(op) => {
                    strict_marker_step!(
                        2,
                        Bytes,
                        BytesFrame,
                        BytesFrameKind::BinSecond { op, x: arg!(0) },
                        arg!(1)
                    );
                }
                StrictPrimitiveAction::Conversion(kind) => {
                    strict_marker_step!(1, Conversion, ConversionFrame, kind, arg!(0));
                }
                StrictPrimitiveAction::None => {}
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
                let printed = self.print_program(value)?;
                self.write_bfile_bytes(ptr, &printed)?;
                let unit = self.prim("I");
                Some((3, self.pair(unit, arg!(2))))
            }
            Some(IoSerialize) if args_len >= 3 => {
                let ptr = self.eval_pointer_value(arg!(0))?;
                let value = self.reduce_node_whnf(arg!(1), FORCE_REDUCTION_LIMIT)?;
                let serialized = self.serialize_program(value)?;
                self.write_bfile_bytes(ptr, &serialized)?;
                let unit = self.prim("I");
                Some((3, self.pair(unit, arg!(2))))
            }
            Some(IoDeserialize) if args_len >= 2 => {
                let ptr = self.eval_pointer_value(arg!(0))?;
                let value = self.deserialize_bfile(ptr)?;
                Some((2, self.pair(value, arg!(1))))
            }
            Some(IoGetArgRef) if args_len >= 1 => {
                let arg_array = self.arg_ref_array();
                Some((1, self.pair(arg_array, arg!(0))))
            }
            Some(IoThid) if args_len >= 1 => {
                let thread = self.push_node(Node::ThreadId(1));
                Some((1, self.pair(thread, arg!(0))))
            }
            Some(IoYield) if args_len >= 1 => {
                let unit = self.prim("I");
                Some((1, self.pair(unit, arg!(0))))
            }
            Some(IoGetMaskingState) if args_len >= 1 => {
                let state = self.int(self.masking_state);
                Some((1, self.pair(state, arg!(0))))
            }
            Some(IoSetMaskingState) if args_len >= 2 => {
                self.masking_state = self.eval_int(arg!(0))?;
                let unit = self.prim("I");
                Some((2, self.pair(unit, arg!(1))))
            }
            Some(Dynsym) if args_len >= 1 => {
                let name = self.eval_ffi_name(arg!(0))?;
                Some((1, self.push_node(Node::ffi(name))))
            }
            Some(IoThreadStatus) if args_len >= 2 => {
                self.eval_thread_id(arg!(0))?;
                let status = self.int(0);
                Some((2, self.pair(status, arg!(1))))
            }
            Some(IoNewMVar) if args_len >= 1 => {
                let mvar = self.push_node(Node::MVar(None));
                Some((1, self.pair(mvar, arg!(0))))
            }
            Some(IoTakeMVar) if args_len >= 2 => {
                let mvar = self.eval_mvar_id(arg!(0))?;
                let value = self.take_mvar(mvar)?.ok_or(EvalError::InvalidMVar)?;
                Some((2, self.pair(value, arg!(1))))
            }
            Some(IoReadMVar) if args_len >= 2 => {
                let mvar = self.eval_mvar_id(arg!(0))?;
                let value = self.read_mvar(mvar)?.ok_or(EvalError::InvalidMVar)?;
                Some((2, self.pair(value, arg!(1))))
            }
            Some(IoPutMVar) if args_len >= 3 => {
                let mvar = self.eval_mvar_id(arg!(0))?;
                self.put_mvar(mvar, arg!(1))?;
                let unit = self.prim("I");
                Some((3, self.pair(unit, arg!(2))))
            }
            Some(IoTryTakeMVar) if args_len >= 2 => {
                let mvar = self.eval_mvar_id(arg!(0))?;
                let value = match self.take_mvar(mvar)? {
                    Some(value) => self.just(value),
                    None => self.nothing(),
                };
                Some((2, self.pair(value, arg!(1))))
            }
            Some(IoTryReadMVar) if args_len >= 2 => {
                let mvar = self.eval_mvar_id(arg!(0))?;
                let value = match self.read_mvar(mvar)? {
                    Some(value) => self.just(value),
                    None => self.nothing(),
                };
                Some((2, self.pair(value, arg!(1))))
            }
            Some(IoTryPutMVar) if args_len >= 3 => {
                let mvar = self.eval_mvar_id(arg!(0))?;
                let value = if self.try_put_mvar(mvar, arg!(1))? {
                    self.prim("A")
                } else {
                    self.prim("K")
                };
                Some((3, self.pair(value, arg!(2))))
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
