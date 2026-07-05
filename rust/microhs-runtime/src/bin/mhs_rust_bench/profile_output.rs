use super::metrics::{millis, nanos_millis, print_gc_events};
use super::runner::ProfileBench;

pub(super) fn print_profile(profile: &ProfileBench, top: usize) {
    println!("profile_total_ms: {:.3}", millis(profile.elapsed));
    println!("profile_steps: {}", profile.steps);
    println!("profile_sink: {}", profile.serialize_sink);
    println!("profile_step_limited: {}", profile.step_limited);
    println!("profile_nodes_before: {}", profile.nodes_before);
    println!("profile_nodes_after: {}", profile.nodes_after);
    println!("profile_gc_collections: {}", profile.gc.collections);
    println!(
        "profile_gc_freed_nodes_total: {}",
        profile.gc.freed_nodes_total
    );
    println!("profile_gc_last_live_nodes: {}", profile.gc.last_live_nodes);
    println!("profile_gc_last_free_nodes: {}", profile.gc.last_free_nodes);
    println!(
        "profile_gc_high_water_nodes: {}",
        profile.gc.high_water_nodes
    );
    println!("profile_gc_current_nodes: {}", profile.gc.current_nodes);
    println!(
        "profile_gc_current_free_nodes: {}",
        profile.gc.current_free_nodes
    );
    println!(
        "profile_gc_last_pause_ms: {:.3}",
        nanos_millis(profile.gc.last_pause_nanos)
    );
    println!(
        "profile_gc_total_pause_ms: {:.3}",
        nanos_millis(profile.gc.total_pause_nanos)
    );
    #[cfg(feature = "gc-phase-profile")]
    {
        println!(
            "profile_gc_last_mark_ms: {:.3}",
            nanos_millis(profile.gc.last_mark_nanos)
        );
        println!(
            "profile_gc_total_mark_ms: {:.3}",
            nanos_millis(profile.gc.total_mark_nanos)
        );
        println!(
            "profile_gc_last_sweep_ms: {:.3}",
            nanos_millis(profile.gc.last_sweep_nanos)
        );
        println!(
            "profile_gc_total_sweep_ms: {:.3}",
            nanos_millis(profile.gc.total_sweep_nanos)
        );
    }
    println!(
        "profile_gc_last_allocations_since_collect: {}",
        profile.gc.last_allocations_since_collect
    );
    println!(
        "profile_gc_current_allocations_since_collect: {}",
        profile.gc.current_allocations_since_collect
    );
    print_gc_events("profile_gc_events", &profile.gc);
    println!(
        "profile_node_growth: {}",
        profile.nodes_after.saturating_sub(profile.nodes_before)
    );
    println!("profile_step_attempts: {}", profile.profile.step_attempts);
    println!(
        "profile_successful_steps: {}",
        profile.profile.successful_steps
    );
    println!("profile_reductions: {}", profile.profile.reductions);
    println!(
        "profile_app_allocations: {}",
        profile.profile.app_allocations
    );
    println!(
        "profile_arg_materializations: {}",
        profile.profile.arg_materializations
    );
    println!(
        "profile_arg_materialized_nodes: {}",
        profile.profile.arg_materialized_nodes
    );
    println!("profile_stack_rewrites: {}", profile.profile.stack_rewrites);
    println!(
        "profile_stack_rewrite_apps: {}",
        profile.profile.stack_rewrite_apps
    );
    println!(
        "profile_stack_rewrite_indirections: {}",
        profile.profile.stack_rewrite_indirections
    );
    println!(
        "profile_stack_app_updates: {}",
        profile.profile.stack_app_updates
    );
    println!(
        "profile_stack_app_update_apps: {}",
        profile.profile.stack_app_update_apps
    );
    println!(
        "profile_stack_descent_pushes: {}",
        profile.profile.stack_descent_pushes
    );
    println!(
        "profile_stack_arg_reads: {}",
        profile.profile.stack_arg_reads
    );
    println!(
        "profile_stack_arg_batches: {}",
        profile.profile.stack_arg_batches
    );
    println!(
        "profile_persistent_forces: {}",
        profile.profile.persistent_forces
    );
    println!(
        "profile_persistent_fallbacks: {}",
        profile.profile.persistent_fallbacks
    );
    println!(
        "profile_fallback_eval_loop_steps: {}",
        profile.profile.fallback_eval_loop_steps
    );
    println!(
        "profile_eval_frame_pushes: {}",
        profile.profile.eval_frame_pushes
    );
    println!(
        "profile_max_spine_arity: {}",
        profile.profile.max_spine_arity
    );
    println!("profile_resolve_calls: {}", profile.profile.resolve_calls);
    println!(
        "profile_resolve_indirections: {}",
        profile.profile.resolve_indirections
    );
    println!(
        "profile_max_resolve_chain: {}",
        profile.profile.max_resolve_chain
    );
    println!("profile_top_head_attempts:");
    for (head, count) in profile.profile.top_head_attempts(top) {
        println!("  {head}: {count}");
    }
    println!("profile_top_head_reductions:");
    for (head, count) in profile.profile.top_head_reductions(top) {
        println!("  {head}: {count}");
    }
    println!("profile_resolve_chain:");
    for (depth, count) in &profile.profile.resolve_chain {
        println!("  {depth}: {count}");
    }
    println!("profile_shortcut_hits:");
    for (shortcut, count) in profile.profile.top_shortcut_hits(top) {
        println!("  {shortcut}: {count}");
    }
    println!("profile_app_allocation_sites:");
    for (site, count) in profile.profile.top_app_allocation_sites(top) {
        println!("  {site}: {count}");
    }
    println!("profile_eval_frame_push_kinds:");
    for (kind, count) in profile.profile.top_eval_frame_push_kinds(top) {
        println!("  {kind}: {count}");
    }
    println!("profile_stack_fallback_heads:");
    for (head, count) in profile.profile.top_stack_fallback_heads(top) {
        println!("  {head}: {count}");
    }
}
