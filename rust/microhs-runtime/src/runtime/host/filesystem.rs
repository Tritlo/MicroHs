use super::*;

#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
unsafe extern "C" {
    pub(in crate::runtime) fn mhs_host_result_copy(dst: *mut u8, len: usize) -> usize;
    pub(in crate::runtime) fn mhs_host_getenv(name_ptr: *const u8, name_len: usize) -> isize;
    pub(in crate::runtime) fn mhs_host_setenv(
        name_ptr: *const u8,
        name_len: usize,
        value_ptr: *const u8,
        value_len: usize,
        overwrite: i64,
    ) -> i64;
    pub(in crate::runtime) fn mhs_host_unsetenv(name_ptr: *const u8, name_len: usize) -> i64;
    pub(in crate::runtime) fn mhs_host_environ() -> isize;
    pub(in crate::runtime) fn mhs_host_remove(path_ptr: *const u8, path_len: usize) -> i64;
    pub(in crate::runtime) fn mhs_host_chdir(path_ptr: *const u8, path_len: usize) -> i64;
    pub(in crate::runtime) fn mhs_host_mkdir(
        path_ptr: *const u8,
        path_len: usize,
        mode: i64,
    ) -> i64;
    pub(in crate::runtime) fn mhs_host_getcwd() -> isize;
    pub(in crate::runtime) fn mhs_host_tmpname(
        pre_ptr: *const u8,
        pre_len: usize,
        suf_ptr: *const u8,
        suf_len: usize,
    ) -> isize;
    pub(in crate::runtime) fn mhs_host_get_permissions(path_ptr: *const u8, path_len: usize)
    -> i64;
    pub(in crate::runtime) fn mhs_host_set_permissions(
        path_ptr: *const u8,
        path_len: usize,
        permissions: i64,
    ) -> i64;
    pub(in crate::runtime) fn mhs_host_dir_entries(path_ptr: *const u8, path_len: usize) -> isize;
    pub(in crate::runtime) fn mhs_host_file_open(
        path_ptr: *const u8,
        path_len: usize,
        mode_ptr: *const u8,
        mode_len: usize,
    ) -> i64;
    pub(in crate::runtime) fn mhs_host_file_read(handle: i64, dst: *mut u8, len: usize) -> isize;
    pub(in crate::runtime) fn mhs_host_file_write(handle: i64, src: *const u8, len: usize)
    -> isize;
    pub(in crate::runtime) fn mhs_host_file_flush(handle: i64) -> i64;
    pub(in crate::runtime) fn mhs_host_file_close(handle: i64) -> i64;
}

#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
pub(in crate::runtime) fn host_result_i64(rc: i64) -> HostIntResult {
    if rc < 0 {
        HostIntResult::err((-rc) as i32)
    } else {
        HostIntResult::ok(rc)
    }
}

#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
pub(in crate::runtime) fn host_result_usize(rc: isize) -> Result<usize, i32> {
    if rc < 0 {
        Err((-rc) as i32)
    } else {
        usize::try_from(rc).map_err(|_| errno_i32("EOVERFLOW"))
    }
}

#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
pub(in crate::runtime) fn copy_host_result(len: usize) -> Vec<u8> {
    let mut bytes = vec![0; len];
    if len != 0 {
        let copied = unsafe { mhs_host_result_copy(bytes.as_mut_ptr(), len) };
        bytes.truncate(copied.min(len));
    }
    bytes
}

#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
pub(in crate::runtime) fn host_bytes_result(rc: isize) -> Result<Vec<u8>, i32> {
    let len = host_result_usize(rc)?;
    Ok(copy_host_result(len))
}

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
    #[cfg(target_os = "wasi")]
    wasi_trace_host("getenv", &String::from_utf8_lossy(name));
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
    #[cfg(target_os = "wasi")]
    wasi_trace_host(
        "setenv",
        &format!(
            "name={} value_len={} overwrite={overwrite}",
            String::from_utf8_lossy(name),
            value.len()
        ),
    );
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
    #[cfg(target_os = "wasi")]
    wasi_trace_host("unsetenv", &String::from_utf8_lossy(name));
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

#[cfg(all(unix, not(target_arch = "wasm32")))]
pub(in crate::runtime) fn remove_path_bytes(path: &[u8]) -> HostIntResult {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    let path = std::path::Path::new(OsStr::from_bytes(path));
    match std::fs::remove_file(path) {
        Ok(()) => HostIntResult::ok(0),
        Err(file_err) => match std::fs::remove_dir(path) {
            Ok(()) => HostIntResult::ok(0),
            Err(dir_err) => HostIntResult::os_err(
                io_error_errno(&dir_err).or_else(|| io_error_errno(&file_err)),
            ),
        },
    }
}

#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
pub(in crate::runtime) fn remove_path_bytes(path: &[u8]) -> HostIntResult {
    host_result_i64(unsafe { mhs_host_remove(path.as_ptr(), path.len()) })
}

#[cfg(any(target_os = "wasi", not(any(unix, target_arch = "wasm32"))))]
pub(in crate::runtime) fn remove_path_bytes(path: &[u8]) -> HostIntResult {
    #[cfg(target_os = "wasi")]
    wasi_trace_host("remove", &String::from_utf8_lossy(path));
    let Ok(path) = std::str::from_utf8(path) else {
        return HostIntResult::err(errno_i32("EINVAL"));
    };
    match std::fs::remove_file(path) {
        Ok(()) => HostIntResult::ok(0),
        Err(file_err) => match std::fs::remove_dir(path) {
            Ok(()) => HostIntResult::ok(0),
            Err(dir_err) => HostIntResult::os_err(
                io_error_errno(&dir_err).or_else(|| io_error_errno(&file_err)),
            ),
        },
    }
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
pub(in crate::runtime) fn system_command_bytes(command: Option<&[u8]>) -> i64 {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    use std::os::unix::process::ExitStatusExt;

    let Some(command) = command else {
        return 1;
    };
    match std::process::Command::new("/bin/sh")
        .arg("-c")
        .arg(OsStr::from_bytes(command))
        .status()
    {
        Ok(status) => i64::from(status.into_raw()),
        Err(_) => -1,
    }
}

#[cfg(target_arch = "wasm32")]
pub(in crate::runtime) fn system_command_bytes(command: Option<&[u8]>) -> i64 {
    let _ = command;
    -1
}

#[cfg(not(any(unix, target_arch = "wasm32")))]
pub(in crate::runtime) fn system_command_bytes(command: Option<&[u8]>) -> i64 {
    let Some(command) = command else {
        return 1;
    };
    let Ok(command) = std::str::from_utf8(command) else {
        return -1;
    };
    match std::process::Command::new("cmd")
        .arg("/C")
        .arg(command)
        .status()
    {
        Ok(status) => status.code().map_or(-1, i64::from),
        Err(_) => -1,
    }
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
pub(in crate::runtime) fn chdir_path_bytes(path: &[u8]) -> HostIntResult {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    let path = std::path::Path::new(OsStr::from_bytes(path));
    match std::env::set_current_dir(path) {
        Ok(()) => HostIntResult::ok(0),
        Err(err) => HostIntResult::os_err(io_error_errno(&err)),
    }
}

#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
pub(in crate::runtime) fn chdir_path_bytes(path: &[u8]) -> HostIntResult {
    host_result_i64(unsafe { mhs_host_chdir(path.as_ptr(), path.len()) })
}

#[cfg(any(target_os = "wasi", not(any(unix, target_arch = "wasm32"))))]
pub(in crate::runtime) fn chdir_path_bytes(path: &[u8]) -> HostIntResult {
    #[cfg(target_os = "wasi")]
    wasi_trace_host("chdir", &String::from_utf8_lossy(path));
    let Ok(path) = std::str::from_utf8(path) else {
        return HostIntResult::err(errno_i32("EINVAL"));
    };
    match std::env::set_current_dir(path) {
        Ok(()) => HostIntResult::ok(0),
        Err(err) => HostIntResult::os_err(io_error_errno(&err)),
    }
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
pub(in crate::runtime) fn mkdir_path_bytes(path: &[u8], mode: i64) -> HostIntResult {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    use std::os::unix::fs::DirBuilderExt;

    let Ok(mode) = u32::try_from(mode) else {
        return HostIntResult::err(errno_i32("EINVAL"));
    };
    let path = std::path::Path::new(OsStr::from_bytes(path));
    match std::fs::DirBuilder::new().mode(mode).create(path) {
        Ok(()) => HostIntResult::ok(0),
        Err(err) => HostIntResult::os_err(io_error_errno(&err)),
    }
}

#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
pub(in crate::runtime) fn mkdir_path_bytes(path: &[u8], mode: i64) -> HostIntResult {
    host_result_i64(unsafe { mhs_host_mkdir(path.as_ptr(), path.len(), mode) })
}

#[cfg(any(target_os = "wasi", not(any(unix, target_arch = "wasm32"))))]
pub(in crate::runtime) fn mkdir_path_bytes(path: &[u8], mode: i64) -> HostIntResult {
    #[cfg(target_os = "wasi")]
    wasi_trace_host(
        "mkdir",
        &format!("path={} mode={mode}", String::from_utf8_lossy(path)),
    );
    let _ = mode;
    let Ok(path) = std::str::from_utf8(path) else {
        return HostIntResult::err(errno_i32("EINVAL"));
    };
    match std::fs::create_dir(path) {
        Ok(()) => HostIntResult::ok(0),
        Err(err) => HostIntResult::os_err(io_error_errno(&err)),
    }
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
pub(in crate::runtime) fn current_dir_bytes() -> Result<Vec<u8>, i32> {
    use std::os::unix::ffi::OsStringExt;

    std::env::current_dir()
        .map(|path| path.into_os_string().into_vec())
        .map_err(|err| io_error_errno(&err).unwrap_or_else(|| errno_i32("ENOENT")))
}

#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
pub(in crate::runtime) fn current_dir_bytes() -> Result<Vec<u8>, i32> {
    host_bytes_result(unsafe { mhs_host_getcwd() })
}

#[cfg(any(target_os = "wasi", not(any(unix, target_arch = "wasm32"))))]
pub(in crate::runtime) fn current_dir_bytes() -> Result<Vec<u8>, i32> {
    #[cfg(target_os = "wasi")]
    wasi_trace_host("getcwd", "");
    std::env::current_dir()
        .map(|path| path.to_string_lossy().into_owned().into_bytes())
        .map_err(|err| io_error_errno(&err).unwrap_or_else(|| errno_i32("ENOENT")))
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
pub(in crate::runtime) fn executable_path_bytes() -> Result<Vec<u8>, i32> {
    use std::os::unix::ffi::OsStringExt;

    std::env::current_exe()
        .map(|path| path.into_os_string().into_vec())
        .map_err(|err| io_error_errno(&err).unwrap_or_else(|| errno_i32("ENOENT")))
}

#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
pub(in crate::runtime) fn executable_path_bytes() -> Result<Vec<u8>, i32> {
    Err(errno_i32("ENOSYS"))
}

#[cfg(any(target_os = "wasi", not(any(unix, target_arch = "wasm32"))))]
pub(in crate::runtime) fn executable_path_bytes() -> Result<Vec<u8>, i32> {
    #[cfg(target_os = "wasi")]
    wasi_trace_host("get_executable_path", "");
    std::env::current_exe()
        .map(|path| path.to_string_lossy().into_owned().into_bytes())
        .map_err(|err| io_error_errno(&err).unwrap_or_else(|| errno_i32("ENOENT")))
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
pub(in crate::runtime) fn tmpname_bytes(pre: &[u8], suf: &[u8]) -> Result<Vec<u8>, i32> {
    use std::ffi::{CString, OsString};
    use std::os::unix::ffi::OsStringExt;
    use std::os::unix::io::RawFd;

    let tmpdir = std::env::var_os("TMPDIR")
        .unwrap_or_else(|| OsString::from("/tmp"))
        .into_vec();
    let mut template = Vec::with_capacity(tmpdir.len() + pre.len() + suf.len() + 8);
    template.extend_from_slice(&tmpdir);
    template.push(b'/');
    template.extend_from_slice(pre);
    template.extend_from_slice(b"XXXXXX");
    template.extend_from_slice(suf);
    template.push(0);
    let suffix_len = std::os::raw::c_int::try_from(suf.len()).map_err(|_| errno_i32("EINVAL"))?;
    let path = CString::from_vec_with_nul(template).map_err(|_| errno_i32("EINVAL"))?;
    let mut bytes = path.into_bytes_with_nul();
    // SAFETY: mkstemps mutates the NUL-terminated template in place and returns a file descriptor.
    let fd: RawFd = unsafe { libc::mkstemps(bytes.as_mut_ptr().cast(), suffix_len) };
    if fd < 0 {
        return Err(last_errno());
    }
    // SAFETY: fd came from mkstemps and is not used after this close.
    unsafe {
        libc::close(fd);
    }
    bytes.pop();
    Ok(bytes)
}

#[cfg(target_arch = "wasm32")]
pub(in crate::runtime) fn tmpname_bytes(pre: &[u8], suf: &[u8]) -> Result<Vec<u8>, i32> {
    #[cfg(target_os = "wasi")]
    {
        let _ = (pre, suf);
        Err(errno_i32("ENOSYS"))
    }
    #[cfg(not(target_os = "wasi"))]
    {
        host_bytes_result(unsafe {
            mhs_host_tmpname(pre.as_ptr(), pre.len(), suf.as_ptr(), suf.len())
        })
    }
}

#[cfg(not(any(unix, target_arch = "wasm32")))]
pub(in crate::runtime) fn tmpname_bytes(pre: &[u8], suf: &[u8]) -> Result<Vec<u8>, i32> {
    let pre = std::str::from_utf8(pre).map_err(|_| errno_i32("EINVAL"))?;
    let suf = std::str::from_utf8(suf).map_err(|_| errno_i32("EINVAL"))?;
    let tmpdir = std::env::temp_dir();
    let seed = current_time_nanos() ^ u64::from(std::process::id());
    for attempt in 0..1024 {
        let mut name = String::with_capacity(pre.len() + 6 + suf.len());
        name.push_str(pre);
        name.push_str(
            std::str::from_utf8(&tmp_six(seed.wrapping_add(attempt))).unwrap_or("XXXXXX"),
        );
        name.push_str(suf);
        let path = tmpdir.join(name);
        match std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(_) => return Ok(path.to_string_lossy().into_owned().into_bytes()),
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(err) => return Err(io_error_errno(&err).unwrap_or_else(|| errno_i32("ENOENT"))),
        }
    }
    Err(errno_i32("EEXIST"))
}

#[cfg(not(any(unix, target_arch = "wasm32")))]
pub(in crate::runtime) fn tmp_six(mut value: u64) -> [u8; 6] {
    const ALPHABET: &[u8; 36] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let mut out = [b'0'; 6];
    for byte in &mut out {
        *byte = ALPHABET[(value % 36) as usize];
        value /= 36;
    }
    out
}

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
    #[cfg(target_os = "wasi")]
    wasi_trace_host("get_permissions", &String::from_utf8_lossy(path));
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
    #[cfg(target_os = "wasi")]
    wasi_trace_host(
        "set_permissions",
        &format!(
            "path={} permissions={permissions}",
            String::from_utf8_lossy(path)
        ),
    );
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

#[cfg(all(unix, not(target_arch = "wasm32")))]
pub(in crate::runtime) fn dir_entries_path_bytes(path: &[u8]) -> Result<Vec<Vec<u8>>, i32> {
    use std::ffi::OsStr;
    use std::os::unix::ffi::{OsStrExt, OsStringExt};

    let path = std::path::Path::new(OsStr::from_bytes(path));
    let mut entries = vec![b".".to_vec(), b"..".to_vec()];
    for entry in std::fs::read_dir(path)
        .map_err(|err| io_error_errno(&err).unwrap_or_else(|| errno_i32("ENOENT")))?
    {
        let entry =
            entry.map_err(|err| io_error_errno(&err).unwrap_or_else(|| errno_i32("ENOENT")))?;
        entries.push(entry.file_name().into_vec());
    }
    Ok(entries)
}

#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
pub(in crate::runtime) fn dir_entries_path_bytes(path: &[u8]) -> Result<Vec<Vec<u8>>, i32> {
    let bytes = host_bytes_result(unsafe { mhs_host_dir_entries(path.as_ptr(), path.len()) })?;
    Ok(bytes
        .split(|byte| *byte == 0)
        .filter(|entry| !entry.is_empty())
        .map(Vec::from)
        .collect())
}

#[cfg(any(target_os = "wasi", not(any(unix, target_arch = "wasm32"))))]
pub(in crate::runtime) fn dir_entries_path_bytes(path: &[u8]) -> Result<Vec<Vec<u8>>, i32> {
    #[cfg(target_os = "wasi")]
    wasi_trace_host("opendir", &String::from_utf8_lossy(path));
    let path = std::str::from_utf8(path).map_err(|_| errno_i32("EINVAL"))?;
    let mut entries = vec![b".".to_vec(), b"..".to_vec()];
    for entry in std::fs::read_dir(path)
        .map_err(|err| io_error_errno(&err).unwrap_or_else(|| errno_i32("ENOENT")))?
    {
        let entry =
            entry.map_err(|err| io_error_errno(&err).unwrap_or_else(|| errno_i32("ENOENT")))?;
        entries.push(
            entry
                .file_name()
                .to_string_lossy()
                .into_owned()
                .into_bytes(),
        );
    }
    Ok(entries)
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
pub(in crate::runtime) fn native_fopen_bfile(path: &[u8], mode: &[u8]) -> Result<BFile, i32> {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    let mode = parse_native_file_mode(mode).ok_or_else(|| errno_i32("EINVAL"))?;
    let file = open_native_file(std::path::Path::new(OsStr::from_bytes(path)), mode)?;
    Ok(BFile {
        kind: BFileKind::NativeFile {
            file: std::rc::Rc::new(std::cell::RefCell::new(file)),
            ungot: Vec::new(),
        },
        readable: mode.readable,
        writable: mode.writable,
    })
}

#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
pub(in crate::runtime) fn native_fopen_bfile(path: &[u8], mode: &[u8]) -> Result<BFile, i32> {
    let handle =
        unsafe { mhs_host_file_open(path.as_ptr(), path.len(), mode.as_ptr(), mode.len()) };
    if handle < 0 {
        return Err((-handle) as i32);
    }
    let mode = parse_native_file_mode(mode).ok_or_else(|| errno_i32("EINVAL"))?;
    Ok(BFile {
        kind: BFileKind::BrowserFile {
            handle,
            ungot: Vec::new(),
        },
        readable: mode.readable,
        writable: mode.writable,
    })
}

#[cfg(any(target_os = "wasi", not(any(unix, target_arch = "wasm32"))))]
pub(in crate::runtime) fn native_fopen_bfile(path: &[u8], mode: &[u8]) -> Result<BFile, i32> {
    #[cfg(target_os = "wasi")]
    wasi_trace_host(
        "fopen",
        &format!(
            "path={} mode={}",
            String::from_utf8_lossy(path),
            String::from_utf8_lossy(mode)
        ),
    );
    let path = std::str::from_utf8(path).map_err(|_| errno_i32("EINVAL"))?;
    let mode = parse_native_file_mode(mode).ok_or_else(|| errno_i32("EINVAL"))?;
    let file = open_native_file(std::path::Path::new(path), mode)?;
    Ok(BFile {
        kind: BFileKind::NativeFile {
            file: std::rc::Rc::new(std::cell::RefCell::new(file)),
            ungot: Vec::new(),
        },
        readable: mode.readable,
        writable: mode.writable,
    })
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
pub(in crate::runtime) fn native_fd_bfile(fd: i32) -> Result<BFile, i32> {
    use std::os::unix::io::FromRawFd;

    if fd < 0 {
        return Err(errno_i32("EBADF"));
    }
    // SAFETY: add_fd transfers fd ownership to the BFILE, matching the C runtime closeb_fd path.
    let file = unsafe { std::fs::File::from_raw_fd(fd) };
    Ok(BFile {
        kind: BFileKind::NativeFile {
            file: std::rc::Rc::new(std::cell::RefCell::new(file)),
            ungot: Vec::new(),
        },
        readable: true,
        writable: true,
    })
}

#[cfg(not(all(unix, not(target_arch = "wasm32"))))]
pub(in crate::runtime) fn native_fd_bfile(fd: i32) -> Result<BFile, i32> {
    let _ = fd;
    Err(errno_i32("ENOSYS"))
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
pub(in crate::runtime) fn open_fd_path_bytes(path: &[u8], flags: i32, mode: i64) -> HostIntResult {
    let path = match std::ffi::CString::new(path) {
        Ok(path) => path,
        Err(_) => return HostIntResult::err(errno_i32("EINVAL")),
    };
    let mode = match libc::mode_t::try_from(mode) {
        Ok(mode) => mode,
        Err(_) => return HostIntResult::err(errno_i32("EINVAL")),
    };
    // SAFETY: path is NUL-terminated and flags/mode are plain C values.
    let fd = unsafe { libc::open(path.as_ptr(), flags, mode) };
    if fd < 0 {
        HostIntResult::err(last_errno())
    } else {
        HostIntResult::ok(i64::from(fd))
    }
}

#[cfg(not(all(unix, not(target_arch = "wasm32"))))]
pub(in crate::runtime) fn open_fd_path_bytes(path: &[u8], flags: i32, mode: i64) -> HostIntResult {
    let _ = (path, flags, mode);
    HostIntResult::err(errno_i32("ENOSYS"))
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
pub(in crate::runtime) fn close_fd(fd: i32) -> HostIntResult {
    // SAFETY: close only consumes the integer file descriptor.
    let rc = unsafe { libc::close(fd) };
    if rc < 0 {
        HostIntResult::err(last_errno())
    } else {
        HostIntResult::ok(i64::from(rc))
    }
}

#[cfg(not(all(unix, not(target_arch = "wasm32"))))]
pub(in crate::runtime) fn close_fd(fd: i32) -> HostIntResult {
    let _ = fd;
    HostIntResult::err(errno_i32("ENOSYS"))
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
pub(in crate::runtime) fn fcntl_fd(fd: i32, cmd: i32, arg: i32) -> HostIntResult {
    // SAFETY: this mirrors the C runtime's three-int fcntl wrapper.
    let rc = unsafe { libc::fcntl(fd, cmd, arg) };
    if rc < 0 {
        HostIntResult::err(last_errno())
    } else {
        HostIntResult::ok(i64::from(rc))
    }
}

#[cfg(not(all(unix, not(target_arch = "wasm32"))))]
pub(in crate::runtime) fn fcntl_fd(fd: i32, cmd: i32, arg: i32) -> HostIntResult {
    let _ = (fd, cmd, arg);
    HostIntResult::err(errno_i32("ENOSYS"))
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
pub(in crate::runtime) fn socket_fd(domain: i32, typ: i32, protocol: i32) -> HostIntResult {
    // SAFETY: socket takes plain integer arguments.
    let fd = unsafe { libc::socket(domain, typ, protocol) };
    if fd < 0 {
        HostIntResult::err(last_errno())
    } else {
        HostIntResult::ok(i64::from(fd))
    }
}

#[cfg(not(all(unix, not(target_arch = "wasm32"))))]
pub(in crate::runtime) fn socket_fd(domain: i32, typ: i32, protocol: i32) -> HostIntResult {
    let _ = (domain, typ, protocol);
    HostIntResult::err(errno_i32("ENOSYS"))
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
pub(in crate::runtime) fn bind_socket(fd: i32, addr: &[u8]) -> HostIntResult {
    let len = match libc::socklen_t::try_from(addr.len()) {
        Ok(len) => len,
        Err(_) => return HostIntResult::err(errno_i32("EINVAL")),
    };
    // SAFETY: addr points to len bytes copied from guest memory.
    let rc = unsafe { libc::bind(fd, addr.as_ptr().cast(), len) };
    if rc < 0 {
        HostIntResult::err(last_errno())
    } else {
        HostIntResult::ok(i64::from(rc))
    }
}

#[cfg(not(all(unix, not(target_arch = "wasm32"))))]
pub(in crate::runtime) fn bind_socket(fd: i32, addr: &[u8]) -> HostIntResult {
    let _ = (fd, addr);
    HostIntResult::err(errno_i32("ENOSYS"))
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
pub(in crate::runtime) fn connect_socket(fd: i32, addr: &[u8]) -> HostIntResult {
    let len = match libc::socklen_t::try_from(addr.len()) {
        Ok(len) => len,
        Err(_) => return HostIntResult::err(errno_i32("EINVAL")),
    };
    // SAFETY: addr points to len bytes copied from guest memory.
    let rc = unsafe { libc::connect(fd, addr.as_ptr().cast(), len) };
    if rc < 0 {
        HostIntResult::err(last_errno())
    } else {
        HostIntResult::ok(i64::from(rc))
    }
}

#[cfg(not(all(unix, not(target_arch = "wasm32"))))]
pub(in crate::runtime) fn connect_socket(fd: i32, addr: &[u8]) -> HostIntResult {
    let _ = (fd, addr);
    HostIntResult::err(errno_i32("ENOSYS"))
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
pub(in crate::runtime) fn listen_socket(fd: i32, backlog: i32) -> HostIntResult {
    // SAFETY: listen takes plain integer arguments.
    let rc = unsafe { libc::listen(fd, backlog) };
    if rc < 0 {
        HostIntResult::err(last_errno())
    } else {
        HostIntResult::ok(i64::from(rc))
    }
}

#[cfg(not(all(unix, not(target_arch = "wasm32"))))]
pub(in crate::runtime) fn listen_socket(fd: i32, backlog: i32) -> HostIntResult {
    let _ = (fd, backlog);
    HostIntResult::err(errno_i32("ENOSYS"))
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
pub(in crate::runtime) fn setsockopt_socket(
    fd: i32,
    level: i32,
    optname: i32,
    optval: &[u8],
) -> HostIntResult {
    let len = match libc::socklen_t::try_from(optval.len()) {
        Ok(len) => len,
        Err(_) => return HostIntResult::err(errno_i32("EINVAL")),
    };
    // SAFETY: optval points to len bytes copied from guest memory.
    let rc = unsafe { libc::setsockopt(fd, level, optname, optval.as_ptr().cast(), len) };
    if rc < 0 {
        HostIntResult::err(last_errno())
    } else {
        HostIntResult::ok(i64::from(rc))
    }
}

#[cfg(not(all(unix, not(target_arch = "wasm32"))))]
pub(in crate::runtime) fn setsockopt_socket(
    fd: i32,
    level: i32,
    optname: i32,
    optval: &[u8],
) -> HostIntResult {
    let _ = (fd, level, optname, optval);
    HostIntResult::err(errno_i32("ENOSYS"))
}

pub(in crate::runtime) fn parse_native_file_mode(mode: &[u8]) -> Option<NativeFileMode> {
    let mut normalized = Vec::with_capacity(mode.len());
    for byte in mode {
        if *byte != b'b' {
            normalized.push(*byte);
        }
    }
    let mode = match normalized.as_slice() {
        b"r" => NativeFileMode {
            readable: true,
            writable: false,
            append: false,
            truncate: false,
            create: false,
        },
        b"w" => NativeFileMode {
            readable: false,
            writable: true,
            append: false,
            truncate: true,
            create: true,
        },
        b"a" => NativeFileMode {
            readable: false,
            writable: true,
            append: true,
            truncate: false,
            create: true,
        },
        b"r+" => NativeFileMode {
            readable: true,
            writable: true,
            append: false,
            truncate: false,
            create: false,
        },
        b"w+" => NativeFileMode {
            readable: true,
            writable: true,
            append: false,
            truncate: true,
            create: true,
        },
        b"a+" => NativeFileMode {
            readable: true,
            writable: true,
            append: true,
            truncate: false,
            create: true,
        },
        _ => return None,
    };
    Some(mode)
}

#[cfg(any(not(target_arch = "wasm32"), target_os = "wasi"))]
pub(in crate::runtime) fn open_native_file(
    path: &std::path::Path,
    mode: NativeFileMode,
) -> Result<std::fs::File, i32> {
    std::fs::OpenOptions::new()
        .read(mode.readable)
        .write(mode.writable && !mode.append)
        .append(mode.append)
        .truncate(mode.truncate)
        .create(mode.create)
        .open(path)
        .map_err(|err| io_error_errno(&err).unwrap_or_else(|| errno_i32("ENOENT")))
}
