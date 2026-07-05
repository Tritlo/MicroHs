fn int_to_i32(n: i64) -> Result<i32, EvalError> {
    i32::try_from(n).map_err(|_| EvalError::Overflow)
}

fn validate_js_tags(tags: &[u8]) -> Result<(), EvalError> {
    if tags.is_empty() {
        return Err(EvalError::InvalidByteString);
    }
    for (idx, tag) in tags.iter().copied().enumerate() {
        let ok = matches!(tag, b'I' | b'U' | b'D' | b'F' | b'P' | b'B' | b'J' | b'S')
            || (idx == 0 && tag == b'V');
        if !ok {
            return Err(EvalError::InvalidByteString);
        }
    }
    Ok(())
}

fn host_js_debug(bytes: &[u8]) -> Result<(), EvalError> {
    #[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
    {
        let bytes = nul_terminated(bytes)?;
        unsafe {
            mhs_js_debug(bytes.as_ptr());
        }
        Ok(())
    }
    #[cfg(not(all(target_arch = "wasm32", not(target_os = "wasi"))))]
    {
        let _ = bytes;
        Err(EvalError::UnsupportedJsFfi)
    }
}

fn host_js_eval_run(bytes: &[u8]) -> Result<(), EvalError> {
    #[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
    {
        let bytes = nul_terminated(bytes)?;
        unsafe {
            mhs_js_eval_run(bytes.as_ptr());
        }
        Ok(())
    }
    #[cfg(not(all(target_arch = "wasm32", not(target_os = "wasi"))))]
    {
        let _ = bytes;
        Err(EvalError::UnsupportedJsFfi)
    }
}

fn host_js_eval_call(bytes: &[u8]) -> Result<Vec<u8>, EvalError> {
    #[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
    {
        let bytes = nul_terminated(bytes)?;
        unsafe {
            let ptr = mhs_js_eval_call(bytes.as_ptr());
            copy_host_c_string(ptr)
        }
    }
    #[cfg(not(all(target_arch = "wasm32", not(target_os = "wasi"))))]
    {
        let _ = bytes;
        Err(EvalError::UnsupportedJsFfi)
    }
}

fn host_js_set_haskell_callback(callback: i32) -> Result<(), EvalError> {
    #[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
    {
        unsafe {
            mhs_js_set_haskellCallback(callback);
        }
        Ok(())
    }
    #[cfg(not(all(target_arch = "wasm32", not(target_os = "wasi"))))]
    {
        let _ = callback;
        Err(EvalError::UnsupportedJsFfi)
    }
}

fn host_js_call_void(body: &[u8], arity: usize, args: &[JsArg]) -> Result<(), EvalError> {
    #[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
    {
        let idx = host_js_prepare_call(body, arity, args)?;
        unsafe {
            mhs_js_call_void(idx);
        }
        host_js_check_error()
    }
    #[cfg(not(all(target_arch = "wasm32", not(target_os = "wasi"))))]
    {
        let _ = (body, arity, args);
        Err(EvalError::UnsupportedJsFfi)
    }
}

fn host_js_call_int(body: &[u8], arity: usize, args: &[JsArg]) -> Result<i32, EvalError> {
    #[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
    {
        let idx = host_js_prepare_call(body, arity, args)?;
        let result = unsafe { mhs_js_call_int(idx) };
        host_js_check_error()?;
        Ok(result)
    }
    #[cfg(not(all(target_arch = "wasm32", not(target_os = "wasi"))))]
    {
        let _ = (body, arity, args);
        Err(EvalError::UnsupportedJsFfi)
    }
}

fn host_js_call_uint(body: &[u8], arity: usize, args: &[JsArg]) -> Result<u32, EvalError> {
    #[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
    {
        let idx = host_js_prepare_call(body, arity, args)?;
        let result = unsafe { mhs_js_call_uint(idx) };
        host_js_check_error()?;
        Ok(result)
    }
    #[cfg(not(all(target_arch = "wasm32", not(target_os = "wasi"))))]
    {
        let _ = (body, arity, args);
        Err(EvalError::UnsupportedJsFfi)
    }
}

fn host_js_call_double(body: &[u8], arity: usize, args: &[JsArg]) -> Result<f64, EvalError> {
    #[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
    {
        let idx = host_js_prepare_call(body, arity, args)?;
        let result = unsafe { mhs_js_call_dbl(idx) };
        host_js_check_error()?;
        Ok(result)
    }
    #[cfg(not(all(target_arch = "wasm32", not(target_os = "wasi"))))]
    {
        let _ = (body, arity, args);
        Err(EvalError::UnsupportedJsFfi)
    }
}

fn host_js_call_ptr(body: &[u8], arity: usize, args: &[JsArg]) -> Result<u32, EvalError> {
    #[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
    {
        let idx = host_js_prepare_call(body, arity, args)?;
        let result = unsafe { mhs_js_call_ptr(idx) };
        host_js_check_error()?;
        Ok(result)
    }
    #[cfg(not(all(target_arch = "wasm32", not(target_os = "wasi"))))]
    {
        let _ = (body, arity, args);
        Err(EvalError::UnsupportedJsFfi)
    }
}

fn host_js_call_object(body: &[u8], arity: usize, args: &[JsArg]) -> Result<u32, EvalError> {
    #[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
    {
        let idx = host_js_prepare_call(body, arity, args)?;
        let result = unsafe { mhs_js_call_obj(idx) };
        host_js_check_error()?;
        Ok(result)
    }
    #[cfg(not(all(target_arch = "wasm32", not(target_os = "wasi"))))]
    {
        let _ = (body, arity, args);
        Err(EvalError::UnsupportedJsFfi)
    }
}

fn host_js_call_bool(body: &[u8], arity: usize, args: &[JsArg]) -> Result<bool, EvalError> {
    #[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
    {
        let idx = host_js_prepare_call(body, arity, args)?;
        let result = unsafe { mhs_js_call_bool(idx) != 0 };
        host_js_check_error()?;
        Ok(result)
    }
    #[cfg(not(all(target_arch = "wasm32", not(target_os = "wasi"))))]
    {
        let _ = (body, arity, args);
        Err(EvalError::UnsupportedJsFfi)
    }
}

fn host_js_call_string(body: &[u8], arity: usize, args: &[JsArg]) -> Result<Vec<u8>, EvalError> {
    #[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
    {
        let idx = host_js_prepare_call(body, arity, args)?;
        unsafe {
            let ptr = mhs_js_call_str(idx);
            let len = usize::try_from(mhs_js_slen()).map_err(|_| EvalError::Overflow)?;
            let result = copy_host_bytes(ptr, len)?;
            host_js_check_error()?;
            Ok(result)
        }
    }
    #[cfg(not(all(target_arch = "wasm32", not(target_os = "wasi"))))]
    {
        let _ = (body, arity, args);
        Err(EvalError::UnsupportedJsFfi)
    }
}

fn host_js_make_wrapper(
    program_handle: u32,
    stable_ptr: i64,
    wrapper_index: u32,
) -> Result<u32, EvalError> {
    #[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
    {
        let stable_ptr = u32::try_from(stable_ptr).map_err(|_| EvalError::Overflow)?;
        let result = unsafe { mhs_js_make_wrapper(program_handle, stable_ptr, wrapper_index) };
        host_js_check_error()?;
        Ok(result)
    }
    #[cfg(not(all(target_arch = "wasm32", not(target_os = "wasi"))))]
    {
        let _ = (program_handle, stable_ptr, wrapper_index);
        Err(EvalError::UnsupportedJsFfi)
    }
}

#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
fn host_js_prepare_call(body: &[u8], arity: usize, args: &[JsArg]) -> Result<i32, EvalError> {
    let body = nul_terminated(body)?;
    let arity = i32::try_from(arity).map_err(|_| EvalError::Overflow)?;
    unsafe {
        mhs_js_setup();
        let idx = mhs_js_register(body.as_ptr(), arity);
        mhs_js_argreset();
        for arg in args {
            match arg {
                JsArg::Int(value) => mhs_js_push_int(*value),
                JsArg::UInt(value) => mhs_js_push_uint(*value),
                JsArg::Double(value) => mhs_js_push_dbl(*value),
                JsArg::Object(value) => mhs_js_push_obj(*value),
                JsArg::String(bytes) => {
                    let len = i32::try_from(bytes.len()).map_err(|_| EvalError::Overflow)?;
                    mhs_js_push_str(bytes.as_ptr(), len);
                }
            }
        }
        Ok(idx)
    }
}

#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
fn host_js_check_error() -> Result<(), EvalError> {
    unsafe {
        if mhs_js_haserr() != 0 {
            mhs_js_logerr();
            return Err(EvalError::UnsupportedJsFfi);
        }
    }
    Ok(())
}

#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
fn nul_terminated(bytes: &[u8]) -> Result<Vec<u8>, EvalError> {
    if bytes.contains(&0) {
        return Err(EvalError::InvalidByteString);
    }
    let mut out = Vec::with_capacity(bytes.len() + 1);
    out.extend_from_slice(bytes);
    out.push(0);
    Ok(out)
}

#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
unsafe fn copy_host_c_string(ptr: *const std::os::raw::c_char) -> Result<Vec<u8>, EvalError> {
    if ptr.is_null() {
        return Ok(Vec::new());
    }
    Ok(unsafe { std::ffi::CStr::from_ptr(ptr) }.to_bytes().to_vec())
}

#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
unsafe fn copy_host_bytes(
    ptr: *const std::os::raw::c_char,
    len: usize,
) -> Result<Vec<u8>, EvalError> {
    if ptr.is_null() {
        return if len == 0 {
            Ok(Vec::new())
        } else {
            Err(EvalError::InvalidByteString)
        };
    }
    Ok(unsafe { std::slice::from_raw_parts(ptr.cast::<u8>(), len) }.to_vec())
}

#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
unsafe extern "C" {
    fn mhs_js_debug(ptr: *const u8);
    fn mhs_js_eval_run(ptr: *const u8);
    fn mhs_js_eval_call(ptr: *const u8) -> *const std::os::raw::c_char;
    fn mhs_js_set_haskellCallback(callback: i32);
    fn mhs_js_setup();
    fn mhs_js_register(body: *const u8, arity: i32) -> i32;
    fn mhs_js_argreset();
    fn mhs_js_push_int(value: i32);
    fn mhs_js_push_uint(value: u32);
    fn mhs_js_push_dbl(value: f64);
    fn mhs_js_push_obj(handle: u32);
    fn mhs_js_push_str(ptr: *const u8, len: i32);
    fn mhs_js_call_int(idx: i32) -> i32;
    fn mhs_js_call_uint(idx: i32) -> u32;
    fn mhs_js_call_dbl(idx: i32) -> f64;
    fn mhs_js_call_ptr(idx: i32) -> u32;
    fn mhs_js_call_obj(idx: i32) -> u32;
    fn mhs_js_call_bool(idx: i32) -> i32;
    fn mhs_js_call_str(idx: i32) -> *const std::os::raw::c_char;
    fn mhs_js_call_void(idx: i32);
    fn mhs_js_make_wrapper(program_handle: u32, stable_ptr: u32, wrapper_index: u32) -> u32;
    fn mhs_js_slen() -> i32;
    fn mhs_js_haserr() -> i32;
    fn mhs_js_logerr();
}
