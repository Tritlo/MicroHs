//! Runtime facade and internal module tree for MicroHs comb evaluation.
use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::fmt;
use std::mem::{MaybeUninit, size_of};
#[cfg(any(not(target_arch = "wasm32"), target_os = "wasi"))]
use std::time::Instant;

#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
#[derive(Clone, Copy)]
struct Instant;

#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
impl Instant {
    fn now() -> Self {
        Self
    }

    fn elapsed(self) -> BrowserDuration {
        BrowserDuration
    }
}

#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
struct BrowserDuration;

#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
impl BrowserDuration {
    fn as_nanos(&self) -> u128 {
        0
    }
}

macro_rules! trace_invalid_bytes {
    ($program:expr, $($arg:tt)*) => {{
        if std::env::var_os("MHS_TRACE_INVALID_BYTES").is_some() {
            eprintln!("invalid bytes: reductions={}", $program.reductions);
            eprintln!($($arg)*);
        }
        EvalError::InvalidByteString
    }};
}

mod codecs;
mod core_types;
mod eval_state;
mod host;
mod ops;
mod prims;
mod profile;
mod program;
#[cfg(test)]
mod tests;

pub use self::core_types::{EvalError, JsCallNode, JsValue, Node, Program};
pub(crate) use self::ops::is_runtime_prim_name;
pub use self::prims::{KnownPrim, NodeId, Prim};
pub use self::profile::{EvalProfile, GcStats};

pub fn cell_size_bytes() -> usize {
    size_of::<Cell>()
}

pub(in crate::runtime) use self::codecs::*;
pub(in crate::runtime) use self::core_types::*;
pub(in crate::runtime) use self::eval_state::*;
pub(in crate::runtime) use self::host::*;
pub(in crate::runtime) use self::ops::*;
pub(in crate::runtime) use self::prims::*;
pub(in crate::runtime) use self::profile::*;
