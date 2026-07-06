//! Host support modules for filesystem, FFI, rendering, and platform glue.
use super::*;

mod bigint;
mod ffi_arity;
mod filesystem;
pub(in crate::runtime) mod js_ffi;
mod platform;
mod render;

pub(in crate::runtime) use self::bigint::*;
pub(in crate::runtime) use self::ffi_arity::*;
pub(in crate::runtime) use self::filesystem::*;
#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
pub(crate) use self::js_ffi::JsValue;
pub(in crate::runtime) use self::js_ffi::{
    JsArg, host_js_call_bool, host_js_call_double, host_js_call_int, host_js_call_object,
    host_js_call_ptr, host_js_call_string, host_js_call_uint, host_js_call_void, host_js_debug,
    host_js_eval_call, host_js_eval_run, host_js_make_wrapper, host_js_obj_free,
    host_js_set_haskell_callback, int_to_i32, validate_js_tags,
};
pub(in crate::runtime) use self::platform::*;
pub(in crate::runtime) use self::render::*;
