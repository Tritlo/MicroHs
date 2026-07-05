//! Runtime facade and internal module tree for MicroHs comb evaluation.
use std::cmp::Ordering;
#[cfg(feature = "profile")]
use std::collections::BTreeMap;
use std::collections::{HashMap, HashSet};
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

mod codecs;
mod core_types;
mod eval_state;
mod host;
mod lzma_decode;
mod ops;
mod prims;
mod profile;
mod program;
#[cfg(test)]
mod tests;

pub use self::core_types::{EvalError, JsCallNode, Node, Program};
#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
pub(crate) use self::host::JsValue;
pub(crate) use self::ops::is_runtime_prim_name;
pub use self::prims::{KnownPrim, NodeId, Prim};
#[cfg(feature = "profile")]
pub use self::profile::EvalProfile;
pub use self::profile::GcStats;

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
