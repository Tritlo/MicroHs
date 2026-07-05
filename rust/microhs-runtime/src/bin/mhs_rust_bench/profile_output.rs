fn print_profile(profile: &ProfileBench, top: usize) {
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
        println!(
            "profile_gc_red_i_opportunities: {}",
            profile.gc.red_i_opportunities
        );
        println!(
            "profile_gc_red_k_opportunities: {}",
            profile.gc.red_k_opportunities
        );
        println!(
            "profile_gc_red_a_opportunities: {}",
            profile.gc.red_a_opportunities
        );
        println!(
            "profile_gc_red_bi_opportunities: {}",
            profile.gc.red_bi_opportunities
        );
        println!(
            "profile_gc_red_bxi_opportunities: {}",
            profile.gc.red_bxi_opportunities
        );
        println!(
            "profile_gc_red_ccbi_opportunities: {}",
            profile.gc.red_ccbi_opportunities
        );
        println!(
            "profile_gc_red_cc_opportunities: {}",
            profile.gc.red_cc_opportunities
        );
        println!(
            "profile_gc_red_cci_opportunities: {}",
            profile.gc.red_cci_opportunities
        );
        println!(
            "profile_gc_red_ccbbcp_opportunities: {}",
            profile.gc.red_ccbbcp_opportunities
        );
        println!(
            "profile_gc_red_flip_opportunities: {}",
            profile.gc.red_flip_opportunities
        );
        println!(
            "profile_gc_young_profile_last_slots: {}",
            profile.gc.young_profile_last_slots
        );
        println!(
            "profile_gc_young_profile_last_live: {}",
            profile.gc.young_profile_last_live
        );
        println!(
            "profile_gc_young_profile_last_dead: {}",
            profile.gc.young_profile_last_dead
        );
        println!(
            "profile_gc_young_profile_last_old_to_young_sources: {}",
            profile.gc.young_profile_last_old_to_young_sources
        );
        println!(
            "profile_gc_young_profile_last_old_to_young_edges: {}",
            profile.gc.young_profile_last_old_to_young_edges
        );
        println!(
            "profile_gc_young_profile_total_slots: {}",
            profile.gc.young_profile_total_slots
        );
        println!(
            "profile_gc_young_profile_total_live: {}",
            profile.gc.young_profile_total_live
        );
        println!(
            "profile_gc_young_profile_total_dead: {}",
            profile.gc.young_profile_total_dead
        );
        println!(
            "profile_gc_young_profile_total_old_to_young_sources: {}",
            profile.gc.young_profile_total_old_to_young_sources
        );
        println!(
            "profile_gc_young_profile_total_old_to_young_edges: {}",
            profile.gc.young_profile_total_old_to_young_edges
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
    println!("profile_spine_rewrites: {}", profile.profile.spine_rewrites);
    println!(
        "profile_spine_rewrite_extra_args: {}",
        profile.profile.spine_rewrite_extra_args
    );
    println!("profile_app_rewrites: {}", profile.profile.app_rewrites);
    println!(
        "profile_app_rewrite_extra_args: {}",
        profile.profile.app_rewrite_extra_args
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
        "profile_stack_app_update_allocations: {}",
        profile.profile.stack_app_update_allocations
    );
    println!(
        "profile_stack_rethreads: {}",
        profile.profile.stack_rethreads
    );
    println!(
        "profile_stack_rethread_apps: {}",
        profile.profile.stack_rethread_apps
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
    #[cfg(feature = "eval-phase-profile")]
    print_phase_profile(profile, top);
    let spine_arity_entries: usize = profile
        .profile
        .spine_arity
        .iter()
        .map(|(arity, count)| arity.saturating_mul(*count))
        .sum();
    println!("profile_spine_arity_entries: {spine_arity_entries}");
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
        "profile_strict_redex_snapshots: {}",
        profile.profile.strict_redex_snapshots
    );
    println!(
        "profile_strict_redex_snapshot_apps: {}",
        profile.profile.strict_redex_snapshot_apps
    );
    println!(
        "profile_remaining_app_scans: {}",
        profile.profile.remaining_app_scans
    );
    println!(
        "profile_remaining_app_scan_apps: {}",
        profile.profile.remaining_app_scan_apps
    );
    println!(
        "profile_eval_frame_pushes: {}",
        profile.profile.eval_frame_pushes
    );
    println!(
        "profile_small_int_cache_hits: {}",
        profile.profile.small_int_cache_hits
    );
    println!(
        "profile_small_int_cache_misses: {}",
        profile.profile.small_int_cache_misses
    );
    println!(
        "profile_non_small_int_allocations: {}",
        profile.profile.non_small_int_allocations
    );
    println!("profile_heap_spines: {}", profile.profile.heap_spines);
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
    println!("profile_spine_arity:");
    for (arity, count) in &profile.profile.spine_arity {
        println!("  {arity}: {count}");
    }
    println!("profile_resolve_chain:");
    for (depth, count) in &profile.profile.resolve_chain {
        println!("  {depth}: {count}");
    }
    println!("profile_shortcut_hits:");
    for (shortcut, count) in profile.profile.top_shortcut_hits(top) {
        println!("  {shortcut}: {count}");
    }
    println!("profile_primitive_dispatch_probes:");
    for (probe, count) in profile.profile.top_primitive_dispatch_probes(top) {
        println!("  {probe}: {count}");
    }
    println!("profile_primitive_dispatch_hits:");
    for (hit, count) in profile.profile.top_primitive_dispatch_hits(top) {
        println!("  {hit}: {count}");
    }
    println!("profile_node_allocations:");
    for (kind, count) in profile.profile.top_node_allocations(top) {
        println!("  {kind}: {count}");
    }
    println!("profile_app_allocation_sites:");
    for (site, count) in profile.profile.top_app_allocation_sites(top) {
        println!("  {site}: {count}");
    }
    #[cfg(feature = "eval-phase-profile")]
    {
        println!("profile_app_allocation_site_shapes:");
        for (site, count) in profile.profile.top_app_allocation_site_shapes(top) {
            println!("  {site}: {count}");
        }
        println!("profile_app_allocation_resolved_site_shapes:");
        for (site, count) in profile.profile.top_app_allocation_resolved_site_shapes(top) {
            println!("  {site}: {count}");
        }
        println!("profile_eval_whnf_value_calls:");
        for (kind, count) in profile.profile.top_eval_whnf_value_calls(top) {
            println!("  {kind}: {count}");
        }
        println!("profile_eval_whnf_value_slow:");
        for (kind, count) in profile.profile.top_eval_whnf_value_slow(top) {
            println!("  {kind}: {count}");
        }
        println!("profile_reduce_node_whnf_entry_shapes:");
        for (shape, count) in profile.profile.top_reduce_node_whnf_entry_shapes(top) {
            println!("  {shape}: {count}");
        }
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

#[cfg(feature = "eval-phase-profile")]
fn print_phase_profile(profile: &ProfileBench, top: usize) {
    println!(
        "profile_stack_loop_iterations: {}",
        profile.profile.stack_loop_iterations
    );
    println!(
        "profile_stack_ready_checks: {}",
        profile.profile.stack_ready_checks
    );
    println!(
        "profile_stack_ready_successes: {}",
        profile.profile.stack_ready_successes
    );
    println!(
        "profile_stack_eval_step_calls: {}",
        profile.profile.stack_eval_step_calls
    );
    println!(
        "profile_stack_step_reduced: {}",
        profile.profile.stack_step_reduced
    );
    println!(
        "profile_stack_step_force: {}",
        profile.profile.stack_step_force
    );
    println!(
        "profile_stack_step_whnf: {}",
        profile.profile.stack_step_whnf
    );
    println!(
        "profile_stack_step_fallback: {}",
        profile.profile.stack_step_fallback
    );
    println!(
        "profile_stack_gc_check_ms: {:.3}",
        nanos_millis(profile.profile.stack_gc_check_nanos)
    );
    println!(
        "profile_stack_resolve_ms: {:.3}",
        nanos_millis(profile.profile.stack_resolve_nanos)
    );
    println!(
        "profile_stack_ready_frame_ms: {:.3}",
        nanos_millis(profile.profile.stack_ready_frame_nanos)
    );
    println!(
        "profile_stack_descent_ms: {:.3}",
        nanos_millis(profile.profile.stack_descent_nanos)
    );
    println!(
        "profile_stack_eval_step_ms: {:.3}",
        nanos_millis(profile.profile.stack_eval_step_nanos)
    );
    println!(
        "profile_stack_whnf_finish_ms: {:.3}",
        nanos_millis(profile.profile.stack_whnf_finish_nanos)
    );
    println!(
        "profile_stack_arg_read_ms: {:.3}",
        nanos_millis(profile.profile.stack_arg_read_nanos)
    );
    println!(
        "profile_stack_app_alloc_ms: {:.3}",
        nanos_millis(profile.profile.stack_app_alloc_nanos)
    );
    println!(
        "profile_app_alloc_reused: {}",
        profile.profile.app_alloc_reused
    );
    println!(
        "profile_app_alloc_fresh: {}",
        profile.profile.app_alloc_fresh
    );
    println!(
        "profile_app_alloc_free_pop_ms: {:.3}",
        nanos_millis(profile.profile.app_alloc_free_pop_nanos)
    );
    println!(
        "profile_app_alloc_reused_write_ms: {:.3}",
        nanos_millis(profile.profile.app_alloc_reused_write_nanos)
    );
    println!(
        "profile_app_alloc_fresh_push_ms: {:.3}",
        nanos_millis(profile.profile.app_alloc_fresh_push_nanos)
    );
    println!(
        "profile_stack_apply_rewrite_ms: {:.3}",
        nanos_millis(profile.profile.stack_apply_rewrite_nanos)
    );
    println!(
        "profile_stack_apply_app_ms: {:.3}",
        nanos_millis(profile.profile.stack_apply_app_nanos)
    );
    println!(
        "profile_stack_force_frame_ms: {:.3}",
        nanos_millis(profile.profile.stack_force_frame_nanos)
    );
    println!(
        "profile_stack_inner_descent_ms: {:.3}",
        nanos_millis(profile.profile.stack_inner_descent_nanos)
    );
    println!(
        "profile_profile_step_ms: {:.3}",
        nanos_millis(profile.profile.profile_step_nanos)
    );
    println!(
        "profile_profile_reduction_ms: {:.3}",
        nanos_millis(profile.profile.profile_reduction_nanos)
    );
    println!(
        "profile_profile_stack_head_time_ms: {:.3}",
        nanos_millis(profile.profile.profile_stack_head_time_nanos)
    );
    println!(
        "profile_profile_app_alloc_bookkeeping_ms: {:.3}",
        nanos_millis(profile.profile.profile_app_alloc_bookkeeping_nanos)
    );
    println!("profile_stack_rewrite_arg_patterns:");
    for (pattern, count) in profile.profile.top_stack_rewrite_arg_patterns(top) {
        println!("  {pattern}: {count}");
    }
    println!("profile_stack_rewrite_opportunities:");
    for (opportunity, count) in profile.profile.top_stack_rewrite_opportunities(top) {
        println!("  {opportunity}: {count}");
    }
    println!("profile_stack_head_arities:");
    for (head, count) in profile.profile.top_stack_head_arities(top) {
        println!("  {head}: {count}");
    }
    println!("profile_stack_head_arity_classes:");
    for (head, count) in profile.profile.top_stack_head_arity_classes(top) {
        println!("  {head}: {count}");
    }
    println!("profile_stack_continue_next_heads:");
    for (transition, count) in profile.profile.top_stack_continue_next_heads(top) {
        println!("  {transition}: {count}");
    }
    println!("profile_stack_eval_step_head_ms:");
    for (head, nanos) in profile.profile.top_stack_eval_step_head_times(top) {
        println!("  {head}: {:.3}", nanos_millis(nanos));
    }
    println!("profile_stack_arg_read_head_ms:");
    for (head, nanos) in profile.profile.top_stack_arg_read_head_times(top) {
        println!("  {head}: {:.3}", nanos_millis(nanos));
    }
    println!("profile_stack_app_alloc_site_ms:");
    for (site, nanos) in profile.profile.top_stack_app_alloc_site_times(top) {
        println!("  {site}: {:.3}", nanos_millis(nanos));
    }
    println!("profile_stack_apply_rewrite_head_ms:");
    for (head, nanos) in profile.profile.top_stack_apply_rewrite_head_times(top) {
        println!("  {head}: {:.3}", nanos_millis(nanos));
    }
    println!("profile_stack_apply_app_head_ms:");
    for (head, nanos) in profile.profile.top_stack_apply_app_head_times(top) {
        println!("  {head}: {:.3}", nanos_millis(nanos));
    }
    println!("profile_stack_force_frame_head_ms:");
    for (head, nanos) in profile.profile.top_stack_force_frame_head_times(top) {
        println!("  {head}: {:.3}", nanos_millis(nanos));
    }
    println!("profile_stack_inner_descent_head_ms:");
    for (head, nanos) in profile.profile.top_stack_inner_descent_head_times(top) {
        println!("  {head}: {:.3}", nanos_millis(nanos));
    }
}
