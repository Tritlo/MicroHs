//! Program methods grouped by heap, reducer, host, and serialization concerns.
use super::*;

mod bfile;
mod core;
#[cfg(any(test, feature = "moving-gc"))]
mod evacuate;
mod eval;
mod gc;
mod handles;
mod profile_hooks;
mod public;
#[cfg(any(test, feature = "moving-gc"))]
mod remap;
mod runtime_dispatch;
mod serialize;
mod values;
