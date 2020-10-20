use crate::{DeserializeErr, DeserializeResult};
use alloc::string::String;

pub fn take(amount: usize, data: &[u8]) -> DeserializeResult<&[u8]> {
    if data.len() < amount {
        Err(DeserializeErr::Eof)
    } else {
        Ok(data.split_at(amount).into())
    }
}

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
