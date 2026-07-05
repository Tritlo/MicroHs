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

include!("runtime/prims.rs");
include!("runtime/core_types.rs");
include!("runtime/profile.rs");
include!("runtime/eval_state.rs");
include!("runtime/program.rs");
include!("runtime/ops.rs");
include!("runtime/codecs.rs");
include!("runtime/host.rs");
include!("runtime/tests.rs");
