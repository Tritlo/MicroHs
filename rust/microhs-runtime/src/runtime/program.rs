//! Program methods grouped by heap, reducer, host, and serialization concerns.
use super::*;

mod bfile;
mod core;
mod eval;
mod gc;
mod handles;
mod profile_hooks;
mod public;
mod runtime_dispatch;
mod serialize;
mod values;
