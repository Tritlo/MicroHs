use super::parse_scenario_size;

pub(super) fn make_scenario(scenario: &str) -> Option<Result<Vec<u8>, String>> {
    if let Some(size) = scenario.strip_prefix("array-chain:") {
        return Some(parse_scenario_size("array-chain", size).map(array_chain));
    }
    if let Some(size) = scenario.strip_prefix("io-chain:") {
        return Some(parse_scenario_size("io-chain", size).map(io_chain));
    }
    if let Some(size) = scenario.strip_prefix("io-array-chain:") {
        return Some(parse_scenario_size("io-array-chain", size).map(array_chain));
    }
    if let Some(size) = scenario.strip_prefix("io-bytes-chain:") {
        return Some(io_bytes_chain(size));
    }
    if let Some(size) = scenario.strip_prefix("io-control-chain:") {
        return Some(parse_scenario_size("io-control-chain", size).map(io_control_chain));
    }
    if let Some(size) = scenario.strip_prefix("performio-apply-chain:") {
        return Some(parse_scenario_size("performio-apply-chain", size).map(performio_apply_chain));
    }
    if let Some(size) = scenario.strip_prefix("argref-chain:") {
        return Some(parse_scenario_size("argref-chain", size).map(argref_chain));
    }
    if let Some(size) = scenario.strip_prefix("stdio-chain:") {
        return Some(parse_scenario_size("stdio-chain", size).map(stdio_chain));
    }
    if let Some(size) = scenario.strip_prefix("mvar-chain:") {
        return Some(parse_scenario_size("mvar-chain", size).map(mvar_chain));
    }
    if let Some(size) = scenario.strip_prefix("rnf-chain:") {
        return Some(parse_scenario_size("rnf-chain", size).map(rnf_chain));
    }
    if let Some(size) = scenario.strip_prefix("stableptr-chain:") {
        return Some(parse_scenario_size("stableptr-chain", size).map(stableptr_chain));
    }
    if let Some(size) = scenario.strip_prefix("weak-chain:") {
        return Some(parse_scenario_size("weak-chain", size).map(weak_chain));
    }
    None
}

fn array_chain(size: usize) -> Vec<u8> {
    let last = size - 1;
    let mut items = String::new();
    for _ in 0..size {
        items.push_str("#0 ");
    }
    format!(
        "v8.4\n1\nIO.performIO IO.>> A.write {items}[{size}] :0 @ #{last} @ #42 @ @ A.read _0 @ #{last} @ @ @ }}\n"
    )
    .into_bytes()
}

fn io_chain(size: usize) -> Vec<u8> {
    let mut expr = String::from("IO.return #0 @");
    for idx in 1..size {
        expr = format!("IO.>> {expr} @ IO.return #{idx} @ @");
    }
    format!("v8.4\n0\nIO.performIO {expr} @ }}\n").into_bytes()
}

fn io_bytes_chain(size: &str) -> Result<Vec<u8>, String> {
    let bytes = super::ascii_payload("io-bytes-chain", size)?;
    let last = bytes.len() - 1;
    Ok(format!(
        "v8.4\n1\nIO.performIO IO.>> bswrite \"{bytes}\" :0 @ #{last} @ #42 @ @ bsread _0 @ #{last} @ @ @ }}\n"
    )
    .into_bytes())
}

fn io_control_chain(size: usize) -> Vec<u8> {
    let mut expr = String::from("IO.getmaskingstate");
    for idx in 0..size {
        expr = format!("IO.>> IO.setmaskingstate #{} @ @ {expr} @", idx % 3);
    }
    format!("v8.4\n0\nIO.performIO {expr} @ }}\n").into_bytes()
}

fn performio_apply_chain(size: usize) -> Vec<u8> {
    let mut expr = String::from("#0");
    for _ in 0..size {
        expr = format!("IO.performIO IO.return I @ @ {expr} @");
    }
    format!("v8.4\n0\n{expr} }}\n").into_bytes()
}

fn argref_chain(size: usize) -> Vec<u8> {
    let mut expr = String::from("IO.getArgRef");
    for _ in 1..size {
        expr = format!("IO.>> {expr} @ IO.getArgRef @");
    }
    format!("v8.4\n0\nIO.performIO {expr} @ }}\n").into_bytes()
}

fn stdio_chain(size: usize) -> Vec<u8> {
    let mut expr = String::from("#0");
    for _ in 0..size {
        expr = format!("seq toInt fp2p IO.stdout @ @ @ {expr} @");
    }
    format!("v8.4\n0\n{expr} }}\n").into_bytes()
}

fn cons_chain(size: usize) -> String {
    let mut expr = String::from("#0");
    for idx in 0..size {
        expr = format!("O #{idx} @ {expr} @");
    }
    expr
}

fn mvar_chain(size: usize) -> Vec<u8> {
    let expr = cons_chain(size);
    format!("v8.4\n0\nIO.performIO IO.lazyBind IO.newmvar @ IO.trytakemvar @ @ {expr} @ I @ }}\n")
        .into_bytes()
}

fn rnf_chain(size: usize) -> Vec<u8> {
    let expr = cons_chain(size);
    format!("v8.4\n0\nrnf #0 @ {expr} @ }}\n").into_bytes()
}

fn stableptr_chain(size: usize) -> Vec<u8> {
    let expr = cons_chain(size);
    format!("v8.4\n0\nIO.performIO SPnew {expr} @ @ }}\n").into_bytes()
}

fn weak_chain(size: usize) -> Vec<u8> {
    let expr = cons_chain(size);
    format!("v8.4\n0\nIO.performIO IO.lazyBind Wknew #0 @ {expr} @ @ Wkderef @ @ #0 @ I @ }}\n")
        .into_bytes()
}
