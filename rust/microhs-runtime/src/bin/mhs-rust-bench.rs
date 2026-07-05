use std::env;
use std::fs;
use std::hint::black_box;
use std::mem::size_of;
use std::process::Command;
use std::process::ExitCode;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use microhs_runtime::{
    EvalError, EvalProfile, GcStats, Node, NodeId, Prim, Program, parse_program,
};

include!("mhs_rust_bench/config.rs");

include!("mhs_rust_bench/scenarios.rs");

include!("mhs_rust_bench/runner.rs");

include!("mhs_rust_bench/profile_output.rs");

include!("mhs_rust_bench/runtime_helpers.rs");

include!("mhs_rust_bench/c_compare.rs");

include!("mhs_rust_bench/metrics.rs");
