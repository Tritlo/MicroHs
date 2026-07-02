import { readFile } from "node:fs/promises";

const encoder = new TextEncoder();
const decoder = new TextDecoder();

export async function instantiateMicroHsRuntime(wasm) {
  const bytes = typeof wasm === "string" || wasm instanceof URL ? await readFile(wasm) : wasm;
  const state = {
    reg: [],
    argbuf: [],
    wargs: [],
    obj: [null],
    objfree: [],
    err: null,
    slen: 0,
    wres: undefined,
    exports: null,
    memory: null,
  };
  state.intern = (value) => {
    const handle = state.objfree.length ? state.objfree.pop() : state.obj.length;
    state.obj[handle] = value;
    return handle;
  };

  const imports = { env: makeImports(state) };
  const { instance, module } = await WebAssembly.instantiate(bytes, imports);
  state.exports = instance.exports;
  state.memory = instance.exports.memory;

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
          throw new Error("MicroHs program parse failed");
        }
        return handle;
      } finally {
        state.exports.mhs_rust_dealloc(ptr, bytes.length);
      }
    },
    reduce(handle, limit = 100000) {
      return state.exports.mhs_rust_program_reduce(handle, limit) === 0;
    },
    render(handle) {
      const ptr = state.exports.mhs_rust_program_render(handle);
      const len = state.exports.mhs_rust_result_len();
      if (ptr === 0) {
        throw new Error("MicroHs render failed");
      }
      return readUtf8(state, ptr, len);
    },
    freeProgram(handle) {
      state.exports.mhs_rust_program_free(handle);
    },
  };
}

function makeImports(state) {
  return {
    mhs_js_debug(ptr) {
      console.log(readCString(state, ptr));
    },
    mhs_js_eval_run(ptr) {
      globalThis.eval(readCString(state, ptr));
    },
    mhs_js_eval_call(ptr) {
      return writeHostString(state, JSON.stringify(globalThis.eval(readCString(state, ptr))), true);
    },
    mhs_js_set_haskellCallback(callback) {
      globalThis._haskellCallback = callback;
    },
    mhs_js_setup() {},
    mhs_js_register(bodyp, arity) {
      const body = readCString(state, bodyp);
      const names = [];
      for (let idx = 0; idx < arity; idx += 1) names.push(`$${idx}`);
      names.push(body);
      let fn;
      try {
        fn = Function.apply(null, names);
      } catch (error) {
        const message = String(error);
        fn = () => {
          throw new Error(message);
        };
      }
      return state.reg.push(fn) - 1;
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
    mhs_js_push_obj(handle) {
      state.argbuf.push(state.obj[handle]);
    },
    mhs_js_push_str(ptr, len) {
      state.argbuf.push(readUtf8(state, ptr, len));
    },
    mhs_js_call_int(idx) {
      return callJs(state, idx, 0, (value) => value | 0);
    },
    mhs_js_call_dbl(idx) {
      return callJs(state, idx, 0, (value) => +value);
    },
    mhs_js_call_ptr(idx) {
      return callJs(state, idx, 0, (value) => value >>> 0);
    },
    mhs_js_call_obj(idx) {
      return callJs(state, idx, 0, (value) => state.intern(value));
    },
    mhs_js_call_bool(idx) {
      return callJs(state, idx, 0, (value) => (value ? 1 : 0));
    },
    mhs_js_call_str(idx) {
      try {
        return writeHostString(state, String(state.reg[idx].apply(null, state.argbuf)), false);
      } catch (error) {
        state.err = String(error);
        return writeHostString(state, "", false);
      }
    },
    mhs_js_call_void(idx) {
      callJs(state, idx, undefined, () => undefined);
    },
    mhs_js_make_wrapper(programHandle, stablePtr, wrapperIndex) {
      const fn = (...args) => {
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
      };
      return state.intern(fn);
    },
    mhs_js_arg_int(index) {
      try {
        return state.wargs[state.wargs.length - 1][index] | 0;
      } catch {
        return 0;
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
  try {
    return convert(state.reg[idx].apply(null, state.argbuf));
  } catch (error) {
    state.err = String(error);
    return fallback;
  }
}

function allocBytes(state, bytes) {
  const ptr = state.exports.mhs_rust_alloc(bytes.length);
  if (ptr === 0 && bytes.length !== 0) {
    throw new Error("MicroHs wasm allocation failed");
  }
  new Uint8Array(state.memory.buffer, ptr, bytes.length).set(bytes);
  return ptr;
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
  return decoder.decode(new Uint8Array(state.memory.buffer, ptr, len));
}
