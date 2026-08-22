pub mod parse;
pub mod runtime;
#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
mod wasm;

pub use parse::{ParseError, parse_program};
#[cfg(feature = "profile")]
pub use runtime::EvalProfile;
pub use runtime::{EvalError, GcStats, KnownPrim, Node, NodeId, Prim, Program, cell_size_bytes};
