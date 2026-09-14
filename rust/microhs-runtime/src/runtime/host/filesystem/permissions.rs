//! Host file-permission operations.
use super::*;

#[cfg(all(unix, not(target_arch = "wasm32")))]
pub(in crate::runtime) fn get_permissions_path_bytes(path: &[u8]) -> HostIntResult {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    use std::os::unix::fs::PermissionsExt;

    let path = std::path::Path::new(OsStr::from_bytes(path));
    let metadata = match std::fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(err) => return HostIntResult::os_err(io_error_errno(&err)),
    };
    let mode = metadata.permissions().mode();
    let mut permissions = 0;
    if mode & 0o400 != 0 {
        permissions |= 4;
    }
    if mode & 0o200 != 0 {
        permissions |= 2;
    }
    if mode & 0o100 != 0 {
        permissions |= if metadata.is_dir() { 8 } else { 1 };
    }
    HostIntResult::ok(permissions)
}

#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
pub(in crate::runtime) fn get_permissions_path_bytes(path: &[u8]) -> HostIntResult {
    host_result_i64(unsafe { mhs_host_get_permissions(path.as_ptr(), path.len()) })
}

#[cfg(any(target_os = "wasi", not(any(unix, target_arch = "wasm32"))))]
pub(in crate::runtime) fn get_permissions_path_bytes(path: &[u8]) -> HostIntResult {
    let Ok(path) = std::str::from_utf8(path) else {
        return HostIntResult::err(errno_i32("EINVAL"));
    };
    let metadata = match std::fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(err) => return HostIntResult::os_err(io_error_errno(&err)),
    };
    let mut permissions = 4;
    if !metadata.permissions().readonly() {
        permissions |= 2;
    }
    if metadata.is_dir() {
        permissions |= 8;
    }
    HostIntResult::ok(permissions)
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
pub(in crate::runtime) fn set_permissions_path_bytes(
    path: &[u8],
    permissions: i64,
) -> HostIntResult {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    use std::os::unix::fs::PermissionsExt;

    unsafe extern "C" {
        fn umask(mask: u32) -> u32;
    }

    let Ok(permissions) = u32::try_from(permissions) else {
        return HostIntResult::err(errno_i32("EINVAL"));
    };
    let path = std::path::Path::new(OsStr::from_bytes(path));
    let file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(err) => return HostIntResult::os_err(io_error_errno(&err)),
    };
    let metadata = match file.metadata() {
        Ok(metadata) => metadata,
        Err(err) => return HostIntResult::os_err(io_error_errno(&err)),
    };
    let mut user_mode = 0;
    if permissions & 4 != 0 {
        user_mode |= 0o400;
    }
    if permissions & 2 != 0 {
        user_mode |= 0o200;
    }
    if permissions & 1 != 0 || permissions & 8 != 0 {
        user_mode |= 0o100;
    }
    let mut mode = user_mode | (user_mode >> 3) | (user_mode >> 6);
    // SAFETY: umask is process-global like in the C runtime. We restore it immediately.
    let mask = unsafe { umask(0) };
    // SAFETY: restores the mask value just read above.
    unsafe {
        umask(mask);
    }
    mode &= !mask;
    mode |= metadata.permissions().mode() & !0o777;
    match file.set_permissions(std::fs::Permissions::from_mode(mode)) {
        Ok(()) => HostIntResult::ok(0),
        Err(err) => HostIntResult::os_err(io_error_errno(&err)),
    }
}

#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
pub(in crate::runtime) fn set_permissions_path_bytes(
    path: &[u8],
    permissions: i64,
) -> HostIntResult {
    host_result_i64(unsafe { mhs_host_set_permissions(path.as_ptr(), path.len(), permissions) })
}

#[cfg(any(target_os = "wasi", not(any(unix, target_arch = "wasm32"))))]
pub(in crate::runtime) fn set_permissions_path_bytes(
    path: &[u8],
    permissions: i64,
) -> HostIntResult {
    let _ = permissions;
    let Ok(path) = std::str::from_utf8(path) else {
        return HostIntResult::err(errno_i32("EINVAL"));
    };
    let metadata = match std::fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(err) => return HostIntResult::os_err(io_error_errno(&err)),
    };
    let mut permissions = metadata.permissions();
    permissions.set_readonly(false);
    match std::fs::set_permissions(path, permissions) {
        Ok(()) => HostIntResult::ok(0),
        Err(err) => HostIntResult::os_err(io_error_errno(&err)),
    }
}
