//! Platform constants, errno mapping, and time helpers.
pub(in crate::runtime) fn size_of_i64<T>() -> i64 {
    std::mem::size_of::<T>() as i64
}

pub(in crate::runtime) fn format_float(value: f64) -> String {
    let mut out = value.to_string();
    if out == "NaN" {
        out = "nan".to_owned();
    }
    if out != "nan"
        && out != "-nan"
        && out != "inf"
        && out != "-inf"
        && !out.contains('.')
        && !out.contains('e')
        && !out.contains('E')
    {
        out.push_str(".0");
    }
    out
}

pub(in crate::runtime) fn current_time_micro() -> i64 {
    #[cfg(target_arch = "wasm32")]
    {
        0
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::time::{SystemTime, UNIX_EPOCH};

        let Ok(duration) = SystemTime::now().duration_since(UNIX_EPOCH) else {
            return 0;
        };
        i64::try_from(duration.as_micros()).unwrap_or(i64::MAX)
    }
}

#[cfg(not(any(unix, target_arch = "wasm32")))]
pub(in crate::runtime) fn current_time_nanos() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};

    let Ok(duration) = SystemTime::now().duration_since(UNIX_EPOCH) else {
        return 0;
    };
    u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX)
}

#[cfg(all(
    any(target_os = "linux", target_os = "android"),
    not(target_arch = "wasm32")
))]
pub(in crate::runtime) fn cpu_time() -> (u64, u64) {
    let mut ts = std::mem::MaybeUninit::<libc::timespec>::uninit();
    // SAFETY: clock_gettime writes the timespec on success. The pointer is valid for one call.
    let rc = unsafe { libc::clock_gettime(libc::CLOCK_PROCESS_CPUTIME_ID, ts.as_mut_ptr()) };
    if rc != 0 {
        return (0, 0);
    }
    // SAFETY: the call above succeeded, so the timespec has been initialized.
    let ts = unsafe { ts.assume_init() };
    (
        u64::try_from(ts.tv_sec).unwrap_or(0),
        u64::try_from(ts.tv_nsec).unwrap_or(0),
    )
}

#[cfg(any(
    target_arch = "wasm32",
    not(any(target_os = "linux", target_os = "android"))
))]
pub(in crate::runtime) fn cpu_time() -> (u64, u64) {
    (0, 0)
}

pub(in crate::runtime) fn errno_i32(name: &str) -> i32 {
    errno_constant(name).unwrap_or(-1) as i32
}

pub(in crate::runtime) fn host_constant(name: &str) -> Option<i64> {
    #[cfg(all(unix, not(target_arch = "wasm32")))]
    {
        Some(i64::from(match name {
            "F_SETFL" => libc::F_SETFL,
            "O_NONBLOCK" => libc::O_NONBLOCK,
            "SOL_SOCKET" => libc::SOL_SOCKET,
            "SO_DEBUG" => libc::SO_DEBUG,
            "SO_ERROR" => libc::SO_ERROR,
            "SO_REUSEADDR" => libc::SO_REUSEADDR,
            "SO_TYPE" => libc::SO_TYPE,
            _ => return None,
        }))
    }
    #[cfg(not(all(unix, not(target_arch = "wasm32"))))]
    {
        const HOST_CONSTANTS: &[&str] = &[
            "F_SETFL",
            "O_NONBLOCK",
            "SOL_SOCKET",
            "SO_DEBUG",
            "SO_ERROR",
            "SO_REUSEADDR",
            "SO_TYPE",
        ];
        HOST_CONSTANTS.contains(&name).then_some(-1)
    }
}

#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
pub(in crate::runtime) fn last_errno() -> i32 {
    std::io::Error::last_os_error()
        .raw_os_error()
        .unwrap_or_else(|| errno_i32("ENOENT"))
}

#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
pub(in crate::runtime) fn io_error_errno(error: &std::io::Error) -> Option<i32> {
    error.raw_os_error()
}

pub(in crate::runtime) fn strerror_bytes(errno: i32) -> Vec<u8> {
    std::io::Error::from_raw_os_error(errno)
        .to_string()
        .into_bytes()
}

#[cfg(all(
    any(target_os = "linux", target_os = "android"),
    not(target_arch = "wasm32")
))]
pub(in crate::runtime) fn errno_constant(name: &str) -> Option<i64> {
    Some(i64::from(match name {
        "EOK" => 0,
        "E2BIG" => libc::E2BIG,
        "EACCES" => libc::EACCES,
        "EADDRINUSE" => libc::EADDRINUSE,
        "EADDRNOTAVAIL" => libc::EADDRNOTAVAIL,
        "EADV" => libc::EADV,
        "EAFNOSUPPORT" => libc::EAFNOSUPPORT,
        "EAGAIN" => libc::EAGAIN,
        "EALREADY" => libc::EALREADY,
        "EBADF" => libc::EBADF,
        "EBADMSG" => libc::EBADMSG,
        "EBADRPC" => -1,
        "EBUSY" => libc::EBUSY,
        "ECHILD" => libc::ECHILD,
        "ECOMM" => libc::ECOMM,
        "ECONNABORTED" => libc::ECONNABORTED,
        "ECONNREFUSED" => libc::ECONNREFUSED,
        "ECONNRESET" => libc::ECONNRESET,
        "EDEADLK" => libc::EDEADLK,
        "EDESTADDRREQ" => libc::EDESTADDRREQ,
        "EDIRTY" => -1,
        "EDOM" => libc::EDOM,
        "EDQUOT" => libc::EDQUOT,
        "EEXIST" => libc::EEXIST,
        "EFAULT" => libc::EFAULT,
        "EFBIG" => libc::EFBIG,
        "EFTYPE" => -1,
        "EHOSTDOWN" => libc::EHOSTDOWN,
        "EHOSTUNREACH" => libc::EHOSTUNREACH,
        "EIDRM" => libc::EIDRM,
        "EILSEQ" => libc::EILSEQ,
        "EINPROGRESS" => libc::EINPROGRESS,
        "EINTR" => libc::EINTR,
        "EINVAL" => libc::EINVAL,
        "EIO" => libc::EIO,
        "EISCONN" => libc::EISCONN,
        "EISDIR" => libc::EISDIR,
        "ELOOP" => libc::ELOOP,
        "EMFILE" => libc::EMFILE,
        "EMLINK" => libc::EMLINK,
        "EMSGSIZE" => libc::EMSGSIZE,
        "EMULTIHOP" => libc::EMULTIHOP,
        "ENAMETOOLONG" => libc::ENAMETOOLONG,
        "ENETDOWN" => libc::ENETDOWN,
        "ENETRESET" => libc::ENETRESET,
        "ENETUNREACH" => libc::ENETUNREACH,
        "ENFILE" => libc::ENFILE,
        "ENOBUFS" => libc::ENOBUFS,
        "ENODATA" => libc::ENODATA,
        "ENODEV" => libc::ENODEV,
        "ENOENT" => libc::ENOENT,
        "ENOEXEC" => libc::ENOEXEC,
        "ENOLCK" => libc::ENOLCK,
        "ENOLINK" => libc::ENOLINK,
        "ENOMEM" => libc::ENOMEM,
        "ENOMSG" => libc::ENOMSG,
        "ENONET" => libc::ENONET,
        "ENOPROTOOPT" => libc::ENOPROTOOPT,
        "ENOSPC" => libc::ENOSPC,
        "ENOSR" => libc::ENOSR,
        "ENOSTR" => libc::ENOSTR,
        "ENOSYS" => libc::ENOSYS,
        "ENOTBLK" => libc::ENOTBLK,
        "ENOTCONN" => libc::ENOTCONN,
        "ENOTDIR" => libc::ENOTDIR,
        "ENOTEMPTY" => libc::ENOTEMPTY,
        "ENOTSOCK" => libc::ENOTSOCK,
        "ENOTSUP" => libc::ENOTSUP,
        "ENOTTY" => libc::ENOTTY,
        "ENXIO" => libc::ENXIO,
        "EOPNOTSUPP" => libc::EOPNOTSUPP,
        "EPERM" => libc::EPERM,
        "EPFNOSUPPORT" => libc::EPFNOSUPPORT,
        "EPIPE" => libc::EPIPE,
        "EPROCLIM" => -1,
        "EPROCUNAVAIL" => -1,
        "EPROGMISMATCH" => -1,
        "EPROGUNAVAIL" => -1,
        "EPROTO" => libc::EPROTO,
        "EPROTONOSUPPORT" => libc::EPROTONOSUPPORT,
        "EPROTOTYPE" => libc::EPROTOTYPE,
        "ERANGE" => libc::ERANGE,
        "EREMCHG" => libc::EREMCHG,
        "EREMOTE" => libc::EREMOTE,
        "EROFS" => libc::EROFS,
        "ERPCMISMATCH" => -1,
        "ERREMOTE" => -1,
        "ESHUTDOWN" => libc::ESHUTDOWN,
        "ESOCKTNOSUPPORT" => libc::ESOCKTNOSUPPORT,
        "ESPIPE" => libc::ESPIPE,
        "ESRCH" => libc::ESRCH,
        "ESRMNT" => libc::ESRMNT,
        "ESTALE" => libc::ESTALE,
        "ETIME" => libc::ETIME,
        "ETIMEDOUT" => libc::ETIMEDOUT,
        "ETOOMANYREFS" => libc::ETOOMANYREFS,
        "ETXTBSY" => libc::ETXTBSY,
        "EUSERS" => libc::EUSERS,
        "EWOULDBLOCK" => libc::EWOULDBLOCK,
        "EXDEV" => libc::EXDEV,
        _ => return None,
    }))
}

#[cfg(any(
    target_arch = "wasm32",
    not(any(target_os = "linux", target_os = "android"))
))]
pub(in crate::runtime) fn errno_constant(name: &str) -> Option<i64> {
    if name == "EOK" {
        return Some(0);
    }
    const ERRNO_NAMES: &[&str] = &[
        "E2BIG",
        "EACCES",
        "EADDRINUSE",
        "EADDRNOTAVAIL",
        "EADV",
        "EAFNOSUPPORT",
        "EAGAIN",
        "EALREADY",
        "EBADF",
        "EBADMSG",
        "EBADRPC",
        "EBUSY",
        "ECHILD",
        "ECOMM",
        "ECONNABORTED",
        "ECONNREFUSED",
        "ECONNRESET",
        "EDEADLK",
        "EDESTADDRREQ",
        "EDIRTY",
        "EDOM",
        "EDQUOT",
        "EEXIST",
        "EFAULT",
        "EFBIG",
        "EFTYPE",
        "EHOSTDOWN",
        "EHOSTUNREACH",
        "EIDRM",
        "EILSEQ",
        "EINPROGRESS",
        "EINTR",
        "EINVAL",
        "EIO",
        "EISCONN",
        "EISDIR",
        "ELOOP",
        "EMFILE",
        "EMLINK",
        "EMSGSIZE",
        "EMULTIHOP",
        "ENAMETOOLONG",
        "ENETDOWN",
        "ENETRESET",
        "ENETUNREACH",
        "ENFILE",
        "ENOBUFS",
        "ENODATA",
        "ENODEV",
        "ENOENT",
        "ENOEXEC",
        "ENOLCK",
        "ENOLINK",
        "ENOMEM",
        "ENOMSG",
        "ENONET",
        "ENOPROTOOPT",
        "ENOSPC",
        "ENOSR",
        "ENOSTR",
        "ENOSYS",
        "ENOTBLK",
        "ENOTCONN",
        "ENOTDIR",
        "ENOTEMPTY",
        "ENOTSOCK",
        "ENOTSUP",
        "ENOTTY",
        "ENXIO",
        "EOPNOTSUPP",
        "EPERM",
        "EPFNOSUPPORT",
        "EPIPE",
        "EPROCLIM",
        "EPROCUNAVAIL",
        "EPROGMISMATCH",
        "EPROGUNAVAIL",
        "EPROTO",
        "EPROTONOSUPPORT",
        "EPROTOTYPE",
        "ERANGE",
        "EREMCHG",
        "EREMOTE",
        "EROFS",
        "ERPCMISMATCH",
        "ERREMOTE",
        "ESHUTDOWN",
        "ESOCKTNOSUPPORT",
        "ESPIPE",
        "ESRCH",
        "ESRMNT",
        "ESTALE",
        "ETIME",
        "ETIMEDOUT",
        "ETOOMANYREFS",
        "ETXTBSY",
        "EUSERS",
        "EWOULDBLOCK",
        "EXDEV",
    ];
    ERRNO_NAMES.contains(&name).then_some(-1)
}
