//! Minimal bigint operations used by MicroHs integer FFI primitives.
use super::*;

pub(in crate::runtime) const MPZ_BASE: u32 = 1_000_000_000;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::runtime) struct MpzValue {
    pub(in crate::runtime) negative: bool,
    pub(in crate::runtime) digits: Vec<u32>,
}

impl MpzValue {
    pub(in crate::runtime) fn zero() -> Self {
        Self {
            negative: false,
            digits: Vec::new(),
        }
    }

    pub(in crate::runtime) fn one() -> Self {
        Self {
            negative: false,
            digits: vec![1],
        }
    }

    pub(in crate::runtime) fn from_u64(mut value: u64) -> Self {
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

    pub(in crate::runtime) fn from_i64(value: i64) -> Self {
        let mut out = Self::from_u64(value.unsigned_abs());
        out.negative = value < 0 && !out.is_zero();
        out
    }

    pub(in crate::runtime) fn parse_decimal(bytes: &[u8]) -> Result<Self, ()> {
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

    pub(in crate::runtime) fn to_decimal_bytes(&self) -> Vec<u8> {
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

    pub(in crate::runtime) fn normalize(&mut self) {
        while self.digits.last() == Some(&0) {
            self.digits.pop();
        }
        if self.digits.is_empty() {
            self.negative = false;
        }
    }

    pub(in crate::runtime) fn normalized(mut self) -> Self {
        self.normalize();
        self
    }

    pub(in crate::runtime) fn is_zero(&self) -> bool {
        self.digits.is_empty()
    }

    pub(in crate::runtime) fn abs(&self) -> Self {
        let mut out = self.clone();
        out.negative = false;
        out
    }

    pub(in crate::runtime) fn cmp_abs(&self, other: &Self) -> Ordering {
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

    pub(in crate::runtime) fn cmp(&self, other: &Self) -> Ordering {
        match (self.negative, other.negative) {
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            (false, false) => self.cmp_abs(other),
            (true, true) => other.cmp_abs(self),
        }
    }

    pub(in crate::runtime) fn abs_add(&self, other: &Self) -> Self {
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

    pub(in crate::runtime) fn abs_sub(&self, other: &Self) -> Self {
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

    pub(in crate::runtime) fn add(&self, other: &Self) -> Self {
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

    pub(in crate::runtime) fn sub(&self, other: &Self) -> Self {
        let mut neg_other = other.clone();
        if !neg_other.is_zero() {
            neg_other.negative = !neg_other.negative;
        }
        self.add(&neg_other)
    }

    pub(in crate::runtime) fn mul(&self, other: &Self) -> Self {
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

    pub(in crate::runtime) fn mul_small_mut(&mut self, value: u32) {
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

    pub(in crate::runtime) fn add_small_mut(&mut self, value: u32) {
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

    pub(in crate::runtime) fn div2_mut(&mut self) -> bool {
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

    pub(in crate::runtime) fn shl1_mut(&mut self) {
        self.mul_small_mut(2);
    }

    pub(in crate::runtime) fn shl_bits(mut self, bits: usize) -> Self {
        for _ in 0..bits {
            self.shl1_mut();
        }
        self
    }

    pub(in crate::runtime) fn shr_abs_bits(&self, bits: usize) -> (Self, bool) {
        let mut out = self.abs();
        let mut dropped = false;
        for _ in 0..bits {
            dropped |= out.div2_mut();
        }
        (out, dropped)
    }

    pub(in crate::runtime) fn fdiv_q_2exp(&self, bits: usize) -> Self {
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

    pub(in crate::runtime) fn to_bits_abs(&self) -> Vec<bool> {
        let mut tmp = self.abs();
        let mut bits = Vec::new();
        while !tmp.is_zero() {
            bits.push(tmp.div2_mut());
        }
        bits
    }

    pub(in crate::runtime) fn from_bits_abs(bits: &[bool]) -> Self {
        let mut out = Self::zero();
        for bit in bits.iter().rev() {
            out.shl1_mut();
            if *bit {
                out.add_small_mut(1);
            }
        }
        out
    }

    pub(in crate::runtime) fn one_shl(bits: usize) -> Self {
        Self::one().shl_bits(bits)
    }

    pub(in crate::runtime) fn div_rem_abs(
        &self,
        divisor: &Self,
    ) -> Result<(Self, Self), EvalError> {
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

    pub(in crate::runtime) fn tdiv_qr(&self, divisor: &Self) -> Result<(Self, Self), EvalError> {
        let (mut quot, mut rem) = self.abs().div_rem_abs(&divisor.abs())?;
        quot.negative = self.negative != divisor.negative && !quot.is_zero();
        rem.negative = self.negative && !rem.is_zero();
        Ok((quot, rem))
    }

    pub(in crate::runtime) fn bit_len(&self) -> usize {
        if self.digits.is_empty() {
            return 0;
        }
        const EXACT_LIMBS: usize = 4;
        let base = u128::from(MPZ_BASE);
        if self.digits.len() <= EXACT_LIMBS {
            let mut value = 0_u128;
            for digit in self.digits.iter().rev() {
                value = value * base + u128::from(*digit);
            }
            return value.bit_width() as usize;
        }

        let lower_limbs = self.digits.len() - EXACT_LIMBS;
        let mut prefix = 0_u128;
        for digit in self.digits[lower_limbs..].iter().rev() {
            prefix = prefix * base + u128::from(*digit);
        }
        let scale = lower_limbs as f64 * f64::from(MPZ_BASE).log2();
        let lower_log = (prefix as f64).log2() + scale;
        let upper_log = ((prefix + 1) as f64).log2() + scale;
        let lower_floor = lower_log.floor();
        let upper_floor = (upper_log - 1.0e-12).floor();
        if lower_floor == upper_floor {
            return lower_floor as usize + 1;
        }

        self.to_bits_abs().len()
    }

    pub(in crate::runtime) fn to_twos_bits(&self, width: usize) -> Vec<bool> {
        let mut bits = if self.negative {
            Self::one_shl(width).sub(&self.abs()).to_bits_abs()
        } else {
            self.to_bits_abs()
        };
        bits.resize(width, false);
        bits
    }

    pub(in crate::runtime) fn from_twos_bits(bits: &[bool]) -> Self {
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

    pub(in crate::runtime) fn bitwise(&self, other: &Self, op: fn(bool, bool) -> bool) -> Self {
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

    pub(in crate::runtime) fn bitand(&self, other: &Self) -> Self {
        self.bitwise(other, |left, right| left & right)
    }

    pub(in crate::runtime) fn bitor(&self, other: &Self) -> Self {
        self.bitwise(other, |left, right| left | right)
    }

    pub(in crate::runtime) fn bitxor(&self, other: &Self) -> Self {
        self.bitwise(other, |left, right| left ^ right)
    }

    pub(in crate::runtime) fn test_bit_abs(&self, bit: usize) -> bool {
        self.to_bits_abs().get(bit).copied().unwrap_or(false)
    }

    pub(in crate::runtime) fn test_bit_signed(&self, bit: usize) -> bool {
        if !self.negative {
            return self.test_bit_abs(bit);
        }
        let shifted = self.fdiv_q_2exp(bit);
        shifted.abs().test_bit_abs(0)
    }

    pub(in crate::runtime) fn signed_popcount(&self) -> Result<i64, EvalError> {
        let count = i64::try_from(self.to_bits_abs().into_iter().filter(|bit| *bit).count())
            .map_err(|_| EvalError::Overflow)?;
        Ok(if self.negative { -count } else { count })
    }

    pub(in crate::runtime) fn log2(&self) -> Result<i64, EvalError> {
        i64::try_from(self.bit_len().saturating_sub(1)).map_err(|_| EvalError::Overflow)
    }

    pub(in crate::runtime) fn to_u64_low(&self) -> u64 {
        let mut out = 0_u64;
        let mut place = 1_u64;
        for digit in &self.digits {
            out = out.wrapping_add(u64::from(*digit).wrapping_mul(place));
            place = place.wrapping_mul(u64::from(MPZ_BASE));
        }
        out
    }

    pub(in crate::runtime) fn to_i64_wrapping(&self) -> i64 {
        let low = self.to_u64_low();
        if self.negative {
            0_u64.wrapping_sub(low) as i64
        } else {
            low as i64
        }
    }

    /// Convert to `f64` exactly like GMP `mpz_get_d`: truncate toward zero (keep the top
    /// 53 significant magnitude bits and discard the rest — never round to nearest),
    /// saturating to +/-infinity only when the magnitude is too large for a finite
    /// double (matching `mpz_get_d`).
    #[cold]
    #[inline(never)]
    pub(in crate::runtime) fn to_f64(&self) -> f64 {
        if self.is_zero() {
            return 0.0;
        }
        // `to_bits_abs` is little-endian (bit 0 = LSB) and its top bit is set, so its
        // length is the bit length of |n| with the leading 1 at index `bit_len - 1`.
        let bits = self.to_bits_abs();
        let bit_len = bits.len() as u64;
        let exponent = (bit_len - 1) + 1023;
        if exponent >= 2047 {
            // Magnitude >= 2^1024: no finite double, so GMP yields +/-infinity.
            return if self.negative {
                f64::NEG_INFINITY
            } else {
                f64::INFINITY
            };
        }
        // The 52 fraction bits are the ones immediately below the leading 1, truncated:
        // any magnitude bits below index `bit_len - 53` are simply discarded.
        let mut fraction = 0_u64;
        for i in 0..52_i64 {
            let pos = bit_len as i64 - 2 - i;
            if pos >= 0 && bits[pos as usize] {
                fraction |= 1_u64 << (51 - i);
            }
        }
        let sign = if self.negative { 1_u64 << 63 } else { 0 };
        f64::from_bits(sign | (exponent << 52) | fraction)
    }
}
