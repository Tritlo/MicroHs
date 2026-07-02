use std::alloc::{Layout, alloc, dealloc};
use std::cell::RefCell;

use crate::{JsValue, Program, parse_program};

const CALLBACK_LIMIT: usize = 100_000;

thread_local! {
    static PROGRAMS: RefCell<Vec<Option<Program>>> = const { RefCell::new(Vec::new()) };
    static ACTIVE_PROGRAMS: RefCell<Vec<(u32, *mut Program)>> = const { RefCell::new(Vec::new()) };
    static RESULT_BYTES: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
}

#[unsafe(no_mangle)]
pub extern "C" fn mhs_rust_alloc(len: usize) -> *mut u8 {
    if len == 0 {
        return std::ptr::null_mut();
    }
    let Ok(layout) = Layout::from_size_align(len, 1) else {
        return std::ptr::null_mut();
    };
    unsafe { alloc(layout) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn mhs_rust_dealloc(ptr: *mut u8, len: usize) {
    if ptr.is_null() || len == 0 {
        return;
    }
    let Ok(layout) = Layout::from_size_align(len, 1) else {
        return;
    };
    unsafe {
        dealloc(ptr, layout);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn mhs_rust_program_new(ptr: *const u8, len: usize) -> u32 {
    if ptr.is_null() && len != 0 {
        return 0;
    }
    let input = if len == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(ptr, len) }
    };
    let Ok(program) = parse_program(input) else {
        return 0;
    };
    insert_program(program).unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn mhs_rust_program_free(handle: u32) {
    let _ = PROGRAMS.try_with(|programs| {
        if let Ok(mut programs) = programs.try_borrow_mut() {
            if let Some(slot) = programs.get_mut(handle as usize) {
                *slot = None;
            }
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn mhs_rust_program_reduce(handle: u32, limit: usize) -> i32 {
    match with_program_mut(handle, |program| program.reduce_whnf(limit).map(|_| ())) {
        Ok(()) => 0,
        Err(()) => 1,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mhs_rust_program_render(handle: u32) -> *const u8 {
    let Ok(bytes) = with_program_mut(handle, |program| {
        Ok(program.render(program.root()).into_bytes())
    }) else {
        clear_result_bytes();
        return std::ptr::null();
    };
    RESULT_BYTES
        .try_with(|result| {
            let mut result = result.try_borrow_mut().map_err(|_| ())?;
            *result = bytes;
            Ok::<*const u8, ()>(result.as_ptr())
        })
        .unwrap_or(Ok(std::ptr::null()))
        .unwrap_or(std::ptr::null())
}

#[unsafe(no_mangle)]
pub extern "C" fn mhs_rust_result_len() -> usize {
    RESULT_BYTES
        .try_with(|result| result.try_borrow().map(|result| result.len()).unwrap_or(0))
        .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn mhs_rust_wrapper_invoke(
    program_handle: u32,
    stable_ptr: u32,
    wrapper_index: u32,
) -> i32 {
    let tags = match with_program_mut(program_handle, |program| {
        program.js_wrapper_tags(wrapper_index).map(str::to_owned)
    }) {
        Ok(tags) => tags,
        Err(()) => return 1,
    };
    let args = match read_wrapper_args(tags.as_bytes()) {
        Ok(args) => args,
        Err(()) => return 1,
    };
    match with_program_mut(program_handle, |program| {
        program.apply_js_wrapper_index(wrapper_index, stable_ptr as usize, &args, CALLBACK_LIMIT)
    }) {
        Ok(value) => match set_wrapper_result(&value) {
            Ok(()) => 0,
            Err(()) => 1,
        },
        Err(()) => 1,
    }
}

fn insert_program(mut program: Program) -> Option<u32> {
    PROGRAMS
        .try_with(|programs| {
            let mut programs = programs.try_borrow_mut().ok()?;
            if programs.is_empty() {
                programs.push(None);
            }
            let handle = programs
                .iter()
                .enumerate()
                .skip(1)
                .find_map(|(index, slot)| slot.is_none().then_some(index))
                .unwrap_or(programs.len());
            let handle = u32::try_from(handle).ok()?;
            program.set_js_program_handle(handle);
            let index = handle as usize;
            if index == programs.len() {
                programs.push(Some(program));
            } else {
                programs[index] = Some(program);
            }
            Some(handle)
        })
        .ok()
        .flatten()
}

fn clear_result_bytes() {
    let _ = RESULT_BYTES.try_with(|result| {
        if let Ok(mut result) = result.try_borrow_mut() {
            result.clear();
        }
    });
}

fn with_program_mut<R>(
    handle: u32,
    f: impl FnOnce(&mut Program) -> Result<R, crate::EvalError>,
) -> Result<R, ()> {
    PROGRAMS.with(|programs| {
        if let Ok(mut programs) = programs.try_borrow_mut() {
            let program = programs
                .get_mut(handle as usize)
                .and_then(Option::as_mut)
                .ok_or(())?;
            let _active = ActiveProgram::push(handle, program)?;
            f(program).map_err(|_| ())
        } else {
            with_active_program_mut(handle, f)
        }
    })
}

struct ActiveProgram;

impl ActiveProgram {
    fn push(handle: u32, program: &mut Program) -> Result<Self, ()> {
        ACTIVE_PROGRAMS.with(|active| {
            active
                .try_borrow_mut()
                .map_err(|_| ())?
                .push((handle, program));
            Ok(Self)
        })
    }
}

impl Drop for ActiveProgram {
    fn drop(&mut self) {
        let _ = ACTIVE_PROGRAMS.try_with(|active| {
            if let Ok(mut active) = active.try_borrow_mut() {
                active.pop();
            }
        });
    }
}

fn with_active_program_mut<R>(
    handle: u32,
    f: impl FnOnce(&mut Program) -> Result<R, crate::EvalError>,
) -> Result<R, ()> {
    ACTIVE_PROGRAMS.with(|active| {
        let program = {
            let active = active.try_borrow().map_err(|_| ())?;
            active
                .iter()
                .rev()
                .find_map(|(active_handle, program)| (*active_handle == handle).then_some(*program))
                .ok_or(())?
        };
        // Wasm callback re-entry is single-threaded; the active guard keeps this pointer scoped
        // to the outer runtime entry while JS is synchronously calling back into the same module.
        let program = unsafe { program.as_mut() }.ok_or(())?;
        f(program).map_err(|_| ())
    })
}

fn read_wrapper_args(tags: &[u8]) -> Result<Vec<JsValue>, ()> {
    let mut args = Vec::with_capacity(tags.len().saturating_sub(1));
    for (idx, tag) in tags[1..].iter().copied().enumerate() {
        let idx = i32::try_from(idx).map_err(|_| ())?;
        let value = match tag {
            b'D' => JsValue::Double(unsafe { mhs_js_arg_dbl(idx) }),
            b'F' => JsValue::Float(unsafe { mhs_js_arg_dbl(idx) } as f32),
            b'B' => JsValue::Bool(unsafe { mhs_js_arg_int(idx) } != 0),
            b'P' => JsValue::Pointer(unsafe { mhs_js_arg_uint(idx) }),
            b'J' => JsValue::Object(unsafe { mhs_js_arg_obj(idx) }),
            b'S' => {
                let ptr = unsafe { mhs_js_arg_str(idx) };
                let len = usize::try_from(unsafe { mhs_js_slen() }).map_err(|_| ())?;
                JsValue::Bytes(copy_host_bytes(ptr, len)?)
            }
            b'U' => JsValue::UInt(unsafe { mhs_js_arg_uint(idx) }),
            b'I' => JsValue::Int(unsafe { mhs_js_arg_int(idx) }),
            _ => return Err(()),
        };
        args.push(value);
    }
    Ok(args)
}

fn set_wrapper_result(value: &JsValue) -> Result<(), ()> {
    unsafe {
        match value {
            JsValue::Unit => mhs_js_set_res_undef(),
            JsValue::Int(value) => mhs_js_set_res_num(f64::from(*value)),
            JsValue::UInt(value) | JsValue::Pointer(value) => mhs_js_set_res_num(f64::from(*value)),
            JsValue::Double(value) => mhs_js_set_res_num(*value),
            JsValue::Float(value) => mhs_js_set_res_num(f64::from(*value)),
            JsValue::Bool(value) => mhs_js_set_res_num(f64::from(i32::from(*value))),
            JsValue::Object(value) => mhs_js_set_res_obj(*value),
            JsValue::Bytes(bytes) => {
                let len = i32::try_from(bytes.len()).map_err(|_| ())?;
                mhs_js_set_res_str(bytes.as_ptr(), len);
            }
        }
    }
    Ok(())
}

fn copy_host_bytes(ptr: *const std::os::raw::c_char, len: usize) -> Result<Vec<u8>, ()> {
    if ptr.is_null() {
        return if len == 0 { Ok(Vec::new()) } else { Err(()) };
    }
    Ok(unsafe { std::slice::from_raw_parts(ptr.cast::<u8>(), len) }.to_vec())
}

unsafe extern "C" {
    fn mhs_js_arg_int(index: i32) -> i32;
    fn mhs_js_arg_uint(index: i32) -> u32;
    fn mhs_js_arg_dbl(index: i32) -> f64;
    fn mhs_js_arg_obj(index: i32) -> u32;
    fn mhs_js_arg_str(index: i32) -> *const std::os::raw::c_char;
    fn mhs_js_set_res_num(value: f64);
    fn mhs_js_set_res_obj(handle: u32);
    fn mhs_js_set_res_str(ptr: *const u8, len: i32);
    fn mhs_js_set_res_undef();
    fn mhs_js_slen() -> i32;
}
