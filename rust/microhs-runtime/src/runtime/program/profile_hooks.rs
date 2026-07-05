//! Program-side hooks for evaluation and GC profiling counters.
use super::*;

impl Program {
    #[cold]
    pub(in crate::runtime) fn profile_head_key(&self, head: NodeId) -> String {
        match self.node_for_debug(head) {
            Node::App(_, _) => "App".to_owned(),
            Node::Indir(_) => "Indir".to_owned(),
            Node::Free(_) => "Free".to_owned(),
            Node::Prim(name) => format!("Prim:{name}"),
            Node::Int(_) => "Int".to_owned(),
            Node::Int64(_) => "Int64".to_owned(),
            Node::Float64(_) => "Float64".to_owned(),
            Node::Float32(_) => "Float32".to_owned(),
            Node::ThreadId(_) => "ThreadId".to_owned(),
            Node::Ptr(_) => "Ptr".to_owned(),
            Node::RawFunPtr(_) => "RawFunPtr".to_owned(),
            Node::ForeignPtr(_) => "ForeignPtr".to_owned(),
            Node::Weak(_) => "Weak".to_owned(),
            Node::MVar(_) => "MVar".to_owned(),
            Node::BigInt(_) => "BigInt".to_owned(),
            Node::Bytes(_) => "Bytes".to_owned(),
            Node::BytesView(_) => "BytesView".to_owned(),
            Node::MutableBytes(_) => "MutableBytes".to_owned(),
            Node::Array(_) => "Array".to_owned(),
            Node::Ffi(name) => format!("Ffi:{name}"),
            Node::JsCall(call) => format!("JsCall:{}", call.tags),
            Node::JsWrap { tags } => format!("JsWrap:{tags}"),
            Node::FunPtr(name) => format!("FunPtr:{name}"),
            Node::Tick(_) => "Tick".to_owned(),
        }
    }

    #[cold]
    pub(in crate::runtime) fn profile_step(
        &mut self,
        head: NodeId,
        arity: usize,
        heap_spine: bool,
    ) -> ProfileHead {
        #[cfg(feature = "eval-phase-profile")]
        let started = Instant::now();
        let key = self.profile_head_key(head);
        #[cfg(feature = "eval-phase-profile")]
        let known_head = self.cell(head).prim();
        let profile = self.profile.as_mut().expect("profile checked");
        profile.step_attempts += 1;
        *profile.head_attempts.entry(key.clone()).or_default() += 1;
        #[cfg(feature = "eval-phase-profile")]
        {
            let mut arity_key = String::with_capacity(key.len() + 8);
            arity_key.push_str(&key);
            arity_key.push('@');
            arity_key.push_str(&arity.to_string());
            *profile.stack_head_arities.entry(arity_key).or_default() += 1;
            if let Some(Prim::Known(known)) = known_head {
                if let Some(required) = profile_known_reducing_arity(known) {
                    let class = match arity.cmp(&required) {
                        std::cmp::Ordering::Less => "under",
                        std::cmp::Ordering::Equal => "exact",
                        std::cmp::Ordering::Greater => "extra",
                    };
                    let mut class_key = String::with_capacity(key.len() + class.len() + 1);
                    class_key.push_str(&key);
                    class_key.push('@');
                    class_key.push_str(class);
                    *profile
                        .stack_head_arity_classes
                        .entry(class_key)
                        .or_default() += 1;
                }
            }
        }
        *profile.spine_arity.entry(arity).or_default() += 1;
        if heap_spine {
            profile.heap_spines += 1;
        }
        profile.max_spine_arity = profile.max_spine_arity.max(arity);
        #[cfg(feature = "eval-phase-profile")]
        {
            profile.profile_step_nanos = profile
                .profile_step_nanos
                .saturating_add(started.elapsed().as_nanos());
        }
        Some(head)
    }

    #[cold]
    pub(in crate::runtime) fn profile_reduction(&mut self, head: ProfileHead, reductions: usize) {
        let Some(head) = head else {
            return;
        };
        #[cfg(feature = "eval-phase-profile")]
        let started = Instant::now();
        let key = self.profile_head_key(head);
        let Some(profile) = self.profile.as_mut() else {
            return;
        };
        profile.successful_steps += 1;
        profile.reductions += reductions;
        *profile.head_reductions.entry(key).or_default() += reductions;
        #[cfg(feature = "eval-phase-profile")]
        {
            profile.profile_reduction_nanos = profile
                .profile_reduction_nanos
                .saturating_add(started.elapsed().as_nanos());
        }
    }

    #[cfg(feature = "eval-phase-profile")]
    #[cold]
    pub(in crate::runtime) fn profile_stack_eval_step_head_time(
        &mut self,
        head: ProfileHead,
        nanos: u128,
    ) {
        let Some(head) = head else {
            return;
        };
        let started = Instant::now();
        let key = self.profile_head_key(head);
        let Some(profile) = self.profile.as_mut() else {
            return;
        };
        *profile.stack_eval_step_head_nanos.entry(key).or_default() += nanos;
        profile.profile_stack_head_time_nanos = profile
            .profile_stack_head_time_nanos
            .saturating_add(started.elapsed().as_nanos());
    }

    #[cfg(feature = "eval-phase-profile")]
    #[cold]
    pub(in crate::runtime) fn profile_stack_arg_read_head_time(
        &mut self,
        head: ProfileHead,
        nanos: u128,
    ) {
        let Some(head) = head else {
            return;
        };
        let started = Instant::now();
        let key = self.profile_head_key(head);
        let Some(profile) = self.profile.as_mut() else {
            return;
        };
        *profile.stack_arg_read_head_nanos.entry(key).or_default() += nanos;
        profile.profile_stack_head_time_nanos = profile
            .profile_stack_head_time_nanos
            .saturating_add(started.elapsed().as_nanos());
    }

    #[cfg(feature = "eval-phase-profile")]
    #[cold]
    pub(in crate::runtime) fn profile_stack_app_alloc_site_time(
        &mut self,
        site: &'static str,
        nanos: u128,
    ) {
        let started = Instant::now();
        let Some(profile) = self.profile.as_mut() else {
            return;
        };
        *profile
            .stack_app_alloc_site_nanos
            .entry(site.to_owned())
            .or_default() += nanos;
        profile.profile_stack_head_time_nanos = profile
            .profile_stack_head_time_nanos
            .saturating_add(started.elapsed().as_nanos());
    }

    #[cfg(feature = "eval-phase-profile")]
    #[cold]
    pub(in crate::runtime) fn profile_stack_apply_rewrite_head_time(
        &mut self,
        head: ProfileHead,
        nanos: u128,
    ) {
        let Some(head) = head else {
            return;
        };
        let started = Instant::now();
        let key = self.profile_head_key(head);
        let Some(profile) = self.profile.as_mut() else {
            return;
        };
        *profile
            .stack_apply_rewrite_head_nanos
            .entry(key)
            .or_default() += nanos;
        profile.profile_stack_head_time_nanos = profile
            .profile_stack_head_time_nanos
            .saturating_add(started.elapsed().as_nanos());
    }

    #[cfg(feature = "eval-phase-profile")]
    #[cold]
    pub(in crate::runtime) fn profile_stack_apply_app_head_time(
        &mut self,
        head: ProfileHead,
        nanos: u128,
    ) {
        let Some(head) = head else {
            return;
        };
        let started = Instant::now();
        let key = self.profile_head_key(head);
        let Some(profile) = self.profile.as_mut() else {
            return;
        };
        *profile.stack_apply_app_head_nanos.entry(key).or_default() += nanos;
        profile.profile_stack_head_time_nanos = profile
            .profile_stack_head_time_nanos
            .saturating_add(started.elapsed().as_nanos());
    }

    #[cfg(feature = "eval-phase-profile")]
    #[cold]
    pub(in crate::runtime) fn profile_stack_force_frame_head_time(
        &mut self,
        head: ProfileHead,
        nanos: u128,
    ) {
        let Some(head) = head else {
            return;
        };
        let started = Instant::now();
        let key = self.profile_head_key(head);
        let Some(profile) = self.profile.as_mut() else {
            return;
        };
        *profile.stack_force_frame_head_nanos.entry(key).or_default() += nanos;
        profile.profile_stack_head_time_nanos = profile
            .profile_stack_head_time_nanos
            .saturating_add(started.elapsed().as_nanos());
    }

    #[cfg(feature = "eval-phase-profile")]
    #[cold]
    pub(in crate::runtime) fn profile_stack_inner_descent_head_time(
        &mut self,
        head: ProfileHead,
        nanos: u128,
    ) {
        let Some(head) = head else {
            return;
        };
        let started = Instant::now();
        let key = self.profile_head_key(head);
        let Some(profile) = self.profile.as_mut() else {
            return;
        };
        *profile
            .stack_inner_descent_head_nanos
            .entry(key)
            .or_default() += nanos;
        profile.profile_stack_head_time_nanos = profile
            .profile_stack_head_time_nanos
            .saturating_add(started.elapsed().as_nanos());
    }

    #[cfg(feature = "eval-phase-profile")]
    #[cold]
    pub(in crate::runtime) fn profile_stack_continue_next_head(
        &mut self,
        from: ProfileHead,
        next: NodeId,
        arity: usize,
    ) {
        let Some(from) = from else {
            return;
        };
        let started = Instant::now();
        let from_key = self.profile_head_key(from);
        let next_key = self.profile_head_key(next);
        let mut key = String::with_capacity(from_key.len() + next_key.len() + 16);
        key.push_str(&from_key);
        key.push_str("->");
        key.push_str(&next_key);
        key.push('@');
        key.push_str(&arity.to_string());
        let Some(profile) = self.profile.as_mut() else {
            return;
        };
        *profile.stack_continue_next_heads.entry(key).or_default() += 1;
        profile.profile_stack_head_time_nanos = profile
            .profile_stack_head_time_nanos
            .saturating_add(started.elapsed().as_nanos());
    }

    #[cfg(feature = "eval-phase-profile")]
    pub(in crate::runtime) fn profile_node_shape_key(&self, id: NodeId) -> String {
        let cell = self.cell(id);
        match cell.tag() {
            CellTag::App => {
                let fun = cell.id_payload();
                match self.cell(fun).prim() {
                    Some(prim) => format!("App({})", prim.name()),
                    None => format!("App({})", self.profile_cell_shape_key(self.cell(fun))),
                }
            }
            CellTag::Cold => self
                .cold_node(id)
                .map(cold_profile_key)
                .unwrap_or("Cold")
                .to_owned(),
            _ => self.profile_cell_shape_key(cell).to_owned(),
        }
    }

    #[cfg(feature = "eval-phase-profile")]
    pub(in crate::runtime) fn profile_resolved_node_shape_key(&self, id: NodeId) -> String {
        let Some(id) = self.gc_profile_resolved_id(id) else {
            return "Free".to_owned();
        };
        let cell = self.cell(id);
        match cell.tag() {
            CellTag::App => {
                let fun = cell.id_payload();
                let fun = self.gc_profile_resolved_id(fun).unwrap_or(fun);
                match self.cell(fun).prim() {
                    Some(prim) => format!("App({})", prim.name()),
                    None => format!("App({})", self.profile_cell_shape_key(self.cell(fun))),
                }
            }
            CellTag::Cold => self
                .cold_node(id)
                .map(cold_profile_key)
                .unwrap_or("Cold")
                .to_owned(),
            _ => self.profile_cell_shape_key(cell).to_owned(),
        }
    }

    #[cfg(feature = "eval-phase-profile")]
    pub(in crate::runtime) fn profile_eval_whnf_value(
        &mut self,
        kind: &'static str,
        immediate: bool,
    ) {
        let Some(profile) = self.profile.as_mut() else {
            return;
        };
        *profile
            .eval_whnf_value_calls
            .entry(kind.to_owned())
            .or_default() += 1;
        if !immediate {
            *profile
                .eval_whnf_value_slow
                .entry(kind.to_owned())
                .or_default() += 1;
        }
    }

    #[cfg(feature = "eval-phase-profile")]
    pub(in crate::runtime) fn profile_reduce_node_whnf_entry(&mut self, root: NodeId) {
        if self.profile.is_none() {
            return;
        }
        let shape = self.profile_resolved_node_shape_key(root);
        if let Some(profile) = self.profile.as_mut() {
            *profile
                .reduce_node_whnf_entry_shapes
                .entry(shape)
                .or_default() += 1;
        }
    }

    #[cfg(feature = "eval-phase-profile")]
    pub(in crate::runtime) fn profile_cell_shape_key(&self, cell: Cell) -> &'static str {
        match cell.tag() {
            CellTag::App => "App",
            CellTag::Indir => "Indir",
            CellTag::Free => "Free",
            CellTag::KnownPrim | CellTag::RuntimePrim => {
                cell.prim_name().expect("primitive tag must decode")
            }
            CellTag::Int => "Int",
            CellTag::Float32 => "Float32",
            CellTag::ThreadId => "ThreadId",
            CellTag::Cold => "Cold",
        }
    }

    #[cfg(feature = "eval-phase-profile")]
    #[cold]
    pub(in crate::runtime) fn profile_stack_arg_read_time(&mut self, nanos: u128) {
        if let Some(profile) = self.profile.as_mut() {
            profile.stack_arg_read_nanos = profile.stack_arg_read_nanos.saturating_add(nanos);
        }
    }

    #[cfg(feature = "eval-phase-profile")]
    #[cold]
    pub(in crate::runtime) fn profile_stack_app_alloc_time(&mut self, nanos: u128) {
        if let Some(profile) = self.profile.as_mut() {
            profile.stack_app_alloc_nanos = profile.stack_app_alloc_nanos.saturating_add(nanos);
        }
    }

    #[cfg(feature = "eval-phase-profile")]
    #[cold]
    pub(in crate::runtime) fn profile_stack_apply_rewrite_time(&mut self, nanos: u128) {
        if let Some(profile) = self.profile.as_mut() {
            profile.stack_apply_rewrite_nanos =
                profile.stack_apply_rewrite_nanos.saturating_add(nanos);
        }
    }

    #[cfg(feature = "eval-phase-profile")]
    #[cold]
    pub(in crate::runtime) fn profile_stack_apply_app_time(&mut self, nanos: u128) {
        if let Some(profile) = self.profile.as_mut() {
            profile.stack_apply_app_nanos = profile.stack_apply_app_nanos.saturating_add(nanos);
        }
    }

    #[cfg(feature = "eval-phase-profile")]
    #[cold]
    pub(in crate::runtime) fn profile_stack_force_frame_time(&mut self, nanos: u128) {
        if let Some(profile) = self.profile.as_mut() {
            profile.stack_force_frame_nanos = profile.stack_force_frame_nanos.saturating_add(nanos);
        }
    }

    #[cfg(feature = "eval-phase-profile")]
    #[cold]
    pub(in crate::runtime) fn profile_stack_inner_descent_time(&mut self, nanos: u128) {
        if let Some(profile) = self.profile.as_mut() {
            profile.stack_inner_descent_nanos =
                profile.stack_inner_descent_nanos.saturating_add(nanos);
        }
    }

    #[cfg(feature = "eval-phase-profile")]
    #[cold]
    pub(in crate::runtime) fn profile_app_alloc_bookkeeping_time(&mut self, nanos: u128) {
        if let Some(profile) = self.profile.as_mut() {
            profile.profile_app_alloc_bookkeeping_nanos = profile
                .profile_app_alloc_bookkeeping_nanos
                .saturating_add(nanos);
        }
    }

    #[cold]
    pub(in crate::runtime) fn profile_resolve_chain(&mut self, depth: usize) {
        let Some(profile) = self.profile.as_mut() else {
            return;
        };
        profile.resolve_calls += 1;
        profile.resolve_indirections += depth;
        profile.max_resolve_chain = profile.max_resolve_chain.max(depth);
        *profile.resolve_chain.entry(depth).or_default() += 1;
    }

    #[cold]
    pub(in crate::runtime) fn profile_shortcut(&mut self, key: &'static str, count: usize) {
        if count == 0 {
            return;
        }
        let Some(profile) = self.profile.as_mut() else {
            return;
        };
        if let Some(existing) = profile.shortcut_hits.get_mut(key) {
            *existing += count;
        } else {
            profile.shortcut_hits.insert(key.to_owned(), count);
        }
    }

    #[cold]
    pub(in crate::runtime) fn profile_arg_materialization(&mut self, nodes: usize) {
        if let Some(profile) = self.profile.as_mut() {
            profile.arg_materializations += 1;
            profile.arg_materialized_nodes += nodes;
        }
    }

    #[cold]
    pub(in crate::runtime) fn profile_spine_rewrite(&mut self, extra_args: usize) {
        if let Some(profile) = self.profile.as_mut() {
            profile.spine_rewrites += 1;
            profile.spine_rewrite_extra_args += extra_args;
        }
    }

    #[cold]
    pub(in crate::runtime) fn profile_app_rewrite(&mut self, extra_args: usize) {
        if let Some(profile) = self.profile.as_mut() {
            profile.app_rewrites += 1;
            profile.app_rewrite_extra_args += extra_args;
        }
    }

    #[cold]
    pub(in crate::runtime) fn profile_stack_rewrite(
        &mut self,
        used: usize,
        wrote_indirection: bool,
    ) {
        if let Some(profile) = self.profile.as_mut() {
            profile.stack_rewrites += 1;
            profile.stack_rewrite_apps += used;
            if wrote_indirection {
                profile.stack_rewrite_indirections += 1;
            }
        }
    }

    #[cold]
    pub(in crate::runtime) fn profile_stack_app_update(&mut self, used: usize) {
        if let Some(profile) = self.profile.as_mut() {
            profile.stack_app_updates += 1;
            profile.stack_app_update_apps += used;
            if used == 0 {
                profile.stack_app_update_allocations += 1;
            }
        }
    }

    #[cold]
    pub(in crate::runtime) fn profile_stack_rethread(&mut self, apps: usize) {
        if let Some(profile) = self.profile.as_mut() {
            profile.stack_rethreads += 1;
            profile.stack_rethread_apps += apps;
        }
    }

    #[cold]
    pub(in crate::runtime) fn profile_stack_descent_push(&mut self) {
        if let Some(profile) = self.profile.as_mut() {
            profile.stack_descent_pushes += 1;
        }
    }

    #[cold]
    pub(in crate::runtime) fn profile_stack_arg_reads(&mut self, reads: usize) {
        if let Some(profile) = self.profile.as_mut() {
            profile.stack_arg_reads += reads;
        }
    }

    #[cold]
    pub(in crate::runtime) fn profile_stack_arg_batch(&mut self) {
        if let Some(profile) = self.profile.as_mut() {
            profile.stack_arg_batches += 1;
        }
    }

    #[cold]
    pub(in crate::runtime) fn profile_persistent_force(&mut self) {
        if let Some(profile) = self.profile.as_mut() {
            profile.persistent_forces += 1;
        }
    }

    #[cold]
    pub(in crate::runtime) fn profile_persistent_fallback(&mut self) {
        if let Some(profile) = self.profile.as_mut() {
            profile.persistent_fallbacks += 1;
        }
    }

    #[cold]
    pub(in crate::runtime) fn profile_stack_fallback_head(&mut self, head: NodeId) {
        if self.profile.is_some() {
            let key = self.profile_head_key(head);
            if let Some(profile) = self.profile.as_mut() {
                *profile.stack_fallback_heads.entry(key).or_insert(0) += 1;
            }
        }
    }

    #[cold]
    pub(in crate::runtime) fn profile_fallback_eval_loop_step(&mut self) {
        if let Some(profile) = self.profile.as_mut() {
            profile.fallback_eval_loop_steps += 1;
        }
    }

    #[cold]
    pub(in crate::runtime) fn profile_eval_frame_push(&mut self, kind: &'static str) {
        if let Some(profile) = self.profile.as_mut() {
            profile.eval_frame_pushes += 1;
            *profile
                .eval_frame_push_kinds
                .entry(kind.to_owned())
                .or_default() += 1;
        }
    }

    #[cold]
    pub(in crate::runtime) fn profile_small_int_cache_hit(&mut self) {
        if let Some(profile) = self.profile.as_mut() {
            profile.small_int_cache_hits += 1;
        }
    }

    #[cold]
    pub(in crate::runtime) fn profile_small_int_cache_miss(&mut self) {
        if let Some(profile) = self.profile.as_mut() {
            profile.small_int_cache_misses += 1;
        }
    }

    #[cold]
    pub(in crate::runtime) fn profile_non_small_int_allocation(&mut self) {
        if let Some(profile) = self.profile.as_mut() {
            profile.non_small_int_allocations += 1;
        }
    }

    #[cold]
    pub(in crate::runtime) fn profile_primitive_dispatch_probe(&mut self, key: &'static str) {
        if let Some(profile) = self.profile.as_mut() {
            *profile
                .primitive_dispatch_probes
                .entry(key.to_owned())
                .or_default() += 1;
        }
    }

    #[cold]
    pub(in crate::runtime) fn profile_primitive_dispatch_hit(&mut self, key: &'static str) {
        if let Some(profile) = self.profile.as_mut() {
            *profile
                .primitive_dispatch_hits
                .entry(key.to_owned())
                .or_default() += 1;
        }
    }

    #[cold]
    pub(in crate::runtime) fn profile_strict_primitive_dispatch(
        &mut self,
        args_len: usize,
        action: StrictPrimitiveAction,
    ) {
        macro_rules! probe {
            ($min_args:expr, $key:literal, $pattern:pat) => {
                if args_len >= $min_args {
                    self.profile_primitive_dispatch_probe($key);
                    if matches!(action, $pattern) {
                        self.profile_primitive_dispatch_hit($key);
                        return;
                    }
                }
            };
        }

        probe!(2, "strict_int_binop", StrictPrimitiveAction::IntBin(_));
        probe!(1, "strict_int_unop", StrictPrimitiveAction::IntUn(_));
        probe!(2, "strict_int64_binop", StrictPrimitiveAction::Int64Bin(_));
        probe!(1, "strict_int64_unop", StrictPrimitiveAction::Int64Un(_));
        probe!(
            2,
            "strict_float64_binop",
            StrictPrimitiveAction::Float64Bin(_)
        );
        probe!(
            1,
            "strict_float64_unop",
            StrictPrimitiveAction::Float64Un(_)
        );
        probe!(
            2,
            "strict_float32_binop",
            StrictPrimitiveAction::Float32Bin(_)
        );
        probe!(
            1,
            "strict_float32_unop",
            StrictPrimitiveAction::Float32Un(_)
        );
        probe!(2, "strict_bytes_binop", StrictPrimitiveAction::BytesBin(_));
        probe!(1, "strict_conversion", StrictPrimitiveAction::Conversion(_));
    }

    #[cold]
    pub(in crate::runtime) fn profile_node_allocation(&mut self, node: &Node) {
        if let Some(profile) = self.profile.as_mut() {
            *profile
                .node_allocations
                .entry(node_allocation_key(node).to_owned())
                .or_default() += 1;
        }
    }
}
