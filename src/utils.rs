use crate::{DeserializeErr, DeserializeResult, Deserialized};

#[inline]
pub fn read_one_byte(data: &[u8]) -> DeserializeResult<u8> {
    match data.split_first() {
        Some((byte, rest)) => Deserialized::ok(*byte, rest),
        None => Err(DeserializeErr::Eof),
    }
}

#[inline]
pub const fn take(amount: usize) -> impl for<'b> Fn(&'b [u8]) -> DeserializeResult<'b, &'b [u8]> {
    move |data| {
        if data.len() < amount {
            Err(DeserializeErr::Eof)
        } else {
            Ok(data.split_at(amount).into())
        }
    }
}

#[inline]
pub fn read_long(data: &[u8]) -> DeserializeResult<u64> {
    Ok(take(8)(data)?.map(move |bytes| {
        (bytes[0] as u64) << 56
            | (bytes[1] as u64) << 48
            | (bytes[2] as u64) << 40
            | (bytes[3] as u64) << 32
            | (bytes[4] as u64) << 24
            | (bytes[5] as u64) << 16
            | (bytes[6] as u64) << 8
            | (bytes[7] as u64)
    }))
}

#[inline]
pub fn write_long(v: u64) -> [u8; 8] {
    [
        (v >> 56) as u8,
        (v >> 48) as u8,
        (v >> 40) as u8,
        (v >> 32) as u8,
        (v >> 24) as u8,
        (v >> 16) as u8,
        (v >> 8) as u8,
        v as u8,
    ]
}

#[inline]
pub fn read_int(data: &[u8]) -> DeserializeResult<u32> {
    Ok(take(4)(data)?.map(move |bytes| {
        (bytes[0] as u32) << 24
            | (bytes[1] as u32) << 16
            | (bytes[2] as u32) << 8
            | (bytes[3] as u32)
    }))
}

#[inline]
pub fn write_int(v: u32) -> [u8; 4] {
    [(v >> 24) as u8, (v >> 16) as u8, (v >> 8) as u8, v as u8]
}

#[inline]
pub fn read_short(data: &[u8]) -> DeserializeResult<u16> {
    Ok(take(2)(data)?.map(move |bytes| (bytes[0] as u16) << 8 | (bytes[1] as u16)))
}

#[inline]
pub fn write_short(v: u16) -> [u8; 2] {
    [(v >> 8) as u8, v as u8]
}

#[inline]
pub fn hex(data: &[u8]) -> String {
    let mut str = String::with_capacity(data.len() * 2);
    for byte_ref in data {
        let byte = *byte_ref;
        str.push(hex_char_for(byte >> 4));
        str.push(hex_char_for(byte & 0xF));
    }
    str
}

const ZERO_ASCII_CODE: u8 = 48;
const LOWER_A_ASCII_CODE: u8 = 97;

#[inline]
fn hex_char_for(half: u8) -> char {
    if half > 0xF {
        panic!("not defined for > 0xF (operates on half a byte)");
    }

    if half < 10 {
        (half + ZERO_ASCII_CODE) as char
    } else {
        (half + (LOWER_A_ASCII_CODE - 10)) as char
    }
}

#[inline]
pub fn parse_hex_char(data: u8) -> Option<u8> {
    const UPPER_A_ASCII_CODE: u8 = 65;
    const LOWER_F_ASCII_CODE: u8 = 102;
    const UPPER_F_ASCII_CODE: u8 = 70;
    const NINE_ASCII_CODE: u8 = 57;

    if data >= LOWER_A_ASCII_CODE {
        if data > LOWER_F_ASCII_CODE {
            None
        } else {
            Some(10 + (data - LOWER_A_ASCII_CODE))
        }
    } else if data >= UPPER_A_ASCII_CODE {
        if data > UPPER_F_ASCII_CODE {
            None
        } else {
            Some(10 + (data - UPPER_A_ASCII_CODE))
        }
    } else if data >= ZERO_ASCII_CODE {
        if data > NINE_ASCII_CODE {
            None
        } else {
            Some(data - ZERO_ASCII_CODE)
        }
    } else {
        None
    }
}
