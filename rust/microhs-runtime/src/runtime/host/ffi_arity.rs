fn ffi_arity(name: &str) -> Option<usize> {
    if errno_constant(name).is_some() {
        return Some(0);
    }
    if host_constant(name).is_some() {
        return Some(0);
    }
    Some(match name {
        "GETRAW"
        | "GETTIMEMICRO"
        | "islinux"
        | "ismacos"
        | "iswindows"
        | "sizeof_char"
        | "sizeof_short"
        | "sizeof_int"
        | "sizeof_long"
        | "sizeof_llong"
        | "sizeof_size_t"
        | "want_gmp"
        | "want_imath"
        | "&closeb"
        | "&free"
        | "&errno"
        | "errno"
        | "environ"
        | "get_executable_path"
        | "new_mpz"
        | "openb_wr_mem" => 0,
        "malloc"
        | "free"
        | "strlen"
        | "getenv"
        | "unsetenv"
        | "putchar"
        | "remove"
        | "system"
        | "chdir"
        | "get_permissions"
        | "add_fd"
        | "opendir"
        | "readdir"
        | "closedir"
        | "c_d_name"
        | "close"
        | "js_debug"
        | "js_eval_run"
        | "js_eval_call"
        | "js_set_haskellCallback"
        | "add_FILE"
        | "add_utf8"
        | "add_crlf"
        | "add_rle_compressor"
        | "add_rle_decompressor"
        | "add_base64_encoder"
        | "add_base64_decoder"
        | "add_lz77_compressor"
        | "add_lz77_decompressor"
        | "add_bwt_compressor"
        | "add_bwt_decompressor"
        | "add_lzma_compressor"
        | "add_lzma_decompressor"
        | "closeb"
        | "flushb"
        | "getb"
        | "peekPtr"
        | "peekWord"
        | "peek_uint8"
        | "peek_uint16"
        | "peek_uint32"
        | "peek_uint64"
        | "peek_int8"
        | "peek_int16"
        | "peek_int32"
        | "peek_int64"
        | "peek_char"
        | "peek_schar"
        | "peek_uchar"
        | "peek_short"
        | "peek_ushort"
        | "peek_int"
        | "peek_uint"
        | "peek_long"
        | "peek_ulong"
        | "peek_llong"
        | "peek_ullong"
        | "peek_size_t"
        | "peek_flt32"
        | "peek_flt64"
        | "mpz_get_d"
        | "mpz_get_f"
        | "mpz_get_si"
        | "mpz_get_si64"
        | "mpz_log2"
        | "mpz_popcount"
        | "acos"
        | "asin"
        | "atan"
        | "cos"
        | "exp"
        | "log"
        | "sin"
        | "sqrt"
        | "tan"
        | "acosf"
        | "asinf"
        | "atanf"
        | "cosf"
        | "expf"
        | "logf"
        | "sinf"
        | "sqrtf"
        | "tanf" => 1,
        "calloc" | "realloc" | "strcpy" | "fopen" | "tmpname" | "add_buf" | "mkdir" | "getcwd"
        | "set_permissions" | "md5BFILE" | "md5String" | "pokePtr" | "pokeWord" | "poke_uint8"
        | "poke_uint16" | "poke_uint32" | "poke_uint64" | "poke_int8" | "poke_int16"
        | "poke_int32" | "poke_int64" | "poke_char" | "poke_schar" | "poke_uchar"
        | "poke_short" | "poke_ushort" | "poke_int" | "poke_uint" | "poke_long" | "poke_ulong"
        | "poke_llong" | "poke_ullong" | "poke_size_t" | "poke_flt32" | "poke_flt64"
        | "openb_rd_mem" | "getcpu" | "gettimeofday" | "listen" | "mpz_abs" | "mpz_cmp"
        | "mpz_init_set_si" | "mpz_init_set_si64" | "mpz_init_set_ui" | "mpz_init_set_ui64"
        | "mpz_neg" | "mpz_tstbit" | "putb" | "ungetb" | "atan2" | "pow" | "scalbn" | "atan2f"
        | "powf" | "scalbnf" => 2,
        "memcpy" | "memmove" | "setenv" | "md5Array" | "get_mem" | "readb" | "writeb" | "open"
        | "accept" | "bind" | "connect" | "fcntl" | "lz77c" | "mpz_add" | "mpz_and"
        | "mpz_fdiv_q_2exp" | "mpz_ior" | "mpz_mul" | "mpz_mul_2exp" | "mpz_sub" | "mpz_xor"
        | "socket" => 3,
        "recv" | "send" => 4,
        "mpz_tdiv_qr" => 4,
        "getsockopt" | "setsockopt" => 5,
        "strerror_r" => 3,
        _ => return None,
    })
}

fn is_unary_math_ffi_candidate(name: &str) -> bool {
    let bytes = name.as_bytes();
    match bytes.first() {
        Some(b'a') => {
            bytes.starts_with(b"ac") || bytes.starts_with(b"as") || bytes.starts_with(b"at")
        }
        Some(b'c') => bytes.starts_with(b"co"),
        Some(b'e') => bytes.starts_with(b"ex"),
        Some(b'l') => bytes.starts_with(b"lo"),
        Some(b's') => bytes.starts_with(b"si") || bytes.starts_with(b"sq"),
        Some(b't') => bytes.starts_with(b"ta"),
        _ => false,
    }
}
