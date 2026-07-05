//! Compression and encoding helpers used by BFILE primitives.
use super::*;

pub(in crate::runtime) const MD5_S: [u32; 64] = [
    7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 5, 9, 14, 20, 5, 9, 14, 20, 5, 9,
    14, 20, 5, 9, 14, 20, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 6, 10, 15,
    21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21,
];

pub(in crate::runtime) const MD5_K: [u32; 64] = [
    0xd76aa478, 0xe8c7b756, 0x242070db, 0xc1bdceee, 0xf57c0faf, 0x4787c62a, 0xa8304613, 0xfd469501,
    0x698098d8, 0x8b44f7af, 0xffff5bb1, 0x895cd7be, 0x6b901122, 0xfd987193, 0xa679438e, 0x49b40821,
    0xf61e2562, 0xc040b340, 0x265e5a51, 0xe9b6c7aa, 0xd62f105d, 0x02441453, 0xd8a1e681, 0xe7d3fbc8,
    0x21e1cde6, 0xc33707d6, 0xf4d50d87, 0x455a14ed, 0xa9e3e905, 0xfcefa3f8, 0x676f02d9, 0x8d2a4c8a,
    0xfffa3942, 0x8771f681, 0x6d9d6122, 0xfde5380c, 0xa4beea44, 0x4bdecfa9, 0xf6bb4b60, 0xbebfbc70,
    0x289b7ec6, 0xeaa127fa, 0xd4ef3085, 0x04881d05, 0xd9d4d039, 0xe6db99e5, 0x1fa27cf8, 0xc4ac5665,
    0xf4292244, 0x432aff97, 0xab9423a7, 0xfc93a039, 0x655b59c3, 0x8f0ccc92, 0xffeff47d, 0x85845dd1,
    0x6fa87e4f, 0xfe2ce6e0, 0xa3014314, 0x4e0811a1, 0xf7537e82, 0xbd3af235, 0x2ad7d2bb, 0xeb86d391,
];

pub(in crate::runtime) struct Md5Context {
    pub(in crate::runtime) size: u64,
    pub(in crate::runtime) state: [u32; 4],
    pub(in crate::runtime) input: [u8; 64],
}

impl Md5Context {
    pub(in crate::runtime) fn new() -> Self {
        Self {
            size: 0,
            state: [0x67452301, 0xefcdab89, 0x98badcfe, 0x10325476],
            input: [0; 64],
        }
    }

    pub(in crate::runtime) fn update(&mut self, bytes: &[u8]) {
        let mut offset = (self.size % 64) as usize;
        self.size = self.size.wrapping_add(bytes.len() as u64);
        for &byte in bytes {
            self.input[offset] = byte;
            offset += 1;
            if offset == 64 {
                md5_step(&mut self.state, &md5_block_words(&self.input));
                offset = 0;
            }
        }
    }

    pub(in crate::runtime) fn finalize(mut self) -> [u8; 16] {
        let offset = (self.size % 64) as usize;
        let padding_len = if offset < 56 {
            56 - offset
        } else {
            120 - offset
        };
        let mut padding = [0; 64];
        padding[0] = 0x80;
        self.update(&padding[..padding_len]);
        self.size = self.size.wrapping_sub(padding_len as u64);

        let mut block = md5_block_words(&self.input);
        let bit_len = self.size.wrapping_mul(8);
        block[14] = bit_len as u32;
        block[15] = (bit_len >> 32) as u32;
        md5_step(&mut self.state, &block);

        let mut digest = [0; 16];
        for (idx, word) in self.state.iter().enumerate() {
            digest[idx * 4..idx * 4 + 4].copy_from_slice(&word.to_le_bytes());
        }
        digest
    }
}

pub(in crate::runtime) fn md5_bytes(bytes: &[u8]) -> [u8; 16] {
    let mut ctx = Md5Context::new();
    ctx.update(bytes);
    ctx.finalize()
}

pub(in crate::runtime) fn md5_block_words(input: &[u8; 64]) -> [u32; 16] {
    let mut out = [0; 16];
    for (idx, word) in out.iter_mut().enumerate() {
        let start = idx * 4;
        *word = u32::from_le_bytes([
            input[start],
            input[start + 1],
            input[start + 2],
            input[start + 3],
        ]);
    }
    out
}

pub(in crate::runtime) fn md5_step(state: &mut [u32; 4], input: &[u32; 16]) {
    let mut a = state[0];
    let mut b = state[1];
    let mut c = state[2];
    let mut d = state[3];

    for idx in 0..64 {
        let (e, word_idx) = match idx / 16 {
            0 => ((b & c) | (!b & d), idx),
            1 => ((b & d) | (c & !d), (idx * 5 + 1) % 16),
            2 => (b ^ c ^ d, (idx * 3 + 5) % 16),
            _ => (c ^ (b | !d), (idx * 7) % 16),
        };
        let old_d = d;
        d = c;
        c = b;
        b = b.wrapping_add(
            a.wrapping_add(e)
                .wrapping_add(MD5_K[idx])
                .wrapping_add(input[word_idx])
                .rotate_left(MD5_S[idx]),
        );
        a = old_d;
    }

    state[0] = state[0].wrapping_add(a);
    state[1] = state[1].wrapping_add(b);
    state[2] = state[2].wrapping_add(c);
    state[3] = state[3].wrapping_add(d);
}

pub(in crate::runtime) fn rle_pending_bytes(count: usize, byte: i64) -> Result<Vec<u8>, EvalError> {
    if count == 0 {
        return Ok(Vec::new());
    }
    if !(0..128).contains(&byte) {
        return Err(EvalError::InvalidHandle);
    }
    let byte = byte as u8;
    if count > 2 {
        let mut out = Vec::new();
        push_rle_rep(count - 1, &mut out)?;
        out.push(byte);
        Ok(out)
    } else {
        Ok(vec![byte; count])
    }
}

pub(in crate::runtime) fn push_rle_rep(n: usize, out: &mut Vec<u8>) -> Result<(), EvalError> {
    if n > 127 {
        push_rle_rep(n / 128, out)?;
    }
    let digit = u8::try_from(n % 128).map_err(|_| EvalError::Overflow)?;
    out.push(digit | 0x80);
    Ok(())
}

pub(in crate::runtime) enum Base64Input {
    Value(i32),
    Whitespace,
    Invalid,
}

pub(in crate::runtime) const BASE64_ALPHABET: &[u8; 65] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/=";
pub(in crate::runtime) const BASE64_PAD: usize = 64;

pub(in crate::runtime) fn base64_decode_value(byte: u8) -> Base64Input {
    match byte {
        b'A'..=b'Z' => Base64Input::Value(i32::from(byte - b'A')),
        b'a'..=b'z' => Base64Input::Value(i32::from(byte - b'a') + 26),
        b'0'..=b'9' => Base64Input::Value(i32::from(byte - b'0') + 52),
        b'+' => Base64Input::Value(62),
        b'/' => Base64Input::Value(63),
        b'=' => Base64Input::Value(-3),
        b' ' | b'\t' | b'\n' | b'\r' => Base64Input::Whitespace,
        _ => Base64Input::Invalid,
    }
}

pub(in crate::runtime) fn base64_full_quad_bytes(
    encbuf: &[u8; 3],
    linelen: usize,
    outcol: &mut usize,
) -> Vec<u8> {
    let x = ((u32::from(encbuf[0])) << 16) | ((u32::from(encbuf[1])) << 8) | u32::from(encbuf[2]);
    base64_quad_bytes(
        [
            ((x >> 18) & 0x3f) as usize,
            ((x >> 12) & 0x3f) as usize,
            ((x >> 6) & 0x3f) as usize,
            (x & 0x3f) as usize,
        ],
        linelen,
        outcol,
    )
}

pub(in crate::runtime) fn base64_pending_bytes(
    encbuf: &[u8; 3],
    encpos: &usize,
    linelen: usize,
    outcol: &mut usize,
) -> Result<Vec<u8>, EvalError> {
    Ok(match *encpos {
        0 => Vec::new(),
        1 => {
            let x = (u32::from(encbuf[0])) << 16;
            base64_quad_bytes(
                [
                    ((x >> 18) & 0x3f) as usize,
                    ((x >> 12) & 0x3f) as usize,
                    BASE64_PAD,
                    BASE64_PAD,
                ],
                linelen,
                outcol,
            )
        }
        2 => {
            let x = ((u32::from(encbuf[0])) << 16) | ((u32::from(encbuf[1])) << 8);
            base64_quad_bytes(
                [
                    ((x >> 18) & 0x3f) as usize,
                    ((x >> 12) & 0x3f) as usize,
                    ((x >> 6) & 0x3f) as usize,
                    BASE64_PAD,
                ],
                linelen,
                outcol,
            )
        }
        _ => return Err(EvalError::InvalidHandle),
    })
}

pub(in crate::runtime) fn base64_quad_bytes(
    indices: [usize; 4],
    linelen: usize,
    outcol: &mut usize,
) -> Vec<u8> {
    let mut out = Vec::with_capacity(5);
    if linelen != 0 && *outcol + 4 > linelen {
        out.push(b'\n');
        *outcol = 0;
    }
    for index in indices {
        out.push(BASE64_ALPHABET[index]);
    }
    *outcol += 4;
    out
}

pub(in crate::runtime) const LZ77_MAXWIN: usize = 8_192;
pub(in crate::runtime) const LZ77_MAXLEN: usize = 9 + 255;
pub(in crate::runtime) const LZ77_MINMATCH: usize = 3;
pub(in crate::runtime) const LZ77_MINOFFS: usize = 1;
pub(in crate::runtime) const LZ77_MAXLIT: usize = 32;
pub(in crate::runtime) const LZ77_HASHBITS: usize = 11;
pub(in crate::runtime) const LZ77_HASHSIZE: usize = 1 << LZ77_HASHBITS;
pub(in crate::runtime) const LZ77_NUMPREV: usize = 16;

pub(in crate::runtime) fn lz77_decompress(src: &[u8]) -> Result<Vec<u8>, EvalError> {
    let mut out = Vec::with_capacity(100_000);
    let mut src_pos = 0;
    while src_pos < src.len() {
        let op = src[src_pos];
        src_pos += 1;
        let opx = usize::from(op & 0x1f);
        let op = op >> 5;
        if op == 0 {
            let len = opx.checked_add(1).ok_or(EvalError::Overflow)?;
            let end = src_pos.checked_add(len).ok_or(EvalError::Overflow)?;
            let bytes = src.get(src_pos..end).ok_or(EvalError::InvalidByteString)?;
            out.extend_from_slice(bytes);
            src_pos = end;
        } else {
            let lo = *src.get(src_pos).ok_or(EvalError::InvalidByteString)?;
            src_pos += 1;
            let offs = opx
                .checked_mul(256)
                .and_then(|offs| offs.checked_add(usize::from(lo)))
                .and_then(|offs| offs.checked_add(LZ77_MINOFFS))
                .ok_or(EvalError::Overflow)?;
            let len = if op == 7 {
                let extra = *src.get(src_pos).ok_or(EvalError::InvalidByteString)?;
                src_pos += 1;
                9usize
                    .checked_add(usize::from(extra))
                    .ok_or(EvalError::Overflow)?
            } else {
                2usize
                    .checked_add(usize::from(op))
                    .ok_or(EvalError::Overflow)?
            };
            if offs > out.len() {
                return Err(EvalError::InvalidByteString);
            }
            let start = out.len() - offs;
            for idx in 0..len {
                let byte = *out.get(start + idx).ok_or(EvalError::InvalidByteString)?;
                out.push(byte);
            }
        }
    }
    Ok(out)
}

pub(in crate::runtime) fn lz77_compress(src: &[u8]) -> Result<Vec<u8>, EvalError> {
    let mut out = Vec::with_capacity(25_000);
    let mut hashes = [[usize::MAX; LZ77_NUMPREV]; LZ77_HASHSIZE];
    let mut cur = 0;
    while cur < src.len() {
        let mut match_offs = 0;
        let mut match_len = 0;
        let mut len = 0;
        while len < src.len() - cur {
            (match_len, match_offs) = lz77_find_longest_match(src, cur + len, &hashes);
            if match_len >= LZ77_MINMATCH {
                break;
            }
            lz77_update_hash(src, cur + len, &mut hashes);
            len += 1;
        }
        while len != 0 {
            let n = len.min(LZ77_MAXLIT);
            out.push(u8::try_from(n - 1).map_err(|_| EvalError::Overflow)?);
            out.extend_from_slice(&src[cur..cur + n]);
            cur += n;
            len -= n;
        }
        if match_len >= LZ77_MINMATCH {
            for _ in 0..match_len {
                lz77_update_hash(src, cur, &mut hashes);
                cur += 1;
            }
            let match_offs = match_offs
                .checked_sub(LZ77_MINOFFS)
                .ok_or(EvalError::Overflow)?;
            let match_len = match_len.checked_sub(2).ok_or(EvalError::Overflow)?;
            let hi = match_offs >> 8;
            let lo = match_offs & 0xff;
            if match_len < 7 {
                out.push(u8::try_from((match_len << 5) + hi).map_err(|_| EvalError::Overflow)?);
                out.push(u8::try_from(lo).map_err(|_| EvalError::Overflow)?);
            } else {
                out.push(u8::try_from((7 << 5) + hi).map_err(|_| EvalError::Overflow)?);
                out.push(u8::try_from(lo).map_err(|_| EvalError::Overflow)?);
                out.push(u8::try_from(match_len - 7).map_err(|_| EvalError::Overflow)?);
            }
        }
    }
    Ok(out)
}

pub(in crate::runtime) fn lz77_hash(bytes: &[u8]) -> usize {
    let mut hash = 5381usize;
    for byte in bytes.iter().take(4) {
        hash = hash.wrapping_mul(33).wrapping_add(usize::from(*byte));
    }
    hash & (LZ77_HASHSIZE - 1)
}

pub(in crate::runtime) fn lz77_find_longest_match(
    src: &[u8],
    cur: usize,
    hashes: &[[usize; LZ77_NUMPREV]; LZ77_HASHSIZE],
) -> (usize, usize) {
    let win_end = cur + 1;
    let win_len = win_end.min(LZ77_MAXWIN);
    let offsets = &hashes[lz77_hash(&src[cur..])];
    let mut match_len = 0;
    let mut match_offs = 0;
    for offset in offsets {
        if *offset == usize::MAX {
            break;
        }
        if *offset > cur {
            break;
        }
        let offs = cur - *offset;
        if !(LZ77_MINOFFS..win_len).contains(&offs) {
            break;
        }
        let len = lz77_match_len(src, cur, cur - offs);
        if len > match_len {
            match_len = len;
            match_offs = offs;
        }
    }
    (match_len, match_offs)
}

pub(in crate::runtime) fn lz77_match_len(src: &[u8], cur: usize, win: usize) -> usize {
    let mut len = 0;
    while cur + len < src.len() && len < LZ77_MAXLEN && src[cur + len] == src[win + len] {
        len += 1;
    }
    len
}

pub(in crate::runtime) fn lz77_update_hash(
    src: &[u8],
    cur: usize,
    hashes: &mut [[usize; LZ77_NUMPREV]; LZ77_HASHSIZE],
) {
    let slots = &mut hashes[lz77_hash(&src[cur..])];
    for idx in (1..LZ77_NUMPREV).rev() {
        slots[idx] = slots[idx - 1];
    }
    slots[0] = cur;
}

pub(in crate::runtime) fn u32_le_bytes(n: usize) -> Result<[u8; 4], EvalError> {
    let n = u32::try_from(n).map_err(|_| EvalError::Overflow)?;
    Ok(n.to_le_bytes())
}

pub(in crate::runtime) fn u64_le_bytes(n: usize) -> Result<[u8; 8], EvalError> {
    let n = u64::try_from(n).map_err(|_| EvalError::Overflow)?;
    Ok(n.to_le_bytes())
}

pub(in crate::runtime) fn lzma_compress_payload(input: &[u8]) -> Result<Vec<u8>, EvalError> {
    let props = lzma_sdk_rs::LzmaProps::for_level(5, u32::MAX);
    let raw = lzma_sdk_rs::encode(input, &props);
    let mut out = Vec::with_capacity(13 + raw.len());
    out.extend_from_slice(&lzma_sdk_rs::decoder_props(&props));
    out.extend_from_slice(&u64_le_bytes(input.len())?);
    out.extend_from_slice(&raw);
    Ok(out)
}

pub(in crate::runtime) fn lzma_decompress_payload(input: &[u8]) -> Result<Vec<u8>, EvalError> {
    if input.len() < 13 {
        return Err(EvalError::InvalidByteString);
    }
    let props: [u8; 5] = input[..5]
        .try_into()
        .map_err(|_| EvalError::InvalidByteString)?;
    let out_len = u64::from_le_bytes(
        input[5..13]
            .try_into()
            .map_err(|_| EvalError::InvalidByteString)?,
    );
    let out_len = usize::try_from(out_len).map_err(|_| EvalError::Overflow)?;
    crate::lzma_decode::decode_raw_checked(&input[13..], &props, out_len)
        .ok_or(EvalError::InvalidByteString)
}

pub(in crate::runtime) fn bwt_encode(data: &[u8]) -> Result<(usize, Vec<u8>), EvalError> {
    if data.is_empty() {
        return Ok((0, Vec::new()));
    }
    let mut rotations: Vec<usize> = (0..data.len()).collect();
    rotations.sort_by(|a, b| bwt_compare_rotation(data, *a, *b));
    let mut zero = 0;
    let mut last = Vec::with_capacity(data.len());
    for (idx, offset) in rotations.into_iter().enumerate() {
        last.push(data[(offset + data.len() - 1) % data.len()]);
        if offset == 0 {
            zero = idx;
        }
    }
    Ok((zero, last))
}

pub(in crate::runtime) fn bwt_compare_rotation(data: &[u8], a: usize, b: usize) -> Ordering {
    if a == b {
        return Ordering::Equal;
    }
    for offset in 0..data.len() {
        let left = data[(a + offset) % data.len()];
        let right = data[(b + offset) % data.len()];
        match left.cmp(&right) {
            Ordering::Equal => {}
            ordering => return ordering,
        }
    }
    Ordering::Equal
}

pub(in crate::runtime) fn bwt_decode(data: &[u8], zero: usize) -> Result<Vec<u8>, EvalError> {
    if data.is_empty() {
        if zero == 0 {
            return Ok(Vec::new());
        }
        return Err(EvalError::InvalidByteString);
    }
    if zero >= data.len() {
        return Err(EvalError::InvalidByteString);
    }
    let mut count = [0usize; 256];
    let mut pred = Vec::with_capacity(data.len());
    for byte in data {
        let index = usize::from(*byte);
        pred.push(count[index]);
        count[index] = count[index].checked_add(1).ok_or(EvalError::Overflow)?;
    }
    let mut sum = 0usize;
    for item in &mut count {
        let previous = *item;
        *item = sum;
        sum = sum.checked_add(previous).ok_or(EvalError::Overflow)?;
    }
    let mut out = vec![0; data.len()];
    let mut index = zero;
    for pos in (0..data.len()).rev() {
        let byte = data[index];
        out[pos] = byte;
        index = pred[index]
            .checked_add(count[usize::from(byte)])
            .ok_or(EvalError::Overflow)?;
        if pos != 0 && index >= data.len() {
            return Err(EvalError::InvalidByteString);
        }
    }
    Ok(out)
}
