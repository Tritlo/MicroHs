use super::{lzma_envelope_for_bytes, parse_scenario_size, repeated_io_then};

pub(super) fn make_scenario(scenario: &str) -> Option<Result<Vec<u8>, String>> {
    if let Some(size) = scenario.strip_prefix("utf8-bfile-read-chain:") {
        return Some(
            parse_scenario_size("utf8-bfile-read-chain", size).map(|size| {
                bfile_read_chain(
                    size,
                    b"IO.lazyBind ^openb_rd_mem fp2p bs2fp $3 ",
                    &[0xc0, 0x80, b'A'],
                    b" @ @ @ #3 @ @ ^add_utf8 @",
                )
            }),
        );
    }
    if let Some(size) = scenario.strip_prefix("crlf-bfile-read-chain:") {
        return Some(
            parse_scenario_size("crlf-bfile-read-chain", size).map(|size| {
                bfile_read_chain(
                    size,
                    b"IO.lazyBind ^openb_rd_mem fp2p bs2fp $3 ",
                    b"\r\nA",
                    b" @ @ @ #3 @ @ ^add_crlf @",
                )
            }),
        );
    }
    if let Some(size) = scenario.strip_prefix("base64-bfile-read-chain:") {
        return Some(
            parse_scenario_size("base64-bfile-read-chain", size).map(|size| {
                bfile_read_chain(
                    size,
                    b"IO.lazyBind ^openb_rd_mem fp2p bs2fp $6 ",
                    b"Q Q==\n",
                    b" @ @ @ #6 @ @ ^add_base64_decoder @",
                )
            }),
        );
    }
    if let Some(size) = scenario.strip_prefix("rle-bfile-read-chain:") {
        return Some(
            parse_scenario_size("rle-bfile-read-chain", size).map(|size| {
                bfile_read_chain(
                    size,
                    b"IO.lazyBind ^openb_rd_mem fp2p bs2fp $2 ",
                    &[0x81, 0x01],
                    b" @ @ @ #2 @ @ ^add_rle_decompressor @",
                )
            }),
        );
    }
    if let Some(size) = scenario.strip_prefix("lz77-bfile-read-chain:") {
        return Some(
            parse_scenario_size("lz77-bfile-read-chain", size).map(|size| {
                let mut payload = b"LZ1".to_vec();
                payload.extend_from_slice(&[2, 0, 0, 0, 0, b'A']);
                bfile_read_chain(
                    size,
                    b"IO.lazyBind ^openb_rd_mem fp2p bs2fp $9 ",
                    &payload,
                    b" @ @ @ #9 @ @ ^add_lz77_decompressor @",
                )
            }),
        );
    }
    if let Some(size) = scenario.strip_prefix("bwt-bfile-read-chain:") {
        return Some(
            parse_scenario_size("bwt-bfile-read-chain", size).map(|size| {
                let mut payload = b"BW1".to_vec();
                payload.extend_from_slice(&[1, 0, 0, 0, 0, 0, 0, 0, b'A']);
                bfile_read_chain(
                    size,
                    b"IO.lazyBind ^openb_rd_mem fp2p bs2fp $12 ",
                    &payload,
                    b" @ @ @ #12 @ @ ^add_bwt_decompressor @",
                )
            }),
        );
    }
    if let Some(size) = scenario.strip_prefix("lzma-bfile-read-chain:") {
        return Some(lzma_bfile_read_chain(size));
    }
    if let Some(size) = scenario.strip_prefix("buf-bfile-read-chain:") {
        return Some(parse_scenario_size("buf-bfile-read-chain", size).map(buf_bfile_read_chain));
    }
    if let Some(size) = scenario.strip_prefix("bfile-read-chain:") {
        return Some(parse_scenario_size("bfile-read-chain", size).map(plain_bfile_read_chain));
    }
    None
}

fn bfile_read_chain(size: usize, prefix: &[u8], payload: &[u8], suffix: &[u8]) -> Vec<u8> {
    let mut open = prefix.to_vec();
    open.extend_from_slice(payload);
    open.extend_from_slice(suffix);
    let mut action = b"IO.lazyBind ".to_vec();
    action.extend_from_slice(&open);
    action.extend_from_slice(b" @ ^getb @");
    repeated_io_then(size, &action)
}

fn lzma_bfile_read_chain(size: &str) -> Result<Vec<u8>, String> {
    let size = parse_scenario_size("lzma-bfile-read-chain", size)?;
    let payload = lzma_envelope_for_bytes(b"A")?;
    let mut open = format!("IO.lazyBind ^openb_rd_mem fp2p bs2fp ${} ", payload.len()).into_bytes();
    open.extend_from_slice(&payload);
    open.extend_from_slice(
        format!(" @ @ @ #{} @ @ ^add_lzma_decompressor @", payload.len()).as_bytes(),
    );
    let mut action = b"IO.lazyBind ".to_vec();
    action.extend_from_slice(&open);
    action.extend_from_slice(b" @ ^getb @");
    Ok(repeated_io_then(size, &action))
}

fn buf_bfile_read_chain(size: usize) -> Vec<u8> {
    let payload = "microhs-rust-buffered";
    let open = format!(
        "IO.lazyBind ^openb_rd_mem fp2p bs2fp \"{payload}\" @ @ @ #{} @ @ R #4 @ ^add_buf @ @",
        payload.len()
    );
    let action = format!("IO.lazyBind {open} @ ^getb @");
    repeat_action(size, &action)
}

fn plain_bfile_read_chain(size: usize) -> Vec<u8> {
    let payload = "microhs-rust-bfile";
    let action = format!(
        "IO.lazyBind ^openb_rd_mem fp2p bs2fp \"{payload}\" @ @ @ #{} @ @ ^getb @",
        payload.len()
    );
    repeat_action(size, &action)
}

fn repeat_action(size: usize, action: &str) -> Vec<u8> {
    let mut expr = action.to_owned();
    for _ in 1..size {
        expr = format!("IO.>> {expr} @ {action} @");
    }
    format!("v8.4\n0\nIO.performIO {expr} @ }}\n").into_bytes()
}
