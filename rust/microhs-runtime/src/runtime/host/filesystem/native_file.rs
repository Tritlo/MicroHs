//! Native BFILE construction and file-open mode parsing.
use super::*;

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
