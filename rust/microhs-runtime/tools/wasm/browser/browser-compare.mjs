import createCModule from "./browser-bench-c.mjs";
import { instantiateMicroHsRuntime } from "./host.mjs";

const encoder = new TextEncoder();

const DEFAULT_SCENARIOS = [
  "identity-chain:1000",
  "arith-chain:200",
  "io-control-chain:200",
  "performio-apply-chain:200",
  "ffi-mem-chain:200",
  "bfile-read-chain:200",
  "zoo-chain:300",
  "data-chain:300",
];
const DEFAULT_ITERS = 200;
const DEFAULT_WARMUP_ITERS = 20;
const DEFAULT_REDUCE_LIMIT = 1_000_000_000;

export async function runComparison(options = {}) {
  const scenarios = options.scenarios?.length ? options.scenarios : DEFAULT_SCENARIOS;
  const iters = options.iters ?? DEFAULT_ITERS;
  const warmupIters = options.warmupIters ?? DEFAULT_WARMUP_ITERS;
  const reduceLimit = options.reduceLimit ?? DEFAULT_REDUCE_LIMIT;
  const rustWasm =
    options.rustWasm ??
    new URL("../../../../../target/wasm32-unknown-unknown/release/microhs_runtime.wasm", import.meta.url);
  const runtime = await instantiateMicroHsRuntime(rustWasm);
  const rows = [];

  for (const scenario of scenarios) {
    const input = makeScenario(scenario);
    const rust = runRustScenario(runtime, input, { iters, warmupIters, reduceLimit });
    const c = await runCScenario(input, { iters, warmupIters });
    const row = {
      scenario,
      iters,
      warmupIters,
      bytes: input.length,
      rust,
      c,
      ratio: rust.nsPerIter / c.nsPerIter,
      sinkMatch: rust.sink === c.sink,
    };
    rows.push(row);
    options.onRow?.(row);
  }

  return rows;
}

function runRustScenario(runtime, input, options) {
  for (let idx = 0; idx < options.warmupIters; idx += 1) {
    runRustOnce(runtime, input, options.reduceLimit);
  }

  let sink = 0;
  let stepChunks = 0;
  const start = performance.now();
  for (let idx = 0; idx < options.iters; idx += 1) {
    const once = runRustOnce(runtime, input, options.reduceLimit);
    sink += once.sink;
    stepChunks += once.stepChunks;
  }
  const totalMs = performance.now() - start;
  return {
    totalMs,
    nsPerIter: (totalMs * 1_000_000) / options.iters,
    sink,
    stepChunks,
  };
}

function runRustOnce(runtime, input, reduceLimit) {
  const handle = runtime.newProgram(input);
  let stepChunks = 0;
  try {
    while (!runtime.reduce(handle, reduceLimit)) {
      stepChunks += 1;
      if (stepChunks > 1000) {
        throw new Error("Rust reduce did not finish within 1000 chunks");
      }
    }
    return {
      sink: bytesSink(runtime.serialize(handle)),
      stepChunks,
    };
  } finally {
    runtime.freeProgram(handle);
  }
}

async function runCScenario(input, options) {
  const stdout = [];
  const stderr = [];
  const module = await createCModule({
    noInitialRun: true,
    print(line) {
      stdout.push(line);
    },
    printErr(line) {
      stderr.push(line);
    },
  });
  module.FS.writeFile("/input.comb", input);

  try {
    module.callMain([
      "--mode",
      "whnf",
      "--warmup-iters",
      String(options.warmupIters),
      "--iters",
      String(options.iters),
      "/input.comb",
    ]);
  } catch (error) {
    if (!isEmscriptenExitZero(error)) {
      throw new Error(`C benchmark failed: ${String(error)}\n${stderr.join("\n")}`);
    }
  }

  const parsed = parseCOutput(stdout.join("\n"));
  if (!Number.isFinite(parsed.nsPerIter) || !Number.isFinite(parsed.sink)) {
    throw new Error(`C benchmark output was incomplete:\n${stdout.join("\n")}`);
  }
  return parsed;
}

function isEmscriptenExitZero(error) {
  return error?.status === 0 || /exit\(0\)/.test(String(error));
}

function parseCOutput(output) {
  const parsed = {};
  for (const line of output.split(/\r?\n/)) {
    const idx = line.indexOf(": ");
    if (idx < 0) continue;
    parsed[line.slice(0, idx)] = line.slice(idx + 2);
  }
  return {
    totalMs: Number(parsed.c_parse_eval_serialize_total_ms),
    nsPerIter: Number(parsed.c_parse_eval_serialize_ns_per_iter),
    sink: Number(parsed.c_bench_sink),
  };
}

function bytesSink(bytes) {
  let sink = bytes.length;
  if (bytes.length > 0) {
    sink += bytes[0];
  }
  return sink;
}

export function makeScenario(scenario) {
  const [name, rawSize] = scenario.split(":");
  const size = Number(rawSize);
  if (!Number.isSafeInteger(size) || size <= 0) {
    throw new Error(`invalid scenario size: ${scenario}`);
  }

  switch (name) {
    case "identity-chain":
      return encode(identityChain(size));
    case "arith-chain":
      return encode(arithChain(size));
    case "io-control-chain":
      return encode(ioControlChain(size));
    case "performio-apply-chain":
      return encode(performIoApplyChain(size));
    case "ffi-mem-chain":
      return encode(ffiMemChain(size));
    case "bfile-read-chain":
      return encode(bfileReadChain(size));
    case "zoo-chain":
      return encode(zooChain(size));
    case "data-chain":
      return encode(dataChain(size));
    default:
      throw new Error(`unsupported browser scenario: ${scenario}`);
  }
}

function encode(text) {
  return encoder.encode(text);
}

function identityChain(size) {
  let out = "v8.4\n0\nI";
  for (let idx = 1; idx < size; idx += 1) {
    out += " I @";
  }
  return `${out} #1 @ }\n`;
}

function arithChain(size) {
  let expr = "#0";
  for (let idx = 0; idx < size; idx += 1) {
    expr = `+ ${expr} @ #1 @`;
  }
  return `v8.4\n0\n${expr} }\n`;
}

function ioControlChain(size) {
  let expr = "IO.getmaskingstate";
  for (let idx = 0; idx < size; idx += 1) {
    expr = `IO.>> IO.setmaskingstate #${idx % 3} @ @ ${expr} @`;
  }
  return `v8.4\n0\nIO.performIO ${expr} @ }\n`;
}

function performIoApplyChain(size) {
  let expr = "#0";
  for (let idx = 0; idx < size; idx += 1) {
    expr = `IO.performIO IO.return I @ @ ${expr} @`;
  }
  return `v8.4\n0\n${expr} }\n`;
}

function ffiMemChain(size) {
  const action = "IO.lazyBind ^calloc #1 @ #1 @ @ ^peek_uint8 @";
  let expr = action;
  for (let idx = 1; idx < size; idx += 1) {
    expr = `IO.>> ${expr} @ ${action} @`;
  }
  return `v8.4\n0\nIO.performIO ${expr} @ }\n`;
}

function bfileReadChain(size) {
  const payload = "microhs-rust-bfile";
  const action = `IO.lazyBind ^openb_rd_mem fp2p bs2fp "${payload}" @ @ @ #${payload.length} @ @ ^getb @`;
  let expr = action;
  for (let idx = 1; idx < size; idx += 1) {
    expr = `IO.>> ${expr} @ ${action} @`;
  }
  return `v8.4\n0\nIO.performIO ${expr} @ }\n`;
}

function zooChain(size) {
  let expr = "#1";
  for (let idx = 0; idx < size; idx += 1) {
    switch (idx % 11) {
      case 0:
        expr = `S' K @ K @ K @ ${expr} @ #0 @`;
        break;
      case 1:
        expr = `B' K @ ${expr} @ K @ #0 @`;
        break;
      case 2:
        expr = `Z K @ ${expr} @ #0 @ #1 @`;
        break;
      case 3:
        expr = `J ${expr} @ #0 @ I @`;
        break;
      case 4:
        expr = `L ${expr} @ I @ #0 @`;
        break;
      case 5:
        expr = `KK #0 @ ${expr} @ #1 @`;
        break;
      case 6:
        expr = `KA #0 @ #1 @ ${expr} @`;
        break;
      case 7:
        expr = `C' A @ K @ ${expr} @ #0 @`;
        break;
      case 8:
        expr = `R #0 @ K @ ${expr} @`;
        break;
      case 9:
        expr = `O ${expr} @ #0 @ #1 @ K @`;
        break;
      default:
        expr = `C'B K @ K @ ${expr} @ #0 @`;
        break;
    }
  }
  return `v8.4\n0\n${expr} }\n`;
}

function dataChain(size) {
  let expr = "#1";
  for (let idx = 0; idx < size; idx += 1) {
    if (idx % 3 === 0) {
      expr = `TAG${idx % 33} ${expr} @ A @`;
    } else if (idx % 3 === 1) {
      expr = `T3 ${expr} @ #0 @ #1 @ K3 @ #0 @`;
    } else {
      expr = `T4 ${expr} @ #0 @ #1 @ #2 @ K4 @ #0 @`;
    }
  }
  return `v8.4\n0\n${expr} }\n`;
}

function parseArgs(argv) {
  const options = {
    scenarios: [],
    iters: DEFAULT_ITERS,
    warmupIters: DEFAULT_WARMUP_ITERS,
    reduceLimit: DEFAULT_REDUCE_LIMIT,
    json: false,
  };

  for (let idx = 0; idx < argv.length; idx += 1) {
    const arg = argv[idx];
    switch (arg) {
      case "--scenario":
        options.scenarios.push(requiredValue(argv, ++idx, arg));
        break;
      case "--iters":
        options.iters = parsePositiveInt(requiredValue(argv, ++idx, arg), arg);
        break;
      case "--warmup-iters":
        options.warmupIters = parseNonNegativeInt(requiredValue(argv, ++idx, arg), arg);
        break;
      case "--reduce-limit":
        options.reduceLimit = parsePositiveInt(requiredValue(argv, ++idx, arg), arg);
        break;
      case "--json":
        options.json = true;
        break;
      case "-h":
      case "--help":
        printUsage();
        process.exit(0);
        break;
      default:
        throw new Error(`unknown argument: ${arg}`);
    }
  }

  return options;
}

function requiredValue(argv, idx, arg) {
  if (idx >= argv.length) {
    throw new Error(`${arg} requires a value`);
  }
  return argv[idx];
}

function parsePositiveInt(value, arg) {
  const parsed = Number(value);
  if (!Number.isSafeInteger(parsed) || parsed <= 0) {
    throw new Error(`${arg} must be a positive integer`);
  }
  return parsed;
}

function parseNonNegativeInt(value, arg) {
  const parsed = Number(value);
  if (!Number.isSafeInteger(parsed) || parsed < 0) {
    throw new Error(`${arg} must be a non-negative integer`);
  }
  return parsed;
}

function printUsage() {
  console.log(
    "usage: node rust/microhs-runtime/tools/wasm/browser/browser-compare.mjs " +
      "[--iters N] [--warmup-iters N] [--scenario NAME:N] [--json]"
  );
}

function printRow(row) {
  const ratio = row.ratio.toFixed(2);
  const sink = row.sinkMatch ? "match" : `mismatch rust=${row.rust.sink} c=${row.c.sink}`;
  console.log(
    `${row.scenario.padEnd(26)} ` +
      `rust=${row.rust.nsPerIter.toFixed(0).padStart(10)} ns/iter ` +
      `c=${row.c.nsPerIter.toFixed(0).padStart(10)} ns/iter ` +
      `ratio=${ratio.padStart(5)} sink=${sink}`
  );
}

function isNodeMain() {
  if (typeof process === "undefined" || !process.argv?.[1]) {
    return false;
  }
  const script = process.argv[1].startsWith("/")
    ? process.argv[1]
    : `${process.cwd()}/${process.argv[1]}`;
  return import.meta.url === new URL(`file://${script}`).href;
}

if (isNodeMain()) {
  try {
    const options = parseArgs(process.argv.slice(2));
    const rows = await runComparison({
      ...options,
      onRow: options.json ? undefined : printRow,
    });
    if (options.json) {
      console.log(JSON.stringify(rows, null, 2));
    }
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exit(1);
  }
}
