use super::parse_scenario_size;

pub(super) fn make_scenario(scenario: &str) -> Option<Result<Vec<u8>, String>> {
    if let Some(size) = scenario.strip_prefix("ffi-chain:") {
        return Some(parse_scenario_size("ffi-chain", size).map(ffi_chain));
    }
    if let Some(size) = scenario.strip_prefix("ffi-math-chain:") {
        return Some(parse_scenario_size("ffi-math-chain", size).map(ffi_math_chain));
    }
    if let Some(size) = scenario.strip_prefix("ffi-const-chain:") {
        return Some(parse_scenario_size("ffi-const-chain", size).map(ffi_const_chain));
    }
    if let Some(size) = scenario.strip_prefix("ffi-mem-chain:") {
        return Some(parse_scenario_size("ffi-mem-chain", size).map(ffi_mem_chain));
    }
    if let Some(size) = scenario.strip_prefix("ffi-wide-mem-chain:") {
        return Some(parse_scenario_size("ffi-wide-mem-chain", size).map(ffi_wide_mem_chain));
    }
    if let Some(size) = scenario.strip_prefix("ffi-word-mem-chain:") {
        return Some(parse_scenario_size("ffi-word-mem-chain", size).map(ffi_word_mem_chain));
    }
    if let Some(size) = scenario.strip_prefix("ffi-ptr-mem-chain:") {
        return Some(parse_scenario_size("ffi-ptr-mem-chain", size).map(ffi_ptr_mem_chain));
    }
    if let Some(size) = scenario.strip_prefix("ffi-strcpy-chain:") {
        return Some(parse_scenario_size("ffi-strcpy-chain", size).map(ffi_strcpy_chain));
    }
    if let Some(size) = scenario.strip_prefix("md5-string-chain:") {
        return Some(parse_scenario_size("md5-string-chain", size).map(md5_string_chain));
    }
    if let Some(size) = scenario.strip_prefix("getenv-chain:") {
        return Some(parse_scenario_size("getenv-chain", size).map(getenv_chain));
    }
    if let Some(size) = scenario.strip_prefix("env-set-chain:") {
        return Some(parse_scenario_size("env-set-chain", size).map(env_set_chain));
    }
    if let Some(size) = scenario.strip_prefix("errno-chain:") {
        return Some(parse_scenario_size("errno-chain", size).map(errno_chain));
    }
    if let Some(size) = scenario.strip_prefix("getcwd-chain:") {
        return Some(parse_scenario_size("getcwd-chain", size).map(getcwd_chain));
    }
    if let Some(size) = scenario.strip_prefix("dir-read-chain:") {
        return Some(parse_scenario_size("dir-read-chain", size).map(dir_read_chain));
    }
    if let Some(size) = scenario.strip_prefix("remove-missing-chain:") {
        return Some(parse_scenario_size("remove-missing-chain", size).map(remove_missing_chain));
    }
    if let Some(size) = scenario.strip_prefix("file-read-close-chain:") {
        return Some(parse_scenario_size("file-read-close-chain", size).map(file_read_close_chain));
    }
    None
}

fn repeat_action(size: usize, action: &str) -> Vec<u8> {
    let mut expr = action.to_owned();
    for _ in 1..size {
        expr = format!("IO.>> {expr} @ {action} @");
    }
    format!("v8.4\n0\nIO.performIO {expr} @ }}\n").into_bytes()
}

fn ffi_chain(size: usize) -> Vec<u8> {
    let mut expr = String::from("^islinux");
    for _ in 1..size {
        expr = format!("IO.>> {expr} @ ^islinux @");
    }
    format!("v8.4\n0\nIO.performIO {expr} @ }}\n").into_bytes()
}

fn ffi_math_chain(size: usize) -> Vec<u8> {
    let mut expr = String::from("^sqrt &9 @");
    for _ in 1..size {
        expr = format!("IO.>> {expr} @ ^sqrt &9 @ @");
    }
    format!("v8.4\n0\nIO.performIO {expr} @ }}\n").into_bytes()
}

fn ffi_const_chain(size: usize) -> Vec<u8> {
    let mut expr = String::from("^sizeof_int");
    for _ in 1..size {
        expr = format!("IO.>> {expr} @ ^sizeof_int @");
    }
    format!("v8.4\n0\nIO.performIO {expr} @ }}\n").into_bytes()
}

fn ffi_mem_chain(size: usize) -> Vec<u8> {
    repeat_action(size, "IO.lazyBind ^calloc #1 @ #1 @ @ ^peek_uint8 @")
}

fn ffi_wide_mem_chain(size: usize) -> Vec<u8> {
    repeat_action(
        size,
        "IO.lazyBind ^calloc #1 @ #8 @ @ S S K IO.>> @ @ S S K ^poke_uint64 @ @ I @ @ K ##123456789 @ @ @ @ S K ^peek_uint64 @ @ I @ @ @",
    )
}

fn ffi_word_mem_chain(size: usize) -> Vec<u8> {
    repeat_action(
        size,
        "IO.lazyBind ^calloc #1 @ #8 @ @ S S K IO.>> @ @ S S K ^pokeWord @ @ I @ @ K #123456789 @ @ @ @ S K ^peekWord @ @ I @ @ @",
    )
}

fn ffi_ptr_mem_chain(size: usize) -> Vec<u8> {
    let ptr_action = "IO.lazyBind ^calloc #1 @ #8 @ @ S S K IO.>> @ @ S S K ^pokePtr @ @ I @ @ K toPtr #42 @ @ @ @ @ S K ^peekPtr @ @ I @ @ @";
    let convert = "S K IO.return @ @ S K toInt @ @ I @ @";
    let action = format!("IO.lazyBind {ptr_action} @ {convert} @");
    repeat_action(size, &action)
}

fn ffi_strcpy_chain(size: usize) -> Vec<u8> {
    let mut source = String::from("fp2p bs2fp $17 microhs-rust-ffi");
    source.push('\0');
    source.push_str(" @ @");
    let action = format!(
        "IO.lazyBind ^calloc #1 @ #17 @ @ S S K IO.>> @ @ S S K ^strcpy @ @ I @ @ K {source} @ @ @ @ S K ^strlen @ @ I @ @ @"
    );
    repeat_action(size, &action)
}

fn md5_string_chain(size: usize) -> Vec<u8> {
    let text = "The quick fox jumps over the lazy dog.";
    let mut source = format!("fp2p bs2fp ${} {text}", text.len() + 1);
    source.push('\0');
    source.push_str(" @ @");
    let action = format!(
        "IO.lazyBind ^calloc #1 @ #16 @ @ S S K IO.>> @ @ S S K ^md5String @ @ K {source} @ @ @ I @ @ @ S K ^peek_uint8 @ @ I @ @ @"
    );
    repeat_action(size, &action)
}

fn getenv_chain(size: usize) -> Vec<u8> {
    let mut key = String::from("fp2p bs2fp $5 PATH");
    key.push('\0');
    key.push_str(" @ @");
    let action = format!("IO.lazyBind ^getenv {key} @ @ ^strlen @");
    repeat_action(size, &action)
}

fn env_set_chain(size: usize) -> Vec<u8> {
    let mut key = String::from("fp2p bs2fp $23 MICROHS_RUST_ENV_BENCH");
    key.push('\0');
    key.push_str(" @ @");
    let mut value = String::from("fp2p bs2fp $4 xyz");
    value.push('\0');
    value.push_str(" @ @");
    let action =
        format!("IO.>> ^setenv {key} @ {value} @ #1 @ @ IO.lazyBind ^getenv {key} @ @ ^strlen @ @");
    repeat_action(size, &action)
}

fn errno_chain(size: usize) -> Vec<u8> {
    repeat_action(
        size,
        "IO.lazyBind ^&errno @ S S K IO.>> @ @ S S K ^poke_int @ @ I @ @ K #2 @ @ @ @ S K ^peek_int @ @ I @ @ @",
    )
}

fn getcwd_chain(size: usize) -> Vec<u8> {
    repeat_action(
        size,
        "IO.lazyBind ^calloc #1 @ #4096 @ @ S S K IO.>> @ @ S S K ^getcwd @ @ I @ @ K #4096 @ @ @ @ S K ^strlen @ @ I @ @ @",
    )
}

fn dir_read_chain(size: usize) -> Vec<u8> {
    let mut path = String::from("fp2p bs2fp $2 .");
    path.push('\0');
    path.push_str(" @ @");
    let action = format!(
        "IO.lazyBind ^opendir {path} @ @ S S K IO.>> @ @ S S K IO.lazyBind @ @ S K ^readdir @ @ I @ @ @ K S S K IO.lazyBind @ @ S K ^c_d_name @ @ I @ @ @ K ^strlen @ @ @ @ @ @ S K ^closedir @ @ I @ @ @"
    );
    repeat_action(size, &action)
}

fn remove_missing_chain(size: usize) -> Vec<u8> {
    let mut path = String::from("fp2p bs2fp $28 microhs-rust-missing-remove");
    path.push('\0');
    path.push_str(" @ @");
    let action = format!("^remove {path} @");
    repeat_action(size, &action)
}

fn file_read_close_chain(size: usize) -> Vec<u8> {
    let mut path = String::from("fp2p bs2fp $11 Cargo.toml");
    path.push('\0');
    path.push_str(" @ @");
    let mut mode = String::from("fp2p bs2fp $3 rb");
    mode.push('\0');
    mode.push_str(" @ @");
    let open =
        format!("IO.lazyBind IO.lazyBind ^fopen {path} @ {mode} @ @ ^add_FILE @ @ ^add_utf8 @");
    let read_close = "S S K IO.>> @ @ ^getb @ @ ^closeb @";
    let action = format!("IO.lazyBind {open} @ {read_close} @");
    repeat_action(size, &action)
}
