//! Program-side hooks for evaluation and GC profiling counters.
use super::*;

#[cfg(feature = "profile")]
impl Program {
    #[inline]
    pub(in crate::runtime) fn profiling_enabled(&self) -> bool {
        self.profile.is_some()
    }

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
    pub(in crate::runtime) fn profile_step(&mut self, head: NodeId, arity: usize) -> ProfileHead {
        let key = self.profile_head_key(head);
        let profile = self.profile.as_mut().expect("profile checked");
        profile.step_attempts += 1;
        *profile.head_attempts.entry(key.clone()).or_default() += 1;
        profile.max_spine_arity = profile.max_spine_arity.max(arity);
        ProfileHead::from_node(head)
    }

    #[cold]
    pub(in crate::runtime) fn profile_reduction(&mut self, head: ProfileHead, reductions: usize) {
        let Some(head) = head.node() else {
            return;
        };
        let key = self.profile_head_key(head);
        let Some(profile) = self.profile.as_mut() else {
            return;
        };
        profile.successful_steps += 1;
        profile.reductions += reductions;
        *profile.head_reductions.entry(key).or_default() += reductions;
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
    pub(in crate::runtime) fn profile_strict_force(&mut self) {
        if let Some(profile) = self.profile.as_mut() {
            profile.strict_forces += 1;
        }
    }

    #[cold]
    pub(in crate::runtime) fn profile_fallback_entry(&mut self) {
        if let Some(profile) = self.profile.as_mut() {
            profile.fallback_entries += 1;
        }
    }

    #[cold]
    pub(in crate::runtime) fn profile_stack_fallback_head(&mut self, head: NodeId) {
        if self.profiling_enabled() {
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
}

#[cfg(not(feature = "profile"))]
impl Program {
    #[inline]
    pub(in crate::runtime) fn profiling_enabled(&self) -> bool {
        false
    }

    #[inline]
    pub(in crate::runtime) fn profile_step(&mut self, _head: NodeId, _arity: usize) -> ProfileHead {
        ProfileHead::none()
    }

    #[inline]
    pub(in crate::runtime) fn profile_reduction(&mut self, _head: ProfileHead, _reductions: usize) {
    }

    #[inline]
    pub(in crate::runtime) fn profile_resolve_chain(&mut self, _depth: usize) {}

    #[inline]
    pub(in crate::runtime) fn profile_shortcut(&mut self, _key: &'static str, _count: usize) {}

    #[inline]
    pub(in crate::runtime) fn profile_arg_materialization(&mut self, _nodes: usize) {}

    #[inline]
    pub(in crate::runtime) fn profile_stack_rewrite(
        &mut self,
        _used: usize,
        _wrote_indirection: bool,
    ) {
    }

    #[inline]
    pub(in crate::runtime) fn profile_stack_app_update(&mut self, _used: usize) {}

    #[inline]
    pub(in crate::runtime) fn profile_stack_descent_push(&mut self) {}

    #[inline]
    pub(in crate::runtime) fn profile_stack_arg_reads(&mut self, _reads: usize) {}

    #[inline]
    pub(in crate::runtime) fn profile_stack_arg_batch(&mut self) {}

    #[inline]
    pub(in crate::runtime) fn profile_strict_force(&mut self) {}

    #[inline]
    pub(in crate::runtime) fn profile_fallback_entry(&mut self) {}

    #[inline]
    pub(in crate::runtime) fn profile_stack_fallback_head(&mut self, _head: NodeId) {}

    #[inline]
    pub(in crate::runtime) fn profile_fallback_eval_loop_step(&mut self) {}

    #[inline]
    pub(in crate::runtime) fn profile_eval_frame_push(&mut self, _kind: &'static str) {}
}
