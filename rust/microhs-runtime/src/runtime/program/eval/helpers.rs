impl Program {
    fn ignored_io_action_reductions(
        &mut self,
        action: NodeId,
        budget: usize,
    ) -> Result<Option<usize>, EvalError> {
        self.io_action_reductions(action, budget, IGNORED_IO_SHORTCUT_RECURSION_LIMIT)
    }

    fn io_action_reductions(
        &mut self,
        action: NodeId,
        budget: usize,
        depth: usize,
    ) -> Result<Option<usize>, EvalError> {
        if budget == 0 || depth == 0 {
            return Ok(None);
        }

        let spine = self.spine(action)?;
        let head = spine.head;
        let args = spine.args();
        use KnownPrim::*;
        match self.cell(head).prim() {
            Some(Prim::Known(IoReturn)) if args.len() == 1 => Ok(Some(1)),
            Some(Prim::Known(IoThen)) if args.len() == 2 && budget >= 2 => {
                let Some(right_reductions) =
                    self.io_action_reductions(args[1], budget - 1, depth - 1)?
                else {
                    return Ok(None);
                };
                let remaining_budget = budget.saturating_sub(right_reductions + 1);
                if remaining_budget == 0 {
                    return Ok(None);
                }
                let Some(left_reductions) =
                    self.io_action_reductions(args[0], remaining_budget, depth - 1)?
                else {
                    return Ok(None);
                };
                Ok(Some(left_reductions + right_reductions + 1))
            }
            Some(Prim::Known(IoLazyBind))
                if args.len() == 2
                    && budget >= 2
                    && self.direct_ffi_continuation_accepts_result(args[1])? =>
            {
                let Some(action_reductions) =
                    self.io_action_reductions(args[0], budget - 1, depth - 1)?
                else {
                    return Ok(None);
                };
                let remaining_budget = budget.saturating_sub(action_reductions + 1);
                if remaining_budget == 0 {
                    return Ok(None);
                }
                Ok(Some(action_reductions + 2))
            }
            Some(Prim::Known(IoGetArgRef | IoGetMaskingState | IoYield)) if args.is_empty() => {
                Ok(Some(1))
            }
            Some(Prim::Known(IoSetMaskingState)) if args.len() == 1 => Ok(Some(1)),
            _ if matches!(self.cold_node(head), Some(Node::Ffi(_))) => {
                let Some(Node::Ffi(name)) = self.cold_node(head).cloned() else {
                    unreachable!();
                };
                let arity =
                    ffi_arity(&name).ok_or_else(|| EvalError::UnknownFfi(name.to_string()))?;
                Ok((args.len() == arity).then_some(1))
            }
            _ => Ok(None),
        }
    }

    fn run_ignored_io_action(
        &mut self,
        action: NodeId,
        world: NodeId,
    ) -> Result<Option<NodeId>, EvalError> {
        Ok(self
            .run_io_action(action, world, IGNORED_IO_SHORTCUT_RECURSION_LIMIT)?
            .map(|(_, world)| world))
    }

    fn io_return_action_result(&mut self, action: NodeId) -> Result<Option<NodeId>, EvalError> {
        let action = self.resolve_profiled(action)?;
        let Some((fun, result)) = self.cell(action).app_fields() else {
            return Ok(None);
        };
        let fun = self.resolve_profiled(fun)?;
        Ok(match self.cell(fun).prim() {
            Some(Prim::Known(KnownPrim::IoReturn)) => Some(result),
            _ => None,
        })
    }

    fn run_io_action(
        &mut self,
        action: NodeId,
        world: NodeId,
        depth: usize,
    ) -> Result<Option<(NodeId, NodeId)>, EvalError> {
        if depth == 0 {
            return Ok(None);
        }

        let spine = self.spine(action)?;
        let head = spine.head;
        let args = spine.args();
        use KnownPrim::*;
        match self.cell(head).prim() {
            Some(Prim::Known(IoReturn)) if args.len() == 1 => Ok(Some((args[0], world))),
            Some(Prim::Known(IoThen)) if args.len() == 2 => {
                let Some((_, world)) = self.run_io_action(args[0], world, depth - 1)? else {
                    return Ok(None);
                };
                self.run_io_action(args[1], world, depth - 1)
            }
            Some(Prim::Known(IoLazyBind))
                if args.len() == 2 && self.direct_ffi_continuation_accepts_result(args[1])? =>
            {
                let Some((result, world)) = self.run_io_action(args[0], world, depth - 1)? else {
                    return Ok(None);
                };
                let next = self.app(args[1], result);
                self.run_io_action(next, world, depth - 1)
            }
            Some(Prim::Known(IoGetArgRef)) if args.is_empty() => {
                let result = self.arg_ref_array();
                Ok(Some((result, world)))
            }
            Some(Prim::Known(IoGetMaskingState)) if args.is_empty() => {
                let result = self.int(self.masking_state);
                Ok(Some((result, world)))
            }
            Some(Prim::Known(IoSetMaskingState)) if args.len() == 1 => {
                self.masking_state = self.eval_int(args[0])?;
                let result = self.prim("I");
                Ok(Some((result, world)))
            }
            Some(Prim::Known(IoYield)) if args.is_empty() => {
                let result = self.prim("I");
                Ok(Some((result, world)))
            }
            _ if matches!(self.cold_node(head), Some(Node::Ffi(_))) => {
                let Some(Node::Ffi(name)) = self.cold_node(head).cloned() else {
                    unreachable!();
                };
                let arity =
                    ffi_arity(&name).ok_or_else(|| EvalError::UnknownFfi(name.to_string()))?;
                if args.len() != arity {
                    return Ok(None);
                }
                let mut ffi_args = Vec::with_capacity(args.len() + 1);
                ffi_args.extend_from_slice(args);
                ffi_args.push(world);
                let Some((_, pair)) = self.ffi_call(&name, &ffi_args)? else {
                    return Ok(None);
                };
                self.pair_fields(pair)
            }
            _ => Ok(None),
        }
    }

    fn direct_ffi_continuation_accepts_result(&mut self, cont: NodeId) -> Result<bool, EvalError> {
        let cont = self.resolve_profiled(cont)?;
        let Some(Node::Ffi(name)) = self.cold_node(cont) else {
            return Ok(false);
        };
        let Some(arity) = ffi_arity(&name) else {
            return Err(EvalError::UnknownFfi(name.to_string()));
        };
        Ok(arity == 1)
    }

    fn pair_fields(&mut self, pair: NodeId) -> Result<Option<(NodeId, NodeId)>, EvalError> {
        let pair = self.resolve_profiled(pair)?;
        let Some((result_pair, world)) = self.cell(pair).app_fields() else {
            return Ok(None);
        };
        let result_pair = self.resolve_profiled(result_pair)?;
        let Some((pair_constructor, result)) = self.cell(result_pair).app_fields() else {
            return Ok(None);
        };
        let pair_constructor = self.resolve_profiled(pair_constructor)?;
        Ok(match self.cell(pair_constructor).prim() {
            Some(Prim::Known(KnownPrim::P)) => Some((result, world)),
            _ => None,
        })
    }

    fn selector_pair_field(
        &mut self,
        selector: NodeId,
        pair: NodeId,
    ) -> Result<Option<NodeId>, EvalError> {
        let selector = self.resolve_profiled(selector)?;
        use KnownPrim::*;
        let field = match self.cell(selector).prim() {
            Some(Prim::Known(K)) => 0,
            Some(Prim::Known(A)) => 1,
            _ => return Ok(None),
        };
        let pair = self.reduce_node_whnf(pair, FORCE_REDUCTION_LIMIT)?;
        let Some((result, world)) = self.pair_fields(pair)? else {
            return Ok(None);
        };
        Ok(Some(if field == 0 { result } else { world }))
    }

    fn tuple_first_field_selector_extra(
        &mut self,
        selector: NodeId,
        fields: usize,
        available_extra: usize,
    ) -> Result<Option<usize>, EvalError> {
        let selector = self.resolve_profiled(selector)?;
        use KnownPrim::*;
        let arity = match self.cell(selector).prim() {
            Some(Prim::Known(K2)) => 3,
            Some(Prim::Known(K3)) => 4,
            Some(Prim::Known(K4)) => 5,
            _ => return Ok(None),
        };
        if arity < fields {
            return Ok(None);
        }
        let extra = arity - fields;
        Ok((extra <= available_extra).then_some(extra))
    }

    fn spine(&mut self, root: NodeId) -> Result<Spine, EvalError> {
        let mut node = self.resolve_profiled(root)?;
        let mut inline_args = [const { MaybeUninit::uninit() }; INLINE_SPINE];
        let mut inline_apps = [const { MaybeUninit::uninit() }; INLINE_SPINE];
        let mut inline_len = 0;
        let mut heap: Option<(Vec<NodeId>, Vec<NodeId>)> = None;
        while let Some((fun, arg)) = self.cell(node).app_fields() {
            if let Some((args, apps)) = &mut heap {
                args.push(arg);
                apps.push(node);
            } else if inline_len < INLINE_SPINE {
                inline_args[inline_len].write(arg);
                inline_apps[inline_len].write(node);
                inline_len += 1;
            } else {
                let mut args = Vec::with_capacity(INLINE_SPINE * 2);
                let mut apps = Vec::with_capacity(INLINE_SPINE * 2);
                for idx in 0..inline_len {
                    // SAFETY: indices below inline_len were written above.
                    args.push(unsafe { inline_args[idx].assume_init() });
                    // SAFETY: indices below inline_len were written above.
                    apps.push(unsafe { inline_apps[idx].assume_init() });
                }
                args.push(arg);
                apps.push(node);
                heap = Some((args, apps));
            }
            node = self.resolve_profiled(fun)?;
        }
        let storage = if let Some((mut args, mut apps)) = heap {
            args.reverse();
            apps.reverse();
            SpineStorage::Heap { args, apps }
        } else {
            inline_args[..inline_len].reverse();
            inline_apps[..inline_len].reverse();
            SpineStorage::Inline {
                args: inline_args,
                apps: inline_apps,
                len: inline_len,
            }
        };
        Ok(Spine {
            head: node,
            storage,
        })
    }

    fn apply_reduction_spine(
        &mut self,
        node: &mut NodeId,
        used: usize,
        apps: &[NodeId],
    ) -> Result<bool, EvalError> {
        let mut in_place = false;
        // Match the C reducer's update point: the consumed redex root is shared
        // even when the current evaluation has extra arguments on the spine.
        if used > 0 && used < apps.len() {
            self.set_app_cell_at(apps[used - 1].index(), Cell::indir(Some(*node)));
            in_place = true;
        }
        for app in &apps[used..] {
            let Some((_, arg)) = self.cell(*app).app_fields() else {
                return Err(EvalError::DanglingIndirection(*app));
            };
            self.set_app_cell_at(app.index(), Cell::app(*node, arg));
            *node = *app;
            in_place = true;
        }
        Ok(in_place)
    }

    fn is_identity_alias_node(&mut self, id: NodeId) -> Result<bool, EvalError> {
        let id = self.resolve_profiled(id)?;
        Ok(matches!(
            self.cell(id).prim(),
            Some(Prim::Known(KnownPrim::I | KnownPrim::Ord | KnownPrim::Chr))
        ))
    }

    #[inline]
    fn app(&mut self, fun: NodeId, arg: NodeId) -> NodeId {
        if self.profile.is_some() {
            self.app_alloc_bookkeeping_cold("<generic app()>", fun, arg);
        }
        #[cfg(feature = "eval-phase-profile")]
        let started = self.profile.is_some().then(Instant::now);
        let node = self.push_app_node(fun, arg);
        #[cfg(feature = "eval-phase-profile")]
        if let Some(started) = started {
            let nanos = started.elapsed().as_nanos();
            self.profile_stack_app_alloc_time(nanos);
            self.profile_stack_app_alloc_site_time("<generic app()>", nanos);
        }
        node
    }

    #[inline]
    fn app_with_site(&mut self, key: &'static str, fun: NodeId, arg: NodeId) -> NodeId {
        if self.profile.is_some() {
            self.app_alloc_bookkeeping_cold(key, fun, arg);
        }
        #[cfg(feature = "eval-phase-profile")]
        let started = self.profile.is_some().then(Instant::now);
        let node = self.push_app_node(fun, arg);
        #[cfg(feature = "eval-phase-profile")]
        if let Some(started) = started {
            let nanos = started.elapsed().as_nanos();
            self.profile_stack_app_alloc_time(nanos);
            self.profile_stack_app_alloc_site_time(key, nanos);
        }
        node
    }

    #[cold]
    #[inline(never)]
    fn app_alloc_bookkeeping_cold(&mut self, key: &'static str, fun: NodeId, arg: NodeId) {
        #[cfg(feature = "eval-phase-profile")]
        let profile_started = self.profile.is_some().then(Instant::now);
        #[cfg(feature = "eval-phase-profile")]
        let site_shape = self.profile.is_some().then(|| {
            let fun_shape = self.profile_node_shape_key(fun);
            let arg_shape = self.profile_node_shape_key(arg);
            let mut shape =
                String::with_capacity(key.len() + fun_shape.len() + arg_shape.len() + 6);
            shape.push_str(key);
            shape.push_str(": ");
            shape.push_str(&fun_shape);
            shape.push(' ');
            shape.push_str(&arg_shape);
            shape
        });
        #[cfg(feature = "eval-phase-profile")]
        let resolved_site_shape = self.profile.is_some().then(|| {
            let fun_shape = self.profile_resolved_node_shape_key(fun);
            let arg_shape = self.profile_resolved_node_shape_key(arg);
            let mut shape =
                String::with_capacity(key.len() + fun_shape.len() + arg_shape.len() + 6);
            shape.push_str(key);
            shape.push_str(": ");
            shape.push_str(&fun_shape);
            shape.push(' ');
            shape.push_str(&arg_shape);
            shape
        });
        if let Some(profile) = self.profile.as_mut() {
            profile.app_allocations += 1;
            *profile
                .node_allocations
                .entry(node_allocation_key(&Node::App(fun, arg)).to_owned())
                .or_default() += 1;
            *profile
                .app_allocation_sites
                .entry(key.to_owned())
                .or_default() += 1;
            #[cfg(feature = "eval-phase-profile")]
            if let Some(site_shape) = site_shape {
                *profile
                    .app_allocation_site_shapes
                    .entry(site_shape)
                    .or_default() += 1;
            }
            #[cfg(feature = "eval-phase-profile")]
            if let Some(resolved_site_shape) = resolved_site_shape {
                *profile
                    .app_allocation_resolved_site_shapes
                    .entry(resolved_site_shape)
                    .or_default() += 1;
            }
        }
        #[cfg(feature = "eval-phase-profile")]
        if let Some(started) = profile_started {
            self.profile_app_alloc_bookkeeping_time(started.elapsed().as_nanos());
        }
    }

    fn prim(&mut self, name: &str) -> NodeId {
        let cached = match name {
            "A" => self.prim_cache.a,
            "B" => self.prim_cache.b,
            "C" => self.prim_cache.c,
            "I" => self.prim_cache.i,
            "K" => self.prim_cache.k,
            "K2" => self.prim_cache.k2,
            "K3" => self.prim_cache.k3,
            "O" => self.prim_cache.o,
            "P" => self.prim_cache.p,
            "U" => self.prim_cache.u,
            "Y" => self.prim_cache.y,
            "Z" => self.prim_cache.z,
            "IO.>>=" => self.prim_cache.io_bind,
            "IO.performIO" => self.prim_cache.io_perform_io,
            _ => None,
        };
        if let Some(id) = cached {
            return id;
        }

        let id = self.push_node(Node::prim(name));
        match name {
            "A" => self.prim_cache.a = Some(id),
            "B" => self.prim_cache.b = Some(id),
            "C" => self.prim_cache.c = Some(id),
            "I" => self.prim_cache.i = Some(id),
            "K" => self.prim_cache.k = Some(id),
            "K2" => self.prim_cache.k2 = Some(id),
            "K3" => self.prim_cache.k3 = Some(id),
            "O" => self.prim_cache.o = Some(id),
            "P" => self.prim_cache.p = Some(id),
            "U" => self.prim_cache.u = Some(id),
            "Y" => self.prim_cache.y = Some(id),
            "Z" => self.prim_cache.z = Some(id),
            "IO.>>=" => self.prim_cache.io_bind = Some(id),
            "IO.performIO" => self.prim_cache.io_perform_io = Some(id),
            _ => {}
        }
        id
    }

    fn int(&mut self, value: i64) -> NodeId {
        let Some(index) = small_int_index(value) else {
            if self.profile.is_some() {
                self.profile_non_small_int_allocation();
            }
            return self.push_node(Node::Int(value));
        };
        if let Some(id) = self.small_ints[index] {
            if self.profile.is_some() {
                self.profile_small_int_cache_hit();
            }
            return id;
        }
        if self.profile.is_some() {
            self.profile_small_int_cache_miss();
        }
        let id = self.push_node(Node::Int(value));
        self.small_ints[index] = Some(id);
        id
    }

    fn push_value_node(&mut self, node: Node) -> NodeId {
        match node {
            Node::Int(value) => self.int(value),
            Node::Prim(prim) => self.prim(prim.name()),
            node => self.push_node(node),
        }
    }

    fn world(&mut self) -> NodeId {
        if let Some(world) = self.world {
            return world;
        }
        let world = self.push_node(Node::Int(99_999));
        self.world = Some(world);
        world
    }

    fn fst(&mut self) -> NodeId {
        if let Some(fst) = self.compound_cache.fst {
            return fst;
        }
        let u = self.prim("U");
        let k = self.prim("K");
        let fst = self.app(u, k);
        self.compound_cache.fst = Some(fst);
        fst
    }

    fn snd(&mut self) -> NodeId {
        if let Some(snd) = self.compound_cache.snd {
            return snd;
        }
        let u = self.prim("U");
        let a = self.prim("A");
        let snd = self.app(u, a);
        self.compound_cache.snd = Some(snd);
        snd
    }

    fn pair(&mut self, result: NodeId, world: NodeId) -> NodeId {
        let pair = self.prim("P");
        let result_pair = self.app(pair, result);
        self.app(result_pair, world)
    }

    fn unit_pair(&mut self, world: NodeId) -> NodeId {
        let pair_unit = if let Some(pair_unit) = self.compound_cache.pair_unit {
            pair_unit
        } else {
            let pair = self.prim("P");
            let unit = self.prim("I");
            let pair_unit = self.app(pair, unit);
            self.compound_cache.pair_unit = Some(pair_unit);
            pair_unit
        };
        self.app(pair_unit, world)
    }

    fn just(&mut self, value: NodeId) -> NodeId {
        let just = if let Some(just) = self.compound_cache.just {
            just
        } else {
            let z = self.prim("Z");
            let u = self.prim("U");
            let just = self.app(z, u);
            self.compound_cache.just = Some(just);
            just
        };
        self.app(just, value)
    }

    fn nothing(&mut self) -> NodeId {
        self.prim("K")
    }

    fn catch_result(
        &mut self,
        action: NodeId,
        handler: NodeId,
        world: NodeId,
    ) -> Result<NodeId, EvalError> {
        let old_mask = self.masking_state;
        match self.reduce_node_whnf(action, FORCE_REDUCTION_LIMIT) {
            Ok(result) => Ok(result),
            Err(EvalError::Raised(exn)) => {
                self.masking_state = MASK_INTERRUPTIBLE;
                let handled = self.app(handler, exn);
                let bind = self.prim("IO.>>=");
                let handled_bind = self.app(bind, handled);
                let b_prime = self.prim("B'");
                let then = self.prim("IO.>>");
                let restore = self.prim("IO.setmaskingstate");
                let old_mask = self.int(old_mask);
                let restore = self.app(restore, old_mask);
                let restore_then = self.app(b_prime, then);
                let restore_then = self.app(restore_then, restore);
                let ret = self.prim("IO.return");
                let continuation = self.app(restore_then, ret);
                let caught = self.app(handled_bind, continuation);
                Ok(self.app(caught, world))
            }
            Err(err) => Err(err),
        }
    }

    fn rts_exception(&mut self, code: i64) -> EvalError {
        let exn = self.int(code);
        EvalError::Raised(exn)
    }

    fn arithmetic_eval_error(&mut self, err: EvalError) -> EvalError {
        match err {
            EvalError::DivideByZero => self.rts_exception(RTS_EXN_DIVIDE_BY_ZERO),
            EvalError::Overflow => self.rts_exception(RTS_EXN_OVERFLOW),
            other => other,
        }
    }

    fn ordering(&mut self, ord: Ordering) -> NodeId {
        let name = match ord {
            Ordering::Less => "K2",
            Ordering::Equal => "KK",
            Ordering::Greater => "KA",
        };
        self.prim(name)
    }

    fn bool_value_node(value: bool) -> Node {
        Node::Prim(Prim::Known(if value { KnownPrim::A } else { KnownPrim::K }))
    }

    fn ordering_value_node(ord: Ordering) -> Node {
        let known = match ord {
            Ordering::Less => KnownPrim::K2,
            Ordering::Equal => KnownPrim::KK,
            Ordering::Greater => KnownPrim::KA,
        };
        Node::Prim(Prim::Known(known))
    }

    fn int_result_node(&mut self, result: IntResult) -> NodeId {
        match result {
            IntResult::Int(n) => self.int(n),
            IntResult::Bool(b) => self.prim(if b { "A" } else { "K" }),
            IntResult::Ordering(ord) => self.ordering(ord),
        }
    }

    fn int_result_value_node(result: IntResult) -> Node {
        match result {
            IntResult::Int(n) => Node::Int(n),
            IntResult::Bool(b) => Self::bool_value_node(b),
            IntResult::Ordering(ord) => Self::ordering_value_node(ord),
        }
    }
}
