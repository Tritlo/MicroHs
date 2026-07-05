//! Native file-descriptor and socket wrappers.
use super::*;

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
