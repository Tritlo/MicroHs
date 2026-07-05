fn int_to_i32(n: i64) -> Result<i32, EvalError> {
    i32::try_from(n).map_err(|_| EvalError::Overflow)
}

fn validate_js_tags(tags: &[u8]) -> Result<(), EvalError> {
    if tags.is_empty() {
        return Err(EvalError::InvalidByteString);
    }
    for (idx, tag) in tags.iter().copied().enumerate() {
        let ok = matches!(tag, b'I' | b'U' | b'D' | b'F' | b'P' | b'B' | b'J' | b'S')
            || (idx == 0 && tag == b'V');
        if !ok {
            return Err(EvalError::InvalidByteString);
        }
    }
    Ok(())
}

fn host_js_debug(bytes: &[u8]) -> Result<(), EvalError> {
    #[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
    {
        let bytes = nul_terminated(bytes)?;
        unsafe {
            mhs_js_debug(bytes.as_ptr());
        }
        Ok(())
    }
    #[cfg(not(all(target_arch = "wasm32", not(target_os = "wasi"))))]
    {
        let _ = bytes;
        Err(EvalError::UnsupportedJsFfi)
    }
}

fn host_js_eval_run(bytes: &[u8]) -> Result<(), EvalError> {
    #[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
    {
        let bytes = nul_terminated(bytes)?;
        unsafe {
            mhs_js_eval_run(bytes.as_ptr());
        }
        Ok(())
    }
    #[cfg(not(all(target_arch = "wasm32", not(target_os = "wasi"))))]
    {
        let _ = bytes;
        Err(EvalError::UnsupportedJsFfi)
    }
}

fn host_js_eval_call(bytes: &[u8]) -> Result<Vec<u8>, EvalError> {
    #[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
    {
        let bytes = nul_terminated(bytes)?;
        unsafe {
            let ptr = mhs_js_eval_call(bytes.as_ptr());
            copy_host_c_string(ptr)
        }
    }
    #[cfg(not(all(target_arch = "wasm32", not(target_os = "wasi"))))]
    {
        let _ = bytes;
        Err(EvalError::UnsupportedJsFfi)
    }
}

fn host_js_set_haskell_callback(callback: i32) -> Result<(), EvalError> {
    #[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
    {
        unsafe {
            mhs_js_set_haskellCallback(callback);
        }
        Ok(())
    }
    #[cfg(not(all(target_arch = "wasm32", not(target_os = "wasi"))))]
    {
        let _ = callback;
        Err(EvalError::UnsupportedJsFfi)
    }
}

fn host_js_call_void(body: &[u8], arity: usize, args: &[JsArg]) -> Result<(), EvalError> {
    #[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
    {
        let idx = host_js_prepare_call(body, arity, args)?;
        unsafe {
            mhs_js_call_void(idx);
        }
        host_js_check_error()
    }
    #[cfg(not(all(target_arch = "wasm32", not(target_os = "wasi"))))]
    {
        let _ = (body, arity, args);
        Err(EvalError::UnsupportedJsFfi)
    }
}

fn host_js_call_int(body: &[u8], arity: usize, args: &[JsArg]) -> Result<i32, EvalError> {
    #[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
    {
        let idx = host_js_prepare_call(body, arity, args)?;
        let result = unsafe { mhs_js_call_int(idx) };
        host_js_check_error()?;
        Ok(result)
    }
    #[cfg(not(all(target_arch = "wasm32", not(target_os = "wasi"))))]
    {
        let _ = (body, arity, args);
        Err(EvalError::UnsupportedJsFfi)
    }
}

fn host_js_call_uint(body: &[u8], arity: usize, args: &[JsArg]) -> Result<u32, EvalError> {
    #[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
    {
        let idx = host_js_prepare_call(body, arity, args)?;
        let result = unsafe { mhs_js_call_uint(idx) };
        host_js_check_error()?;
        Ok(result)
    }
    #[cfg(not(all(target_arch = "wasm32", not(target_os = "wasi"))))]
    {
        let _ = (body, arity, args);
        Err(EvalError::UnsupportedJsFfi)
    }
}

fn host_js_call_double(body: &[u8], arity: usize, args: &[JsArg]) -> Result<f64, EvalError> {
    #[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
    {
        let idx = host_js_prepare_call(body, arity, args)?;
        let result = unsafe { mhs_js_call_dbl(idx) };
        host_js_check_error()?;
        Ok(result)
    }
    #[cfg(not(all(target_arch = "wasm32", not(target_os = "wasi"))))]
    {
        let _ = (body, arity, args);
        Err(EvalError::UnsupportedJsFfi)
    }
}

fn host_js_call_ptr(body: &[u8], arity: usize, args: &[JsArg]) -> Result<u32, EvalError> {
    #[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
    {
        let idx = host_js_prepare_call(body, arity, args)?;
        let result = unsafe { mhs_js_call_ptr(idx) };
        host_js_check_error()?;
        Ok(result)
    }
    #[cfg(not(all(target_arch = "wasm32", not(target_os = "wasi"))))]
    {
        let _ = (body, arity, args);
        Err(EvalError::UnsupportedJsFfi)
    }
}

fn host_js_call_object(body: &[u8], arity: usize, args: &[JsArg]) -> Result<u32, EvalError> {
    #[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
    {
        let idx = host_js_prepare_call(body, arity, args)?;
        let result = unsafe { mhs_js_call_obj(idx) };
        host_js_check_error()?;
        Ok(result)
    }
    #[cfg(not(all(target_arch = "wasm32", not(target_os = "wasi"))))]
    {
        let _ = (body, arity, args);
        Err(EvalError::UnsupportedJsFfi)
    }
}

fn host_js_call_bool(body: &[u8], arity: usize, args: &[JsArg]) -> Result<bool, EvalError> {
    #[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
    {
        let idx = host_js_prepare_call(body, arity, args)?;
        let result = unsafe { mhs_js_call_bool(idx) != 0 };
        host_js_check_error()?;
        Ok(result)
    }
    #[cfg(not(all(target_arch = "wasm32", not(target_os = "wasi"))))]
    {
        let _ = (body, arity, args);
        Err(EvalError::UnsupportedJsFfi)
    }
}

fn host_js_call_string(body: &[u8], arity: usize, args: &[JsArg]) -> Result<Vec<u8>, EvalError> {
    #[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
    {
        let idx = host_js_prepare_call(body, arity, args)?;
        unsafe {
            let ptr = mhs_js_call_str(idx);
            let len = usize::try_from(mhs_js_slen()).map_err(|_| EvalError::Overflow)?;
            let result = copy_host_bytes(ptr, len)?;
            host_js_check_error()?;
            Ok(result)
        }
    }
    #[cfg(not(all(target_arch = "wasm32", not(target_os = "wasi"))))]
    {
        let _ = (body, arity, args);
        Err(EvalError::UnsupportedJsFfi)
    }
}

fn host_js_make_wrapper(
    program_handle: u32,
    stable_ptr: i64,
    wrapper_index: u32,
) -> Result<u32, EvalError> {
    #[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
    {
        let stable_ptr = u32::try_from(stable_ptr).map_err(|_| EvalError::Overflow)?;
        let result = unsafe { mhs_js_make_wrapper(program_handle, stable_ptr, wrapper_index) };
        host_js_check_error()?;
        Ok(result)
    }
    #[cfg(not(all(target_arch = "wasm32", not(target_os = "wasi"))))]
    {
        let _ = (program_handle, stable_ptr, wrapper_index);
        Err(EvalError::UnsupportedJsFfi)
    }
}

#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
fn host_js_prepare_call(body: &[u8], arity: usize, args: &[JsArg]) -> Result<i32, EvalError> {
    let body = nul_terminated(body)?;
    let arity = i32::try_from(arity).map_err(|_| EvalError::Overflow)?;
    unsafe {
        mhs_js_setup();
        let idx = mhs_js_register(body.as_ptr(), arity);
        mhs_js_argreset();
        for arg in args {
            match arg {
                JsArg::Int(value) => mhs_js_push_int(*value),
                JsArg::UInt(value) => mhs_js_push_uint(*value),
                JsArg::Double(value) => mhs_js_push_dbl(*value),
                JsArg::Object(value) => mhs_js_push_obj(*value),
                JsArg::String(bytes) => {
                    let len = i32::try_from(bytes.len()).map_err(|_| EvalError::Overflow)?;
                    mhs_js_push_str(bytes.as_ptr(), len);
                }
            }
        }
        Ok(idx)
    }
}

#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
fn host_js_check_error() -> Result<(), EvalError> {
    unsafe {
        if mhs_js_haserr() != 0 {
            mhs_js_logerr();
            return Err(EvalError::UnsupportedJsFfi);
        }
    }
    Ok(())
}

#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
fn nul_terminated(bytes: &[u8]) -> Result<Vec<u8>, EvalError> {
    if bytes.contains(&0) {
        return Err(EvalError::InvalidByteString);
    }
    let mut out = Vec::with_capacity(bytes.len() + 1);
    out.extend_from_slice(bytes);
    out.push(0);
    Ok(out)
}

#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
unsafe fn copy_host_c_string(ptr: *const std::os::raw::c_char) -> Result<Vec<u8>, EvalError> {
    if ptr.is_null() {
        return Ok(Vec::new());
    }
    Ok(unsafe { std::ffi::CStr::from_ptr(ptr) }.to_bytes().to_vec())
}

#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
unsafe fn copy_host_bytes(
    ptr: *const std::os::raw::c_char,
    len: usize,
) -> Result<Vec<u8>, EvalError> {
    if ptr.is_null() {
        return if len == 0 {
            Ok(Vec::new())
        } else {
            Err(EvalError::InvalidByteString)
        };
    }
    Ok(unsafe { std::slice::from_raw_parts(ptr.cast::<u8>(), len) }.to_vec())
}

#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
unsafe extern "C" {
    fn mhs_js_debug(ptr: *const u8);
    fn mhs_js_eval_run(ptr: *const u8);
    fn mhs_js_eval_call(ptr: *const u8) -> *const std::os::raw::c_char;
    fn mhs_js_set_haskellCallback(callback: i32);
    fn mhs_js_setup();
    fn mhs_js_register(body: *const u8, arity: i32) -> i32;
    fn mhs_js_argreset();
    fn mhs_js_push_int(value: i32);
    fn mhs_js_push_uint(value: u32);
    fn mhs_js_push_dbl(value: f64);
    fn mhs_js_push_obj(handle: u32);
    fn mhs_js_push_str(ptr: *const u8, len: i32);
    fn mhs_js_call_int(idx: i32) -> i32;
    fn mhs_js_call_uint(idx: i32) -> u32;
    fn mhs_js_call_dbl(idx: i32) -> f64;
    fn mhs_js_call_ptr(idx: i32) -> u32;
    fn mhs_js_call_obj(idx: i32) -> u32;
    fn mhs_js_call_bool(idx: i32) -> i32;
    fn mhs_js_call_str(idx: i32) -> *const std::os::raw::c_char;
    fn mhs_js_call_void(idx: i32);
    fn mhs_js_make_wrapper(program_handle: u32, stable_ptr: u32, wrapper_index: u32) -> u32;
    fn mhs_js_slen() -> i32;
    fn mhs_js_haserr() -> i32;
    fn mhs_js_logerr();
}

const MPZ_BASE: u32 = 1_000_000_000;

#[derive(Clone, Debug, Eq, PartialEq)]
struct MpzValue {
    negative: bool,
    digits: Vec<u32>,
}

impl MpzValue {
    fn zero() -> Self {
        Self {
            negative: false,
            digits: Vec::new(),
        }
    }

    fn one() -> Self {
        Self {
            negative: false,
            digits: vec![1],
        }
    }

    fn from_u64(mut value: u64) -> Self {
        let mut digits = Vec::new();
        let base = u64::from(MPZ_BASE);
        while value != 0 {
            digits.push((value % base) as u32);
            value /= base;
        }
        Self {
            negative: false,
            digits,
        }
    }

    fn from_i64(value: i64) -> Self {
        let mut out = Self::from_u64(value.unsigned_abs());
        out.negative = value < 0 && !out.is_zero();
        out
    }

    fn parse_decimal(bytes: &[u8]) -> Result<Self, ()> {
        let (negative, digits) = match bytes {
            [b'-', rest @ ..] => (true, rest),
            [b'+', rest @ ..] => (false, rest),
            rest => (false, rest),
        };
        if digits.is_empty() {
            return Err(());
        }
        let mut value = Self::zero();
        for &byte in digits {
            if !byte.is_ascii_digit() {
                return Err(());
            }
            value.mul_small_mut(10);
            value.add_small_mut(u32::from(byte - b'0'));
        }
        value.negative = negative && !value.is_zero();
        Ok(value)
    }

    fn to_decimal_bytes(&self) -> Vec<u8> {
        if self.is_zero() {
            return b"0".to_vec();
        }
        let mut out = Vec::new();
        if self.negative {
            out.push(b'-');
        }
        let mut digits = self.digits.iter().rev();
        if let Some(first) = digits.next() {
            out.extend(first.to_string().into_bytes());
        }
        for digit in digits {
            out.extend(format!("{digit:09}").into_bytes());
        }
        out
    }

    fn normalize(&mut self) {
        while self.digits.last() == Some(&0) {
            self.digits.pop();
        }
        if self.digits.is_empty() {
            self.negative = false;
        }
    }

    fn normalized(mut self) -> Self {
        self.normalize();
        self
    }

    fn is_zero(&self) -> bool {
        self.digits.is_empty()
    }

    fn abs(&self) -> Self {
        let mut out = self.clone();
        out.negative = false;
        out
    }

    fn cmp_abs(&self, other: &Self) -> Ordering {
        match self.digits.len().cmp(&other.digits.len()) {
            Ordering::Equal => {
                for (left, right) in self.digits.iter().rev().zip(other.digits.iter().rev()) {
                    match left.cmp(right) {
                        Ordering::Equal => {}
                        ordering => return ordering,
                    }
                }
                Ordering::Equal
            }
            ordering => ordering,
        }
    }

    fn cmp(&self, other: &Self) -> Ordering {
        match (self.negative, other.negative) {
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            (false, false) => self.cmp_abs(other),
            (true, true) => other.cmp_abs(self),
        }
    }

    fn abs_add(&self, other: &Self) -> Self {
        let mut out = Vec::with_capacity(self.digits.len().max(other.digits.len()) + 1);
        let mut carry = 0_u64;
        let base = u64::from(MPZ_BASE);
        let len = self.digits.len().max(other.digits.len());
        for idx in 0..len {
            let left = u64::from(*self.digits.get(idx).unwrap_or(&0));
            let right = u64::from(*other.digits.get(idx).unwrap_or(&0));
            let sum = left + right + carry;
            out.push((sum % base) as u32);
            carry = sum / base;
        }
        if carry != 0 {
            out.push(carry as u32);
        }
        Self {
            negative: false,
            digits: out,
        }
        .normalized()
    }

    fn abs_sub(&self, other: &Self) -> Self {
        debug_assert!(self.cmp_abs(other) != Ordering::Less);
        let mut out = Vec::with_capacity(self.digits.len());
        let mut borrow = 0_i64;
        let base = i64::from(MPZ_BASE);
        for idx in 0..self.digits.len() {
            let left = i64::from(self.digits[idx]) - borrow;
            let right = i64::from(*other.digits.get(idx).unwrap_or(&0));
            if left < right {
                out.push((left + base - right) as u32);
                borrow = 1;
            } else {
                out.push((left - right) as u32);
                borrow = 0;
            }
        }
        Self {
            negative: false,
            digits: out,
        }
        .normalized()
    }

    fn add(&self, other: &Self) -> Self {
        if self.negative == other.negative {
            let mut out = self.abs_add(other);
            out.negative = self.negative && !out.is_zero();
            return out;
        }
        match self.cmp_abs(other) {
            Ordering::Greater => {
                let mut out = self.abs_sub(other);
                out.negative = self.negative && !out.is_zero();
                out
            }
            Ordering::Less => {
                let mut out = other.abs_sub(self);
                out.negative = other.negative && !out.is_zero();
                out
            }
            Ordering::Equal => Self::zero(),
        }
    }

    fn sub(&self, other: &Self) -> Self {
        let mut neg_other = other.clone();
        if !neg_other.is_zero() {
            neg_other.negative = !neg_other.negative;
        }
        self.add(&neg_other)
    }

    fn mul(&self, other: &Self) -> Self {
        if self.is_zero() || other.is_zero() {
            return Self::zero();
        }
        let base = u64::from(MPZ_BASE);
        let mut out = vec![0_u64; self.digits.len() + other.digits.len()];
        for (i, &left) in self.digits.iter().enumerate() {
            let mut carry = 0_u64;
            for (j, &right) in other.digits.iter().enumerate() {
                let idx = i + j;
                let raw = out[idx] + u64::from(left) * u64::from(right) + carry;
                out[idx] = raw % base;
                carry = raw / base;
            }
            if carry != 0 {
                out[i + other.digits.len()] += carry;
            }
        }
        let mut digits = Vec::with_capacity(out.len());
        let mut carry = 0_u64;
        for raw in out {
            let raw = raw + carry;
            digits.push((raw % base) as u32);
            carry = raw / base;
        }
        while carry != 0 {
            digits.push((carry % base) as u32);
            carry /= base;
        }
        Self {
            negative: self.negative != other.negative,
            digits,
        }
        .normalized()
    }

    fn mul_small_mut(&mut self, value: u32) {
        if self.is_zero() || value == 1 {
            return;
        }
        if value == 0 {
            self.digits.clear();
            self.negative = false;
            return;
        }
        let base = u64::from(MPZ_BASE);
        let mut carry = 0_u64;
        for digit in &mut self.digits {
            let raw = u64::from(*digit) * u64::from(value) + carry;
            *digit = (raw % base) as u32;
            carry = raw / base;
        }
        while carry != 0 {
            self.digits.push((carry % base) as u32);
            carry /= base;
        }
    }

    fn add_small_mut(&mut self, value: u32) {
        if value == 0 {
            return;
        }
        let base = u64::from(MPZ_BASE);
        let mut carry = u64::from(value);
        let mut idx = 0;
        while carry != 0 {
            if idx == self.digits.len() {
                self.digits.push(0);
            }
            let raw = u64::from(self.digits[idx]) + carry;
            self.digits[idx] = (raw % base) as u32;
            carry = raw / base;
            idx += 1;
        }
    }

    fn div2_mut(&mut self) -> bool {
        let mut rem = 0_u64;
        let base = u64::from(MPZ_BASE);
        for digit in self.digits.iter_mut().rev() {
            let raw = rem * base + u64::from(*digit);
            *digit = (raw / 2) as u32;
            rem = raw % 2;
        }
        self.normalize();
        rem != 0
    }

    fn shl1_mut(&mut self) {
        self.mul_small_mut(2);
    }

    fn shl_bits(mut self, bits: usize) -> Self {
        for _ in 0..bits {
            self.shl1_mut();
        }
        self
    }

    fn shr_abs_bits(&self, bits: usize) -> (Self, bool) {
        let mut out = self.abs();
        let mut dropped = false;
        for _ in 0..bits {
            dropped |= out.div2_mut();
        }
        (out, dropped)
    }

    fn fdiv_q_2exp(&self, bits: usize) -> Self {
        let (mut quot, dropped) = self.shr_abs_bits(bits);
        if self.negative {
            if dropped {
                quot.add_small_mut(1);
            }
            if !quot.is_zero() {
                quot.negative = true;
            }
        }
        quot
    }

    fn to_bits_abs(&self) -> Vec<bool> {
        let mut tmp = self.abs();
        let mut bits = Vec::new();
        while !tmp.is_zero() {
            bits.push(tmp.div2_mut());
        }
        bits
    }

    fn from_bits_abs(bits: &[bool]) -> Self {
        let mut out = Self::zero();
        for bit in bits.iter().rev() {
            out.shl1_mut();
            if *bit {
                out.add_small_mut(1);
            }
        }
        out
    }

    fn one_shl(bits: usize) -> Self {
        Self::one().shl_bits(bits)
    }

    fn div_rem_abs(&self, divisor: &Self) -> Result<(Self, Self), EvalError> {
        if divisor.is_zero() {
            return Err(EvalError::InvalidByteString);
        }
        if self.cmp_abs(divisor) == Ordering::Less {
            return Ok((Self::zero(), self.abs()));
        }
        let bits = self.to_bits_abs();
        let mut quot = Self::zero();
        let mut rem = Self::zero();
        for bit in bits.iter().rev() {
            rem.shl1_mut();
            if *bit {
                rem.add_small_mut(1);
            }
            quot.shl1_mut();
            if rem.cmp_abs(divisor) != Ordering::Less {
                rem = rem.abs_sub(divisor);
                quot.add_small_mut(1);
            }
        }
        Ok((quot, rem))
    }

    fn tdiv_qr(&self, divisor: &Self) -> Result<(Self, Self), EvalError> {
        let (mut quot, mut rem) = self.abs().div_rem_abs(&divisor.abs())?;
        quot.negative = self.negative != divisor.negative && !quot.is_zero();
        rem.negative = self.negative && !rem.is_zero();
        Ok((quot, rem))
    }

    fn bit_len(&self) -> usize {
        self.to_bits_abs().len()
    }

    fn to_twos_bits(&self, width: usize) -> Vec<bool> {
        let mut bits = if self.negative {
            Self::one_shl(width).sub(&self.abs()).to_bits_abs()
        } else {
            self.to_bits_abs()
        };
        bits.resize(width, false);
        bits
    }

    fn from_twos_bits(bits: &[bool]) -> Self {
        if bits.last() != Some(&true) {
            return Self::from_bits_abs(bits);
        }
        let unsigned = Self::from_bits_abs(bits);
        let mut out = Self::one_shl(bits.len()).sub(&unsigned);
        if !out.is_zero() {
            out.negative = true;
        }
        out
    }

    fn bitwise(&self, other: &Self, op: fn(bool, bool) -> bool) -> Self {
        let width = self.bit_len().max(other.bit_len()) + 1;
        let left = self.to_twos_bits(width);
        let right = other.to_twos_bits(width);
        let bits: Vec<bool> = left
            .into_iter()
            .zip(right)
            .map(|(left, right)| op(left, right))
            .collect();
        Self::from_twos_bits(&bits)
    }

    fn bitand(&self, other: &Self) -> Self {
        self.bitwise(other, |left, right| left & right)
    }

    fn bitor(&self, other: &Self) -> Self {
        self.bitwise(other, |left, right| left | right)
    }

    fn bitxor(&self, other: &Self) -> Self {
        self.bitwise(other, |left, right| left ^ right)
    }

    fn test_bit_abs(&self, bit: usize) -> bool {
        self.to_bits_abs().get(bit).copied().unwrap_or(false)
    }

    fn test_bit_signed(&self, bit: usize) -> bool {
        if !self.negative {
            return self.test_bit_abs(bit);
        }
        let shifted = self.fdiv_q_2exp(bit);
        shifted.abs().test_bit_abs(0)
    }

    fn signed_popcount(&self) -> Result<i64, EvalError> {
        let count = i64::try_from(self.to_bits_abs().into_iter().filter(|bit| *bit).count())
            .map_err(|_| EvalError::Overflow)?;
        Ok(if self.negative { -count } else { count })
    }

    fn log2(&self) -> Result<i64, EvalError> {
        i64::try_from(self.bit_len().saturating_sub(1)).map_err(|_| EvalError::Overflow)
    }

    fn to_u64_low(&self) -> u64 {
        let mut out = 0_u64;
        for (idx, bit) in self.to_bits_abs().into_iter().take(64).enumerate() {
            if bit {
                out |= 1_u64 << idx;
            }
        }
        out
    }

    fn to_i64_wrapping(&self) -> i64 {
        let low = self.to_u64_low();
        if self.negative {
            0_u64.wrapping_sub(low) as i64
        } else {
            low as i64
        }
    }

    #[cold]
    #[inline(never)]
    fn to_f64(&self) -> f64 {
        let decimal = self.to_decimal_bytes();
        mpz_decimal_to_f64(&decimal)
    }
}

#[cold]
#[inline(never)]
fn mpz_decimal_to_f64(decimal: &[u8]) -> f64 {
    let text = std::str::from_utf8(decimal).expect("mpz decimal bytes are ASCII");
    text.parse::<f64>()
        .expect("mpz decimal bytes should parse as f64")
}

fn size_of_i64<T>() -> i64 {
    std::mem::size_of::<T>() as i64
}

fn format_float(value: f64) -> String {
    let mut out = value.to_string();
    if out == "NaN" {
        out = "nan".to_owned();
    }
    if out != "nan"
        && out != "-nan"
        && out != "inf"
        && out != "-inf"
        && !out.contains('.')
        && !out.contains('e')
        && !out.contains('E')
    {
        out.push_str(".0");
    }
    out
}

fn current_time_micro() -> i64 {
    #[cfg(target_arch = "wasm32")]
    {
        0
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::time::{SystemTime, UNIX_EPOCH};

        let Ok(duration) = SystemTime::now().duration_since(UNIX_EPOCH) else {
            return 0;
        };
        i64::try_from(duration.as_micros()).unwrap_or(i64::MAX)
    }
}

#[cfg(not(any(unix, target_arch = "wasm32")))]
fn current_time_nanos() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};

    let Ok(duration) = SystemTime::now().duration_since(UNIX_EPOCH) else {
        return 0;
    };
    u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX)
}

#[cfg(all(
    any(target_os = "linux", target_os = "android"),
    not(target_arch = "wasm32")
))]
fn cpu_time() -> (u64, u64) {
    let mut ts = std::mem::MaybeUninit::<libc::timespec>::uninit();
    // SAFETY: clock_gettime writes the timespec on success. The pointer is valid for one call.
    let rc = unsafe { libc::clock_gettime(libc::CLOCK_PROCESS_CPUTIME_ID, ts.as_mut_ptr()) };
    if rc != 0 {
        return (0, 0);
    }
    // SAFETY: the call above succeeded, so the timespec has been initialized.
    let ts = unsafe { ts.assume_init() };
    (
        u64::try_from(ts.tv_sec).unwrap_or(0),
        u64::try_from(ts.tv_nsec).unwrap_or(0),
    )
}

#[cfg(any(
    target_arch = "wasm32",
    not(any(target_os = "linux", target_os = "android"))
))]
fn cpu_time() -> (u64, u64) {
    (0, 0)
}

fn errno_i32(name: &str) -> i32 {
    errno_constant(name).unwrap_or(-1) as i32
}

fn host_constant(name: &str) -> Option<i64> {
    #[cfg(all(unix, not(target_arch = "wasm32")))]
    {
        return Some(i64::from(match name {
            "F_SETFL" => libc::F_SETFL,
            "O_NONBLOCK" => libc::O_NONBLOCK,
            "SOL_SOCKET" => libc::SOL_SOCKET,
            "SO_DEBUG" => libc::SO_DEBUG,
            "SO_ERROR" => libc::SO_ERROR,
            "SO_REUSEADDR" => libc::SO_REUSEADDR,
            "SO_TYPE" => libc::SO_TYPE,
            _ => return None,
        }));
    }
    #[cfg(not(all(unix, not(target_arch = "wasm32"))))]
    {
        const HOST_CONSTANTS: &[&str] = &[
            "F_SETFL",
            "O_NONBLOCK",
            "SOL_SOCKET",
            "SO_DEBUG",
            "SO_ERROR",
            "SO_REUSEADDR",
            "SO_TYPE",
        ];
        HOST_CONSTANTS.contains(&name).then_some(-1)
    }
}

#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
fn last_errno() -> i32 {
    std::io::Error::last_os_error()
        .raw_os_error()
        .unwrap_or_else(|| errno_i32("ENOENT"))
}

#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
fn io_error_errno(error: &std::io::Error) -> Option<i32> {
    error.raw_os_error()
}

fn strerror_bytes(errno: i32) -> Vec<u8> {
    std::io::Error::from_raw_os_error(errno)
        .to_string()
        .into_bytes()
}

#[cfg(target_os = "wasi")]
fn wasi_trace_enabled() -> bool {
    std::env::var_os("MHS_WASI_TRACE").is_some()
}

#[cfg(target_os = "wasi")]
fn wasi_trace_host(event: &str, detail: &str) {
    use std::sync::atomic::{AtomicUsize, Ordering};

    static COUNT: AtomicUsize = AtomicUsize::new(0);
    if !wasi_trace_enabled() {
        return;
    }
    let count = COUNT.fetch_add(1, Ordering::Relaxed);
    if count < 256 {
        eprintln!("wasi_host[{count}] {event} {detail}");
    } else if count == 256 {
        eprintln!("wasi_host trace capped");
    }
}

#[cfg(target_os = "wasi")]
fn wasi_trace_every(event: &str, interval: usize) {
    use std::sync::atomic::{AtomicUsize, Ordering};

    static COUNT: AtomicUsize = AtomicUsize::new(0);
    if !wasi_trace_enabled() {
        return;
    }
    let count = COUNT.fetch_add(1, Ordering::Relaxed) + 1;
    if count % interval == 0 {
        eprintln!("wasi_host {event} count={count}");
    }
}

#[cfg(all(
    any(target_os = "linux", target_os = "android"),
    not(target_arch = "wasm32")
))]
fn errno_constant(name: &str) -> Option<i64> {
    Some(i64::from(match name {
        "EOK" => 0,
        "E2BIG" => libc::E2BIG,
        "EACCES" => libc::EACCES,
        "EADDRINUSE" => libc::EADDRINUSE,
        "EADDRNOTAVAIL" => libc::EADDRNOTAVAIL,
        "EADV" => libc::EADV,
        "EAFNOSUPPORT" => libc::EAFNOSUPPORT,
        "EAGAIN" => libc::EAGAIN,
        "EALREADY" => libc::EALREADY,
        "EBADF" => libc::EBADF,
        "EBADMSG" => libc::EBADMSG,
        "EBADRPC" => -1,
        "EBUSY" => libc::EBUSY,
        "ECHILD" => libc::ECHILD,
        "ECOMM" => libc::ECOMM,
        "ECONNABORTED" => libc::ECONNABORTED,
        "ECONNREFUSED" => libc::ECONNREFUSED,
        "ECONNRESET" => libc::ECONNRESET,
        "EDEADLK" => libc::EDEADLK,
        "EDESTADDRREQ" => libc::EDESTADDRREQ,
        "EDIRTY" => -1,
        "EDOM" => libc::EDOM,
        "EDQUOT" => libc::EDQUOT,
        "EEXIST" => libc::EEXIST,
        "EFAULT" => libc::EFAULT,
        "EFBIG" => libc::EFBIG,
        "EFTYPE" => -1,
        "EHOSTDOWN" => libc::EHOSTDOWN,
        "EHOSTUNREACH" => libc::EHOSTUNREACH,
        "EIDRM" => libc::EIDRM,
        "EILSEQ" => libc::EILSEQ,
        "EINPROGRESS" => libc::EINPROGRESS,
        "EINTR" => libc::EINTR,
        "EINVAL" => libc::EINVAL,
        "EIO" => libc::EIO,
        "EISCONN" => libc::EISCONN,
        "EISDIR" => libc::EISDIR,
        "ELOOP" => libc::ELOOP,
        "EMFILE" => libc::EMFILE,
        "EMLINK" => libc::EMLINK,
        "EMSGSIZE" => libc::EMSGSIZE,
        "EMULTIHOP" => libc::EMULTIHOP,
        "ENAMETOOLONG" => libc::ENAMETOOLONG,
        "ENETDOWN" => libc::ENETDOWN,
        "ENETRESET" => libc::ENETRESET,
        "ENETUNREACH" => libc::ENETUNREACH,
        "ENFILE" => libc::ENFILE,
        "ENOBUFS" => libc::ENOBUFS,
        "ENODATA" => libc::ENODATA,
        "ENODEV" => libc::ENODEV,
        "ENOENT" => libc::ENOENT,
        "ENOEXEC" => libc::ENOEXEC,
        "ENOLCK" => libc::ENOLCK,
        "ENOLINK" => libc::ENOLINK,
        "ENOMEM" => libc::ENOMEM,
        "ENOMSG" => libc::ENOMSG,
        "ENONET" => libc::ENONET,
        "ENOPROTOOPT" => libc::ENOPROTOOPT,
        "ENOSPC" => libc::ENOSPC,
        "ENOSR" => libc::ENOSR,
        "ENOSTR" => libc::ENOSTR,
        "ENOSYS" => libc::ENOSYS,
        "ENOTBLK" => libc::ENOTBLK,
        "ENOTCONN" => libc::ENOTCONN,
        "ENOTDIR" => libc::ENOTDIR,
        "ENOTEMPTY" => libc::ENOTEMPTY,
        "ENOTSOCK" => libc::ENOTSOCK,
        "ENOTSUP" => libc::ENOTSUP,
        "ENOTTY" => libc::ENOTTY,
        "ENXIO" => libc::ENXIO,
        "EOPNOTSUPP" => libc::EOPNOTSUPP,
        "EPERM" => libc::EPERM,
        "EPFNOSUPPORT" => libc::EPFNOSUPPORT,
        "EPIPE" => libc::EPIPE,
        "EPROCLIM" => -1,
        "EPROCUNAVAIL" => -1,
        "EPROGMISMATCH" => -1,
        "EPROGUNAVAIL" => -1,
        "EPROTO" => libc::EPROTO,
        "EPROTONOSUPPORT" => libc::EPROTONOSUPPORT,
        "EPROTOTYPE" => libc::EPROTOTYPE,
        "ERANGE" => libc::ERANGE,
        "EREMCHG" => libc::EREMCHG,
        "EREMOTE" => libc::EREMOTE,
        "EROFS" => libc::EROFS,
        "ERPCMISMATCH" => -1,
        "ERREMOTE" => -1,
        "ESHUTDOWN" => libc::ESHUTDOWN,
        "ESOCKTNOSUPPORT" => libc::ESOCKTNOSUPPORT,
        "ESPIPE" => libc::ESPIPE,
        "ESRCH" => libc::ESRCH,
        "ESRMNT" => libc::ESRMNT,
        "ESTALE" => libc::ESTALE,
        "ETIME" => libc::ETIME,
        "ETIMEDOUT" => libc::ETIMEDOUT,
        "ETOOMANYREFS" => libc::ETOOMANYREFS,
        "ETXTBSY" => libc::ETXTBSY,
        "EUSERS" => libc::EUSERS,
        "EWOULDBLOCK" => libc::EWOULDBLOCK,
        "EXDEV" => libc::EXDEV,
        _ => return None,
    }))
}

#[cfg(any(
    target_arch = "wasm32",
    not(any(target_os = "linux", target_os = "android"))
))]
fn errno_constant(name: &str) -> Option<i64> {
    if name == "EOK" {
        return Some(0);
    }
    const ERRNO_NAMES: &[&str] = &[
        "E2BIG",
        "EACCES",
        "EADDRINUSE",
        "EADDRNOTAVAIL",
        "EADV",
        "EAFNOSUPPORT",
        "EAGAIN",
        "EALREADY",
        "EBADF",
        "EBADMSG",
        "EBADRPC",
        "EBUSY",
        "ECHILD",
        "ECOMM",
        "ECONNABORTED",
        "ECONNREFUSED",
        "ECONNRESET",
        "EDEADLK",
        "EDESTADDRREQ",
        "EDIRTY",
        "EDOM",
        "EDQUOT",
        "EEXIST",
        "EFAULT",
        "EFBIG",
        "EFTYPE",
        "EHOSTDOWN",
        "EHOSTUNREACH",
        "EIDRM",
        "EILSEQ",
        "EINPROGRESS",
        "EINTR",
        "EINVAL",
        "EIO",
        "EISCONN",
        "EISDIR",
        "ELOOP",
        "EMFILE",
        "EMLINK",
        "EMSGSIZE",
        "EMULTIHOP",
        "ENAMETOOLONG",
        "ENETDOWN",
        "ENETRESET",
        "ENETUNREACH",
        "ENFILE",
        "ENOBUFS",
        "ENODATA",
        "ENODEV",
        "ENOENT",
        "ENOEXEC",
        "ENOLCK",
        "ENOLINK",
        "ENOMEM",
        "ENOMSG",
        "ENONET",
        "ENOPROTOOPT",
        "ENOSPC",
        "ENOSR",
        "ENOSTR",
        "ENOSYS",
        "ENOTBLK",
        "ENOTCONN",
        "ENOTDIR",
        "ENOTEMPTY",
        "ENOTSOCK",
        "ENOTSUP",
        "ENOTTY",
        "ENXIO",
        "EOPNOTSUPP",
        "EPERM",
        "EPFNOSUPPORT",
        "EPIPE",
        "EPROCLIM",
        "EPROCUNAVAIL",
        "EPROGMISMATCH",
        "EPROGUNAVAIL",
        "EPROTO",
        "EPROTONOSUPPORT",
        "EPROTOTYPE",
        "ERANGE",
        "EREMCHG",
        "EREMOTE",
        "EROFS",
        "ERPCMISMATCH",
        "ERREMOTE",
        "ESHUTDOWN",
        "ESOCKTNOSUPPORT",
        "ESPIPE",
        "ESRCH",
        "ESRMNT",
        "ESTALE",
        "ETIME",
        "ETIMEDOUT",
        "ETOOMANYREFS",
        "ETXTBSY",
        "EUSERS",
        "EWOULDBLOCK",
        "EXDEV",
    ];
    ERRNO_NAMES.contains(&name).then_some(-1)
}

#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
unsafe extern "C" {
    fn mhs_host_result_copy(dst: *mut u8, len: usize) -> usize;
    fn mhs_host_getenv(name_ptr: *const u8, name_len: usize) -> isize;
    fn mhs_host_setenv(
        name_ptr: *const u8,
        name_len: usize,
        value_ptr: *const u8,
        value_len: usize,
        overwrite: i64,
    ) -> i64;
    fn mhs_host_unsetenv(name_ptr: *const u8, name_len: usize) -> i64;
    fn mhs_host_environ() -> isize;
    fn mhs_host_remove(path_ptr: *const u8, path_len: usize) -> i64;
    fn mhs_host_chdir(path_ptr: *const u8, path_len: usize) -> i64;
    fn mhs_host_mkdir(path_ptr: *const u8, path_len: usize, mode: i64) -> i64;
    fn mhs_host_getcwd() -> isize;
    fn mhs_host_tmpname(
        pre_ptr: *const u8,
        pre_len: usize,
        suf_ptr: *const u8,
        suf_len: usize,
    ) -> isize;
    fn mhs_host_get_permissions(path_ptr: *const u8, path_len: usize) -> i64;
    fn mhs_host_set_permissions(path_ptr: *const u8, path_len: usize, permissions: i64) -> i64;
    fn mhs_host_dir_entries(path_ptr: *const u8, path_len: usize) -> isize;
    fn mhs_host_file_open(
        path_ptr: *const u8,
        path_len: usize,
        mode_ptr: *const u8,
        mode_len: usize,
    ) -> i64;
    fn mhs_host_file_read(handle: i64, dst: *mut u8, len: usize) -> isize;
    fn mhs_host_file_write(handle: i64, src: *const u8, len: usize) -> isize;
    fn mhs_host_file_flush(handle: i64) -> i64;
    fn mhs_host_file_close(handle: i64) -> i64;
}

#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
fn host_result_i64(rc: i64) -> HostIntResult {
    if rc < 0 {
        HostIntResult::err((-rc) as i32)
    } else {
        HostIntResult::ok(rc)
    }
}

#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
fn host_result_usize(rc: isize) -> Result<usize, i32> {
    if rc < 0 {
        Err((-rc) as i32)
    } else {
        usize::try_from(rc).map_err(|_| errno_i32("EOVERFLOW"))
    }
}

#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
fn copy_host_result(len: usize) -> Vec<u8> {
    let mut bytes = vec![0; len];
    if len != 0 {
        let copied = unsafe { mhs_host_result_copy(bytes.as_mut_ptr(), len) };
        bytes.truncate(copied.min(len));
    }
    bytes
}

#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
fn host_bytes_result(rc: isize) -> Result<Vec<u8>, i32> {
    let len = host_result_usize(rc)?;
    Ok(copy_host_result(len))
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
fn getenv_bytes(name: &[u8]) -> Option<Vec<u8>> {
    use std::ffi::OsStr;
    use std::os::unix::ffi::{OsStrExt, OsStringExt};

    std::env::var_os(OsStr::from_bytes(name)).map(|value| value.into_vec())
}

#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
fn getenv_bytes(name: &[u8]) -> Option<Vec<u8>> {
    let len = unsafe { mhs_host_getenv(name.as_ptr(), name.len()) };
    if len < 0 {
        None
    } else {
        Some(copy_host_result(usize::try_from(len).ok()?))
    }
}

#[cfg(any(target_os = "wasi", not(any(unix, target_arch = "wasm32"))))]
fn getenv_bytes(name: &[u8]) -> Option<Vec<u8>> {
    #[cfg(target_os = "wasi")]
    wasi_trace_host("getenv", &String::from_utf8_lossy(name));
    let name = std::str::from_utf8(name).ok()?;
    std::env::var_os(name).map(|value| value.to_string_lossy().into_owned().into_bytes())
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
fn setenv_bytes(name: &[u8], value: &[u8], overwrite: i64) -> HostIntResult {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    if name.is_empty() || name.contains(&b'=') {
        return HostIntResult::err(errno_i32("EINVAL"));
    }
    let name = OsStr::from_bytes(name);
    if overwrite == 0 && std::env::var_os(name).is_some() {
        return HostIntResult::ok(0);
    }
    // SAFETY: MicroHs executes user code on one runtime thread today; this mirrors C's process-global env.
    unsafe {
        std::env::set_var(name, OsStr::from_bytes(value));
    }
    HostIntResult::ok(0)
}

#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
fn setenv_bytes(name: &[u8], value: &[u8], overwrite: i64) -> HostIntResult {
    host_result_i64(unsafe {
        mhs_host_setenv(
            name.as_ptr(),
            name.len(),
            value.as_ptr(),
            value.len(),
            overwrite,
        )
    })
}

#[cfg(any(target_os = "wasi", not(any(unix, target_arch = "wasm32"))))]
fn setenv_bytes(name: &[u8], value: &[u8], overwrite: i64) -> HostIntResult {
    #[cfg(target_os = "wasi")]
    wasi_trace_host(
        "setenv",
        &format!(
            "name={} value_len={} overwrite={overwrite}",
            String::from_utf8_lossy(name),
            value.len()
        ),
    );
    if name.is_empty() || name.contains(&b'=') {
        return HostIntResult::err(errno_i32("EINVAL"));
    }
    let Ok(name) = std::str::from_utf8(name) else {
        return HostIntResult::err(errno_i32("EINVAL"));
    };
    if overwrite == 0 && std::env::var_os(name).is_some() {
        return HostIntResult::ok(0);
    }
    let value = String::from_utf8_lossy(value);
    // SAFETY: MicroHs executes user code on one runtime thread today; this mirrors C's process-global env.
    unsafe {
        std::env::set_var(name, value.as_ref());
    }
    HostIntResult::ok(0)
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
fn unsetenv_bytes(name: &[u8]) -> HostIntResult {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    if name.is_empty() || name.contains(&b'=') {
        return HostIntResult::err(errno_i32("EINVAL"));
    }
    // SAFETY: MicroHs executes user code on one runtime thread today; this mirrors C's process-global env.
    unsafe {
        std::env::remove_var(OsStr::from_bytes(name));
    }
    HostIntResult::ok(0)
}

#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
fn unsetenv_bytes(name: &[u8]) -> HostIntResult {
    host_result_i64(unsafe { mhs_host_unsetenv(name.as_ptr(), name.len()) })
}

#[cfg(any(target_os = "wasi", not(any(unix, target_arch = "wasm32"))))]
fn unsetenv_bytes(name: &[u8]) -> HostIntResult {
    #[cfg(target_os = "wasi")]
    wasi_trace_host("unsetenv", &String::from_utf8_lossy(name));
    if name.is_empty() || name.contains(&b'=') {
        return HostIntResult::err(errno_i32("EINVAL"));
    }
    let Ok(name) = std::str::from_utf8(name) else {
        return HostIntResult::err(errno_i32("EINVAL"));
    };
    // SAFETY: MicroHs executes user code on one runtime thread today; this mirrors C's process-global env.
    unsafe {
        std::env::remove_var(name);
    }
    HostIntResult::ok(0)
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
fn environ_bytes() -> Vec<Vec<u8>> {
    use std::os::unix::ffi::OsStringExt;

    std::env::vars_os()
        .map(|(name, value)| {
            let mut bytes = name.into_vec();
            bytes.push(b'=');
            bytes.extend(value.into_vec());
            bytes
        })
        .collect()
}

#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
fn environ_bytes() -> Vec<Vec<u8>> {
    let Ok(bytes) = host_bytes_result(unsafe { mhs_host_environ() }) else {
        return Vec::new();
    };
    bytes
        .split(|byte| *byte == 0)
        .filter(|entry| !entry.is_empty())
        .map(Vec::from)
        .collect()
}

#[cfg(any(target_os = "wasi", not(any(unix, target_arch = "wasm32"))))]
fn environ_bytes() -> Vec<Vec<u8>> {
    std::env::vars_os()
        .map(|(name, value)| {
            let mut bytes = name.to_string_lossy().into_owned().into_bytes();
            bytes.push(b'=');
            bytes.extend(value.to_string_lossy().into_owned().into_bytes());
            bytes
        })
        .collect()
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
fn remove_path_bytes(path: &[u8]) -> HostIntResult {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    let path = std::path::Path::new(OsStr::from_bytes(path));
    match std::fs::remove_file(path) {
        Ok(()) => HostIntResult::ok(0),
        Err(file_err) => match std::fs::remove_dir(path) {
            Ok(()) => HostIntResult::ok(0),
            Err(dir_err) => HostIntResult::os_err(
                io_error_errno(&dir_err).or_else(|| io_error_errno(&file_err)),
            ),
        },
    }
}

#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
fn remove_path_bytes(path: &[u8]) -> HostIntResult {
    host_result_i64(unsafe { mhs_host_remove(path.as_ptr(), path.len()) })
}

#[cfg(any(target_os = "wasi", not(any(unix, target_arch = "wasm32"))))]
fn remove_path_bytes(path: &[u8]) -> HostIntResult {
    #[cfg(target_os = "wasi")]
    wasi_trace_host("remove", &String::from_utf8_lossy(path));
    let Ok(path) = std::str::from_utf8(path) else {
        return HostIntResult::err(errno_i32("EINVAL"));
    };
    match std::fs::remove_file(path) {
        Ok(()) => HostIntResult::ok(0),
        Err(file_err) => match std::fs::remove_dir(path) {
            Ok(()) => HostIntResult::ok(0),
            Err(dir_err) => HostIntResult::os_err(
                io_error_errno(&dir_err).or_else(|| io_error_errno(&file_err)),
            ),
        },
    }
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
fn system_command_bytes(command: Option<&[u8]>) -> i64 {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    use std::os::unix::process::ExitStatusExt;

    let Some(command) = command else {
        return 1;
    };
    match std::process::Command::new("/bin/sh")
        .arg("-c")
        .arg(OsStr::from_bytes(command))
        .status()
    {
        Ok(status) => i64::from(status.into_raw()),
        Err(_) => -1,
    }
}

#[cfg(target_arch = "wasm32")]
fn system_command_bytes(command: Option<&[u8]>) -> i64 {
    let _ = command;
    -1
}

#[cfg(not(any(unix, target_arch = "wasm32")))]
fn system_command_bytes(command: Option<&[u8]>) -> i64 {
    let Some(command) = command else {
        return 1;
    };
    let Ok(command) = std::str::from_utf8(command) else {
        return -1;
    };
    match std::process::Command::new("cmd")
        .arg("/C")
        .arg(command)
        .status()
    {
        Ok(status) => status.code().map_or(-1, i64::from),
        Err(_) => -1,
    }
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
fn chdir_path_bytes(path: &[u8]) -> HostIntResult {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    let path = std::path::Path::new(OsStr::from_bytes(path));
    match std::env::set_current_dir(path) {
        Ok(()) => HostIntResult::ok(0),
        Err(err) => HostIntResult::os_err(io_error_errno(&err)),
    }
}

#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
fn chdir_path_bytes(path: &[u8]) -> HostIntResult {
    host_result_i64(unsafe { mhs_host_chdir(path.as_ptr(), path.len()) })
}

#[cfg(any(target_os = "wasi", not(any(unix, target_arch = "wasm32"))))]
fn chdir_path_bytes(path: &[u8]) -> HostIntResult {
    #[cfg(target_os = "wasi")]
    wasi_trace_host("chdir", &String::from_utf8_lossy(path));
    let Ok(path) = std::str::from_utf8(path) else {
        return HostIntResult::err(errno_i32("EINVAL"));
    };
    match std::env::set_current_dir(path) {
        Ok(()) => HostIntResult::ok(0),
        Err(err) => HostIntResult::os_err(io_error_errno(&err)),
    }
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
fn mkdir_path_bytes(path: &[u8], mode: i64) -> HostIntResult {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    use std::os::unix::fs::DirBuilderExt;

    let Ok(mode) = u32::try_from(mode) else {
        return HostIntResult::err(errno_i32("EINVAL"));
    };
    let path = std::path::Path::new(OsStr::from_bytes(path));
    match std::fs::DirBuilder::new().mode(mode).create(path) {
        Ok(()) => HostIntResult::ok(0),
        Err(err) => HostIntResult::os_err(io_error_errno(&err)),
    }
}

#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
fn mkdir_path_bytes(path: &[u8], mode: i64) -> HostIntResult {
    host_result_i64(unsafe { mhs_host_mkdir(path.as_ptr(), path.len(), mode) })
}

#[cfg(any(target_os = "wasi", not(any(unix, target_arch = "wasm32"))))]
fn mkdir_path_bytes(path: &[u8], mode: i64) -> HostIntResult {
    #[cfg(target_os = "wasi")]
    wasi_trace_host(
        "mkdir",
        &format!("path={} mode={mode}", String::from_utf8_lossy(path)),
    );
    let _ = mode;
    let Ok(path) = std::str::from_utf8(path) else {
        return HostIntResult::err(errno_i32("EINVAL"));
    };
    match std::fs::create_dir(path) {
        Ok(()) => HostIntResult::ok(0),
        Err(err) => HostIntResult::os_err(io_error_errno(&err)),
    }
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
fn current_dir_bytes() -> Result<Vec<u8>, i32> {
    use std::os::unix::ffi::OsStringExt;

    std::env::current_dir()
        .map(|path| path.into_os_string().into_vec())
        .map_err(|err| io_error_errno(&err).unwrap_or_else(|| errno_i32("ENOENT")))
}

#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
fn current_dir_bytes() -> Result<Vec<u8>, i32> {
    host_bytes_result(unsafe { mhs_host_getcwd() })
}

#[cfg(any(target_os = "wasi", not(any(unix, target_arch = "wasm32"))))]
fn current_dir_bytes() -> Result<Vec<u8>, i32> {
    #[cfg(target_os = "wasi")]
    wasi_trace_host("getcwd", "");
    std::env::current_dir()
        .map(|path| path.to_string_lossy().into_owned().into_bytes())
        .map_err(|err| io_error_errno(&err).unwrap_or_else(|| errno_i32("ENOENT")))
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
fn executable_path_bytes() -> Result<Vec<u8>, i32> {
    use std::os::unix::ffi::OsStringExt;

    std::env::current_exe()
        .map(|path| path.into_os_string().into_vec())
        .map_err(|err| io_error_errno(&err).unwrap_or_else(|| errno_i32("ENOENT")))
}

#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
fn executable_path_bytes() -> Result<Vec<u8>, i32> {
    Err(errno_i32("ENOSYS"))
}

#[cfg(any(target_os = "wasi", not(any(unix, target_arch = "wasm32"))))]
fn executable_path_bytes() -> Result<Vec<u8>, i32> {
    #[cfg(target_os = "wasi")]
    wasi_trace_host("get_executable_path", "");
    std::env::current_exe()
        .map(|path| path.to_string_lossy().into_owned().into_bytes())
        .map_err(|err| io_error_errno(&err).unwrap_or_else(|| errno_i32("ENOENT")))
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
fn tmpname_bytes(pre: &[u8], suf: &[u8]) -> Result<Vec<u8>, i32> {
    use std::ffi::{CString, OsString};
    use std::os::unix::ffi::OsStringExt;
    use std::os::unix::io::RawFd;

    let tmpdir = std::env::var_os("TMPDIR")
        .unwrap_or_else(|| OsString::from("/tmp"))
        .into_vec();
    let mut template = Vec::with_capacity(tmpdir.len() + pre.len() + suf.len() + 8);
    template.extend_from_slice(&tmpdir);
    template.push(b'/');
    template.extend_from_slice(pre);
    template.extend_from_slice(b"XXXXXX");
    template.extend_from_slice(suf);
    template.push(0);
    let suffix_len = std::os::raw::c_int::try_from(suf.len()).map_err(|_| errno_i32("EINVAL"))?;
    let path = CString::from_vec_with_nul(template).map_err(|_| errno_i32("EINVAL"))?;
    let mut bytes = path.into_bytes_with_nul();
    // SAFETY: mkstemps mutates the NUL-terminated template in place and returns a file descriptor.
    let fd: RawFd = unsafe { libc::mkstemps(bytes.as_mut_ptr().cast(), suffix_len) };
    if fd < 0 {
        return Err(last_errno());
    }
    // SAFETY: fd came from mkstemps and is not used after this close.
    unsafe {
        libc::close(fd);
    }
    bytes.pop();
    Ok(bytes)
}

#[cfg(target_arch = "wasm32")]
fn tmpname_bytes(pre: &[u8], suf: &[u8]) -> Result<Vec<u8>, i32> {
    #[cfg(target_os = "wasi")]
    {
        let _ = (pre, suf);
        Err(errno_i32("ENOSYS"))
    }
    #[cfg(not(target_os = "wasi"))]
    {
        host_bytes_result(unsafe {
            mhs_host_tmpname(pre.as_ptr(), pre.len(), suf.as_ptr(), suf.len())
        })
    }
}

#[cfg(not(any(unix, target_arch = "wasm32")))]
fn tmpname_bytes(pre: &[u8], suf: &[u8]) -> Result<Vec<u8>, i32> {
    let pre = std::str::from_utf8(pre).map_err(|_| errno_i32("EINVAL"))?;
    let suf = std::str::from_utf8(suf).map_err(|_| errno_i32("EINVAL"))?;
    let tmpdir = std::env::temp_dir();
    let seed = current_time_nanos() ^ u64::from(std::process::id());
    for attempt in 0..1024 {
        let mut name = String::with_capacity(pre.len() + 6 + suf.len());
        name.push_str(pre);
        name.push_str(
            std::str::from_utf8(&tmp_six(seed.wrapping_add(attempt))).unwrap_or("XXXXXX"),
        );
        name.push_str(suf);
        let path = tmpdir.join(name);
        match std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(_) => return Ok(path.to_string_lossy().into_owned().into_bytes()),
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(err) => return Err(io_error_errno(&err).unwrap_or_else(|| errno_i32("ENOENT"))),
        }
    }
    Err(errno_i32("EEXIST"))
}

#[cfg(not(any(unix, target_arch = "wasm32")))]
fn tmp_six(mut value: u64) -> [u8; 6] {
    const ALPHABET: &[u8; 36] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let mut out = [b'0'; 6];
    for byte in &mut out {
        *byte = ALPHABET[(value % 36) as usize];
        value /= 36;
    }
    out
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
fn get_permissions_path_bytes(path: &[u8]) -> HostIntResult {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    use std::os::unix::fs::PermissionsExt;

    let path = std::path::Path::new(OsStr::from_bytes(path));
    let metadata = match std::fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(err) => return HostIntResult::os_err(io_error_errno(&err)),
    };
    let mode = metadata.permissions().mode();
    let mut permissions = 0;
    if mode & 0o400 != 0 {
        permissions |= 4;
    }
    if mode & 0o200 != 0 {
        permissions |= 2;
    }
    if mode & 0o100 != 0 {
        permissions |= if metadata.is_dir() { 8 } else { 1 };
    }
    HostIntResult::ok(permissions)
}

#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
fn get_permissions_path_bytes(path: &[u8]) -> HostIntResult {
    host_result_i64(unsafe { mhs_host_get_permissions(path.as_ptr(), path.len()) })
}

#[cfg(any(target_os = "wasi", not(any(unix, target_arch = "wasm32"))))]
fn get_permissions_path_bytes(path: &[u8]) -> HostIntResult {
    #[cfg(target_os = "wasi")]
    wasi_trace_host("get_permissions", &String::from_utf8_lossy(path));
    let Ok(path) = std::str::from_utf8(path) else {
        return HostIntResult::err(errno_i32("EINVAL"));
    };
    let metadata = match std::fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(err) => return HostIntResult::os_err(io_error_errno(&err)),
    };
    let mut permissions = 4;
    if !metadata.permissions().readonly() {
        permissions |= 2;
    }
    if metadata.is_dir() {
        permissions |= 8;
    }
    HostIntResult::ok(permissions)
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
fn set_permissions_path_bytes(path: &[u8], permissions: i64) -> HostIntResult {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    use std::os::unix::fs::PermissionsExt;

    unsafe extern "C" {
        fn umask(mask: u32) -> u32;
    }

    let Ok(permissions) = u32::try_from(permissions) else {
        return HostIntResult::err(errno_i32("EINVAL"));
    };
    let path = std::path::Path::new(OsStr::from_bytes(path));
    let file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(err) => return HostIntResult::os_err(io_error_errno(&err)),
    };
    let metadata = match file.metadata() {
        Ok(metadata) => metadata,
        Err(err) => return HostIntResult::os_err(io_error_errno(&err)),
    };
    let mut user_mode = 0;
    if permissions & 4 != 0 {
        user_mode |= 0o400;
    }
    if permissions & 2 != 0 {
        user_mode |= 0o200;
    }
    if permissions & 1 != 0 || permissions & 8 != 0 {
        user_mode |= 0o100;
    }
    let mut mode = user_mode | (user_mode >> 3) | (user_mode >> 6);
    // SAFETY: umask is process-global like in the C runtime. We restore it immediately.
    let mask = unsafe { umask(0) };
    // SAFETY: restores the mask value just read above.
    unsafe {
        umask(mask);
    }
    mode &= !mask;
    mode |= metadata.permissions().mode() & !0o777;
    match file.set_permissions(std::fs::Permissions::from_mode(mode)) {
        Ok(()) => HostIntResult::ok(0),
        Err(err) => HostIntResult::os_err(io_error_errno(&err)),
    }
}

#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
fn set_permissions_path_bytes(path: &[u8], permissions: i64) -> HostIntResult {
    host_result_i64(unsafe { mhs_host_set_permissions(path.as_ptr(), path.len(), permissions) })
}

#[cfg(any(target_os = "wasi", not(any(unix, target_arch = "wasm32"))))]
fn set_permissions_path_bytes(path: &[u8], permissions: i64) -> HostIntResult {
    #[cfg(target_os = "wasi")]
    wasi_trace_host(
        "set_permissions",
        &format!(
            "path={} permissions={permissions}",
            String::from_utf8_lossy(path)
        ),
    );
    let _ = permissions;
    let Ok(path) = std::str::from_utf8(path) else {
        return HostIntResult::err(errno_i32("EINVAL"));
    };
    let metadata = match std::fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(err) => return HostIntResult::os_err(io_error_errno(&err)),
    };
    let mut permissions = metadata.permissions();
    permissions.set_readonly(false);
    match std::fs::set_permissions(path, permissions) {
        Ok(()) => HostIntResult::ok(0),
        Err(err) => HostIntResult::os_err(io_error_errno(&err)),
    }
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
fn dir_entries_path_bytes(path: &[u8]) -> Result<Vec<Vec<u8>>, i32> {
    use std::ffi::OsStr;
    use std::os::unix::ffi::{OsStrExt, OsStringExt};

    let path = std::path::Path::new(OsStr::from_bytes(path));
    let mut entries = vec![b".".to_vec(), b"..".to_vec()];
    for entry in std::fs::read_dir(path)
        .map_err(|err| io_error_errno(&err).unwrap_or_else(|| errno_i32("ENOENT")))?
    {
        let entry =
            entry.map_err(|err| io_error_errno(&err).unwrap_or_else(|| errno_i32("ENOENT")))?;
        entries.push(entry.file_name().into_vec());
    }
    Ok(entries)
}

#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
fn dir_entries_path_bytes(path: &[u8]) -> Result<Vec<Vec<u8>>, i32> {
    let bytes = host_bytes_result(unsafe { mhs_host_dir_entries(path.as_ptr(), path.len()) })?;
    Ok(bytes
        .split(|byte| *byte == 0)
        .filter(|entry| !entry.is_empty())
        .map(Vec::from)
        .collect())
}

#[cfg(any(target_os = "wasi", not(any(unix, target_arch = "wasm32"))))]
fn dir_entries_path_bytes(path: &[u8]) -> Result<Vec<Vec<u8>>, i32> {
    #[cfg(target_os = "wasi")]
    wasi_trace_host("opendir", &String::from_utf8_lossy(path));
    let path = std::str::from_utf8(path).map_err(|_| errno_i32("EINVAL"))?;
    let mut entries = vec![b".".to_vec(), b"..".to_vec()];
    for entry in std::fs::read_dir(path)
        .map_err(|err| io_error_errno(&err).unwrap_or_else(|| errno_i32("ENOENT")))?
    {
        let entry =
            entry.map_err(|err| io_error_errno(&err).unwrap_or_else(|| errno_i32("ENOENT")))?;
        entries.push(
            entry
                .file_name()
                .to_string_lossy()
                .into_owned()
                .into_bytes(),
        );
    }
    Ok(entries)
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
fn native_fopen_bfile(path: &[u8], mode: &[u8]) -> Result<BFile, i32> {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    let mode = parse_native_file_mode(mode).ok_or_else(|| errno_i32("EINVAL"))?;
    let file = open_native_file(std::path::Path::new(OsStr::from_bytes(path)), mode)?;
    Ok(BFile {
        kind: BFileKind::NativeFile {
            file: std::rc::Rc::new(std::cell::RefCell::new(file)),
            ungot: Vec::new(),
        },
        readable: mode.readable,
        writable: mode.writable,
    })
}

#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
fn native_fopen_bfile(path: &[u8], mode: &[u8]) -> Result<BFile, i32> {
    let handle =
        unsafe { mhs_host_file_open(path.as_ptr(), path.len(), mode.as_ptr(), mode.len()) };
    if handle < 0 {
        return Err((-handle) as i32);
    }
    let mode = parse_native_file_mode(mode).ok_or_else(|| errno_i32("EINVAL"))?;
    Ok(BFile {
        kind: BFileKind::BrowserFile {
            handle,
            ungot: Vec::new(),
        },
        readable: mode.readable,
        writable: mode.writable,
    })
}

#[cfg(any(target_os = "wasi", not(any(unix, target_arch = "wasm32"))))]
fn native_fopen_bfile(path: &[u8], mode: &[u8]) -> Result<BFile, i32> {
    #[cfg(target_os = "wasi")]
    wasi_trace_host(
        "fopen",
        &format!(
            "path={} mode={}",
            String::from_utf8_lossy(path),
            String::from_utf8_lossy(mode)
        ),
    );
    let path = std::str::from_utf8(path).map_err(|_| errno_i32("EINVAL"))?;
    let mode = parse_native_file_mode(mode).ok_or_else(|| errno_i32("EINVAL"))?;
    let file = open_native_file(std::path::Path::new(path), mode)?;
    Ok(BFile {
        kind: BFileKind::NativeFile {
            file: std::rc::Rc::new(std::cell::RefCell::new(file)),
            ungot: Vec::new(),
        },
        readable: mode.readable,
        writable: mode.writable,
    })
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
fn native_fd_bfile(fd: i32) -> Result<BFile, i32> {
    use std::os::unix::io::FromRawFd;

    if fd < 0 {
        return Err(errno_i32("EBADF"));
    }
    // SAFETY: add_fd transfers fd ownership to the BFILE, matching the C runtime closeb_fd path.
    let file = unsafe { std::fs::File::from_raw_fd(fd) };
    Ok(BFile {
        kind: BFileKind::NativeFile {
            file: std::rc::Rc::new(std::cell::RefCell::new(file)),
            ungot: Vec::new(),
        },
        readable: true,
        writable: true,
    })
}

#[cfg(not(all(unix, not(target_arch = "wasm32"))))]
fn native_fd_bfile(fd: i32) -> Result<BFile, i32> {
    let _ = fd;
    Err(errno_i32("ENOSYS"))
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
fn open_fd_path_bytes(path: &[u8], flags: i32, mode: i64) -> HostIntResult {
    let path = match std::ffi::CString::new(path) {
        Ok(path) => path,
        Err(_) => return HostIntResult::err(errno_i32("EINVAL")),
    };
    let mode = match libc::mode_t::try_from(mode) {
        Ok(mode) => mode,
        Err(_) => return HostIntResult::err(errno_i32("EINVAL")),
    };
    // SAFETY: path is NUL-terminated and flags/mode are plain C values.
    let fd = unsafe { libc::open(path.as_ptr(), flags, mode) };
    if fd < 0 {
        HostIntResult::err(last_errno())
    } else {
        HostIntResult::ok(i64::from(fd))
    }
}

#[cfg(not(all(unix, not(target_arch = "wasm32"))))]
fn open_fd_path_bytes(path: &[u8], flags: i32, mode: i64) -> HostIntResult {
    let _ = (path, flags, mode);
    HostIntResult::err(errno_i32("ENOSYS"))
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
fn close_fd(fd: i32) -> HostIntResult {
    // SAFETY: close only consumes the integer file descriptor.
    let rc = unsafe { libc::close(fd) };
    if rc < 0 {
        HostIntResult::err(last_errno())
    } else {
        HostIntResult::ok(i64::from(rc))
    }
}

#[cfg(not(all(unix, not(target_arch = "wasm32"))))]
fn close_fd(fd: i32) -> HostIntResult {
    let _ = fd;
    HostIntResult::err(errno_i32("ENOSYS"))
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
fn fcntl_fd(fd: i32, cmd: i32, arg: i32) -> HostIntResult {
    // SAFETY: this mirrors the C runtime's three-int fcntl wrapper.
    let rc = unsafe { libc::fcntl(fd, cmd, arg) };
    if rc < 0 {
        HostIntResult::err(last_errno())
    } else {
        HostIntResult::ok(i64::from(rc))
    }
}

#[cfg(not(all(unix, not(target_arch = "wasm32"))))]
fn fcntl_fd(fd: i32, cmd: i32, arg: i32) -> HostIntResult {
    let _ = (fd, cmd, arg);
    HostIntResult::err(errno_i32("ENOSYS"))
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
fn socket_fd(domain: i32, typ: i32, protocol: i32) -> HostIntResult {
    // SAFETY: socket takes plain integer arguments.
    let fd = unsafe { libc::socket(domain, typ, protocol) };
    if fd < 0 {
        HostIntResult::err(last_errno())
    } else {
        HostIntResult::ok(i64::from(fd))
    }
}

#[cfg(not(all(unix, not(target_arch = "wasm32"))))]
fn socket_fd(domain: i32, typ: i32, protocol: i32) -> HostIntResult {
    let _ = (domain, typ, protocol);
    HostIntResult::err(errno_i32("ENOSYS"))
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
fn bind_socket(fd: i32, addr: &[u8]) -> HostIntResult {
    let len = match libc::socklen_t::try_from(addr.len()) {
        Ok(len) => len,
        Err(_) => return HostIntResult::err(errno_i32("EINVAL")),
    };
    // SAFETY: addr points to len bytes copied from guest memory.
    let rc = unsafe { libc::bind(fd, addr.as_ptr().cast(), len) };
    if rc < 0 {
        HostIntResult::err(last_errno())
    } else {
        HostIntResult::ok(i64::from(rc))
    }
}

#[cfg(not(all(unix, not(target_arch = "wasm32"))))]
fn bind_socket(fd: i32, addr: &[u8]) -> HostIntResult {
    let _ = (fd, addr);
    HostIntResult::err(errno_i32("ENOSYS"))
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
fn connect_socket(fd: i32, addr: &[u8]) -> HostIntResult {
    let len = match libc::socklen_t::try_from(addr.len()) {
        Ok(len) => len,
        Err(_) => return HostIntResult::err(errno_i32("EINVAL")),
    };
    // SAFETY: addr points to len bytes copied from guest memory.
    let rc = unsafe { libc::connect(fd, addr.as_ptr().cast(), len) };
    if rc < 0 {
        HostIntResult::err(last_errno())
    } else {
        HostIntResult::ok(i64::from(rc))
    }
}

#[cfg(not(all(unix, not(target_arch = "wasm32"))))]
fn connect_socket(fd: i32, addr: &[u8]) -> HostIntResult {
    let _ = (fd, addr);
    HostIntResult::err(errno_i32("ENOSYS"))
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
fn listen_socket(fd: i32, backlog: i32) -> HostIntResult {
    // SAFETY: listen takes plain integer arguments.
    let rc = unsafe { libc::listen(fd, backlog) };
    if rc < 0 {
        HostIntResult::err(last_errno())
    } else {
        HostIntResult::ok(i64::from(rc))
    }
}

#[cfg(not(all(unix, not(target_arch = "wasm32"))))]
fn listen_socket(fd: i32, backlog: i32) -> HostIntResult {
    let _ = (fd, backlog);
    HostIntResult::err(errno_i32("ENOSYS"))
}

#[cfg(all(unix, not(target_arch = "wasm32")))]
fn setsockopt_socket(fd: i32, level: i32, optname: i32, optval: &[u8]) -> HostIntResult {
    let len = match libc::socklen_t::try_from(optval.len()) {
        Ok(len) => len,
        Err(_) => return HostIntResult::err(errno_i32("EINVAL")),
    };
    // SAFETY: optval points to len bytes copied from guest memory.
    let rc = unsafe { libc::setsockopt(fd, level, optname, optval.as_ptr().cast(), len) };
    if rc < 0 {
        HostIntResult::err(last_errno())
    } else {
        HostIntResult::ok(i64::from(rc))
    }
}

#[cfg(not(all(unix, not(target_arch = "wasm32"))))]
fn setsockopt_socket(fd: i32, level: i32, optname: i32, optval: &[u8]) -> HostIntResult {
    let _ = (fd, level, optname, optval);
    HostIntResult::err(errno_i32("ENOSYS"))
}

fn parse_native_file_mode(mode: &[u8]) -> Option<NativeFileMode> {
    let mut normalized = Vec::with_capacity(mode.len());
    for byte in mode {
        if *byte != b'b' {
            normalized.push(*byte);
        }
    }
    let mode = match normalized.as_slice() {
        b"r" => NativeFileMode {
            readable: true,
            writable: false,
            append: false,
            truncate: false,
            create: false,
        },
        b"w" => NativeFileMode {
            readable: false,
            writable: true,
            append: false,
            truncate: true,
            create: true,
        },
        b"a" => NativeFileMode {
            readable: false,
            writable: true,
            append: true,
            truncate: false,
            create: true,
        },
        b"r+" => NativeFileMode {
            readable: true,
            writable: true,
            append: false,
            truncate: false,
            create: false,
        },
        b"w+" => NativeFileMode {
            readable: true,
            writable: true,
            append: false,
            truncate: true,
            create: true,
        },
        b"a+" => NativeFileMode {
            readable: true,
            writable: true,
            append: true,
            truncate: false,
            create: true,
        },
        _ => return None,
    };
    Some(mode)
}

#[cfg(any(not(target_arch = "wasm32"), target_os = "wasi"))]
fn open_native_file(path: &std::path::Path, mode: NativeFileMode) -> Result<std::fs::File, i32> {
    std::fs::OpenOptions::new()
        .read(mode.readable)
        .write(mode.writable && !mode.append)
        .append(mode.append)
        .truncate(mode.truncate)
        .create(mode.create)
        .open(path)
        .map_err(|err| io_error_errno(&err).unwrap_or_else(|| errno_i32("ENOENT")))
}

fn ffi_arity(name: &str) -> Option<usize> {
    if errno_constant(name).is_some() {
        return Some(0);
    }
    if host_constant(name).is_some() {
        return Some(0);
    }
    Some(match name {
        "GETRAW"
        | "GETTIMEMICRO"
        | "islinux"
        | "ismacos"
        | "iswindows"
        | "sizeof_char"
        | "sizeof_short"
        | "sizeof_int"
        | "sizeof_long"
        | "sizeof_llong"
        | "sizeof_size_t"
        | "want_gmp"
        | "want_imath"
        | "&closeb"
        | "&free"
        | "&errno"
        | "errno"
        | "environ"
        | "get_executable_path"
        | "new_mpz"
        | "openb_wr_mem" => 0,
        "malloc"
        | "free"
        | "strlen"
        | "getenv"
        | "unsetenv"
        | "putchar"
        | "remove"
        | "system"
        | "chdir"
        | "get_permissions"
        | "add_fd"
        | "opendir"
        | "readdir"
        | "closedir"
        | "c_d_name"
        | "close"
        | "js_debug"
        | "js_eval_run"
        | "js_eval_call"
        | "js_set_haskellCallback"
        | "add_FILE"
        | "add_utf8"
        | "add_crlf"
        | "add_rle_compressor"
        | "add_rle_decompressor"
        | "add_base64_encoder"
        | "add_base64_decoder"
        | "add_lz77_compressor"
        | "add_lz77_decompressor"
        | "add_bwt_compressor"
        | "add_bwt_decompressor"
        | "add_lzma_compressor"
        | "add_lzma_decompressor"
        | "closeb"
        | "flushb"
        | "getb"
        | "peekPtr"
        | "peekWord"
        | "peek_uint8"
        | "peek_uint16"
        | "peek_uint32"
        | "peek_uint64"
        | "peek_int8"
        | "peek_int16"
        | "peek_int32"
        | "peek_int64"
        | "peek_char"
        | "peek_schar"
        | "peek_uchar"
        | "peek_short"
        | "peek_ushort"
        | "peek_int"
        | "peek_uint"
        | "peek_long"
        | "peek_ulong"
        | "peek_llong"
        | "peek_ullong"
        | "peek_size_t"
        | "peek_flt32"
        | "peek_flt64"
        | "mpz_get_d"
        | "mpz_get_f"
        | "mpz_get_si"
        | "mpz_get_si64"
        | "mpz_log2"
        | "mpz_popcount"
        | "acos"
        | "asin"
        | "atan"
        | "cos"
        | "exp"
        | "log"
        | "sin"
        | "sqrt"
        | "tan"
        | "acosf"
        | "asinf"
        | "atanf"
        | "cosf"
        | "expf"
        | "logf"
        | "sinf"
        | "sqrtf"
        | "tanf" => 1,
        "calloc" | "realloc" | "strcpy" | "fopen" | "tmpname" | "add_buf" | "mkdir" | "getcwd"
        | "set_permissions" | "md5BFILE" | "md5String" | "pokePtr" | "pokeWord" | "poke_uint8"
        | "poke_uint16" | "poke_uint32" | "poke_uint64" | "poke_int8" | "poke_int16"
        | "poke_int32" | "poke_int64" | "poke_char" | "poke_schar" | "poke_uchar"
        | "poke_short" | "poke_ushort" | "poke_int" | "poke_uint" | "poke_long" | "poke_ulong"
        | "poke_llong" | "poke_ullong" | "poke_size_t" | "poke_flt32" | "poke_flt64"
        | "openb_rd_mem" | "getcpu" | "gettimeofday" | "listen" | "mpz_abs" | "mpz_cmp"
        | "mpz_init_set_si" | "mpz_init_set_si64" | "mpz_init_set_ui" | "mpz_init_set_ui64"
        | "mpz_neg" | "mpz_tstbit" | "putb" | "ungetb" | "atan2" | "pow" | "scalbn" | "atan2f"
        | "powf" | "scalbnf" => 2,
        "memcpy" | "memmove" | "setenv" | "md5Array" | "get_mem" | "readb" | "writeb" | "open"
        | "accept" | "bind" | "connect" | "fcntl" | "lz77c" | "mpz_add" | "mpz_and"
        | "mpz_fdiv_q_2exp" | "mpz_ior" | "mpz_mul" | "mpz_mul_2exp" | "mpz_sub" | "mpz_xor"
        | "socket" => 3,
        "recv" | "send" => 4,
        "mpz_tdiv_qr" => 4,
        "getsockopt" | "setsockopt" => 5,
        "strerror_r" => 3,
        _ => return None,
    })
}

fn is_unary_math_ffi_candidate(name: &str) -> bool {
    let bytes = name.as_bytes();
    match bytes.first() {
        Some(b'a') => {
            bytes.starts_with(b"ac") || bytes.starts_with(b"as") || bytes.starts_with(b"at")
        }
        Some(b'c') => bytes.starts_with(b"co"),
        Some(b'e') => bytes.starts_with(b"ex"),
        Some(b'l') => bytes.starts_with(b"lo"),
        Some(b's') => bytes.starts_with(b"si") || bytes.starts_with(b"sq"),
        Some(b't') => bytes.starts_with(b"ta"),
        _ => false,
    }
}

fn c_string_len(bytes: &[u8]) -> usize {
    bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len())
}

fn std_handle(name: &str) -> Option<StdHandle> {
    Some(match name {
        "IO.stdin" => StdHandle::Stdin,
        "IO.stdout" => StdHandle::Stdout,
        "IO.stderr" => StdHandle::Stderr,
        _ => return None,
    })
}

fn std_handle_ptr(name: &str) -> Option<i64> {
    Some(match std_handle(name)? {
        StdHandle::Stdin => -1,
        StdHandle::Stdout => -2,
        StdHandle::Stderr => -3,
    })
}

fn handle_from_ptr(ptr: i64) -> Option<StdHandle> {
    Some(match ptr {
        -1 => StdHandle::Stdin,
        -2 => StdHandle::Stdout,
        -3 => StdHandle::Stderr,
        _ => return None,
    })
}

fn handle_name_from_ptr(ptr: i64) -> Option<&'static str> {
    Some(match handle_from_ptr(ptr)? {
        StdHandle::Stdin => "IO.stdin",
        StdHandle::Stdout => "IO.stdout",
        StdHandle::Stderr => "IO.stderr",
    })
}

fn push_display<T: fmt::Display>(out: &mut Vec<u8>, value: T) {
    out.extend_from_slice(value.to_string().as_bytes());
}

fn serialize_ptr(ptr: i64, out: &mut Vec<u8>) {
    match ptr {
        -1 => out.extend_from_slice(b"fp2p IO.stdin @"),
        -2 => out.extend_from_slice(b"fp2p IO.stdout @"),
        -3 => out.extend_from_slice(b"fp2p IO.stderr @"),
        _ => {
            out.extend_from_slice(b"toPtr #");
            push_display(out, ptr);
            out.extend_from_slice(b" @");
        }
    }
}

fn serialize_bytes_comb(bytes: &[u8], out: &mut Vec<u8>) {
    if bytes.len() > 100 {
        out.push(b'$');
        push_display(out, bytes.len());
        out.push(b' ');
        out.extend_from_slice(bytes);
    } else {
        serialize_bytes_quoted(bytes, out);
    }
}

fn serialize_bigint_decimal(bytes: &[u8], out: &mut Vec<u8>) {
    out.push(b'%');
    out.extend_from_slice(bytes);
    out.push(b'"');
}

fn serialize_bytes_quoted(bytes: &[u8], out: &mut Vec<u8>) {
    out.push(b'"');
    for &byte in bytes {
        match byte {
            b'"' | b'\\' | b'^' | b'|' => {
                out.push(b'\\');
                out.push(byte);
            }
            0xff => out.extend_from_slice(b"\\_"),
            0x20..=0x7e => out.push(byte),
            0x00..=0x1f => {
                out.push(b'^');
                out.push(byte | 0x20);
            }
            0x7f => out.extend_from_slice(b"\\?"),
            0x80..=0x9f => {
                out.push(b'^');
                out.push(byte & 0x1f | 0x40);
            }
            0xa0..=0xfe => {
                out.push(b'|');
                out.push(byte & 0x7f);
            }
        }
    }
    out.push(b'"');
}

fn head_utf8(bytes: &[u8]) -> Result<(u32, usize), EvalError> {
    let c1 = *bytes.first().ok_or(EvalError::InvalidByteString)?;
    if c1 & 0x80 == 0 {
        return Ok((c1 as u32, 1));
    }

    let c2 = *bytes.get(1).ok_or(EvalError::InvalidByteString)?;
    if c1 & 0xe0 == 0xc0 {
        return Ok(((((c1 & 0x1f) as u32) << 6) | ((c2 & 0x3f) as u32), 2));
    }

    let c3 = *bytes.get(2).ok_or(EvalError::InvalidByteString)?;
    if c1 & 0xf0 == 0xe0 {
        return Ok((
            (((c1 & 0x0f) as u32) << 12) | (((c2 & 0x3f) as u32) << 6) | ((c3 & 0x3f) as u32),
            3,
        ));
    }

    let c4 = *bytes.get(3).ok_or(EvalError::InvalidByteString)?;
    if c1 & 0xf8 == 0xf0 {
        return Ok((
            (((c1 & 0x07) as u32) << 18)
                | (((c2 & 0x3f) as u32) << 12)
                | (((c3 & 0x3f) as u32) << 6)
                | ((c4 & 0x3f) as u32),
            4,
        ));
    }

    Err(EvalError::InvalidByteString)
}

fn head_utf8_string(bytes: &[u8]) -> Result<Option<(u32, usize)>, EvalError> {
    let Some(&c1) = bytes.first() else {
        return Ok(None);
    };
    if c1 & 0x80 == 0 {
        return Ok(Some((c1 as u32, 1)));
    }

    let Some(&c2) = bytes.get(1) else {
        return Ok(None);
    };
    if c1 & 0xe0 == 0xc0 {
        let c = (((c1 & 0x1f) as u32) << 6) | ((c2 & 0x3f) as u32);
        if 0 < c && c < 0x80 {
            return Err(EvalError::InvalidByteString);
        }
        return Ok(Some((c, 2)));
    }

    let Some(&c3) = bytes.get(2) else {
        return Ok(None);
    };
    if c1 & 0xf0 == 0xe0 {
        let c = (((c1 & 0x0f) as u32) << 12) | (((c2 & 0x3f) as u32) << 6) | ((c3 & 0x3f) as u32);
        if c < 0x800 {
            return Err(EvalError::InvalidByteString);
        }
        return Ok(Some((c, 3)));
    }

    let Some(&c4) = bytes.get(3) else {
        return Ok(None);
    };
    if c1 & 0xf8 == 0xf0 {
        let c = (((c1 & 0x07) as u32) << 18)
            | (((c2 & 0x3f) as u32) << 12)
            | (((c3 & 0x3f) as u32) << 6)
            | ((c4 & 0x3f) as u32);
        if c < 0x10000 {
            return Err(EvalError::InvalidByteString);
        }
        return Ok(Some((c, 4)));
    }

    Err(EvalError::InvalidByteString)
}

fn decode_utf8_string_bytes(mut bytes: &[u8]) -> Result<Vec<u32>, EvalError> {
    let mut values = Vec::new();
    while let Some((value, offset)) = head_utf8_string(bytes)? {
        values.push(value);
        bytes = &bytes[offset..];
    }
    Ok(values)
}

fn modified_utf8(n: i64) -> Result<Vec<u8>, EvalError> {
    let mut c = u32::try_from(n).map_err(|_| EvalError::InvalidByteString)?;
    if c & 0x1ff800 == 0xd800 {
        c = 0xfffd;
    }
    if c > 0 && c < 0x80 {
        Ok(vec![c as u8])
    } else if c < 0x800 {
        Ok(vec![0xc0 | (c >> 6) as u8, 0x80 | (c & 0x3f) as u8])
    } else if c < 0x10000 {
        Ok(vec![
            0xe0 | (c >> 12) as u8,
            0x80 | ((c >> 6) & 0x3f) as u8,
            0x80 | (c & 0x3f) as u8,
        ])
    } else if c < 0x110000 {
        Ok(vec![
            0xf0 | (c >> 18) as u8,
            0x80 | ((c >> 12) & 0x3f) as u8,
            0x80 | ((c >> 6) & 0x3f) as u8,
            0x80 | (c & 0x3f) as u8,
        ])
    } else {
        Err(EvalError::InvalidByteString)
    }
}

fn render_bytes(bytes: &[u8], out: &mut String) {
    out.push('"');
    for &byte in bytes {
        match byte {
            b'\\' => out.push_str("\\\\"),
            b'"' => out.push_str("\\\""),
            b'\n' => out.push_str("\\n"),
            b'\r' => out.push_str("\\r"),
            b'\t' => out.push_str("\\t"),
            0x20..=0x7e => out.push(byte as char),
            _ => {
                out.push_str("\\x");
                out.push(nibble(byte >> 4));
                out.push(nibble(byte & 0x0f));
            }
        }
    }
    out.push('"');
}

fn nibble(n: u8) -> char {
    match n {
        0..=9 => (b'0' + n) as char,
        10..=15 => (b'a' + n - 10) as char,
        _ => unreachable!(),
    }
}
