//! Browser-host filesystem FFI declarations and result helpers.
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
