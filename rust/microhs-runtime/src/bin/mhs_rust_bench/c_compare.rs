struct CInProcessBench {
    mode: BenchMode,
    ns_per_iter: f64,
    sink: usize,
}

fn bench_c_mhsbench(
    input: &[u8],
    c_mhsbench: &str,
    mode: BenchMode,
    program_args: &[Vec<u8>],
    warmup_iters: usize,
    iters: usize,
) -> Result<CInProcessBench, String> {
    let file = temp_comb_file();
    fs::write(&file, input).map_err(|err| format!("{}: {err}", file.display()))?;
    let iters_arg = iters.to_string();
    let warmup_iters_arg = warmup_iters.to_string();
    let file_arg = file.to_str().ok_or("non-utf8 temp file")?;
    let mut command = Command::new(c_mhsbench);
    command.args([
        "--mode",
        mode.as_str(),
        "--warmup-iters",
        &warmup_iters_arg,
        "--iters",
        &iters_arg,
        file_arg,
    ]);
    if !program_args.is_empty() {
        command.arg("--");
        for arg in program_args {
            command.arg(String::from_utf8_lossy(arg).as_ref());
        }
    }
    let output = command
        .output()
        .map_err(|err| format!("{c_mhsbench}: {err}"))?;
    let _ = fs::remove_file(&file);
    if !output.status.success() {
        return Err(format!(
            "{c_mhsbench} exited with {}\n{}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let stdout = String::from_utf8(output.stdout).map_err(|_| "non-utf8 mhsbench output")?;
    let ns_per_iter = stdout
        .lines()
        .find_map(|line| {
            line.strip_prefix("c_parse_eval_serialize_ns_per_iter: ")
                .or_else(|| line.strip_prefix("c_parse_reduce_serialize_ns_per_iter: "))
        })
        .ok_or_else(|| format!("missing timing in mhsbench output:\n{stdout}"))?
        .parse()
        .map_err(|_| format!("invalid timing in mhsbench output:\n{stdout}"))?;
    let sink = stdout
        .lines()
        .find_map(|line| line.strip_prefix("c_bench_sink: "))
        .ok_or_else(|| format!("missing sink in mhsbench output:\n{stdout}"))?
        .parse()
        .map_err(|_| format!("invalid sink in mhsbench output:\n{stdout}"))?;
    Ok(CInProcessBench {
        mode,
        ns_per_iter,
        sink,
    })
}

fn temp_comb_file() -> std::path::PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    env::temp_dir().join(format!(
        "mhs-rust-bench-{}-{stamp}.comb",
        std::process::id()
    ))
}
