//! Rendering and byte/string serialization helpers.
use super::*;

pub(in crate::runtime) fn c_string_len(bytes: &[u8]) -> usize {
    bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len())
}

pub(in crate::runtime) fn std_handle(name: &str) -> Option<StdHandle> {
    Some(match name {
        "IO.stdin" => StdHandle::Stdin,
        "IO.stdout" => StdHandle::Stdout,
        "IO.stderr" => StdHandle::Stderr,
        _ => return None,
    })
}

pub(in crate::runtime) fn std_handle_ptr(name: &str) -> Option<i64> {
    Some(match std_handle(name)? {
        StdHandle::Stdin => -1,
        StdHandle::Stdout => -2,
        StdHandle::Stderr => -3,
    })
}

pub(in crate::runtime) fn handle_from_ptr(ptr: i64) -> Option<StdHandle> {
    Some(match ptr {
        -1 => StdHandle::Stdin,
        -2 => StdHandle::Stdout,
        -3 => StdHandle::Stderr,
        _ => return None,
    })
}

pub(in crate::runtime) fn handle_name_from_ptr(ptr: i64) -> Option<&'static str> {
    Some(match handle_from_ptr(ptr)? {
        StdHandle::Stdin => "IO.stdin",
        StdHandle::Stdout => "IO.stdout",
        StdHandle::Stderr => "IO.stderr",
    })
}

pub(in crate::runtime) fn push_display<T: fmt::Display>(out: &mut Vec<u8>, value: T) {
    out.extend_from_slice(value.to_string().as_bytes());
}

pub(in crate::runtime) fn serialize_ptr(ptr: i64, out: &mut Vec<u8>) {
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

pub(in crate::runtime) fn serialize_bytes_comb(bytes: &[u8], out: &mut Vec<u8>) {
    if bytes.len() > 100 {
        out.push(b'$');
        push_display(out, bytes.len());
        out.push(b' ');
        out.extend_from_slice(bytes);
    } else {
        serialize_bytes_quoted(bytes, out);
    }
}

pub(in crate::runtime) fn serialize_bigint_decimal(bytes: &[u8], out: &mut Vec<u8>) {
    out.push(b'%');
    out.extend_from_slice(bytes);
    out.push(b'"');
}

pub(in crate::runtime) fn serialize_bytes_quoted(bytes: &[u8], out: &mut Vec<u8>) {
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

pub(in crate::runtime) fn head_utf8(bytes: &[u8]) -> Result<(u32, usize), EvalError> {
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

pub(in crate::runtime) fn head_utf8_string(
    bytes: &[u8],
) -> Result<Option<(u32, usize)>, EvalError> {
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

pub(in crate::runtime) fn decode_utf8_string_bytes(
    mut bytes: &[u8],
) -> Result<Vec<u32>, EvalError> {
    let mut values = Vec::new();
    while let Some((value, offset)) = head_utf8_string(bytes)? {
        values.push(value);
        bytes = &bytes[offset..];
    }
    Ok(values)
}

pub(in crate::runtime) fn modified_utf8(n: i64) -> Result<Vec<u8>, EvalError> {
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

pub(in crate::runtime) fn render_bytes(bytes: &[u8], out: &mut String) {
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

pub(in crate::runtime) fn nibble(n: u8) -> char {
    match n {
        0..=9 => (b'0' + n) as char,
        10..=15 => (b'a' + n - 10) as char,
        _ => unreachable!(),
    }
}
