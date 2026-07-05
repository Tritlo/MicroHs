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

#[cfg(feature = "profile")]
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
    pub stack_descent_pushes: usize,
    pub stack_arg_reads: usize,
    pub stack_arg_batches: usize,
    pub strict_forces: usize,
    pub fallback_entries: usize,
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
}

#[cfg(feature = "profile")]
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
}

#[cfg(feature = "profile")]
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
