//! Non-browser JavaScript FFI stubs.
use super::*;

pub(in crate::runtime) fn host_js_debug(bytes: &[u8]) -> Result<(), EvalError> {
    let _ = bytes;
    Err(EvalError::UnsupportedJsFfi)
}

pub(in crate::runtime) fn host_js_eval_run(bytes: &[u8]) -> Result<(), EvalError> {
    let _ = bytes;
    Err(EvalError::UnsupportedJsFfi)
}

pub(in crate::runtime) fn host_js_eval_call(bytes: &[u8]) -> Result<Vec<u8>, EvalError> {
    let _ = bytes;
    Err(EvalError::UnsupportedJsFfi)
}

pub(in crate::runtime) fn host_js_set_haskell_callback(callback: i32) -> Result<(), EvalError> {
    let _ = callback;
    Err(EvalError::UnsupportedJsFfi)
}

pub(in crate::runtime) fn host_js_call_void(
    body: &[u8],
    arity: usize,
    args: &[JsArg],
) -> Result<(), EvalError> {
    let _ = (body, arity, args);
    Err(EvalError::UnsupportedJsFfi)
}

pub(in crate::runtime) fn host_js_call_int(
    body: &[u8],
    arity: usize,
    args: &[JsArg],
) -> Result<i32, EvalError> {
    let _ = (body, arity, args);
    Err(EvalError::UnsupportedJsFfi)
}

pub(in crate::runtime) fn host_js_call_uint(
    body: &[u8],
    arity: usize,
    args: &[JsArg],
) -> Result<u32, EvalError> {
    let _ = (body, arity, args);
    Err(EvalError::UnsupportedJsFfi)
}

pub(in crate::runtime) fn host_js_call_double(
    body: &[u8],
    arity: usize,
    args: &[JsArg],
) -> Result<f64, EvalError> {
    let _ = (body, arity, args);
    Err(EvalError::UnsupportedJsFfi)
}

pub(in crate::runtime) fn host_js_call_ptr(
    body: &[u8],
    arity: usize,
    args: &[JsArg],
) -> Result<i64, EvalError> {
    let _ = (body, arity, args);
    Err(EvalError::UnsupportedJsFfi)
}

pub(in crate::runtime) fn host_js_call_object(
    body: &[u8],
    arity: usize,
    args: &[JsArg],
) -> Result<u32, EvalError> {
    let _ = (body, arity, args);
    Err(EvalError::UnsupportedJsFfi)
}

pub(in crate::runtime) fn host_js_call_bool(
    body: &[u8],
    arity: usize,
    args: &[JsArg],
) -> Result<bool, EvalError> {
    let _ = (body, arity, args);
    Err(EvalError::UnsupportedJsFfi)
}

pub(in crate::runtime) fn host_js_call_string(
    body: &[u8],
    arity: usize,
    args: &[JsArg],
) -> Result<Vec<u8>, EvalError> {
    let _ = (body, arity, args);
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
