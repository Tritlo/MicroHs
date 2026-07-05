use std::env;
use std::fs;
use std::hint::black_box;
use std::mem::size_of;
use std::process::Command;
use std::process::ExitCode;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[cfg(feature = "profile")]
use microhs_runtime::EvalProfile;
use microhs_runtime::{
    EvalError, GcStats, Node, NodeId, Prim, Program, cell_size_bytes, parse_program,
};

#[path = "mhs_rust_bench/c_compare.rs"]
mod c_compare;

#[path = "mhs_rust_bench/config.rs"]
mod config;

#[path = "mhs_rust_bench/metrics.rs"]
mod metrics;

#[path = "mhs_rust_bench/profile_output.rs"]
#[cfg(feature = "profile")]
mod profile_output;

#[path = "mhs_rust_bench/runner.rs"]
mod runner;

#[path = "mhs_rust_bench/runtime_helpers.rs"]
mod runtime_helpers;

#[path = "mhs_rust_bench/scenarios.rs"]
mod scenarios;

fn main() -> ExitCode {
    config::main()
}
