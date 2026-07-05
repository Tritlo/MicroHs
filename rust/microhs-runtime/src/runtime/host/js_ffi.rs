//! Browser JavaScript FFI bridge for wasm builds.
use super::*;

#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
mod browser;
#[cfg(not(all(target_arch = "wasm32", not(target_os = "wasi"))))]
mod unsupported;

#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
pub(in crate::runtime) use self::browser::*;
#[cfg(not(all(target_arch = "wasm32", not(target_os = "wasi"))))]
pub(in crate::runtime) use self::unsupported::*;

pub(in crate::runtime) fn int_to_i32(n: i64) -> Result<i32, EvalError> {
    i32::try_from(n).map_err(|_| EvalError::Overflow)
}

pub(in crate::runtime) fn validate_js_tags(tags: &[u8]) -> Result<(), EvalError> {
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
