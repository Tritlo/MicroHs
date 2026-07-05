//! Fast explicit-stack reducer for hot combinator evaluation.
use super::*;

impl Program {
    pub(in crate::runtime) fn stack_eval_step(
        &mut self,
        mut head: NodeId,
        stack: &mut EvalStack,
        scratch_args: &mut Vec<NodeId>,
        budget: usize,
        profile_resolve: bool,
    ) -> Result<StackStep, EvalError> {
        let mut carried_reductions = 0;
        'eval: loop {
            let profiling = self.profile.is_some();
            if stack.app_len() == 0 {
                if let Some((next, reductions)) = self.finish_ready_stack_frame(stack, head)? {
                    carried_reductions += reductions;
                    if carried_reductions >= budget {
                        return Ok(StackStep::Reduced {
                            node: next,
                            reductions: carried_reductions,
                        });
                    }
                    #[cfg(feature = "eval-phase-profile")]
                    let started = profiling.then(Instant::now);
                    head = self.descend_stack_from(next, stack, profile_resolve, profiling)?;
                    #[cfg(feature = "eval-phase-profile")]
                    if let Some(started) = started {
                        self.profile_stack_inner_descent_time(started.elapsed().as_nanos());
                    }
                    continue 'eval;
                }
            }

            let args_len = stack.app_len();
            let profile_head = if profiling {
                self.profile_step(head, args_len)
            } else {
                None
            };

            macro_rules! arg {
                ($idx:expr) => {{
                    if profiling {
                        self.profile_stack_arg_reads(1);
                    }
                    #[cfg(feature = "eval-phase-profile")]
                    let started = profiling.then(Instant::now);
                    let arg = stack.arg(&self.nodes, $idx);
                    #[cfg(feature = "eval-phase-profile")]
                    if let Some(started) = started {
                        let nanos = started.elapsed().as_nanos();
                        self.profile_stack_arg_read_time(nanos);
                    }
                    arg
                }};
            }
            macro_rules! take_args {
                ($reads:expr, $method:ident) => {{
                    if profiling {
                        self.profile_stack_arg_batch();
                        self.profile_stack_arg_reads($reads);
                    }
                    #[cfg(feature = "eval-phase-profile")]
                    let started = profiling.then(Instant::now);
                    let args = stack.$method(&self.nodes);
                    #[cfg(feature = "eval-phase-profile")]
                    if let Some(started) = started {
                        let nanos = started.elapsed().as_nanos();
                        self.profile_stack_arg_read_time(nanos);
                    }
                    args
                }};
            }
            macro_rules! app_site {
                ($key:literal, $fun:expr, $arg:expr) => {{
                    let fun = $fun;
                    let arg = $arg;
                    self.app_with_site($key, fun, arg)
                }};
            }
            macro_rules! finish_reduction {
                ($node:expr, $reductions:expr) => {{
                    let reductions = carried_reductions + $reductions;
                    if profile_head.is_some() {
                        self.profile_reduction(profile_head, $reductions);
                    }
                    return Ok(StackStep::Reduced {
                        node: $node,
                        reductions,
                    });
                }};
            }
            macro_rules! rewrite_step {
                ($used:expr, $node:expr, $reductions:expr) => {{
                    #[cfg(feature = "eval-phase-profile")]
                    let started = profiling.then(Instant::now);
                    let node = self.apply_stack_rewrite(stack, $used, $node);
                    #[cfg(feature = "eval-phase-profile")]
                    if let Some(started) = started {
                        self.profile_stack_apply_rewrite_time(started.elapsed().as_nanos());
                    }
                    finish_reduction!(node, $reductions);
                }};
            }
            macro_rules! rewrite_continue_reductions {
                ($used:expr, $node:expr, $reductions:expr) => {{
                    #[cfg(feature = "eval-phase-profile")]
                    let started = profiling.then(Instant::now);
                    let node = self.apply_stack_rewrite(stack, $used, $node);
                    #[cfg(feature = "eval-phase-profile")]
                    if let Some(started) = started {
                        self.profile_stack_apply_rewrite_time(started.elapsed().as_nanos());
                    }
                    if profile_head.is_some() {
                        self.profile_reduction(profile_head, $reductions);
                    }
                    carried_reductions += $reductions;
                    if carried_reductions >= budget {
                        return Ok(StackStep::Reduced {
                            node,
                            reductions: carried_reductions,
                        });
                    }
                    continue_with!(node);
                }};
            }
            macro_rules! continue_with {
                ($node:expr) => {{
                    #[cfg(feature = "eval-phase-profile")]
                    let started = profiling.then(Instant::now);
                    head = self.descend_stack_from($node, stack, profile_resolve, profiling)?;
                    #[cfg(feature = "eval-phase-profile")]
                    if let Some(started) = started {
                        let nanos = started.elapsed().as_nanos();
                        self.profile_stack_inner_descent_time(nanos);
                    }
                    #[cfg(feature = "eval-phase-profile")]
                    if profiling {
                        self.profile_stack_continue_next_head(profile_head, head, stack.app_len());
                    }
                    continue 'eval;
                }};
            }
            macro_rules! goind_taken {
                ($redex:expr, $used:expr, $node:expr, $reductions:expr) => {{
                    #[cfg(feature = "eval-phase-profile")]
                    let started = profiling.then(Instant::now);
                    let redex = $redex;
                    let node = $node;
                    let wrote_indirection = node != redex;
                    if wrote_indirection {
                        self.set_app_cell_at(redex.index(), Cell::indir(Some(node)));
                    }
                    if profiling {
                        self.profile_stack_rewrite($used, wrote_indirection);
                    }
                    #[cfg(feature = "eval-phase-profile")]
                    if let Some(started) = started {
                        let nanos = started.elapsed().as_nanos();
                        self.profile_stack_apply_rewrite_time(nanos);
                    }
                    if profile_head.is_some() {
                        self.profile_reduction(profile_head, $reductions);
                    }
                    carried_reductions += $reductions;
                    if carried_reductions >= budget {
                        return Ok(StackStep::Reduced {
                            node,
                            reductions: carried_reductions,
                        });
                    }
                    continue_with!(node);
                }};
            }
            macro_rules! app_step_reductions {
                ($used:expr, $fun:expr, $arg:expr, $reductions:expr) => {{
                    let fun = $fun;
                    let arg = $arg;
                    #[cfg(feature = "eval-phase-profile")]
                    let update_started = profiling.then(Instant::now);
                    let node = self.apply_stack_app(stack, $used, fun, arg);
                    #[cfg(feature = "eval-phase-profile")]
                    if let Some(started) = update_started {
                        self.profile_stack_apply_app_time(started.elapsed().as_nanos());
                    }
                    if profile_head.is_some() {
                        self.profile_reduction(profile_head, $reductions);
                    }
                    carried_reductions += $reductions;
                    if carried_reductions >= budget {
                        return Ok(StackStep::Reduced {
                            node,
                            reductions: carried_reductions,
                        });
                    }
                    if profiling {
                        self.profile_stack_descent_push();
                    }
                    stack.push_app(node);
                    #[cfg(feature = "eval-phase-profile")]
                    let started = profiling.then(Instant::now);
                    head = self.descend_stack_from(fun, stack, profile_resolve, profiling)?;
                    #[cfg(feature = "eval-phase-profile")]
                    if let Some(started) = started {
                        let nanos = started.elapsed().as_nanos();
                        self.profile_stack_inner_descent_time(nanos);
                    }
                    #[cfg(feature = "eval-phase-profile")]
                    if profiling {
                        self.profile_stack_continue_next_head(profile_head, head, stack.app_len());
                    }
                    continue 'eval;
                }};
            }
            macro_rules! app_taken_reductions {
                ($redex:expr, $used:expr, $fun:expr, $arg:expr, $reductions:expr) => {{
                    let redex = $redex;
                    let fun = $fun;
                    let arg = $arg;
                    #[cfg(feature = "eval-phase-profile")]
                    let started = profiling.then(Instant::now);
                    if profiling {
                        self.profile_stack_app_update($used);
                    }
                    self.set_app_cell_at(redex.index(), Cell::app(fun, arg));
                    #[cfg(feature = "eval-phase-profile")]
                    if let Some(started) = started {
                        let nanos = started.elapsed().as_nanos();
                        self.profile_stack_apply_app_time(nanos);
                    }
                    if profile_head.is_some() {
                        self.profile_reduction(profile_head, $reductions);
                    }
                    carried_reductions += $reductions;
                    if carried_reductions >= budget {
                        return Ok(StackStep::Reduced {
                            node: redex,
                            reductions: carried_reductions,
                        });
                    }
                    if profiling {
                        self.profile_stack_descent_push();
                    }
                    stack.push_app(redex);
                    #[cfg(feature = "eval-phase-profile")]
                    let started = profiling.then(Instant::now);
                    head = self.descend_stack_from(fun, stack, profile_resolve, profiling)?;
                    #[cfg(feature = "eval-phase-profile")]
                    if let Some(started) = started {
                        let nanos = started.elapsed().as_nanos();
                        self.profile_stack_inner_descent_time(nanos);
                    }
                    #[cfg(feature = "eval-phase-profile")]
                    if profiling {
                        self.profile_stack_continue_next_head(profile_head, head, stack.app_len());
                    }
                    continue 'eval;
                }};
            }
            macro_rules! app_step {
                ($used:expr, $fun:expr, $arg:expr) => {{
                    app_step_reductions!($used, $fun, $arg, 1);
                }};
            }
            macro_rules! app_taken {
                ($redex:expr, $used:expr, $fun:expr, $arg:expr) => {{
                    app_taken_reductions!($redex, $used, $fun, $arg, 1);
                }};
            }
            macro_rules! force_step {
                ($used:expr, $push:ident, $kind:expr, $next:expr, $profile_kind:literal) => {{
                    let app_end = stack.apps.len();
                    let kind = $kind;
                    let next = $next;
                    if profiling {
                        self.profile_persistent_force();
                        self.profile_eval_frame_push($profile_kind);
                    }
                    #[cfg(feature = "eval-phase-profile")]
                    let started = profiling.then(Instant::now);
                    stack.$push(app_end, $used, profile_head, kind);
                    #[cfg(feature = "eval-phase-profile")]
                    if let Some(started) = started {
                        let nanos = started.elapsed().as_nanos();
                        self.profile_stack_force_frame_time(nanos);
                    }
                    continue_with!(next);
                }};
            }

            let head_dispatch = match self.cell_trusted(head).prim() {
                Some(Prim::Known(known)) => EvalHead::Known(known),
                Some(Prim::Runtime(runtime)) => {
                    let action = runtime.strict_action(args_len);
                    EvalHead::Other {
                        action,
                        fallback_name: matches!(action, StrictPrimitiveAction::None)
                            .then_some(runtime.name()),
                    }
                }
                None if args_len > 0 => match self.cold_node(head) {
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
                None => EvalHead::Whnf,
            };

            let known = match head_dispatch {
                EvalHead::Ffi(name) => {
                    if self.profile.is_some() {
                        self.profile_arg_materialization(args_len);
                    }
                    stack.write_args_head_order(&self.nodes, scratch_args)?;
                    let Some((used, node)) = self.ffi_call(&name, scratch_args.as_slice())? else {
                        return Ok(StackStep::Whnf {
                            node: stack.outer_root(head),
                            head,
                            reductions: carried_reductions,
                        });
                    };
                    rewrite_step!(used, node, 1);
                }
                EvalHead::JsCall { tags, body } => {
                    if self.profile.is_some() {
                        self.profile_arg_materialization(args_len);
                    }
                    stack.write_args_head_order(&self.nodes, scratch_args)?;
                    let Some((used, node)) = self.js_call(&tags, &body, scratch_args.as_slice())?
                    else {
                        return Ok(StackStep::Whnf {
                            node: stack.outer_root(head),
                            head,
                            reductions: carried_reductions,
                        });
                    };
                    rewrite_step!(used, node, 1);
                }
                EvalHead::JsWrap { tags } => {
                    if self.profile.is_some() {
                        self.profile_arg_materialization(args_len);
                    }
                    stack.write_args_head_order(&self.nodes, scratch_args)?;
                    let Some((used, node)) = self.js_wrap(&tags, scratch_args.as_slice())? else {
                        return Ok(StackStep::Whnf {
                            node: stack.outer_root(head),
                            head,
                            reductions: carried_reductions,
                        });
                    };
                    rewrite_step!(used, node, 1);
                }
                EvalHead::Known(known) => known,
                EvalHead::Other {
                    action,
                    fallback_name,
                } => {
                    match action {
                        StrictPrimitiveAction::IntBin(op) => {
                            let (redex, x, y) = take_args!(2, take_args2);
                            let y_immediate = self.cell_int_value(y);
                            if let Some(y_value) = y_immediate {
                                if let Some(x_value) = self.cell_int_value(x) {
                                    let result = op
                                        .apply(x_value, y_value)
                                        .map_err(|err| self.arithmetic_eval_error(err))?;
                                    let node = self.apply_stack_redex_value(
                                        redex,
                                        2,
                                        Self::int_result_value_node(result),
                                    );
                                    finish_reduction!(node, 1);
                                }
                                if profiling {
                                    self.profile_persistent_force();
                                    self.profile_eval_frame_push("Int");
                                }
                                #[cfg(feature = "eval-phase-profile")]
                                let started = profiling.then(Instant::now);
                                stack.push_int_frame(
                                    redex,
                                    profile_head,
                                    IntFrameKind::BinFirst { op, y: y_value },
                                );
                                #[cfg(feature = "eval-phase-profile")]
                                if let Some(started) = started {
                                    let nanos = started.elapsed().as_nanos();
                                    self.profile_stack_force_frame_time(nanos);
                                }
                                continue_with!(x);
                            }
                            if profiling {
                                self.profile_persistent_force();
                                self.profile_eval_frame_push("Int");
                            }
                            #[cfg(feature = "eval-phase-profile")]
                            let started = profiling.then(Instant::now);
                            stack.push_int_frame(
                                redex,
                                profile_head,
                                IntFrameKind::BinSecond { op, x },
                            );
                            #[cfg(feature = "eval-phase-profile")]
                            if let Some(started) = started {
                                let nanos = started.elapsed().as_nanos();
                                self.profile_stack_force_frame_time(nanos);
                            }
                            continue_with!(y);
                        }
                        StrictPrimitiveAction::IntUn(op) => {
                            let (redex, x) = take_args!(1, take_args1);
                            if profiling {
                                self.profile_persistent_force();
                                self.profile_eval_frame_push("Int");
                            }
                            #[cfg(feature = "eval-phase-profile")]
                            let started = profiling.then(Instant::now);
                            stack.push_int_frame(redex, profile_head, IntFrameKind::Un { op });
                            #[cfg(feature = "eval-phase-profile")]
                            if let Some(started) = started {
                                let nanos = started.elapsed().as_nanos();
                                self.profile_stack_force_frame_time(nanos);
                            }
                            continue_with!(x);
                        }
                        StrictPrimitiveAction::Int64Bin(op) => {
                            if op.rhs_is_shift() {
                                let app_end = stack.apps.len();
                                let x = arg!(0);
                                let next = arg!(1);
                                if profiling {
                                    self.profile_persistent_force();
                                    self.profile_eval_frame_push("Int64Shift");
                                }
                                #[cfg(feature = "eval-phase-profile")]
                                let started = profiling.then(Instant::now);
                                stack.push_int64_shift_frame(app_end, 2, profile_head, op, x);
                                #[cfg(feature = "eval-phase-profile")]
                                if let Some(started) = started {
                                    let nanos = started.elapsed().as_nanos();
                                    self.profile_stack_force_frame_time(nanos);
                                }
                                continue_with!(next);
                            } else if op.driver_marker_safe() {
                                force_step!(
                                    2,
                                    push_int64_frame,
                                    Int64FrameKind::BinSecond { op, x: arg!(0) },
                                    arg!(1),
                                    "Int64"
                                );
                            }
                        }
                        StrictPrimitiveAction::Int64Un(op) => {
                            force_step!(
                                1,
                                push_int64_frame,
                                Int64FrameKind::Un { op },
                                arg!(0),
                                "Int64"
                            );
                        }
                        StrictPrimitiveAction::Float64Bin(op) => {
                            force_step!(
                                2,
                                push_float64_frame,
                                Float64FrameKind::BinSecond { op, x: arg!(0) },
                                arg!(1),
                                "Float64"
                            );
                        }
                        StrictPrimitiveAction::Float64Un(op) => {
                            force_step!(
                                1,
                                push_float64_frame,
                                Float64FrameKind::Un { op },
                                arg!(0),
                                "Float64"
                            );
                        }
                        StrictPrimitiveAction::Float32Bin(op) => {
                            force_step!(
                                2,
                                push_float32_frame,
                                Float32FrameKind::BinSecond { op, x: arg!(0) },
                                arg!(1),
                                "Float32"
                            );
                        }
                        StrictPrimitiveAction::Float32Un(op) => {
                            force_step!(
                                1,
                                push_float32_frame,
                                Float32FrameKind::Un { op },
                                arg!(0),
                                "Float32"
                            );
                        }
                        StrictPrimitiveAction::BytesBin(op) => {
                            force_step!(
                                2,
                                push_bytes_frame,
                                BytesFrameKind::BinSecond { op, x: arg!(0) },
                                arg!(1),
                                "Bytes"
                            );
                        }
                        StrictPrimitiveAction::Conversion(kind) => {
                            force_step!(1, push_conversion_frame, kind, arg!(0), "Conversion");
                        }
                        StrictPrimitiveAction::None => {}
                    }
                    if let Some(name) = fallback_name {
                        let materialized_args = args_len.min(FALLBACK_PRIM_ARG_PREFIX);
                        if self.profile.is_some() {
                            self.profile_arg_materialization(materialized_args);
                        }
                        stack.write_args_head_order_prefix(
                            &self.nodes,
                            scratch_args,
                            FALLBACK_PRIM_ARG_PREFIX,
                        )?;
                        if let Some((used, node)) = self.fallback_runtime_prim_rewrite(
                            name,
                            scratch_args.as_slice(),
                            args_len,
                        )? {
                            rewrite_step!(used, node, 1);
                        }
                        if args_len != 0 && !is_supported_runtime_prim_name(name) {
                            return Err(EvalError::UnknownPrim(name.to_owned()));
                        }
                    }
                    return Ok(StackStep::Whnf {
                        node: stack.outer_root(head),
                        head,
                        reductions: carried_reductions,
                    });
                }
                EvalHead::Whnf => {
                    return Ok(StackStep::Whnf {
                        node: stack.outer_root(head),
                        head,
                        reductions: carried_reductions,
                    });
                }
            };
            use KnownPrim::*;

            match known {
                IoStrict if args_len >= 2 => {
                    let (redex, action, value) = take_args!(2, take_args2);
                    if profiling {
                        self.profile_persistent_force();
                        self.profile_eval_frame_push("Whnf");
                    }
                    #[cfg(feature = "eval-phase-profile")]
                    let started = profiling.then(Instant::now);
                    stack.push_whnf_frame(
                        redex,
                        2,
                        profile_head,
                        WhnfFrameKind::IoStrict { action, value },
                    );
                    #[cfg(feature = "eval-phase-profile")]
                    if let Some(started) = started {
                        let nanos = started.elapsed().as_nanos();
                        self.profile_stack_force_frame_time(nanos);
                    }
                    continue_with!(value);
                }
                Seq if args_len >= 2 => {
                    let (redex, x, result) = take_args!(2, take_args2);
                    if profiling {
                        self.profile_persistent_force();
                        self.profile_eval_frame_push("Whnf");
                    }
                    #[cfg(feature = "eval-phase-profile")]
                    let started = profiling.then(Instant::now);
                    stack.push_whnf_frame(redex, 2, profile_head, WhnfFrameKind::Seq { result });
                    #[cfg(feature = "eval-phase-profile")]
                    if let Some(started) = started {
                        let nanos = started.elapsed().as_nanos();
                        self.profile_stack_force_frame_time(nanos);
                    }
                    continue_with!(x);
                }
                IsInt if args_len >= 1 => {
                    let (redex, x) = take_args!(1, take_args1);
                    if profiling {
                        self.profile_persistent_force();
                        self.profile_eval_frame_push("Whnf");
                    }
                    #[cfg(feature = "eval-phase-profile")]
                    let started = profiling.then(Instant::now);
                    stack.push_whnf_frame(redex, 1, profile_head, WhnfFrameKind::IsInt);
                    #[cfg(feature = "eval-phase-profile")]
                    if let Some(started) = started {
                        let nanos = started.elapsed().as_nanos();
                        self.profile_stack_force_frame_time(nanos);
                    }
                    continue_with!(x);
                }
                _ => {}
            }

            match known {
                IoPerformIo if args_len >= 1 => {
                    let (redex, io) = take_args!(1, take_args1);
                    let world = self.world();
                    let k = self.prim("K");
                    let action = app_site!("IO.performIO.action", io, world);
                    app_taken!(redex, 1, action, k);
                }
                IoBind if args_len >= 3 => {
                    let (redex, io, k, world) = take_args!(3, take_args3);
                    let action = app_site!("IO.bind.action", io, world);
                    app_taken!(redex, 3, action, k);
                }
                IoThen if args_len >= 3 && budget >= 2 => {
                    let (redex, io, y, world) = take_args!(3, take_args3);
                    let k = self.prim("K");
                    let then = app_site!("IO.then.k", k, y);
                    let action = app_site!("IO.then.action", io, world);
                    app_taken_reductions!(redex, 3, action, then, 2);
                }
                IoThen if args_len >= 2 => {
                    let (redex, io, y) = take_args!(2, take_args2);
                    let bind = self.prim("IO.>>=");
                    let bind_action = app_site!("IO.then.bind_action", bind, io);
                    let k = self.prim("K");
                    let then = app_site!("IO.then.k", k, y);
                    app_taken!(redex, 2, bind_action, then);
                }
                IoReturn if args_len >= 3 => {
                    let (redex, x, world, k) = take_args!(3, take_args3);
                    let kx = app_site!("IO.return.kx", k, x);
                    app_taken!(redex, 3, kx, world);
                }
                I | Ord | Chr if args_len >= 1 => {
                    let mut used = 1;
                    let mut reductions = 1;
                    let mut node = arg!(0);
                    let mut alias_shortcuts = 0;
                    while reductions < budget
                        && used < args_len
                        && self.is_identity_alias_node(node)?
                    {
                        node = arg!(used);
                        used += 1;
                        reductions += 1;
                        alias_shortcuts += 1;
                    }
                    self.profile_shortcut("identity_alias_chain", alias_shortcuts);
                    rewrite_continue_reductions!(used, node, reductions);
                }
                K if args_len >= 2 => {
                    let (redex, x, _) = take_args!(2, take_args2);
                    goind_taken!(redex, 2, x, 1);
                }
                A if args_len >= 2 => {
                    let (redex, _, y) = take_args!(2, take_args2);
                    goind_taken!(redex, 2, y, 1);
                }
                U if args_len >= 2 => {
                    let (redex, x, y) = take_args!(2, take_args2);
                    app_taken!(redex, 2, y, x);
                }
                S if args_len >= 3 => {
                    let (redex, x, y, z) = take_args!(3, take_args3);
                    if carried_reductions + 1 < budget
                        && z != redex
                        && matches!(self.cell_trusted(x).prim(), Some(Prim::Known(I)))
                    {
                        let right = app_site!("S.right", y, z);
                        app_taken_reductions!(redex, 3, z, right, 2);
                    }
                    let left = app_site!("S.left", x, z);
                    let right = app_site!("S.right", y, z);
                    app_taken!(redex, 3, left, right);
                }
                SPrime if args_len >= 4 => {
                    let (redex, x, y, z, w) = take_args!(4, take_args4);
                    let yw = app_site!("S'.yw", y, w);
                    let zw = app_site!("S'.zw", z, w);
                    let left = app_site!("S'.left", x, yw);
                    app_taken!(redex, 4, left, zw);
                }
                B if args_len >= 3 => {
                    let (redex, x, y, z) = take_args!(3, take_args3);
                    let yz = app_site!("B.yz", y, z);
                    app_taken!(redex, 3, x, yz);
                }
                BPrime if args_len >= 4 => {
                    let (redex, x, y, z, w) = take_args!(4, take_args4);
                    let zw = app_site!("B'.zw", z, w);
                    let xy = app_site!("B'.xy", x, y);
                    app_taken!(redex, 4, xy, zw);
                }
                BPrime if args_len >= 2 => {
                    let (redex, x, y) = take_args!(2, take_args2);
                    let xy = app_site!("B'.xy_under", x, y);
                    let b = self.prim("B");
                    app_taken!(redex, 2, b, xy);
                }
                Z if args_len >= 3 => {
                    let (redex, x, y, _) = take_args!(3, take_args3);
                    app_taken!(redex, 3, x, y);
                }
                Z if args_len >= 2 => {
                    let (redex, x, y) = take_args!(2, take_args2);
                    let xy = app_site!("Z.xy_under", x, y);
                    let k = self.prim("K");
                    app_taken!(redex, 2, k, xy);
                }
                J if args_len >= 3 => {
                    let (redex, x, _, z) = take_args!(3, take_args3);
                    app_taken!(redex, 3, z, x);
                }
                L if args_len >= 3 => {
                    let (redex, x, y, _) = take_args!(3, take_args3);
                    app_taken!(redex, 3, y, x);
                }
                KK if args_len >= 3 => {
                    let (redex, _, y, _) = take_args!(3, take_args3);
                    goind_taken!(redex, 3, y, 1);
                }
                KA if args_len >= 3 => {
                    let (redex, _, _, z) = take_args!(3, take_args3);
                    goind_taken!(redex, 3, z, 1);
                }
                C if args_len >= 3 => {
                    let (redex, x, y, z) = take_args!(3, take_args3);
                    let xz = app_site!("C.xz", x, z);
                    app_taken!(redex, 3, xz, y);
                }
                CPrime if args_len >= 4 => {
                    let (redex, x, y, z, w) = take_args!(4, take_args4);
                    let yw = app_site!("C'.yw", y, w);
                    let xyw = app_site!("C'.xyw", x, yw);
                    app_taken!(redex, 4, xyw, z);
                }
                P if args_len >= 3 => {
                    let (redex, x, y, z) = take_args!(3, take_args3);
                    let zx = app_site!("P.zx", z, x);
                    app_taken!(redex, 3, zx, y);
                }
                R if args_len >= 3 => {
                    let (redex, x, y, z) = take_args!(3, take_args3);
                    let yz = app_site!("R.yz", y, z);
                    app_taken!(redex, 3, yz, x);
                }
                R if args_len >= 2 => {
                    let (redex, x, y) = take_args!(2, take_args2);
                    let c = self.prim("C");
                    let cy = app_site!("R.cy_under", c, y);
                    app_taken!(redex, 2, cy, x);
                }
                O if args_len >= 4 => {
                    let (redex, x, y, _, w) = take_args!(4, take_args4);
                    let wx = app_site!("O.wx", w, x);
                    app_taken!(redex, 4, wx, y);
                }
                K2 if args_len >= 3 => {
                    let (redex, x, _, _) = take_args!(3, take_args3);
                    goind_taken!(redex, 3, x, 1);
                }
                K2 if args_len >= 2 => {
                    let (redex, x, _) = take_args!(2, take_args2);
                    let k = self.prim("K");
                    app_taken!(redex, 2, k, x);
                }
                K3 if args_len >= 4 => {
                    let (redex, x, _, _, _) = take_args!(4, take_args4);
                    goind_taken!(redex, 4, x, 1);
                }
                K3 if args_len >= 2 => {
                    let (redex, x, _) = take_args!(2, take_args2);
                    let k2 = self.prim("K2");
                    app_taken!(redex, 2, k2, x);
                }
                K4 if args_len >= 5 => {
                    let (redex, x, _, _, _, _) = take_args!(5, take_args5);
                    goind_taken!(redex, 5, x, 1);
                }
                K4 if args_len >= 2 => {
                    let (redex, x, _) = take_args!(2, take_args2);
                    let k3 = self.prim("K3");
                    app_taken!(redex, 2, k3, x);
                }
                CPrimeB if args_len >= 4 => {
                    let (redex, x, y, z, w) = take_args!(4, take_args4);
                    let yw = app_site!("C'B.yw", y, w);
                    let xz = app_site!("C'B.xz", x, z);
                    app_taken!(redex, 4, xz, yw);
                }
                CPrimeB if args_len >= 3 => {
                    let (redex, x, y, z) = take_args!(3, take_args3);
                    let xz = app_site!("C'B.xz_under", x, z);
                    let b = self.prim("B");
                    let bxz = app_site!("C'B.bxz_under", b, xz);
                    app_taken!(redex, 3, bxz, y);
                }
                Y if args_len >= 1 => {
                    let (redex, x) = take_args!(1, take_args1);
                    app_taken!(redex, 1, x, redex);
                }
                Tag(tag) if args_len >= 2 => {
                    let (redex, x, y) = take_args!(2, take_args2);
                    let tag = self.int(i64::from(tag));
                    let ytag = app_site!("Tag.ytag", y, tag);
                    app_taken!(redex, 2, ytag, x);
                }
                Tuple(fields) if args_len > usize::from(fields) => {
                    let fields = usize::from(fields);
                    let mut n = arg!(fields);
                    for idx in 0..fields - 1 {
                        let arg = arg!(idx);
                        n = app_site!("Tuple.prefix", n, arg);
                    }
                    let last = arg!(fields - 1);
                    app_step!(fields + 1, n, last);
                }
                I | Ord | Chr | K | A | U | S | SPrime | B | BPrime | Z | J | L | KK | KA | C
                | CPrime | P | R | O | K2 | K3 | K4 | CPrimeB | Y | Tag(_) | Tuple(_)
                | IoPerformIo | IoBind | IoThen | IoReturn => {
                    return Ok(StackStep::Whnf {
                        node: stack.outer_root(head),
                        head,
                        reductions: carried_reductions,
                    });
                }
                _ => {
                    self.profile_stack_fallback_head(head);
                    return Ok(StackStep::Fallback {
                        root: stack.outer_root(head),
                        head,
                        reductions: carried_reductions,
                    });
                }
            }
        }
    }
}
