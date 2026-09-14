import {
  ConsoleStdout,
  Directory,
  File,
  OpenFile,
  PreopenDirectory,
  WASI,
  wasi as wasiDefs,
} from "@bjorn3/browser_wasi_shim";
import { createJavascriptImports, type JavascriptImportManifest } from "./javascript-imports.js";

export interface WasiOptions {
  args?: string[];
  env?: Record<string, string>;
  files?: Record<string, string | Uint8Array>;
  directories?: string[];
  stdin?: string | Uint8Array;
  imports?: WebAssembly.Imports;
  javascript?: JavascriptImportManifest;
  onStdout?: (text: string) => void;
  onStderr?: (text: string) => void;
}

export interface WasiResult {
  exitCode: number;
  elapsedMs: number;
  stdout: string;
  stderr: string;
  files: Record<string, Uint8Array>;
}

/** Run a WASI command with an in-memory file system. Paths use the guest root. */
export async function runWasi(
  module: WebAssembly.Module,
  options: WasiOptions = {},
): Promise<WasiResult> {
  const encoder = new TextEncoder();
  const root = new Directory(new Map());
  const bytes = (value: string | Uint8Array): Uint8Array =>
    typeof value === "string" ? encoder.encode(value) : value.slice();
  const parts = (path: string): string[] => {
    const names = path.split("/").filter((name) => name !== "" && name !== ".");
    if (names.includes("..")) throw new Error(`Invalid guest path: ${path}`);
    return names;
  };
  const directory = (names: string[]): Directory => {
    let current = root;
    for (const name of names) {
      let entry = current.contents.get(name);
      if (entry === undefined) {
        entry = new Directory(new Map());
        current.contents.set(name, entry);
      }
      if (!(entry instanceof Directory)) {
        throw new Error(`Guest path is a file: ${names.join("/")}`);
      }
      current = entry;
    }
    return current;
  };
  for (const path of options.directories ?? ["tmp"]) directory(parts(path));
  for (const [path, data] of Object.entries(options.files ?? {})) {
    const names = parts(path);
    const name = names.pop();
    if (name === undefined) throw new Error(`Invalid guest file: ${path}`);
    const parent = directory(names);
    if (parent.contents.has(name)) throw new Error(`Duplicate guest path: ${path}`);
    parent.contents.set(name, new File(bytes(data)));
  }

  let stdout = "";
  let stderr = "";
  const stdoutDecoder = new TextDecoder();
  const stderrDecoder = new TextDecoder();
  const writeStdout = (text: string): void => {
    stdout += text;
    if (text !== "") options.onStdout?.(text);
  };
  const writeStderr = (text: string): void => {
    stderr += text;
    if (text !== "") options.onStderr?.(text);
  };
  const wasi = new WASI(
    options.args ?? ["mhs"],
    Object.entries(options.env ?? {}).map(([name, value]) => `${name}=${value}`),
    [
      new OpenFile(new File(bytes(options.stdin ?? ""))),
      new ConsoleStdout((data) => writeStdout(stdoutDecoder.decode(data, { stream: true }))),
      new ConsoleStdout((data) => writeStderr(stderrDecoder.decode(data, { stream: true }))),
      new PreopenDirectory("/", root.contents),
      new PreopenDirectory(".", root.contents),
    ],
    { debug: false },
  );
  // Version 0.4.2 counts UTF-16 units here. WASI requires UTF-8 byte counts.
  wasi.wasiImport.args_sizes_get = (argc: number, size: number): number => {
    const view = new DataView(wasi.inst.exports.memory.buffer);
    view.setUint32(argc, wasi.args.length, true);
    view.setUint32(size, wasi.args.reduce((n, arg) => n + encoder.encode(arg).length + 1, 0), true);
    return 0;
  };
  // Preview 1 subscriptions occupy 48 bytes. Clock flags are at offset 40.
  // Events occupy 32 bytes, and the fourth argument receives the event count.
  // Reference: WebAssembly/WASI a2b96e81, legacy/preview1/docs.md, poll_oneoff.
  // All descriptors supplied by this adapter use synchronous memory storage.
  let waitCell = typeof document === "undefined" && typeof SharedArrayBuffer !== "undefined"
    ? new Int32Array(new SharedArrayBuffer(4)) : undefined;
  wasi.wasiImport.poll_oneoff = (input: number, output: number, count: number, eventCount: number): number => {
    input >>>= 0; output >>>= 0; count >>>= 0; eventCount >>>= 0;
    const memory = wasi.inst.exports.memory.buffer;
    const view = new DataView(memory);
    const fits = (address: number, size: number): boolean => address + size <= memory.byteLength;
    if (!fits(eventCount, 4)) return wasiDefs.ERRNO_FAULT;
    view.setUint32(eventCount, 0, true);
    if (count === 0) return wasiDefs.ERRNO_INVAL;
    if (!fits(input, count * 48) || !fits(output, count * 32)) return wasiDefs.ERRNO_FAULT;
    const now = (clock: number): bigint => clock === wasiDefs.CLOCKID_REALTIME
      ? BigInt(Date.now()) * 1_000_000n : BigInt(Math.round(performance.now() * 1_000_000));
    const start = [now(wasiDefs.CLOCKID_REALTIME), now(wasiDefs.CLOCKID_MONOTONIC)];
    const subscriptions: {
      userdata: bigint; type: number; error: number; bytes: bigint; clock: number; deadline: bigint;
    }[] = [];
    for (let index = 0; index < count; index++) {
      const pointer = input + index * 48;
      const type = view.getUint8(pointer + 8);
      const event = { userdata: view.getBigUint64(pointer, true), type, error: 0, bytes: 0n, clock: 0, deadline: 0n };
      if (type === wasiDefs.EVENTTYPE_CLOCK) {
        event.clock = view.getUint32(pointer + 16, true);
        const timeout = view.getBigUint64(pointer + 24, true);
        const flags = view.getUint16(pointer + 40, true);
        if (event.clock > wasiDefs.CLOCKID_MONOTONIC || (flags & ~1) !== 0) {
          event.error = wasiDefs.ERRNO_INVAL;
        } else {
          event.deadline = (flags & wasiDefs.SUBCLOCKFLAGS_SUBSCRIPTION_CLOCK_ABSTIME) !== 0
            ? timeout : start[event.clock] + timeout;
        }
      } else if (type === wasiDefs.EVENTTYPE_FD_READ || type === wasiDefs.EVENTTYPE_FD_WRITE) {
        const fd = wasi.fds[view.getUint32(pointer + 16, true)];
        if (fd === undefined) event.error = wasiDefs.ERRNO_BADF;
        else if (fd instanceof OpenFile) {
          if (type === wasiDefs.EVENTTYPE_FD_READ) {
            event.bytes = fd.file.size > fd.file_pos ? fd.file.size - fd.file_pos : 0n;
          } else if (fd.file.readonly) event.error = wasiDefs.ERRNO_BADF;
        } else if (fd instanceof ConsoleStdout) {
          if (type === wasiDefs.EVENTTYPE_FD_READ) event.error = wasiDefs.ERRNO_BADF;
        } else event.error = wasiDefs.ERRNO_ISDIR;
      } else return wasiDefs.ERRNO_INVAL;
      subscriptions.push(event);
    }
    for (;;) {
      const ready = subscriptions.filter((event) => event.error !== 0 ||
        event.type !== wasiDefs.EVENTTYPE_CLOCK || event.deadline <= now(event.clock));
      if (ready.length !== 0) {
        new Uint8Array(memory).fill(0, output, output + ready.length * 32);
        ready.forEach((event, index) => {
          const pointer = output + index * 32;
          view.setBigUint64(pointer, event.userdata, true);
          view.setUint16(pointer + 8, event.error, true);
          view.setUint8(pointer + 10, event.type);
          view.setBigUint64(pointer + 16, event.bytes, true);
        });
        view.setUint32(eventCount, ready.length, true);
        return wasiDefs.ERRNO_SUCCESS;
      }
      const remaining = subscriptions.map((event) => event.deadline - now(event.clock))
        .reduce((left, right) => left < right ? left : right);
      // Workers can suspend with Atomics.wait when shared memory is available.
      // A browser main thread must poll synchronously. It cannot run callbacks
      // while a synchronous WebAssembly import is active.
      if (waitCell !== undefined && remaining > 0n) {
        try { Atomics.wait(waitCell, 0, 0, Number(remaining) / 1_000_000); }
        catch { waitCell = undefined; }
      } else if (remaining > 0n) {
        const finish = performance.now() + Number(remaining) / 1_000_000;
        while (performance.now() < finish) { /* The synchronous import cannot yield. */ }
      }
    }
  };
  const javascript = options.javascript === undefined ? {} : createJavascriptImports(options.javascript);
  const instance = await WebAssembly.instantiate(module, {
    ...options.imports,
    ...(options.javascript === undefined ? {} : {
      javascript: { ...options.imports?.javascript, ...javascript.javascript },
    }),
    wasi_snapshot_preview1: wasi.wasiImport,
  });
  const { memory, _start } = instance.exports;
  if (!(memory instanceof WebAssembly.Memory) || typeof _start !== "function") {
    throw new Error("WASI command must export memory and _start");
  }
  const started = performance.now();
  const exitCode = wasi.start({ exports: { memory, _start: () => _start() } });
  const elapsedMs = performance.now() - started;
  writeStdout(stdoutDecoder.decode());
  writeStderr(stderrDecoder.decode());

  const files: Record<string, Uint8Array> = Object.create(null);
  const collect = (dir: Directory, prefix: string): void => {
    for (const [name, entry] of dir.contents) {
      const path = prefix + name;
      if (entry instanceof File) files[path] = entry.data.slice();
      else if (entry instanceof Directory) collect(entry, path + "/");
    }
  };
  collect(root, "");
  return { exitCode, elapsedMs, stdout, stderr, files };
}
