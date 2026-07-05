fn millis(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1_000.0
}

fn nanos_millis(nanos: u128) -> f64 {
    nanos as f64 / 1_000_000.0
}

fn print_gc_events(label: &str, gc: &GcStats) {
    println!("{label}:");
    for event in &gc.events {
        #[cfg(feature = "gc-phase-profile")]
        println!(
            "  collection={} pause_ms={:.3} mark_ms={:.3} sweep_ms={:.3} live_nodes={} free_nodes={} arena_nodes={} freed_nodes={} allocations_since_collect={} young_slots={} young_live={} young_dead={} old_to_young_sources={} old_to_young_edges={}",
            event.collection,
            nanos_millis(event.pause_nanos),
            nanos_millis(event.mark_nanos),
            nanos_millis(event.sweep_nanos),
            event.live_nodes,
            event.free_nodes,
            event.arena_nodes,
            event.freed_nodes,
            event.allocations_since_collect,
            event.young_profile_slots,
            event.young_profile_live,
            event.young_profile_dead,
            event.young_profile_old_to_young_sources,
            event.young_profile_old_to_young_edges
        );
        #[cfg(not(feature = "gc-phase-profile"))]
        println!(
            "  collection={} pause_ms={:.3} live_nodes={} free_nodes={} arena_nodes={} freed_nodes={} allocations_since_collect={}",
            event.collection,
            nanos_millis(event.pause_nanos),
            event.live_nodes,
            event.free_nodes,
            event.arena_nodes,
            event.freed_nodes,
            event.allocations_since_collect
        );
    }
}

fn nanos_per_iter(duration: Duration, iters: usize) -> f64 {
    duration.as_secs_f64() * 1_000_000_000.0 / iters as f64
}

fn mib_per_s(bytes: usize, iters: usize, duration: Duration) -> f64 {
    let mib = (bytes as f64 * iters as f64) / (1024.0 * 1024.0);
    mib / duration.as_secs_f64()
}
