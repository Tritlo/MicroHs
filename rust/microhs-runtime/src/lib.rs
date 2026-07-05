mod lzma_decode;
pub mod parse;
pub mod runtime;
#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
mod wasm;

pub use parse::{ParseError, parse_program};
pub use runtime::{
    EvalError, EvalProfile, GcStats, JsValue, KnownPrim, Node, NodeId, Prim, Program,
    cell_size_bytes,
};
