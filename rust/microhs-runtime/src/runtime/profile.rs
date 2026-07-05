//! Evaluation and GC profiling data structures.
use super::*;

#[derive(Clone, Debug, Default)]
pub struct GcStats {
    pub collections: usize,
    pub freed_nodes_total: usize,
    pub last_live_nodes: usize,
    pub last_free_nodes: usize,
    pub high_water_nodes: usize,
    pub current_nodes: usize,
    pub current_free_nodes: usize,
    pub last_pause_nanos: u128,
    pub total_pause_nanos: u128,
    #[cfg(feature = "gc-phase-profile")]
    pub last_mark_nanos: u128,
    #[cfg(feature = "gc-phase-profile")]
    pub total_mark_nanos: u128,
    #[cfg(feature = "gc-phase-profile")]
    pub last_sweep_nanos: u128,
    #[cfg(feature = "gc-phase-profile")]
    pub total_sweep_nanos: u128,
    pub last_allocations_since_collect: usize,
    pub current_allocations_since_collect: usize,
    pub events: Vec<GcEventStats>,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct GcEventStats {
    pub collection: usize,
    pub pause_nanos: u128,
    #[cfg(feature = "gc-phase-profile")]
    pub mark_nanos: u128,
    #[cfg(feature = "gc-phase-profile")]
    pub sweep_nanos: u128,
    pub live_nodes: usize,
    pub free_nodes: usize,
    pub arena_nodes: usize,
    pub freed_nodes: usize,
    pub allocations_since_collect: usize,
}

#[derive(Clone, Debug, Default)]
pub struct EvalProfile {
    pub step_attempts: usize,
    pub successful_steps: usize,
    pub reductions: usize,
    pub app_allocations: usize,
    pub arg_materializations: usize,
    pub arg_materialized_nodes: usize,
    pub stack_rewrites: usize,
    pub stack_rewrite_apps: usize,
    pub stack_rewrite_indirections: usize,
    pub stack_app_updates: usize,
    pub stack_app_update_apps: usize,
    pub stack_rethreads: usize,
    pub stack_rethread_apps: usize,
    pub stack_descent_pushes: usize,
    pub stack_arg_reads: usize,
    pub stack_arg_batches: usize,
    #[cfg(feature = "eval-phase-profile")]
    pub stack_loop_iterations: usize,
    #[cfg(feature = "eval-phase-profile")]
    pub stack_ready_checks: usize,
    #[cfg(feature = "eval-phase-profile")]
    pub stack_ready_successes: usize,
    #[cfg(feature = "eval-phase-profile")]
    pub stack_eval_step_calls: usize,
    #[cfg(feature = "eval-phase-profile")]
    pub stack_step_reduced: usize,
    #[cfg(feature = "eval-phase-profile")]
    pub stack_step_whnf: usize,
    #[cfg(feature = "eval-phase-profile")]
    pub stack_step_fallback: usize,
    #[cfg(feature = "eval-phase-profile")]
    pub stack_gc_check_nanos: u128,
    #[cfg(feature = "eval-phase-profile")]
    pub stack_resolve_nanos: u128,
    #[cfg(feature = "eval-phase-profile")]
    pub stack_ready_frame_nanos: u128,
    #[cfg(feature = "eval-phase-profile")]
    pub stack_descent_nanos: u128,
    #[cfg(feature = "eval-phase-profile")]
    pub stack_eval_step_nanos: u128,
    #[cfg(feature = "eval-phase-profile")]
    pub stack_whnf_finish_nanos: u128,
    #[cfg(feature = "eval-phase-profile")]
    pub stack_arg_read_nanos: u128,
    #[cfg(feature = "eval-phase-profile")]
    pub stack_app_alloc_nanos: u128,
    #[cfg(feature = "eval-phase-profile")]
    pub stack_apply_rewrite_nanos: u128,
    #[cfg(feature = "eval-phase-profile")]
    pub stack_apply_app_nanos: u128,
    #[cfg(feature = "eval-phase-profile")]
    pub stack_force_frame_nanos: u128,
    #[cfg(feature = "eval-phase-profile")]
    pub stack_inner_descent_nanos: u128,
    #[cfg(feature = "eval-phase-profile")]
    pub app_alloc_reused: usize,
    #[cfg(feature = "eval-phase-profile")]
    pub app_alloc_fresh: usize,
    #[cfg(feature = "eval-phase-profile")]
    pub app_alloc_free_pop_nanos: u128,
    #[cfg(feature = "eval-phase-profile")]
    pub app_alloc_reused_write_nanos: u128,
    #[cfg(feature = "eval-phase-profile")]
    pub app_alloc_fresh_push_nanos: u128,
    #[cfg(feature = "eval-phase-profile")]
    pub profile_step_nanos: u128,
    #[cfg(feature = "eval-phase-profile")]
    pub profile_reduction_nanos: u128,
    #[cfg(feature = "eval-phase-profile")]
    pub profile_stack_head_time_nanos: u128,
    #[cfg(feature = "eval-phase-profile")]
    pub profile_app_alloc_bookkeeping_nanos: u128,
    #[cfg(feature = "eval-phase-profile")]
    pub stack_arg_read_head_nanos: HashMap<String, u128>,
    #[cfg(feature = "eval-phase-profile")]
    pub stack_app_alloc_site_nanos: HashMap<String, u128>,
    #[cfg(feature = "eval-phase-profile")]
    pub stack_apply_rewrite_head_nanos: HashMap<String, u128>,
    #[cfg(feature = "eval-phase-profile")]
    pub stack_apply_app_head_nanos: HashMap<String, u128>,
    #[cfg(feature = "eval-phase-profile")]
    pub stack_force_frame_head_nanos: HashMap<String, u128>,
    #[cfg(feature = "eval-phase-profile")]
    pub stack_inner_descent_head_nanos: HashMap<String, u128>,
    #[cfg(feature = "eval-phase-profile")]
    pub stack_head_arities: HashMap<String, usize>,
    #[cfg(feature = "eval-phase-profile")]
    pub stack_head_arity_classes: HashMap<String, usize>,
    #[cfg(feature = "eval-phase-profile")]
    pub stack_continue_next_heads: HashMap<String, usize>,
    #[cfg(feature = "eval-phase-profile")]
    pub app_allocation_resolved_site_shapes: HashMap<String, usize>,
    pub persistent_forces: usize,
    pub persistent_fallbacks: usize,
    pub fallback_eval_loop_steps: usize,
    pub eval_frame_pushes: usize,
    pub max_spine_arity: usize,
    pub resolve_calls: usize,
    pub resolve_indirections: usize,
    pub max_resolve_chain: usize,
    pub head_attempts: HashMap<String, usize>,
    pub head_reductions: HashMap<String, usize>,
    pub resolve_chain: BTreeMap<usize, usize>,
    pub shortcut_hits: HashMap<String, usize>,
    pub app_allocation_sites: HashMap<String, usize>,
    pub eval_frame_push_kinds: HashMap<String, usize>,
    pub stack_fallback_heads: HashMap<String, usize>,
    #[cfg(feature = "eval-phase-profile")]
    pub stack_eval_step_head_nanos: HashMap<String, u128>,
}

impl EvalProfile {
    pub fn top_head_attempts(&self, limit: usize) -> Vec<(&str, usize)> {
        sorted_profile_counts(&self.head_attempts, limit)
    }

    pub fn top_head_reductions(&self, limit: usize) -> Vec<(&str, usize)> {
        sorted_profile_counts(&self.head_reductions, limit)
    }

    pub fn top_shortcut_hits(&self, limit: usize) -> Vec<(&str, usize)> {
        sorted_profile_counts(&self.shortcut_hits, limit)
    }

    pub fn top_app_allocation_sites(&self, limit: usize) -> Vec<(&str, usize)> {
        sorted_profile_counts(&self.app_allocation_sites, limit)
    }

    pub fn top_eval_frame_push_kinds(&self, limit: usize) -> Vec<(&str, usize)> {
        sorted_profile_counts(&self.eval_frame_push_kinds, limit)
    }

    pub fn top_stack_fallback_heads(&self, limit: usize) -> Vec<(&str, usize)> {
        sorted_profile_counts(&self.stack_fallback_heads, limit)
    }

    #[cfg(feature = "eval-phase-profile")]
    pub fn top_stack_eval_step_head_times(&self, limit: usize) -> Vec<(&str, u128)> {
        sorted_profile_times(&self.stack_eval_step_head_nanos, limit)
    }

    #[cfg(feature = "eval-phase-profile")]
    pub fn top_stack_arg_read_head_times(&self, limit: usize) -> Vec<(&str, u128)> {
        sorted_profile_times(&self.stack_arg_read_head_nanos, limit)
    }

    #[cfg(feature = "eval-phase-profile")]
    pub fn top_stack_app_alloc_site_times(&self, limit: usize) -> Vec<(&str, u128)> {
        sorted_profile_times(&self.stack_app_alloc_site_nanos, limit)
    }

    #[cfg(feature = "eval-phase-profile")]
    pub fn top_stack_apply_rewrite_head_times(&self, limit: usize) -> Vec<(&str, u128)> {
        sorted_profile_times(&self.stack_apply_rewrite_head_nanos, limit)
    }

    #[cfg(feature = "eval-phase-profile")]
    pub fn top_stack_apply_app_head_times(&self, limit: usize) -> Vec<(&str, u128)> {
        sorted_profile_times(&self.stack_apply_app_head_nanos, limit)
    }

    #[cfg(feature = "eval-phase-profile")]
    pub fn top_stack_force_frame_head_times(&self, limit: usize) -> Vec<(&str, u128)> {
        sorted_profile_times(&self.stack_force_frame_head_nanos, limit)
    }

    #[cfg(feature = "eval-phase-profile")]
    pub fn top_stack_inner_descent_head_times(&self, limit: usize) -> Vec<(&str, u128)> {
        sorted_profile_times(&self.stack_inner_descent_head_nanos, limit)
    }

    #[cfg(feature = "eval-phase-profile")]
    pub fn top_stack_head_arities(&self, limit: usize) -> Vec<(&str, usize)> {
        sorted_profile_counts(&self.stack_head_arities, limit)
    }

    #[cfg(feature = "eval-phase-profile")]
    pub fn top_stack_head_arity_classes(&self, limit: usize) -> Vec<(&str, usize)> {
        sorted_profile_counts(&self.stack_head_arity_classes, limit)
    }

    #[cfg(feature = "eval-phase-profile")]
    pub fn top_stack_continue_next_heads(&self, limit: usize) -> Vec<(&str, usize)> {
        sorted_profile_counts(&self.stack_continue_next_heads, limit)
    }

    #[cfg(feature = "eval-phase-profile")]
    pub fn top_app_allocation_resolved_site_shapes(&self, limit: usize) -> Vec<(&str, usize)> {
        sorted_profile_counts(&self.app_allocation_resolved_site_shapes, limit)
    }
}

#[cfg(feature = "eval-phase-profile")]
pub(in crate::runtime) fn profile_known_reducing_arity(known: KnownPrim) -> Option<usize> {
    use KnownPrim::*;
    Some(match known {
        I | Ord | Chr | Y | IoPerformIo | Raise | Rnf | IsInt => 1,
        A | K | U | BPrime | Z | R | K2 | K3 | K4 | Tag(_) | Seq | IoStrict | IoThen => 2,
        S | B | C | P | J | L | KK | KA | CPrimeB | IoBind | IoReturn | IoLazyBind => 3,
        SPrime | CPrime | O | IoAtomic | IoPp | IoPrint | IoSerialize | IoDeserialize => 4,
        Tuple(fields) => usize::from(fields) + 1,
        Catch | CatchR | Dynsym | Thnum | IoGc | IoGetArgRef | IoGetMaskingState | IoNewMVar
        | IoPutMVar | IoReadMVar | IoSetMaskingState | IoStderr | IoStdin | IoStdout | IoStats
        | IoTakeMVar | IoThid | IoThreadStatus | IoTryPutMVar | IoTryReadMVar | IoTryTakeMVar
        | IoYield => return None,
    })
}

pub(in crate::runtime) fn sorted_profile_counts(
    map: &HashMap<String, usize>,
    limit: usize,
) -> Vec<(&str, usize)> {
    let mut counts: Vec<_> = map
        .iter()
        .map(|(key, value)| (key.as_str(), *value))
        .collect();
    counts.sort_unstable_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(right.0)));
    counts.truncate(limit);
    counts
}

#[cfg(feature = "eval-phase-profile")]
pub(in crate::runtime) fn sorted_profile_times(
    map: &HashMap<String, u128>,
    limit: usize,
) -> Vec<(&str, u128)> {
    let mut counts: Vec<_> = map
        .iter()
        .map(|(key, value)| (key.as_str(), *value))
        .collect();
    counts.sort_unstable_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(right.0)));
    counts.truncate(limit);
    counts
}

#[cfg(feature = "eval-phase-profile")]
pub(in crate::runtime) fn cold_profile_key(node: &Node) -> &'static str {
    match node {
        Node::ForeignPtr(_) => "ForeignPtr",
        Node::Weak(_) => "Weak",
        Node::MVar(_) => "MVar",
        Node::BigInt(_) => "BigInt",
        Node::Bytes(_) => "Bytes",
        Node::BytesView(_) => "BytesView",
        Node::MutableBytes(_) => "MutableBytes",
        Node::Array(_) => "Array",
        Node::Ffi(_) => "Ffi",
        Node::JsCall(_) => "JsCall",
        Node::JsWrap { .. } => "JsWrap",
        Node::FunPtr(_) => "FunPtr",
        Node::Tick(_) => "Tick",
        Node::App(_, _)
        | Node::Indir(_)
        | Node::Free(_)
        | Node::Prim(_)
        | Node::Int(_)
        | Node::Int64(_)
        | Node::Float64(_)
        | Node::Float32(_)
        | Node::ThreadId(_)
        | Node::Ptr(_)
        | Node::RawFunPtr(_) => "Hot",
    }
}

pub(in crate::runtime) fn serialization_shareable_node(node: &Node) -> bool {
    matches!(
        node,
        Node::App(_, _)
            | Node::ForeignPtr(_)
            | Node::BigInt(_)
            | Node::Bytes(_)
            | Node::MutableBytes(_)
            | Node::Array(_)
    )
}
