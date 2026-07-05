struct ParseBench {
    elapsed: Duration,
}

fn bench_parse(input: &[u8], warmup_iters: usize, iters: usize) -> ParseBench {
    for _ in 0..warmup_iters {
        black_box(parse_once(input));
    }

    let started = Instant::now();
    for _ in 0..iters {
        black_box(parse_once(input));
    }
    ParseBench {
        elapsed: started.elapsed(),
    }
}

fn parse_once(input: &[u8]) -> usize {
    let program = parse_program(black_box(input)).expect("parse benchmark input");
    program.node_count()
}

struct EvalBench {
    elapsed: Duration,
    steps: usize,
    serialize_sink: usize,
    step_limited_iters: usize,
    gc: GcStats,
}

struct ProfileBench {
    elapsed: Duration,
    steps: usize,
    serialize_sink: usize,
    step_limited: bool,
    nodes_before: usize,
    nodes_after: usize,
    gc: GcStats,
    profile: EvalProfile,
}

fn bench_eval(
    input: &[u8],
    mode: BenchMode,
    program_args: &[Vec<u8>],
    executable_path: Option<&[u8]>,
    warmup_iters: usize,
    iters: usize,
    step_limit: Option<usize>,
) -> EvalBench {
    for _ in 0..warmup_iters {
        black_box(eval_once(
            input,
            mode,
            program_args,
            executable_path,
            step_limit,
        ));
    }

    let started = Instant::now();
    let mut steps = 0;
    let mut serialize_sink = 0usize;
    let mut step_limited_iters = 0usize;
    let mut gc = GcStats::default();
    for _ in 0..iters {
        let run = eval_once(input, mode, program_args, executable_path, step_limit);
        steps += run.steps;
        if run.step_limited {
            step_limited_iters += 1;
        }
        let sink = run.serialize_sink;
        serialize_sink = serialize_sink.wrapping_add(sink);
        gc.collections = gc.collections.saturating_add(run.gc.collections);
        gc.freed_nodes_total = gc
            .freed_nodes_total
            .saturating_add(run.gc.freed_nodes_total);
        gc.last_live_nodes = run.gc.last_live_nodes;
        gc.last_free_nodes = run.gc.last_free_nodes;
        gc.high_water_nodes = gc.high_water_nodes.max(run.gc.high_water_nodes);
        gc.current_nodes = run.gc.current_nodes;
        gc.current_free_nodes = run.gc.current_free_nodes;
        gc.last_pause_nanos = run.gc.last_pause_nanos;
        gc.total_pause_nanos = gc
            .total_pause_nanos
            .saturating_add(run.gc.total_pause_nanos);
        #[cfg(feature = "gc-phase-profile")]
        {
            gc.last_mark_nanos = run.gc.last_mark_nanos;
            gc.total_mark_nanos = gc.total_mark_nanos.saturating_add(run.gc.total_mark_nanos);
            gc.last_sweep_nanos = run.gc.last_sweep_nanos;
            gc.total_sweep_nanos = gc
                .total_sweep_nanos
                .saturating_add(run.gc.total_sweep_nanos);
            gc.young_profile_last_slots = run.gc.young_profile_last_slots;
            gc.young_profile_last_live = run.gc.young_profile_last_live;
            gc.young_profile_last_dead = run.gc.young_profile_last_dead;
            gc.young_profile_last_old_to_young_sources =
                run.gc.young_profile_last_old_to_young_sources;
            gc.young_profile_last_old_to_young_edges = run.gc.young_profile_last_old_to_young_edges;
            gc.young_profile_total_slots = gc
                .young_profile_total_slots
                .saturating_add(run.gc.young_profile_total_slots);
            gc.young_profile_total_live = gc
                .young_profile_total_live
                .saturating_add(run.gc.young_profile_total_live);
            gc.young_profile_total_dead = gc
                .young_profile_total_dead
                .saturating_add(run.gc.young_profile_total_dead);
            gc.young_profile_total_old_to_young_sources = gc
                .young_profile_total_old_to_young_sources
                .saturating_add(run.gc.young_profile_total_old_to_young_sources);
            gc.young_profile_total_old_to_young_edges = gc
                .young_profile_total_old_to_young_edges
                .saturating_add(run.gc.young_profile_total_old_to_young_edges);
        }
        gc.last_allocations_since_collect = run.gc.last_allocations_since_collect;
        gc.current_allocations_since_collect = run.gc.current_allocations_since_collect;
        gc.events.extend(run.gc.events);
    }
    black_box(serialize_sink);
    EvalBench {
        elapsed: started.elapsed(),
        steps,
        serialize_sink,
        step_limited_iters,
        gc,
    }
}

struct RunOnce {
    steps: usize,
    serialize_sink: usize,
    step_limited: bool,
    gc: GcStats,
}

fn eval_once(
    input: &[u8],
    mode: BenchMode,
    program_args: &[Vec<u8>],
    executable_path: Option<&[u8]>,
    step_limit: Option<usize>,
) -> RunOnce {
    let mut program = parse_program(black_box(input)).expect("reduce benchmark input");
    program.set_program_args(program_args.to_vec());
    program.set_executable_path(executable_path.map(Vec::from));
    let limit = step_limit.unwrap_or(usize::MAX);
    let mut run = match mode {
        BenchMode::Whnf => {
            let reductions = program.reduction_count();
            match program.reduce_whnf(limit) {
                Ok((root, steps)) => {
                    let serialized = program
                        .serialize_program(root)
                        .expect("serialize benchmark result");
                    let sink = bytes_sink(&serialized);
                    black_box(&serialized);
                    RunOnce {
                        steps,
                        serialize_sink: sink,
                        step_limited: false,
                        gc: GcStats::default(),
                    }
                }
                Err(EvalError::StepLimit { .. }) => RunOnce {
                    steps: program.reduction_count().saturating_sub(reductions),
                    serialize_sink: main_input_sink(input),
                    step_limited: true,
                    gc: GcStats::default(),
                },
                Err(err) => panic!("reduce benchmark input: {err}"),
            }
        }
        BenchMode::Main => reduce_main_or_panic(&mut program, limit, "run benchmark main", input),
    };
    run.gc = program.gc_stats();
    run
}

fn profile_eval(
    input: &[u8],
    mode: BenchMode,
    program_args: &[Vec<u8>],
    executable_path: Option<&[u8]>,
    step_limit: Option<usize>,
) -> ProfileBench {
    let started = Instant::now();
    let mut program = parse_program(black_box(input)).expect("profile benchmark input");
    program.set_program_args(program_args.to_vec());
    program.set_executable_path(executable_path.map(Vec::from));
    let nodes_before = program.node_count();
    program.enable_profile();
    let limit = step_limit.unwrap_or(usize::MAX);
    let run = match mode {
        BenchMode::Whnf => {
            let reductions = program.reduction_count();
            match program.reduce_whnf(limit) {
                Ok((root, steps)) => {
                    let serialized = program
                        .serialize_program(root)
                        .expect("profile serialize benchmark result");
                    let sink = bytes_sink(&serialized);
                    black_box(&serialized);
                    RunOnce {
                        steps,
                        serialize_sink: sink,
                        step_limited: false,
                        gc: GcStats::default(),
                    }
                }
                Err(EvalError::StepLimit { .. }) => RunOnce {
                    steps: program.reduction_count().saturating_sub(reductions),
                    serialize_sink: main_input_sink(input),
                    step_limited: true,
                    gc: GcStats::default(),
                },
                Err(err) => panic!("profile reduce benchmark input: {err}"),
            }
        }
        BenchMode::Main => {
            reduce_main_or_panic(&mut program, limit, "profile run benchmark main", input)
        }
    };
    let elapsed = started.elapsed();
    let nodes_after = program.node_count();
    let gc = program.gc_stats();
    let profile = program.take_profile().expect("profile enabled");
    ProfileBench {
        elapsed,
        steps: run.steps,
        serialize_sink: run.serialize_sink,
        step_limited: run.step_limited,
        nodes_before,
        nodes_after,
        gc,
        profile,
    }
}
