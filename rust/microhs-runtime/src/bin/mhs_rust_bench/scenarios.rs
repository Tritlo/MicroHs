pub(super) fn make_scenario(scenario: &str) -> Result<Vec<u8>, String> {
    if let Some(size) = scenario.strip_prefix("identity-chain:") {
        let size = parse_scenario_size("identity-chain", size)?;
        let mut out = String::from("v8.4\n0\nI");
        for _ in 1..size {
            out.push_str(" I @");
        }
        out.push_str(" #1 @ }\n");
        return Ok(out.into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("arith-chain:") {
        let size = parse_scenario_size("arith-chain", size)?;
        let mut expr = String::from("#0");
        for _ in 0..size {
            expr = format!("+ {expr} @ #1 @");
        }
        return Ok(format!("v8.4\n0\n{expr} }}\n").into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("int64-chain:") {
        let size = parse_scenario_size("int64-chain", size)?;
        let mut expr = String::from("##0");
        for _ in 0..size {
            expr = format!("I+ {expr} @ ##1 @");
        }
        return Ok(format!("v8.4\n0\n{expr} }}\n").into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("float64-chain:") {
        let size = parse_scenario_size("float64-chain", size)?;
        let mut expr = String::from("&0");
        for _ in 0..size {
            expr = format!("d+ {expr} @ &1.25 @");
        }
        return Ok(format!("v8.4\n0\n{expr} }}\n").into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("float32-chain:") {
        let size = parse_scenario_size("float32-chain", size)?;
        let mut expr = String::from("&&0");
        for _ in 0..size {
            expr = format!("f+ {expr} @ &&1.25 @");
        }
        return Ok(format!("v8.4\n0\n{expr} }}\n").into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("bytes-chain:") {
        let size = parse_scenario_size("bytes-chain", size)?;
        let mut expr = String::from("\"\"");
        for idx in 0..size {
            let byte = (b'a' + (idx % 26) as u8) as char;
            expr = format!("bs++ {expr} @ \"{byte}\" @");
        }
        return Ok(format!("v8.4\n0\n{expr} }}\n").into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("foreignptr-slice:") {
        let bytes = ascii_payload("foreignptr-slice", size)?;
        let len = bytes.len() - 1;
        return Ok(
            format!("v8.4\n0\nfp2bs fp+ bs2fp \"{bytes}\" @ @ #1 @ @ #{len} @ }}\n").into_bytes(),
        );
    }
    if let Some(size) = scenario.strip_prefix("cstring-pack:") {
        let bytes = ascii_payload("cstring-pack", size)?;
        let len = bytes.len();
        return Ok(format!(
            "v8.4\n0\nIO.performIO packCStringLen fp2p bs2fp \"{bytes}\" @ @ @ #{len} @ @ }}\n"
        )
        .into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("unpack-chain:") {
        let bytes = ascii_payload("unpack-chain", size)?;
        return Ok(format!("v8.4\n0\nbsunpack \"{bytes}\" @ #0 @ K @ }}\n").into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("fromutf8-chain:") {
        let bytes = ascii_payload("fromutf8-chain", size)?;
        return Ok(format!("v8.4\n0\nfromUTF8 \"{bytes}\" @ #0 @ K @ }}\n").into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("array-chain:") {
        let size = parse_scenario_size("array-chain", size)?;
        let last = size - 1;
        let mut items = String::new();
        for _ in 0..size {
            items.push_str("#0 ");
        }
        return Ok(format!(
            "v8.4\n1\nIO.performIO IO.>> A.write {items}[{size}] :0 @ #{last} @ #42 @ @ A.read _0 @ #{last} @ @ @ }}\n"
        )
        .into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("io-chain:") {
        let size = parse_scenario_size("io-chain", size)?;
        let mut expr = String::from("IO.return #0 @");
        for idx in 1..size {
            expr = format!("IO.>> {expr} @ IO.return #{idx} @ @");
        }
        return Ok(format!("v8.4\n0\nIO.performIO {expr} @ }}\n").into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("io-array-chain:") {
        let size = parse_scenario_size("io-array-chain", size)?;
        let last = size - 1;
        let mut items = String::new();
        for _ in 0..size {
            items.push_str("#0 ");
        }
        return Ok(format!(
            "v8.4\n1\nIO.performIO IO.>> A.write {items}[{size}] :0 @ #{last} @ #42 @ @ A.read _0 @ #{last} @ @ @ }}\n"
        )
        .into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("io-bytes-chain:") {
        let bytes = ascii_payload("io-bytes-chain", size)?;
        let last = bytes.len() - 1;
        return Ok(format!(
            "v8.4\n1\nIO.performIO IO.>> bswrite \"{bytes}\" :0 @ #{last} @ #42 @ @ bsread _0 @ #{last} @ @ @ }}\n"
        )
        .into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("io-control-chain:") {
        let size = parse_scenario_size("io-control-chain", size)?;
        let mut expr = String::from("IO.getmaskingstate");
        for idx in 0..size {
            expr = format!("IO.>> IO.setmaskingstate #{} @ @ {expr} @", idx % 3);
        }
        return Ok(format!("v8.4\n0\nIO.performIO {expr} @ }}\n").into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("performio-apply-chain:") {
        let size = parse_scenario_size("performio-apply-chain", size)?;
        let mut expr = String::from("#0");
        for _ in 0..size {
            expr = format!("IO.performIO IO.return I @ @ {expr} @");
        }
        return Ok(format!("v8.4\n0\n{expr} }}\n").into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("argref-chain:") {
        let size = parse_scenario_size("argref-chain", size)?;
        let mut expr = String::from("IO.getArgRef");
        for _ in 1..size {
            expr = format!("IO.>> {expr} @ IO.getArgRef @");
        }
        return Ok(format!("v8.4\n0\nIO.performIO {expr} @ }}\n").into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("stdio-chain:") {
        let size = parse_scenario_size("stdio-chain", size)?;
        let mut expr = String::from("#0");
        for _ in 0..size {
            expr = format!("seq toInt fp2p IO.stdout @ @ @ {expr} @");
        }
        return Ok(format!("v8.4\n0\n{expr} }}\n").into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("ffi-chain:") {
        let size = parse_scenario_size("ffi-chain", size)?;
        let mut expr = String::from("^islinux");
        for _ in 1..size {
            expr = format!("IO.>> {expr} @ ^islinux @");
        }
        return Ok(format!("v8.4\n0\nIO.performIO {expr} @ }}\n").into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("ffi-math-chain:") {
        let size = parse_scenario_size("ffi-math-chain", size)?;
        let mut expr = String::from("^sqrt &9 @");
        for _ in 1..size {
            expr = format!("IO.>> {expr} @ ^sqrt &9 @ @");
        }
        return Ok(format!("v8.4\n0\nIO.performIO {expr} @ }}\n").into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("ffi-const-chain:") {
        let size = parse_scenario_size("ffi-const-chain", size)?;
        let mut expr = String::from("^sizeof_int");
        for _ in 1..size {
            expr = format!("IO.>> {expr} @ ^sizeof_int @");
        }
        return Ok(format!("v8.4\n0\nIO.performIO {expr} @ }}\n").into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("ffi-mem-chain:") {
        let size = parse_scenario_size("ffi-mem-chain", size)?;
        let action = "IO.lazyBind ^calloc #1 @ #1 @ @ ^peek_uint8 @";
        let mut expr = action.to_owned();
        for _ in 1..size {
            expr = format!("IO.>> {expr} @ {action} @");
        }
        return Ok(format!("v8.4\n0\nIO.performIO {expr} @ }}\n").into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("ffi-wide-mem-chain:") {
        let size = parse_scenario_size("ffi-wide-mem-chain", size)?;
        let action = "IO.lazyBind ^calloc #1 @ #8 @ @ S S K IO.>> @ @ S S K ^poke_uint64 @ @ I @ @ K ##123456789 @ @ @ @ S K ^peek_uint64 @ @ I @ @ @";
        let mut expr = action.to_owned();
        for _ in 1..size {
            expr = format!("IO.>> {expr} @ {action} @");
        }
        return Ok(format!("v8.4\n0\nIO.performIO {expr} @ }}\n").into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("ffi-word-mem-chain:") {
        let size = parse_scenario_size("ffi-word-mem-chain", size)?;
        let action = "IO.lazyBind ^calloc #1 @ #8 @ @ S S K IO.>> @ @ S S K ^pokeWord @ @ I @ @ K #123456789 @ @ @ @ S K ^peekWord @ @ I @ @ @";
        let mut expr = action.to_owned();
        for _ in 1..size {
            expr = format!("IO.>> {expr} @ {action} @");
        }
        return Ok(format!("v8.4\n0\nIO.performIO {expr} @ }}\n").into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("ffi-ptr-mem-chain:") {
        let size = parse_scenario_size("ffi-ptr-mem-chain", size)?;
        let ptr_action = "IO.lazyBind ^calloc #1 @ #8 @ @ S S K IO.>> @ @ S S K ^pokePtr @ @ I @ @ K toPtr #42 @ @ @ @ @ S K ^peekPtr @ @ I @ @ @";
        let convert = "S K IO.return @ @ S K toInt @ @ I @ @";
        let action = format!("IO.lazyBind {ptr_action} @ {convert} @");
        let mut expr = action.to_owned();
        for _ in 1..size {
            expr = format!("IO.>> {expr} @ {action} @");
        }
        return Ok(format!("v8.4\n0\nIO.performIO {expr} @ }}\n").into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("ffi-strcpy-chain:") {
        let size = parse_scenario_size("ffi-strcpy-chain", size)?;
        let mut source = String::from("fp2p bs2fp $17 microhs-rust-ffi");
        source.push('\0');
        source.push_str(" @ @");
        let action = format!(
            "IO.lazyBind ^calloc #1 @ #17 @ @ S S K IO.>> @ @ S S K ^strcpy @ @ I @ @ K {source} @ @ @ @ S K ^strlen @ @ I @ @ @"
        );
        let mut expr = action.clone();
        for _ in 1..size {
            expr = format!("IO.>> {expr} @ {action} @");
        }
        return Ok(format!("v8.4\n0\nIO.performIO {expr} @ }}\n").into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("md5-string-chain:") {
        let size = parse_scenario_size("md5-string-chain", size)?;
        let text = "The quick fox jumps over the lazy dog.";
        let mut source = format!("fp2p bs2fp ${} {text}", text.len() + 1);
        source.push('\0');
        source.push_str(" @ @");
        let action = format!(
            "IO.lazyBind ^calloc #1 @ #16 @ @ S S K IO.>> @ @ S S K ^md5String @ @ K {source} @ @ @ I @ @ @ S K ^peek_uint8 @ @ I @ @ @"
        );
        let mut expr = action.clone();
        for _ in 1..size {
            expr = format!("IO.>> {expr} @ {action} @");
        }
        return Ok(format!("v8.4\n0\nIO.performIO {expr} @ }}\n").into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("getenv-chain:") {
        let size = parse_scenario_size("getenv-chain", size)?;
        let mut key = String::from("fp2p bs2fp $5 PATH");
        key.push('\0');
        key.push_str(" @ @");
        let action = format!("IO.lazyBind ^getenv {key} @ @ ^strlen @");
        let mut expr = action.clone();
        for _ in 1..size {
            expr = format!("IO.>> {expr} @ {action} @");
        }
        return Ok(format!("v8.4\n0\nIO.performIO {expr} @ }}\n").into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("env-set-chain:") {
        let size = parse_scenario_size("env-set-chain", size)?;
        let mut key = String::from("fp2p bs2fp $23 MICROHS_RUST_ENV_BENCH");
        key.push('\0');
        key.push_str(" @ @");
        let mut value = String::from("fp2p bs2fp $4 xyz");
        value.push('\0');
        value.push_str(" @ @");
        let action = format!(
            "IO.>> ^setenv {key} @ {value} @ #1 @ @ IO.lazyBind ^getenv {key} @ @ ^strlen @ @"
        );
        let mut expr = action.clone();
        for _ in 1..size {
            expr = format!("IO.>> {expr} @ {action} @");
        }
        return Ok(format!("v8.4\n0\nIO.performIO {expr} @ }}\n").into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("errno-chain:") {
        let size = parse_scenario_size("errno-chain", size)?;
        let action = "IO.lazyBind ^&errno @ S S K IO.>> @ @ S S K ^poke_int @ @ I @ @ K #2 @ @ @ @ S K ^peek_int @ @ I @ @ @";
        let mut expr = action.to_owned();
        for _ in 1..size {
            expr = format!("IO.>> {expr} @ {action} @");
        }
        return Ok(format!("v8.4\n0\nIO.performIO {expr} @ }}\n").into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("getcwd-chain:") {
        let size = parse_scenario_size("getcwd-chain", size)?;
        let action = "IO.lazyBind ^calloc #1 @ #4096 @ @ S S K IO.>> @ @ S S K ^getcwd @ @ I @ @ K #4096 @ @ @ @ S K ^strlen @ @ I @ @ @";
        let mut expr = action.to_owned();
        for _ in 1..size {
            expr = format!("IO.>> {expr} @ {action} @");
        }
        return Ok(format!("v8.4\n0\nIO.performIO {expr} @ }}\n").into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("dir-read-chain:") {
        let size = parse_scenario_size("dir-read-chain", size)?;
        let mut path = String::from("fp2p bs2fp $2 .");
        path.push('\0');
        path.push_str(" @ @");
        let action = format!(
            "IO.lazyBind ^opendir {path} @ @ S S K IO.>> @ @ S S K IO.lazyBind @ @ S K ^readdir @ @ I @ @ @ K S S K IO.lazyBind @ @ S K ^c_d_name @ @ I @ @ @ K ^strlen @ @ @ @ @ @ S K ^closedir @ @ I @ @ @"
        );
        let mut expr = action.clone();
        for _ in 1..size {
            expr = format!("IO.>> {expr} @ {action} @");
        }
        return Ok(format!("v8.4\n0\nIO.performIO {expr} @ }}\n").into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("remove-missing-chain:") {
        let size = parse_scenario_size("remove-missing-chain", size)?;
        let mut path = String::from("fp2p bs2fp $28 microhs-rust-missing-remove");
        path.push('\0');
        path.push_str(" @ @");
        let action = format!("^remove {path} @");
        let mut expr = action.clone();
        for _ in 1..size {
            expr = format!("IO.>> {expr} @ {action} @");
        }
        return Ok(format!("v8.4\n0\nIO.performIO {expr} @ }}\n").into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("file-read-close-chain:") {
        let size = parse_scenario_size("file-read-close-chain", size)?;
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
        let mut expr = action.clone();
        for _ in 1..size {
            expr = format!("IO.>> {expr} @ {action} @");
        }
        return Ok(format!("v8.4\n0\nIO.performIO {expr} @ }}\n").into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("utf8-bfile-read-chain:") {
        let size = parse_scenario_size("utf8-bfile-read-chain", size)?;
        let mut open = b"IO.lazyBind ^openb_rd_mem fp2p bs2fp $3 ".to_vec();
        open.extend_from_slice(&[0xc0, 0x80, b'A']);
        open.extend_from_slice(b" @ @ @ #3 @ @ ^add_utf8 @");
        let mut action = b"IO.lazyBind ".to_vec();
        action.extend_from_slice(&open);
        action.extend_from_slice(b" @ ^getb @");
        let mut expr = action.clone();
        for _ in 1..size {
            let mut next = b"IO.>> ".to_vec();
            next.extend_from_slice(&expr);
            next.extend_from_slice(b" @ ");
            next.extend_from_slice(&action);
            next.extend_from_slice(b" @");
            expr = next;
        }
        let mut out = b"v8.4\n0\nIO.performIO ".to_vec();
        out.extend_from_slice(&expr);
        out.extend_from_slice(b" @ }\n");
        return Ok(out);
    }
    if let Some(size) = scenario.strip_prefix("crlf-bfile-read-chain:") {
        let size = parse_scenario_size("crlf-bfile-read-chain", size)?;
        let mut open = b"IO.lazyBind ^openb_rd_mem fp2p bs2fp $3 ".to_vec();
        open.extend_from_slice(b"\r\nA");
        open.extend_from_slice(b" @ @ @ #3 @ @ ^add_crlf @");
        let mut action = b"IO.lazyBind ".to_vec();
        action.extend_from_slice(&open);
        action.extend_from_slice(b" @ ^getb @");
        let mut expr = action.clone();
        for _ in 1..size {
            let mut next = b"IO.>> ".to_vec();
            next.extend_from_slice(&expr);
            next.extend_from_slice(b" @ ");
            next.extend_from_slice(&action);
            next.extend_from_slice(b" @");
            expr = next;
        }
        let mut out = b"v8.4\n0\nIO.performIO ".to_vec();
        out.extend_from_slice(&expr);
        out.extend_from_slice(b" @ }\n");
        return Ok(out);
    }
    if let Some(size) = scenario.strip_prefix("base64-bfile-read-chain:") {
        let size = parse_scenario_size("base64-bfile-read-chain", size)?;
        let mut open = b"IO.lazyBind ^openb_rd_mem fp2p bs2fp $6 ".to_vec();
        open.extend_from_slice(b"Q Q==\n");
        open.extend_from_slice(b" @ @ @ #6 @ @ ^add_base64_decoder @");
        let mut action = b"IO.lazyBind ".to_vec();
        action.extend_from_slice(&open);
        action.extend_from_slice(b" @ ^getb @");
        let mut expr = action.clone();
        for _ in 1..size {
            let mut next = b"IO.>> ".to_vec();
            next.extend_from_slice(&expr);
            next.extend_from_slice(b" @ ");
            next.extend_from_slice(&action);
            next.extend_from_slice(b" @");
            expr = next;
        }
        let mut out = b"v8.4\n0\nIO.performIO ".to_vec();
        out.extend_from_slice(&expr);
        out.extend_from_slice(b" @ }\n");
        return Ok(out);
    }
    if let Some(size) = scenario.strip_prefix("rle-bfile-read-chain:") {
        let size = parse_scenario_size("rle-bfile-read-chain", size)?;
        let mut open = b"IO.lazyBind ^openb_rd_mem fp2p bs2fp $2 ".to_vec();
        open.extend_from_slice(&[0x81, 0x01]);
        open.extend_from_slice(b" @ @ @ #2 @ @ ^add_rle_decompressor @");
        let mut action = b"IO.lazyBind ".to_vec();
        action.extend_from_slice(&open);
        action.extend_from_slice(b" @ ^getb @");
        let mut expr = action.clone();
        for _ in 1..size {
            let mut next = b"IO.>> ".to_vec();
            next.extend_from_slice(&expr);
            next.extend_from_slice(b" @ ");
            next.extend_from_slice(&action);
            next.extend_from_slice(b" @");
            expr = next;
        }
        let mut out = b"v8.4\n0\nIO.performIO ".to_vec();
        out.extend_from_slice(&expr);
        out.extend_from_slice(b" @ }\n");
        return Ok(out);
    }
    if let Some(size) = scenario.strip_prefix("lz77-bfile-read-chain:") {
        let size = parse_scenario_size("lz77-bfile-read-chain", size)?;
        let mut open = b"IO.lazyBind ^openb_rd_mem fp2p bs2fp $9 ".to_vec();
        open.extend_from_slice(b"LZ1");
        open.extend_from_slice(&[2, 0, 0, 0, 0, b'A']);
        open.extend_from_slice(b" @ @ @ #9 @ @ ^add_lz77_decompressor @");
        let mut action = b"IO.lazyBind ".to_vec();
        action.extend_from_slice(&open);
        action.extend_from_slice(b" @ ^getb @");
        let mut expr = action.clone();
        for _ in 1..size {
            let mut next = b"IO.>> ".to_vec();
            next.extend_from_slice(&expr);
            next.extend_from_slice(b" @ ");
            next.extend_from_slice(&action);
            next.extend_from_slice(b" @");
            expr = next;
        }
        let mut out = b"v8.4\n0\nIO.performIO ".to_vec();
        out.extend_from_slice(&expr);
        out.extend_from_slice(b" @ }\n");
        return Ok(out);
    }
    if let Some(size) = scenario.strip_prefix("bwt-bfile-read-chain:") {
        let size = parse_scenario_size("bwt-bfile-read-chain", size)?;
        let mut open = b"IO.lazyBind ^openb_rd_mem fp2p bs2fp $12 ".to_vec();
        open.extend_from_slice(b"BW1");
        open.extend_from_slice(&[1, 0, 0, 0, 0, 0, 0, 0, b'A']);
        open.extend_from_slice(b" @ @ @ #12 @ @ ^add_bwt_decompressor @");
        let mut action = b"IO.lazyBind ".to_vec();
        action.extend_from_slice(&open);
        action.extend_from_slice(b" @ ^getb @");
        let mut expr = action.clone();
        for _ in 1..size {
            let mut next = b"IO.>> ".to_vec();
            next.extend_from_slice(&expr);
            next.extend_from_slice(b" @ ");
            next.extend_from_slice(&action);
            next.extend_from_slice(b" @");
            expr = next;
        }
        let mut out = b"v8.4\n0\nIO.performIO ".to_vec();
        out.extend_from_slice(&expr);
        out.extend_from_slice(b" @ }\n");
        return Ok(out);
    }
    if let Some(size) = scenario.strip_prefix("lzma-bfile-read-chain:") {
        let size = parse_scenario_size("lzma-bfile-read-chain", size)?;
        let payload = lzma_envelope_for_bytes(b"A")?;
        let mut open =
            format!("IO.lazyBind ^openb_rd_mem fp2p bs2fp ${} ", payload.len()).into_bytes();
        open.extend_from_slice(&payload);
        open.extend_from_slice(
            format!(" @ @ @ #{} @ @ ^add_lzma_decompressor @", payload.len()).as_bytes(),
        );
        let mut action = b"IO.lazyBind ".to_vec();
        action.extend_from_slice(&open);
        action.extend_from_slice(b" @ ^getb @");
        let mut expr = action.clone();
        for _ in 1..size {
            let mut next = b"IO.>> ".to_vec();
            next.extend_from_slice(&expr);
            next.extend_from_slice(b" @ ");
            next.extend_from_slice(&action);
            next.extend_from_slice(b" @");
            expr = next;
        }
        let mut out = b"v8.4\n0\nIO.performIO ".to_vec();
        out.extend_from_slice(&expr);
        out.extend_from_slice(b" @ }\n");
        return Ok(out);
    }
    if let Some(size) = scenario.strip_prefix("buf-bfile-read-chain:") {
        let size = parse_scenario_size("buf-bfile-read-chain", size)?;
        let payload = "microhs-rust-buffered";
        let open = format!(
            "IO.lazyBind ^openb_rd_mem fp2p bs2fp \"{payload}\" @ @ @ #{} @ @ R #4 @ ^add_buf @ @",
            payload.len()
        );
        let action = format!("IO.lazyBind {open} @ ^getb @");
        let mut expr = action.clone();
        for _ in 1..size {
            expr = format!("IO.>> {expr} @ {action} @");
        }
        return Ok(format!("v8.4\n0\nIO.performIO {expr} @ }}\n").into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("bfile-read-chain:") {
        let size = parse_scenario_size("bfile-read-chain", size)?;
        let payload = "microhs-rust-bfile";
        let action = format!(
            "IO.lazyBind ^openb_rd_mem fp2p bs2fp \"{payload}\" @ @ @ #{} @ @ ^getb @",
            payload.len()
        );
        let mut expr = action.clone();
        for _ in 1..size {
            expr = format!("IO.>> {expr} @ {action} @");
        }
        return Ok(format!("v8.4\n0\nIO.performIO {expr} @ }}\n").into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("ptr-chain:") {
        let size = parse_scenario_size("ptr-chain", size)?;
        let mut expr = String::from("#0");
        for _ in 0..size {
            expr = format!("toInt toPtr {expr} @ @");
        }
        return Ok(format!("v8.4\n0\n{expr} }}\n").into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("mvar-chain:") {
        let size = parse_scenario_size("mvar-chain", size)?;
        let mut expr = String::from("#0");
        for idx in 0..size {
            expr = format!("O #{idx} @ {expr} @");
        }
        return Ok(format!(
            "v8.4\n0\nIO.performIO IO.lazyBind IO.newmvar @ IO.trytakemvar @ @ {expr} @ I @ }}\n"
        )
        .into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("rnf-chain:") {
        let size = parse_scenario_size("rnf-chain", size)?;
        let mut expr = String::from("#0");
        for idx in 0..size {
            expr = format!("O #{idx} @ {expr} @");
        }
        return Ok(format!("v8.4\n0\nrnf #0 @ {expr} @ }}\n").into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("stableptr-chain:") {
        let size = parse_scenario_size("stableptr-chain", size)?;
        let mut expr = String::from("#0");
        for idx in 0..size {
            expr = format!("O #{idx} @ {expr} @");
        }
        return Ok(format!("v8.4\n0\nIO.performIO SPnew {expr} @ @ }}\n").into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("weak-chain:") {
        let size = parse_scenario_size("weak-chain", size)?;
        let mut expr = String::from("#0");
        for idx in 0..size {
            expr = format!("O #{idx} @ {expr} @");
        }
        return Ok(format!(
            "v8.4\n0\nIO.performIO IO.lazyBind Wknew #0 @ {expr} @ @ Wkderef @ @ #0 @ I @ }}\n"
        )
        .into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("zoo-chain:") {
        let size = parse_scenario_size("zoo-chain", size)?;
        let mut expr = String::from("#1");
        for idx in 0..size {
            expr = match idx % 11 {
                0 => format!("S' K @ K @ K @ {expr} @ #0 @"),
                1 => format!("B' K @ {expr} @ K @ #0 @"),
                2 => format!("Z K @ {expr} @ #0 @ #1 @"),
                3 => format!("J {expr} @ #0 @ I @"),
                4 => format!("L {expr} @ I @ #0 @"),
                5 => format!("KK #0 @ {expr} @ #1 @"),
                6 => format!("KA #0 @ #1 @ {expr} @"),
                7 => format!("C' A @ K @ {expr} @ #0 @"),
                8 => format!("R #0 @ K @ {expr} @"),
                9 => format!("O {expr} @ #0 @ #1 @ K @"),
                _ => format!("C'B K @ K @ {expr} @ #0 @"),
            };
        }
        return Ok(format!("v8.4\n0\n{expr} }}\n").into_bytes());
    }
    if let Some(size) = scenario.strip_prefix("data-chain:") {
        let size = parse_scenario_size("data-chain", size)?;
        let mut expr = String::from("#1");
        for idx in 0..size {
            expr = if idx % 3 == 0 {
                format!("TAG{} {expr} @ A @", idx % 33)
            } else if idx % 3 == 1 {
                format!("T3 {expr} @ #0 @ #1 @ K3 @ #0 @")
            } else {
                format!("T4 {expr} @ #0 @ #1 @ #2 @ K4 @ #0 @")
            };
        }
        return Ok(format!("v8.4\n0\n{expr} }}\n").into_bytes());
    }
    Err(format!("unsupported scenario: {scenario}"))
}

fn parse_scenario_size(name: &str, size: &str) -> Result<usize, String> {
    let size: usize = size
        .parse()
        .map_err(|_| format!("invalid {name} size: {size}"))?;
    if size == 0 {
        return Err(format!("{name} size must be greater than zero"));
    }
    Ok(size)
}

fn ascii_payload(name: &str, size: &str) -> Result<String, String> {
    let size = parse_scenario_size(name, size)?;
    let mut bytes = String::with_capacity(size);
    for idx in 0..size {
        bytes.push((b'a' + (idx % 26) as u8) as char);
    }
    Ok(bytes)
}

fn lzma_envelope_for_bytes(input: &[u8]) -> Result<Vec<u8>, String> {
    let props = lzma_sdk_rs::LzmaProps::for_level(5, u32::MAX);
    let raw = lzma_sdk_rs::encode(input, &props);
    let mut payload = Vec::with_capacity(20 + raw.len());
    payload.extend_from_slice(b"LZ2");
    let compressed_len = 13usize
        .checked_add(raw.len())
        .ok_or("lzma payload length overflow")?;
    let compressed_len =
        u32::try_from(compressed_len).map_err(|_| "lzma payload too large for envelope")?;
    payload.extend_from_slice(&compressed_len.to_le_bytes());
    payload.extend_from_slice(&lzma_sdk_rs::decoder_props(&props));
    let input_len = u64::try_from(input.len()).map_err(|_| "lzma input length overflow")?;
    payload.extend_from_slice(&input_len.to_le_bytes());
    payload.extend_from_slice(&raw);
    Ok(payload)
}
