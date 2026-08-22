//! Non-browser JavaScript FFI stubs.
use super::*;

pub(in crate::runtime) fn host_js_debug(bytes: &[u8]) -> Result<(), EvalError> {
    let _ = bytes;
    Err(EvalError::UnsupportedJsFfi)
}

pub(in crate::runtime) fn host_js_eval_run(
    program_handle: u32,
    bytes: &[u8],
) -> Result<(), EvalError> {
    let _ = (program_handle, bytes);
    Err(EvalError::UnsupportedJsFfi)
}

pub(in crate::runtime) fn host_js_eval_call(
    program_handle: u32,
    bytes: &[u8],
) -> Result<Vec<u8>, EvalError> {
    let _ = (program_handle, bytes);
    Err(EvalError::UnsupportedJsFfi)
}

pub(in crate::runtime) fn host_js_set_haskell_callback(callback: i32) -> Result<(), EvalError> {
    let _ = callback;
    Err(EvalError::UnsupportedJsFfi)
}

pub(in crate::runtime) fn host_js_call_void(
    program_handle: u32,
    body: &[u8],
    arity: usize,
    args: &[JsArg],
) -> Result<(), EvalError> {
    let _ = (program_handle, body, arity, args);
    Err(EvalError::UnsupportedJsFfi)
}

pub(in crate::runtime) fn host_js_call_int(
    program_handle: u32,
    body: &[u8],
    arity: usize,
    args: &[JsArg],
) -> Result<i32, EvalError> {
    let _ = (program_handle, body, arity, args);
    Err(EvalError::UnsupportedJsFfi)
}

pub(in crate::runtime) fn host_js_call_uint(
    program_handle: u32,
    body: &[u8],
    arity: usize,
    args: &[JsArg],
) -> Result<u32, EvalError> {
    let _ = (program_handle, body, arity, args);
    Err(EvalError::UnsupportedJsFfi)
}

pub(in crate::runtime) fn host_js_call_double(
    program_handle: u32,
    body: &[u8],
    arity: usize,
    args: &[JsArg],
) -> Result<f64, EvalError> {
    let _ = (program_handle, body, arity, args);
    Err(EvalError::UnsupportedJsFfi)
}

pub(in crate::runtime) fn host_js_call_ptr(
    program_handle: u32,
    body: &[u8],
    arity: usize,
    args: &[JsArg],
) -> Result<i64, EvalError> {
    let _ = (program_handle, body, arity, args);
    Err(EvalError::UnsupportedJsFfi)
}

pub(in crate::runtime) fn host_js_call_object(
    program_handle: u32,
    body: &[u8],
    arity: usize,
    args: &[JsArg],
) -> Result<u32, EvalError> {
    let _ = (program_handle, body, arity, args);
    Err(EvalError::UnsupportedJsFfi)
}

pub(in crate::runtime) fn host_js_call_bool(
    program_handle: u32,
    body: &[u8],
    arity: usize,
    args: &[JsArg],
) -> Result<bool, EvalError> {
    let _ = (program_handle, body, arity, args);
    Err(EvalError::UnsupportedJsFfi)
}

pub(in crate::runtime) fn host_js_call_string(
    program_handle: u32,
    body: &[u8],
    arity: usize,
    args: &[JsArg],
) -> Result<Vec<u8>, EvalError> {
    let _ = (program_handle, body, arity, args);
    Err(EvalError::UnsupportedJsFfi)
}

pub(in crate::runtime) fn host_js_make_wrapper(
    program_handle: u32,
    stable_ptr: i64,
    wrapper_index: u32,
) -> Result<u32, EvalError> {
    let _ = (program_handle, stable_ptr, wrapper_index);
    Err(EvalError::UnsupportedJsFfi)
}

pub(in crate::runtime) fn host_js_obj_free(handle: u32) -> Result<(), EvalError> {
    let _ = handle;
    Ok(())
}
