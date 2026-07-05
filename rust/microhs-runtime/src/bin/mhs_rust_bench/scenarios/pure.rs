use super::{ascii_payload, parse_scenario_size};

pub(super) fn make_scenario(scenario: &str) -> Option<Result<Vec<u8>, String>> {
    if let Some(size) = scenario.strip_prefix("identity-chain:") {
        let size = parse_scenario_size("identity-chain", size);
        return Some(size.map(identity_chain));
    }
    if let Some(size) = scenario.strip_prefix("arith-chain:") {
        let size = parse_scenario_size("arith-chain", size);
        return Some(size.map(|size| numeric_chain("#0", size, "+", "#1")));
    }
    if let Some(size) = scenario.strip_prefix("int64-chain:") {
        let size = parse_scenario_size("int64-chain", size);
        return Some(size.map(|size| numeric_chain("##0", size, "I+", "##1")));
    }
    if let Some(size) = scenario.strip_prefix("float64-chain:") {
        let size = parse_scenario_size("float64-chain", size);
        return Some(size.map(|size| numeric_chain("&0", size, "d+", "&1.25")));
    }
    if let Some(size) = scenario.strip_prefix("float32-chain:") {
        let size = parse_scenario_size("float32-chain", size);
        return Some(size.map(|size| numeric_chain("&&0", size, "f+", "&&1.25")));
    }
    if let Some(size) = scenario.strip_prefix("bytes-chain:") {
        return Some(parse_scenario_size("bytes-chain", size).map(bytes_chain));
    }
    if let Some(size) = scenario.strip_prefix("foreignptr-slice:") {
        return Some(foreignptr_slice(size));
    }
    if let Some(size) = scenario.strip_prefix("cstring-pack:") {
        return Some(cstring_pack(size));
    }
    if let Some(size) = scenario.strip_prefix("unpack-chain:") {
        return Some(unpack_chain(size));
    }
    if let Some(size) = scenario.strip_prefix("fromutf8-chain:") {
        return Some(fromutf8_chain(size));
    }
    if let Some(size) = scenario.strip_prefix("ptr-chain:") {
        return Some(parse_scenario_size("ptr-chain", size).map(ptr_chain));
    }
    if let Some(size) = scenario.strip_prefix("zoo-chain:") {
        return Some(parse_scenario_size("zoo-chain", size).map(zoo_chain));
    }
    if let Some(size) = scenario.strip_prefix("data-chain:") {
        return Some(parse_scenario_size("data-chain", size).map(data_chain));
    }
    None
}

fn identity_chain(size: usize) -> Vec<u8> {
    let mut out = String::from("v8.4\n0\nI");
    for _ in 1..size {
        out.push_str(" I @");
    }
    out.push_str(" #1 @ }\n");
    out.into_bytes()
}

fn numeric_chain(initial: &str, size: usize, op: &str, step: &str) -> Vec<u8> {
    let mut expr = initial.to_owned();
    for _ in 0..size {
        expr = format!("{op} {expr} @ {step} @");
    }
    format!("v8.4\n0\n{expr} }}\n").into_bytes()
}

fn bytes_chain(size: usize) -> Vec<u8> {
    let mut expr = String::from("\"\"");
    for idx in 0..size {
        let byte = (b'a' + (idx % 26) as u8) as char;
        expr = format!("bs++ {expr} @ \"{byte}\" @");
    }
    format!("v8.4\n0\n{expr} }}\n").into_bytes()
}

fn foreignptr_slice(size: &str) -> Result<Vec<u8>, String> {
    let bytes = ascii_payload("foreignptr-slice", size)?;
    let len = bytes.len() - 1;
    Ok(format!("v8.4\n0\nfp2bs fp+ bs2fp \"{bytes}\" @ @ #1 @ @ #{len} @ }}\n").into_bytes())
}

fn cstring_pack(size: &str) -> Result<Vec<u8>, String> {
    let bytes = ascii_payload("cstring-pack", size)?;
    let len = bytes.len();
    Ok(
        format!(
            "v8.4\n0\nIO.performIO packCStringLen fp2p bs2fp \"{bytes}\" @ @ @ #{len} @ @ }}\n"
        )
        .into_bytes(),
    )
}

fn unpack_chain(size: &str) -> Result<Vec<u8>, String> {
    let bytes = ascii_payload("unpack-chain", size)?;
    Ok(format!("v8.4\n0\nbsunpack \"{bytes}\" @ #0 @ K @ }}\n").into_bytes())
}

fn fromutf8_chain(size: &str) -> Result<Vec<u8>, String> {
    let bytes = ascii_payload("fromutf8-chain", size)?;
    Ok(format!("v8.4\n0\nfromUTF8 \"{bytes}\" @ #0 @ K @ }}\n").into_bytes())
}

fn ptr_chain(size: usize) -> Vec<u8> {
    let mut expr = String::from("#0");
    for _ in 0..size {
        expr = format!("toInt toPtr {expr} @ @");
    }
    format!("v8.4\n0\n{expr} }}\n").into_bytes()
}

fn zoo_chain(size: usize) -> Vec<u8> {
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
    format!("v8.4\n0\n{expr} }}\n").into_bytes()
}

fn data_chain(size: usize) -> Vec<u8> {
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
    format!("v8.4\n0\n{expr} }}\n").into_bytes()
}
