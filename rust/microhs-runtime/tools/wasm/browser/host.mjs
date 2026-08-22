const encoder = new TextEncoder();
const decoder = new TextDecoder();
const ERRNO = {
  EPERM: 1,
  ENOENT: 2,
  EBADF: 9,
  EACCES: 13,
  EEXIST: 17,
  ENOTDIR: 20,
  EISDIR: 21,
  EINVAL: 22,
  ENOSYS: 38,
  ENOTEMPTY: 39,
};

export async function instantiateMicroHsRuntime(wasm, options = {}) {
  const state = {
    reg: new Map(),
    regfree: [],
    programRegs: new Map(),
    nextReg: 0,
    livePrograms: new Set(),
    activeProgramHandles: [],
    preparedProgramHandle: 0,
    argbuf: [],
    wargs: [],
    obj: [null],
    objfree: [],
    programPtrs: new Map(),
    nextPtr: 0x100000000,
    err: null,
    slen: 0,
    wres: undefined,
    exports: null,
    memory: null,
    hostResult: new Uint8Array(),
    stringHelpers: null,
    hostFs: makeHostFs(options.host),
    jsExports: {},
    jsExportsHandle: 0,
  };
  state.intern = (value) => {
    const handle = state.objfree.length ? state.objfree.pop() : state.obj.length;
    state.obj[handle] = value;
    return handle;
  };

  const imports = { env: makeImports(state, options) };
  const source = await loadWasmSource(wasm);
  const { instance, module } =
    source instanceof WebAssembly.Module
      ? { instance: await WebAssembly.instantiate(source, imports), module: source }
      : await WebAssembly.instantiate(source, imports);
  state.exports = instance.exports;
  state.memory = instance.exports.memory;
  state.stringHelpers = makeStringHelpers(state);

  return {
    instance,
    module,
    state,
    newProgram(input) {
      const bytes = typeof input === "string" ? encoder.encode(input) : input;
      const ptr = allocBytes(state, bytes);
      try {
        const handle = state.exports.mhs_rust_program_new(ptr, bytes.length);
        if (handle === 0) {
          const detail = readLastError(state);
          throw new Error(`MicroHs program parse failed${detail ? `: ${detail}` : ""}`);
        }
        state.livePrograms.add(handle);
        try {
          state.jsExports = makeJsExports(state, handle);
          state.jsExportsHandle = handle;
          return handle;
        } catch (error) {
          state.exports.mhs_rust_program_free(handle);
          releaseProgram(state, handle);
          throw error;
        }
      } finally {
        state.exports.mhs_rust_dealloc(ptr, bytes.length);
      }
    },
    get exports() {
      return state.jsExports;
    },
    exportObject(handle) {
      return makeJsExports(state, handle);
    },
    reduce(handle, limit = 100000) {
      return state.exports.mhs_rust_program_reduce(handle, limit) === 0;
    },
    setArgs(handle, args) {
      const bytes = nulSeparated(args);
      const ptr = allocBytes(state, bytes);
      try {
        const status = state.exports.mhs_rust_program_set_args(handle, ptr, bytes.length);
        if (status !== 0) {
          throw new Error("MicroHs set args failed");
        }
      } finally {
        state.exports.mhs_rust_dealloc(ptr, bytes.length);
      }
    },
    setExecutablePath(handle, path) {
      const bytes = path == null ? new Uint8Array() : encoder.encode(path);
      const ptr = allocBytes(state, bytes);
      try {
        const status = state.exports.mhs_rust_program_set_executable_path(
          handle,
          ptr,
          bytes.length
        );
        if (status !== 0) {
          throw new Error("MicroHs set executable path failed");
        }
      } finally {
        state.exports.mhs_rust_dealloc(ptr, bytes.length);
      }
    },
    reduceMain(handle, limit = Number.MAX_SAFE_INTEGER) {
      return state.exports.mhs_rust_program_reduce_main(handle, limit);
    },
    reduceMainStatus() {
      if (typeof state.exports.mhs_rust_program_reduce_main_status !== "function") {
        return -1;
      }
      return state.exports.mhs_rust_program_reduce_main_status();
    },
    render(handle) {
      const ptr = state.exports.mhs_rust_program_render(handle);
      const len = state.exports.mhs_rust_result_len();
      if (ptr === 0) {
        throw new Error("MicroHs render failed");
      }
      return readUtf8(state, ptr, len);
    },
    serialize(handle) {
      const ptr = state.exports.mhs_rust_program_serialize(handle);
      const len = state.exports.mhs_rust_result_len();
      if (ptr === 0) {
        throw new Error("MicroHs serialize failed");
      }
      return readBytes(state, ptr, len);
    },
    resultBytes() {
      const ptr = state.exports.mhs_rust_result_ptr();
      const len = state.exports.mhs_rust_result_len();
      if (ptr === 0) {
        return new Uint8Array();
      }
      return readBytes(state, ptr, len);
    },
    resultText() {
      return decoder.decode(this.resultBytes());
    },
    lastError() {
      return readLastError(state);
    },
    stats(handle) {
      if (typeof state.exports.mhs_rust_program_stats !== "function") {
        return null;
      }
      const ptr = state.exports.mhs_rust_program_stats(handle);
      const len = state.exports.mhs_rust_result_len();
      if (ptr === 0) {
        return null;
      }
      const view = new DataView(readBytes(state, ptr, len).buffer);
      return {
        reductions: view.getBigUint64(0, true),
        liveNodes: view.getBigUint64(8, true),
        currentNodes: view.getBigUint64(16, true),
        gcCollections: view.getBigUint64(24, true),
        lastLiveNodes: view.getBigUint64(32, true),
        highWaterNodes: view.getBigUint64(40, true),
      };
    },
    hostWriteFile(path, bytes) {
      state.hostFs.writeFile(path, bytes);
    },
    hostReadFile(path) {
      return state.hostFs.readFile(path);
    },
    hostRemove(path) {
      return state.hostFs.remove(path);
    },
    hostMkdirp(path) {
      state.hostFs.mkdirp(path);
    },
    freeProgram(handle) {
      if (!state.livePrograms.has(handle)) return;
      if (state.exports.mhs_rust_program_free(handle) !== 0) {
        throw new Error("MicroHs program is active and cannot be freed");
      }
      releaseProgram(state, handle);
      if (state.jsExportsHandle === handle) {
        state.jsExports = {};
        state.jsExportsHandle = 0;
      }
    },
  };
}

async function loadWasmSource(wasm) {
  if (wasm instanceof WebAssembly.Module) {
    return wasm;
  }
  if (wasm instanceof ArrayBuffer) {
    return wasm;
  }
  if (ArrayBuffer.isView(wasm)) {
    return wasm;
  }
  if (typeof Response !== "undefined" && wasm instanceof Response) {
    return wasm.arrayBuffer();
  }
  if (typeof wasm === "string" || wasm instanceof URL) {
    if (typeof fetch === "function" && shouldFetch(wasm)) {
      try {
        const response = await fetch(wasm);
        if (!response.ok) {
          throw new Error(`HTTP ${response.status}`);
        }
        return response.arrayBuffer();
      } catch (error) {
        if (!isNodeFileSource(wasm)) {
          throw error;
        }
      }
    }
    const { readFile } = await import("node:fs/promises");
    return readFile(wasm);
  }
  throw new TypeError("unsupported MicroHs wasm source");
}

function shouldFetch(wasm) {
  if (wasm instanceof URL) {
    return wasm.protocol !== "file:";
  }
  return isProbablyBrowser() || /^[a-z][a-z0-9+.-]*:/i.test(wasm);
}

function isProbablyBrowser() {
  return typeof window !== "undefined" && typeof window.document !== "undefined";
}

function isProbablyNode() {
  return typeof process !== "undefined" && process.versions?.node;
}

function isNodeFileSource(wasm) {
  if (!isProbablyNode()) {
    return false;
  }
  if (wasm instanceof URL) {
    return wasm.protocol === "file:";
  }
  return typeof wasm === "string" && !/^[a-z][a-z0-9+.-]*:/i.test(wasm);
}

function makeImports(state, options) {
  return {
    mhs_host_poll(stepsSoFar) {
      try {
        return typeof options.onPoll === "function" && options.onPoll(stepsSoFar) ? 1 : 0;
      } catch {
        return 0;
      }
    },
    mhs_host_result_copy(dst, len) {
      const bytes = state.hostResult.subarray(0, len);
      new Uint8Array(state.memory.buffer, dst, bytes.length).set(bytes);
      return bytes.length;
    },
    mhs_host_getenv(namePtr, nameLen) {
      const value = state.hostFs.getenv(readUtf8(state, namePtr, nameLen));
      if (value == null) return -1;
      state.hostResult = encoder.encode(value);
      return state.hostResult.length;
    },
    mhs_host_setenv(namePtr, nameLen, valuePtr, valueLen, overwrite) {
      return hostCallI64(() =>
        state.hostFs.setenv(
          readUtf8(state, namePtr, nameLen),
          readUtf8(state, valuePtr, valueLen),
          overwrite
        )
      );
    },
    mhs_host_unsetenv(namePtr, nameLen) {
      return hostCallI64(() => state.hostFs.unsetenv(readUtf8(state, namePtr, nameLen)));
    },
    mhs_host_environ() {
      state.hostResult = nulSeparatedBytes(state.hostFs.environ());
      return state.hostResult.length;
    },
    mhs_host_remove(pathPtr, pathLen) {
      return hostCallI64(() => state.hostFs.remove(readUtf8(state, pathPtr, pathLen)));
    },
    mhs_host_chdir(pathPtr, pathLen) {
      return hostCallI64(() => state.hostFs.chdir(readUtf8(state, pathPtr, pathLen)));
    },
    mhs_host_mkdir(pathPtr, pathLen, mode) {
      return hostCallI64(() => state.hostFs.mkdir(readUtf8(state, pathPtr, pathLen), mode));
    },
    mhs_host_getcwd() {
      state.hostResult = encoder.encode(state.hostFs.getcwd());
      return state.hostResult.length;
    },
    mhs_host_tmpname(prePtr, preLen, sufPtr, sufLen) {
      state.hostResult = encoder.encode(
        state.hostFs.tmpname(readUtf8(state, prePtr, preLen), readUtf8(state, sufPtr, sufLen))
      );
      return state.hostResult.length;
    },
    mhs_host_get_permissions(pathPtr, pathLen) {
      return hostCallI64(() => state.hostFs.getPermissions(readUtf8(state, pathPtr, pathLen)));
    },
    mhs_host_set_permissions(pathPtr, pathLen, permissions) {
      return hostCallI64(() =>
        state.hostFs.setPermissions(readUtf8(state, pathPtr, pathLen), permissions)
      );
    },
    mhs_host_dir_entries(pathPtr, pathLen) {
      return hostBytes(state, () =>
        nulSeparatedBytes(state.hostFs.dirEntries(readUtf8(state, pathPtr, pathLen)))
      );
    },
    mhs_host_file_open(pathPtr, pathLen, modePtr, modeLen) {
      return hostCallI64(() =>
        state.hostFs.open(readUtf8(state, pathPtr, pathLen), readUtf8(state, modePtr, modeLen))
      );
    },
    mhs_host_file_read(handle, dst, len) {
      return hostCall(() => {
        const bytes = state.hostFs.read(handle, len);
        new Uint8Array(state.memory.buffer, dst, bytes.length).set(bytes);
        return bytes.length;
      });
    },
    mhs_host_file_write(handle, src, len) {
      return hostCall(() =>
        state.hostFs.write(handle, new Uint8Array(state.memory.buffer, src, len))
      );
    },
    mhs_host_stdio_write(handle, src, len) {
      return hostCall(() => {
        const bytes = new Uint8Array(state.memory.buffer, src, len);
        writeStdio(options, handle, bytes);
        return len;
      });
    },
    mhs_host_file_flush(handle) {
      return hostCallI64(() => state.hostFs.flush(handle));
    },
    mhs_host_file_close(handle) {
      return hostCallI64(() => state.hostFs.close(handle));
    },
    mhs_js_debug(ptr) {
      console.log(readCString(state, ptr));
    },
    mhs_js_eval_run(programHandle, ptr) {
      withActiveProgram(state, programHandle, () =>
        evaluateWithHelpers(state, readCString(state, ptr))
      );
    },
    mhs_js_eval_call(programHandle, ptr) {
      return withActiveProgram(
        state,
        programHandle,
        () =>
          writeHostString(
            state,
            JSON.stringify(evaluateWithHelpers(state, readCString(state, ptr))),
            true
          )
      );
    },
    mhs_js_set_haskellCallback(callback) {
      globalThis._haskellCallback = callback;
    },
    mhs_js_register(programHandle, bodyp, arity) {
      state.preparedProgramHandle = programHandle;
      const body = readCString(state, bodyp);
      let programRegs = state.programRegs.get(programHandle);
      if (programRegs === undefined) {
        programRegs = new Map();
        state.programRegs.set(programHandle, programRegs);
      }
      const key = `${arity}\0${body}`;
      const registered = programRegs.get(key);
      if (registered !== undefined) return registered;
      const names = [];
      for (let idx = 0; idx < arity; idx += 1) names.push(`$${idx}`);
      let fn;
      try {
        fn = Function(
          "helpers",
          `return function(${names.join(",")}) {\n` +
            `const { UTF8ToString, lengthBytesUTF8, stringToNewUTF8 } = helpers;\n` +
            `${body}\n` +
            `};`
        )(state.stringHelpers);
      } catch (error) {
        const message = String(error);
        fn = () => {
          throw new Error(message);
        };
      }
      const idx = state.regfree.length ? state.regfree.pop() : state.nextReg++;
      state.reg.set(idx, fn);
      programRegs.set(key, idx);
      return idx;
    },
    mhs_js_argreset() {
      state.argbuf.length = 0;
      state.err = null;
    },
    mhs_js_push_int(value) {
      state.argbuf.push(value | 0);
    },
    mhs_js_push_uint(value) {
      state.argbuf.push(value >>> 0);
    },
    mhs_js_push_dbl(value) {
      state.argbuf.push(value);
    },
    mhs_js_push_bool(value) {
      state.argbuf.push(value !== 0);
    },
    mhs_js_push_ptr(value) {
      state.argbuf.push(ptrToJs(state, value));
    },
    mhs_js_push_obj(handle) {
      state.argbuf.push(state.obj[handle]);
    },
    mhs_js_push_str(ptr, len) {
      state.argbuf.push(readUtf8(state, ptr, len));
    },
    mhs_js_call_int(idx) {
      return callJs(state, idx, 0, (value) => value | 0);
    },
    mhs_js_call_uint(idx) {
      return callJs(state, idx, 0, (value) => value >>> 0);
    },
    mhs_js_call_dbl(idx) {
      return callJs(state, idx, 0, (value) => +value);
    },
    mhs_js_call_ptr(idx) {
      return callJs(state, idx, 0n, (value) => ptrFromJs(state, value));
    },
    mhs_js_call_obj(idx) {
      return callJs(state, idx, 0, (value) => state.intern(value));
    },
    mhs_js_call_bool(idx) {
      return callJs(state, idx, 0, (value) => (value ? 1 : 0));
    },
    mhs_js_call_str(idx) {
      const value = callJs(state, idx, "", (result) => String(result));
      return writeHostString(state, value, false);
    },
    mhs_js_call_void(idx) {
      callJs(state, idx, undefined, () => undefined);
    },
    mhs_js_make_wrapper(programHandle, stablePtr, wrapperIndex) {
      const fn = (...args) => {
        assertLiveProgram(state, programHandle);
        return withActiveProgram(state, programHandle, () => {
          state.wargs.push(args);
          try {
            const status = state.exports.mhs_rust_wrapper_invoke(
              programHandle,
              stablePtr,
              wrapperIndex
            );
            if (status !== 0) {
              throw new Error("MicroHs wrapper callback failed");
            }
            return state.wres;
          } finally {
            state.wargs.pop();
          }
        });
      };
      return state.intern(fn);
    },
    mhs_js_obj_free(handle) {
      if (
        handle >= 1 &&
        Object.prototype.hasOwnProperty.call(state.obj, handle)
      ) {
        delete state.obj[handle];
        state.objfree.push(handle);
      }
    },
    mhs_js_arg_int(index) {
      try {
        return state.wargs[state.wargs.length - 1][index] | 0;
      } catch {
        return 0;
      }
    },
    mhs_js_arg_uint(index) {
      try {
        return state.wargs[state.wargs.length - 1][index] >>> 0;
      } catch {
        return 0;
      }
    },
    mhs_js_arg_bool(index) {
      try {
        return state.wargs[state.wargs.length - 1][index] ? 1 : 0;
      } catch {
        return 0;
      }
    },
    mhs_js_arg_ptr(index) {
      try {
        return ptrFromJs(state, state.wargs[state.wargs.length - 1][index]);
      } catch {
        return 0n;
      }
    },
    mhs_js_arg_dbl(index) {
      try {
        return +state.wargs[state.wargs.length - 1][index];
      } catch {
        return 0;
      }
    },
    mhs_js_arg_obj(index) {
      try {
        return state.intern(state.wargs[state.wargs.length - 1][index]);
      } catch {
        return 0;
      }
    },
    mhs_js_arg_str(index) {
      try {
        return writeHostString(state, String(state.wargs[state.wargs.length - 1][index]), false);
      } catch {
        return writeHostString(state, "", false);
      }
    },
    mhs_js_set_res_num(value) {
      state.wres = value;
    },
    mhs_js_set_res_bool(value) {
      state.wres = value !== 0;
    },
    mhs_js_set_res_ptr(value) {
      state.wres = ptrToJs(state, value);
    },
    mhs_js_set_res_obj(handle) {
      state.wres = state.obj[handle];
    },
    mhs_js_set_res_str(ptr, len) {
      state.wres = readUtf8(state, ptr, len);
    },
    mhs_js_set_res_undef() {
      state.wres = undefined;
    },
    mhs_js_slen() {
      return state.slen;
    },
    mhs_js_haserr() {
      return state.err === null ? 0 : 1;
    },
    mhs_js_logerr() {
      console.error("MicroHs JavaScript FFI exception:", state.err);
    },
  };
}

function callJs(state, idx, fallback, convert) {
  const programHandle = state.preparedProgramHandle;
  state.preparedProgramHandle = 0;
  try {
    return withActiveProgram(state, programHandle, () =>
      convert(registeredFunction(state, idx).apply(null, state.argbuf))
    );
  } catch (error) {
    state.err = String(error);
    return fallback;
  }
}

function makeJsExports(state, handle) {
  assertLiveProgram(state, handle);
  const exports = Object.create(null);
  const count = state.exports.mhs_rust_js_export_count(handle);
  for (let idx = 0; idx < count; idx += 1) {
    const namePtr = state.exports.mhs_rust_js_export_name(handle, idx);
    const nameLen = state.exports.mhs_rust_result_len();
    if (namePtr === 0) continue;
    const name = readUtf8(state, namePtr, nameLen);
    if (Object.prototype.hasOwnProperty.call(exports, name)) {
      throw new Error(`duplicate MicroHs JavaScript export: ${name}`);
    }
    exports[name] = (...args) => {
      assertLiveProgram(state, handle);
      return withActiveProgram(state, handle, () => {
        state.wargs.push(args);
        try {
          const status = state.exports.mhs_rust_js_export_invoke(handle, idx);
          if (status !== 0) {
            const message = state.exports.mhs_rust_result_len() ? `: ${readResultText(state)}` : "";
            throw new Error(`MicroHs JavaScript export failed${message}`);
          }
          return state.wres;
        } finally {
          state.wargs.pop();
        }
      });
    };
  }
  return exports;
}

function registeredFunction(state, idx) {
  const fn = state.reg.get(idx);
  if (fn === undefined) throw new Error("MicroHs JavaScript function is no longer registered");
  return fn;
}

function readLastError(state) {
  if (
    typeof state.exports.mhs_rust_last_error_ptr !== "function" ||
    typeof state.exports.mhs_rust_last_error_len !== "function"
  ) {
    return "";
  }
  const len = state.exports.mhs_rust_last_error_len();
  if (len === 0) return "";
  return readUtf8(state, state.exports.mhs_rust_last_error_ptr(), len);
}

function assertLiveProgram(state, handle) {
  if (!state.livePrograms.has(handle)) {
    throw new Error("MicroHs program has been freed");
  }
}

function withActiveProgram(state, handle, fn) {
  assertLiveProgram(state, handle);
  state.activeProgramHandles.push(handle);
  try {
    return fn();
  } finally {
    state.activeProgramHandles.pop();
  }
}

function currentProgramHandle(state) {
  return (
    state.activeProgramHandles[state.activeProgramHandles.length - 1] ??
    state.preparedProgramHandle
  );
}

function releaseProgram(state, handle) {
  state.livePrograms.delete(handle);
  state.programPtrs.delete(handle);
  const programRegs = state.programRegs.get(handle);
  if (programRegs !== undefined) {
    for (const idx of programRegs.values()) {
      state.reg.delete(idx);
      state.regfree.push(idx);
    }
    state.programRegs.delete(handle);
  }
  if (state.livePrograms.size === 0) {
    state.reg.clear();
    state.regfree.length = 0;
    state.programRegs.clear();
    state.nextReg = 0;
    state.obj.length = 1;
    state.objfree.length = 0;
    state.programPtrs.clear();
    state.nextPtr = 0x100000000;
    state.activeProgramHandles.length = 0;
    state.preparedProgramHandle = 0;
    state.argbuf.length = 0;
    state.wargs.length = 0;
    state.err = null;
    state.slen = 0;
    state.wres = undefined;
    state.hostResult = new Uint8Array();
  }
}

function makeStringHelpers(state) {
  const UTF8ToString = (ptr, maxBytesToRead, ignoreNul) => {
    try {
      if (maxBytesToRead == null) {
        const len = state.exports.mhs_rust_active_cstring_len(ptrFromJs(state, ptr));
        if (len < 0) return "";
        return decoder.decode(activePointerBytes(state, ptr, len));
      }
      const len = Number(maxBytesToRead);
      if (!Number.isSafeInteger(len) || len <= 0) return "";
      const bytes = activePointerBytes(state, ptr, len);
      if (ignoreNul) return decoder.decode(bytes);
      const nul = bytes.indexOf(0);
      return decoder.decode(nul < 0 ? bytes : bytes.subarray(0, nul));
    } catch {
      return "";
    }
  };
  const lengthBytesUTF8 = (value) => encoder.encode(String(value)).length;
  const stringToNewUTF8 = (value) => {
    const bytes = encoder.encode(String(value));
    const out = new Uint8Array(bytes.length + 1);
    out.set(bytes);
    const scratch = allocBytes(state, out);
    try {
      const ptr = state.exports.mhs_rust_active_alloc(scratch, out.length);
      return ptrToJs(state, ptr);
    } finally {
      state.exports.mhs_rust_dealloc(scratch, out.length);
    }
  };
  return { UTF8ToString, lengthBytesUTF8, stringToNewUTF8 };
}

function evaluateWithHelpers(state, source) {
  const evaluator = Function(
    "helpers",
    "source",
    "const { UTF8ToString, lengthBytesUTF8, stringToNewUTF8 } = helpers; " +
      "return eval(source);"
  );
  return evaluator(state.stringHelpers, source);
}

function activePointerBytes(state, ptr, len) {
  if (len === 0) return new Uint8Array();
  const scratch = state.exports.mhs_rust_alloc(len);
  if (scratch === 0) throw new Error("MicroHs wasm allocation failed");
  try {
    const status = state.exports.mhs_rust_active_read(ptrFromJs(state, ptr), scratch, len);
    if (status !== 0) throw new Error("MicroHs pointer read failed");
    return readBytes(state, scratch, len);
  } finally {
    state.exports.mhs_rust_dealloc(scratch, len);
  }
}

function ptrToJs(state, value) {
  const ptr = BigInt.asIntN(64, BigInt(value));
  if (ptr >= 0n && ptr <= 0xffffffffn) return Number(ptr);
  const programHandle = currentProgramHandle(state);
  assertLiveProgram(state, programHandle);
  let pointers = state.programPtrs.get(programHandle);
  if (pointers === undefined) {
    pointers = { values: new Map(), reverse: new Map() };
    state.programPtrs.set(programHandle, pointers);
  }
  const key = ptr.toString();
  const existing = pointers.reverse.get(key);
  if (existing !== undefined) return existing;
  const handle = state.nextPtr;
  state.nextPtr += 1;
  pointers.values.set(handle, ptr);
  pointers.reverse.set(key, handle);
  return handle;
}

function ptrFromJs(state, value) {
  if (typeof value === "bigint") return BigInt.asIntN(64, value);
  const number = Number(value);
  if (!Number.isFinite(number)) return 0n;
  if (Number.isInteger(number) && number >= 0x100000000) {
    const programHandle = currentProgramHandle(state);
    const pointer = state.programPtrs.get(programHandle)?.values.get(number);
    if (pointer === undefined) throw new Error("MicroHs pointer belongs to another program");
    return pointer;
  }
  return BigInt.asUintN(32, BigInt(number >>> 0));
}

function writeStdio(options, handle, bytes) {
  const sink = Number(handle) === 2 ? options.stderr : options.stdout;
  const out = toBytes(bytes);
  if (sink) {
    sink(out);
  } else if (isProbablyNode()) {
    process[Number(handle) === 2 ? "stderr" : "stdout"].write(Buffer.from(out));
  } else {
    const text = decoder.decode(out);
    if (Number(handle) === 2) {
      console.error(text);
    } else {
      console.log(text);
    }
  }
}

function hostCall(fn) {
  try {
    return fn();
  } catch (error) {
    return -errnoFromError(error);
  }
}

function hostCallI64(fn) {
  return BigInt(hostCall(fn));
}

function hostBytes(state, fn) {
  try {
    const bytes = fn();
    state.hostResult = bytes;
    return bytes.length;
  } catch (error) {
    return -errnoFromError(error);
  }
}

function errnoFromError(error) {
  if (error instanceof HostError) return error.errno;
  return ERRNO.EINVAL;
}

class HostError extends Error {
  constructor(errno, message = `host errno ${errno}`) {
    super(message);
    this.errno = errno;
  }
}

function makeHostFs(options = {}) {
  const files = new Map();
  const dirs = new Set(["/", "/tmp"]);
  const env = new Map(Object.entries(options.env ?? {}));
  const handles = new Map();
  let cwd = "/";
  let nextHandle = 1;
  let tmpCounter = 0;

  function normalize(rawPath) {
    let input = String(rawPath || ".");
    if (!input.startsWith("/")) {
      input = `${cwd}/${input}`;
    }
    const parts = [];
    for (const part of input.split("/")) {
      if (!part || part === ".") continue;
      if (part === "..") {
        parts.pop();
      } else {
        parts.push(part);
      }
    }
    return `/${parts.join("/")}`;
  }

  function parentDir(path) {
    const idx = path.lastIndexOf("/");
    return idx <= 0 ? "/" : path.slice(0, idx);
  }

  function basename(path) {
    const idx = path.lastIndexOf("/");
    return idx < 0 ? path : path.slice(idx + 1);
  }

  function assertParent(path) {
    if (!dirs.has(parentDir(path))) {
      throw new HostError(ERRNO.ENOENT);
    }
  }

  function mkdirp(rawPath) {
    const path = normalize(rawPath);
    let current = "";
    for (const part of path.split("/").filter(Boolean)) {
      current += `/${part}`;
      if (files.has(current)) {
        throw new HostError(ERRNO.ENOTDIR);
      }
      dirs.add(current);
    }
  }

  function writeFile(rawPath, bytes) {
    const path = normalize(rawPath);
    assertParent(path);
    if (dirs.has(path)) {
      throw new HostError(ERRNO.EISDIR);
    }
    files.set(path, toBytes(bytes));
  }

  function readFile(rawPath) {
    const path = normalize(rawPath);
    const bytes = files.get(path);
    if (!bytes) {
      throw new HostError(dirs.has(path) ? ERRNO.EISDIR : ERRNO.ENOENT);
    }
    return bytes.slice();
  }

  function modeFlags(rawMode) {
    const mode = String(rawMode).replaceAll("b", "");
    switch (mode) {
      case "r":
        return { readable: true, writable: false, append: false, truncate: false, create: false };
      case "w":
        return { readable: false, writable: true, append: false, truncate: true, create: true };
      case "a":
        return { readable: false, writable: true, append: true, truncate: false, create: true };
      case "r+":
        return { readable: true, writable: true, append: false, truncate: false, create: false };
      case "w+":
        return { readable: true, writable: true, append: false, truncate: true, create: true };
      case "a+":
        return { readable: true, writable: true, append: true, truncate: false, create: true };
      default:
        throw new HostError(ERRNO.EINVAL);
    }
  }

  function open(rawPath, rawMode) {
    const path = normalize(rawPath);
    const flags = modeFlags(rawMode);
    assertParent(path);
    if (dirs.has(path)) {
      throw new HostError(ERRNO.EISDIR);
    }
    if (!files.has(path)) {
      if (!flags.create) {
        throw new HostError(ERRNO.ENOENT);
      }
      files.set(path, new Uint8Array());
    }
    if (flags.truncate) {
      files.set(path, new Uint8Array());
    }
    const handle = nextHandle++;
    handles.set(handle, {
      path,
      pos: flags.append ? files.get(path).length : 0,
      ...flags,
    });
    return handle;
  }

  function fileHandle(handle, op) {
    const file = handles.get(Number(handle));
    if (!file) {
      throw new HostError(ERRNO.EBADF);
    }
    if (op === "read" && !file.readable) {
      throw new HostError(ERRNO.EBADF);
    }
    if (op === "write" && !file.writable) {
      throw new HostError(ERRNO.EBADF);
    }
    return file;
  }

  function read(handle, len) {
    const file = fileHandle(handle, "read");
    const bytes = files.get(file.path) ?? new Uint8Array();
    const end = Math.min(bytes.length, file.pos + Number(len));
    const out = bytes.slice(file.pos, end);
    file.pos = end;
    return out;
  }

  function write(handle, input) {
    const file = fileHandle(handle, "write");
    const bytes = files.get(file.path) ?? new Uint8Array();
    const src = toBytes(input);
    if (file.append) {
      file.pos = bytes.length;
    }
    const end = file.pos + src.length;
    let out = bytes;
    if (end > out.length) {
      const grown = new Uint8Array(end);
      grown.set(out);
      out = grown;
    }
    out.set(src, file.pos);
    file.pos = end;
    files.set(file.path, out);
    return src.length;
  }

  function flush(handle) {
    fileHandle(handle);
    return 0;
  }

  function close(handle) {
    if (!handles.delete(Number(handle))) {
      throw new HostError(ERRNO.EBADF);
    }
    return 0;
  }

  function remove(rawPath) {
    const path = normalize(rawPath);
    if (files.delete(path)) return 0;
    if (dirs.has(path)) {
      for (const dir of dirs) {
        if (dir !== path && parentDir(dir) === path) {
          throw new HostError(ERRNO.ENOTEMPTY);
        }
      }
      for (const file of files.keys()) {
        if (parentDir(file) === path) {
          throw new HostError(ERRNO.ENOTEMPTY);
        }
      }
      if (path === "/") {
        throw new HostError(ERRNO.EPERM);
      }
      dirs.delete(path);
      return 0;
    }
    throw new HostError(ERRNO.ENOENT);
  }

  function mkdir(rawPath) {
    const path = normalize(rawPath);
    if (files.has(path)) {
      throw new HostError(ERRNO.ENOTDIR);
    }
    if (dirs.has(path)) {
      throw new HostError(ERRNO.EEXIST);
    }
    assertParent(path);
    dirs.add(path);
    return 0;
  }

  function chdir(rawPath) {
    const path = normalize(rawPath);
    if (!dirs.has(path)) {
      throw new HostError(files.has(path) ? ERRNO.ENOTDIR : ERRNO.ENOENT);
    }
    cwd = path;
    return 0;
  }

  function getPermissions(rawPath) {
    const path = normalize(rawPath);
    if (dirs.has(path)) return 14;
    if (files.has(path)) return 6;
    throw new HostError(ERRNO.ENOENT);
  }

  function setPermissions(rawPath) {
    const path = normalize(rawPath);
    if (!dirs.has(path) && !files.has(path)) {
      throw new HostError(ERRNO.ENOENT);
    }
    return 0;
  }

  function dirEntries(rawPath) {
    const path = normalize(rawPath);
    if (!dirs.has(path)) {
      throw new HostError(files.has(path) ? ERRNO.ENOTDIR : ERRNO.ENOENT);
    }
    const names = [".", ".."];
    for (const dir of dirs) {
      if (dir !== path && parentDir(dir) === path) names.push(basename(dir));
    }
    for (const file of files.keys()) {
      if (parentDir(file) === path) names.push(basename(file));
    }
    return names;
  }

  return {
    getenv(name) {
      return env.get(name) ?? null;
    },
    setenv(name, value, overwrite) {
      if (!name || name.includes("=")) throw new HostError(ERRNO.EINVAL);
      if (overwrite || !env.has(name)) env.set(name, value);
      return 0;
    },
    unsetenv(name) {
      if (!name || name.includes("=")) throw new HostError(ERRNO.EINVAL);
      env.delete(name);
      return 0;
    },
    environ() {
      return Array.from(env, ([name, value]) => `${name}=${value}`);
    },
    remove,
    chdir,
    mkdir,
    mkdirp,
    getcwd() {
      return cwd;
    },
    tmpname(pre, suf) {
      tmpCounter += 1;
      return `/tmp/${pre}${String(tmpCounter).padStart(6, "0")}${suf}`;
    },
    getPermissions,
    setPermissions,
    dirEntries,
    open,
    read,
    write,
    flush,
    close,
    writeFile,
    readFile,
  };
}

function toBytes(bytes) {
  if (bytes instanceof Uint8Array) return bytes.slice();
  if (ArrayBuffer.isView(bytes)) {
    return new Uint8Array(bytes.buffer, bytes.byteOffset, bytes.byteLength).slice();
  }
  if (bytes instanceof ArrayBuffer) return new Uint8Array(bytes).slice();
  if (typeof bytes === "string") return encoder.encode(bytes);
  return Uint8Array.from(bytes);
}

function allocBytes(state, bytes) {
  const ptr = state.exports.mhs_rust_alloc(bytes.length);
  if (ptr === 0 && bytes.length !== 0) {
    throw new Error("MicroHs wasm allocation failed");
  }
  new Uint8Array(state.memory.buffer, ptr, bytes.length).set(bytes);
  return ptr;
}

function nulSeparated(values) {
  const encoded = values.map((value) => encoder.encode(String(value)));
  const total = encoded.reduce((acc, bytes) => acc + bytes.length + 1, 0);
  const out = new Uint8Array(total);
  let offset = 0;
  for (const bytes of encoded) {
    out.set(bytes, offset);
    offset += bytes.length + 1;
  }
  return out;
}

function nulSeparatedBytes(values) {
  return nulSeparated(values);
}

function writeHostString(state, value, nulTerminated) {
  const bytes = encoder.encode(value);
  state.slen = bytes.length;
  const out = nulTerminated ? new Uint8Array(bytes.length + 1) : bytes;
  if (nulTerminated) out.set(bytes);
  return allocBytes(state, out);
}

function readCString(state, ptr) {
  const memory = new Uint8Array(state.memory.buffer);
  let end = ptr;
  while (memory[end] !== 0) end += 1;
  return decoder.decode(memory.subarray(ptr, end));
}

function readUtf8(state, ptr, len) {
  return decoder.decode(readBytes(state, ptr, len));
}

function readResultText(state) {
  const ptr = state.exports.mhs_rust_result_ptr();
  const len = state.exports.mhs_rust_result_len();
  return ptr === 0 ? "" : readUtf8(state, ptr, len);
}

function readBytes(state, ptr, len) {
  return new Uint8Array(state.memory.buffer, ptr, len).slice();
}
