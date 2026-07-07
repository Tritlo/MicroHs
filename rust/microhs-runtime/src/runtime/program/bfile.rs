//! BFILE implementation modules for memory, native, and transformed streams.
use super::*;

mod block_io;
mod lifecycle;
mod pointers;
mod read_bytes;
mod setup;
mod utf8_stdio;
mod write_bytes;
