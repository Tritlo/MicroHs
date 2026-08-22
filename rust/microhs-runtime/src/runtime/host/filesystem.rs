//! Host filesystem, directory, environment, and native file operations.
use super::*;

#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
mod browser_host;
mod directory;
mod env;
mod fd_socket;
mod native_file;
mod path_ops;
mod permissions;

#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
pub(in crate::runtime) use self::browser_host::*;
pub(in crate::runtime) use self::directory::*;
pub(in crate::runtime) use self::env::*;
pub(in crate::runtime) use self::fd_socket::*;
pub(in crate::runtime) use self::native_file::*;
pub(in crate::runtime) use self::path_ops::*;
pub(in crate::runtime) use self::permissions::*;
