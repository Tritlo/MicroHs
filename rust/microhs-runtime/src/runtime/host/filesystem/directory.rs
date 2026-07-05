//! Directory listing host operations.
use super::*;

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
