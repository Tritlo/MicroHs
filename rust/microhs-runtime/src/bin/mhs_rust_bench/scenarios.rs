#[path = "scenarios/bfile.rs"]
mod bfile;
#[path = "scenarios/ffi.rs"]
mod ffi;
#[path = "scenarios/io.rs"]
mod io;
#[path = "scenarios/pure.rs"]
mod pure;

pub(super) fn make_scenario(scenario: &str) -> Result<Vec<u8>, String> {
    pure::make_scenario(scenario)
        .or_else(|| io::make_scenario(scenario))
        .or_else(|| ffi::make_scenario(scenario))
        .or_else(|| bfile::make_scenario(scenario))
        .unwrap_or_else(|| Err(format!("unsupported scenario: {scenario}")))
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

fn repeated_io_then(size: usize, action: &[u8]) -> Vec<u8> {
    let mut expr = action.to_vec();
    for _ in 1..size {
        let mut next = b"IO.>> ".to_vec();
        next.extend_from_slice(&expr);
        next.extend_from_slice(b" @ ");
        next.extend_from_slice(action);
        next.extend_from_slice(b" @");
        expr = next;
    }
    let mut out = b"v8.4\n0\nIO.performIO ".to_vec();
    out.extend_from_slice(&expr);
    out.extend_from_slice(b" @ }\n");
    out
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
