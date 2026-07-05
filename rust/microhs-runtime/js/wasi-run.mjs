#!/usr/bin/env node
import { readFile } from "node:fs/promises";
import process from "node:process";
import { WASI } from "node:wasi";

const jsFfiImports = [
  "mhs_js_debug",
  "mhs_js_call_int",
  "mhs_js_haserr",
  "mhs_js_logerr",
  "mhs_js_call_ptr",
  "mhs_js_eval_run",
  "mhs_js_call_bool",
  "mhs_js_call_uint",
  "mhs_js_call_void",
  "mhs_js_eval_call",
  "mhs_js_call_dbl",
  "mhs_js_call_obj",
  "mhs_js_call_str",
  "mhs_js_slen",
  "mhs_js_setup",
  "mhs_js_register",
  "mhs_js_argreset",
  "mhs_js_push_int",
  "mhs_js_push_uint",
  "mhs_js_push_dbl",
  "mhs_js_push_obj",
  "mhs_js_push_str",
  "mhs_js_set_haskellCallback",
  "mhs_js_make_wrapper",
  "mhs_js_arg_dbl",
  "mhs_js_arg_int",
  "mhs_js_arg_uint",
  "mhs_js_arg_obj",
  "mhs_js_arg_str",
  "mhs_js_set_res_undef",
  "mhs_js_set_res_num",
  "mhs_js_set_res_obj",
  "mhs_js_set_res_str",
];

function usage() {
  console.error(
    "usage: node wasi-run.mjs [--dir GUEST=HOST] WASM -- [ARGS...]\n" +
      "       default preopens: /=cwd, /tmp=/tmp"
  );
}

function parseArgs(argv) {
  const preopens = { "/": process.cwd(), "/tmp": "/tmp" };
  let index = 0;
  while (argv[index] === "--dir") {
    const mapping = argv[index + 1];
    const split = mapping?.indexOf("=");
    if (!mapping || split === undefined || split <= 0) {
      throw new Error("--dir expects GUEST=HOST");
    }
    preopens[mapping.slice(0, split)] = mapping.slice(split + 1);
    index += 2;
  }
  if (index >= argv.length) {
    throw new Error("missing wasm path");
  }
  const wasmPath = argv[index];
  const args =
    argv[index + 1] === "--" ? argv.slice(index + 2) : argv.slice(index + 1);
  return { wasmPath, args, preopens };
}

function makeEnvImports() {
  return Object.fromEntries(jsFfiImports.map((name) => [name, () => 0]));
}

async function main() {
  let config;
  try {
    config = parseArgs(process.argv.slice(2));
  } catch (error) {
    console.error(error.message);
    usage();
    process.exitCode = 2;
    return;
  }

  const wasi = new WASI({
    version: "preview1",
    args: [config.wasmPath, ...config.args],
    env: process.env,
    preopens: config.preopens,
  });
  const bytes = await readFile(config.wasmPath);
  const module = await WebAssembly.compile(bytes);
  const instance = await WebAssembly.instantiate(module, {
    env: makeEnvImports(),
    wasi_snapshot_preview1: wasi.wasiImport,
  });
  wasi.start(instance);
}

await main();
