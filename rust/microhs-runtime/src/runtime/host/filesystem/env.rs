//! Environment variable host operations.
use super::*;

#[cfg(all(unix, not(target_arch = "wasm32")))]
pub(in crate::runtime) fn getenv_bytes(name: &[u8]) -> Option<Vec<u8>> {
    use std::ffi::OsStr;
    use std::os::unix::ffi::{OsStrExt, OsStringExt};

    std::env::var_os(OsStr::from_bytes(name)).map(|value| value.into_vec())
}

#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
pub(in crate::runtime) fn getenv_bytes(name: &[u8]) -> Option<Vec<u8>> {
    let len = unsafe { mhs_host_getenv(name.as_ptr(), name.len()) };
    if len < 0 {
        None
    } else {
        Some(copy_host_result(usize::try_from(len).ok()?))
    }
}

#[cfg(any(target_os = "wasi", not(any(unix, target_arch = "wasm32"))))]
pub(in crate::runtime) fn getenv_bytes(name: &[u8]) -> Option<Vec<u8>> {
    let name = std::str::from_utf8(name).ok()?;
    std::env::var_os(name).map(|value| value.to_string_lossy().into_owned().into_bytes())
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
pub(in crate::runtime) fn setenv_bytes(name: &[u8], value: &[u8], overwrite: i64) -> HostIntResult {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    if name.is_empty() || name.contains(&b'=') {
        return HostIntResult::err(errno_i32("EINVAL"));
    }
    let name = OsStr::from_bytes(name);
    if overwrite == 0 && std::env::var_os(name).is_some() {
        return HostIntResult::ok(0);
    }
    // SAFETY: MicroHs executes user code on one runtime thread today; this mirrors C's process-global env.
    unsafe {
        std::env::set_var(name, OsStr::from_bytes(value));
    }
    HostIntResult::ok(0)
}

#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
pub(in crate::runtime) fn setenv_bytes(name: &[u8], value: &[u8], overwrite: i64) -> HostIntResult {
    host_result_i64(unsafe {
        mhs_host_setenv(
            name.as_ptr(),
            name.len(),
            value.as_ptr(),
            value.len(),
            overwrite,
        )
    })
}

#[cfg(any(target_os = "wasi", not(any(unix, target_arch = "wasm32"))))]
pub(in crate::runtime) fn setenv_bytes(name: &[u8], value: &[u8], overwrite: i64) -> HostIntResult {
    if name.is_empty() || name.contains(&b'=') {
        return HostIntResult::err(errno_i32("EINVAL"));
    }
    let Ok(name) = std::str::from_utf8(name) else {
        return HostIntResult::err(errno_i32("EINVAL"));
    };
    if overwrite == 0 && std::env::var_os(name).is_some() {
        return HostIntResult::ok(0);
    }
    let value = String::from_utf8_lossy(value);
    // SAFETY: MicroHs executes user code on one runtime thread today; this mirrors C's process-global env.
    unsafe {
        std::env::set_var(name, value.as_ref());
    }
    HostIntResult::ok(0)
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
pub(in crate::runtime) fn unsetenv_bytes(name: &[u8]) -> HostIntResult {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    if name.is_empty() || name.contains(&b'=') {
        return HostIntResult::err(errno_i32("EINVAL"));
    }
    // SAFETY: MicroHs executes user code on one runtime thread today; this mirrors C's process-global env.
    unsafe {
        std::env::remove_var(OsStr::from_bytes(name));
    }
    HostIntResult::ok(0)
}

#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
pub(in crate::runtime) fn unsetenv_bytes(name: &[u8]) -> HostIntResult {
    host_result_i64(unsafe { mhs_host_unsetenv(name.as_ptr(), name.len()) })
}

#[cfg(any(target_os = "wasi", not(any(unix, target_arch = "wasm32"))))]
pub(in crate::runtime) fn unsetenv_bytes(name: &[u8]) -> HostIntResult {
    if name.is_empty() || name.contains(&b'=') {
        return HostIntResult::err(errno_i32("EINVAL"));
    }
    let Ok(name) = std::str::from_utf8(name) else {
        return HostIntResult::err(errno_i32("EINVAL"));
    };
    // SAFETY: MicroHs executes user code on one runtime thread today; this mirrors C's process-global env.
    unsafe {
        std::env::remove_var(name);
    }
    HostIntResult::ok(0)
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
pub(in crate::runtime) fn environ_bytes() -> Vec<Vec<u8>> {
    use std::os::unix::ffi::OsStringExt;

    std::env::vars_os()
        .map(|(name, value)| {
            let mut bytes = name.into_vec();
            bytes.push(b'=');
            bytes.extend(value.into_vec());
            bytes
        })
        .collect()
}

#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
pub(in crate::runtime) fn environ_bytes() -> Vec<Vec<u8>> {
    let Ok(bytes) = host_bytes_result(unsafe { mhs_host_environ() }) else {
        return Vec::new();
    };
    bytes
        .split(|byte| *byte == 0)
        .filter(|entry| !entry.is_empty())
        .map(Vec::from)
        .collect()
}

#[cfg(any(target_os = "wasi", not(any(unix, target_arch = "wasm32"))))]
pub(in crate::runtime) fn environ_bytes() -> Vec<Vec<u8>> {
    std::env::vars_os()
        .map(|(name, value)| {
            let mut bytes = name.to_string_lossy().into_owned().into_bytes();
            bytes.push(b'=');
            bytes.extend(value.to_string_lossy().into_owned().into_bytes());
            bytes
        })
        .collect()
}
