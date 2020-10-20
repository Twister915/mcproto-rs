use crate::{DeserializeResult, Deserialized, DeserializeErr};
use crate::utils::take;
use core::convert::TryInto;

pub type ProtoByteOrder = BigEndian;

pub trait ByteOrder {
    fn write_u2long(v: u128) -> [u8; 16];

    fn write_2long(v: i128) -> [u8; 16] {
        Self::write_u2long(v as u128)
    }

    fn read_u2long(data: &[u8]) -> DeserializeResult<u128>;

    fn read_2long(data: &[u8]) -> DeserializeResult<i128> {
        Ok(Self::read_u2long(data)?.map(move |data| data as i128))
    }

    fn write_ulong(v: u64) -> [u8; 8];

    fn write_long(v: i64) -> [u8; 8] {
        Self::write_ulong(v as u64)
    }

    fn read_ulong(data: &[u8]) -> DeserializeResult<u64>;

    fn read_long(data: &[u8]) -> DeserializeResult<i64> {
        Ok(Self::read_ulong(data)?.map(move |data| data as i64))
    }

    fn write_uint(v: u32) -> [u8; 4];

    fn write_int(v: i32) -> [u8; 4] {
        Self::write_uint(v as u32)
    }

    fn read_uint(data: &[u8]) -> DeserializeResult<u32>;

    fn read_int(data: &[u8]) -> DeserializeResult<i32> {
        Ok(Self::read_uint(data)?.map(move |data| data as i32))
    }

    fn write_ushort(v: u16) -> [u8; 2];

    fn write_short(v: i16) -> [u8; 2] {
        Self::write_ushort(v as u16)
    }

    fn read_ushort(data: &[u8]) -> DeserializeResult<u16>;

    fn read_short(data: &[u8]) -> DeserializeResult<i16> {
        Ok(Self::read_ushort(data)?.map(move |data| data as i16))
    }

    fn read_ubyte(data: &[u8]) -> DeserializeResult<u8> {
        match data.split_first() {
            Some((byte, rest)) => Deserialized::ok(*byte, rest),
            None => Err(DeserializeErr::Eof)
        }
    }

    fn read_byte(data: &[u8]) -> DeserializeResult<i8> {
        Ok(Self::read_ubyte(data)?.map(move |b| b as i8))
    }

    fn write_float(v: f32) -> [u8; 4];

    fn read_float(data: &[u8]) -> DeserializeResult<f32>;

    fn write_double(v: f64) -> [u8; 8];

    fn read_double(data: &[u8]) -> DeserializeResult<f64>;
}

pub struct BigEndian;

impl ByteOrder for BigEndian {
    fn write_u2long(v: u128) -> [u8; 16] {
        [
            ((v >> 120) as u8),
            ((v >> 112) as u8),
            ((v >> 104) as u8),
            ((v >> 96) as u8),
            ((v >> 88) as u8),
            ((v >> 80) as u8),
            ((v >> 72) as u8),
            ((v >> 64) as u8),
            ((v >> 56) as u8),
            ((v >> 48) as u8),
            ((v >> 40) as u8),
            ((v >> 32) as u8),
            ((v >> 24) as u8),
            ((v >> 16) as u8),
            ((v >> 8) as u8),
            (v as u8),
        ]
    }

    fn read_u2long(data: &[u8]) -> DeserializeResult<'_, u128> {
        Ok(take(16, data)?.map(move |bytes| {
            ((bytes[0] as u128) << 120) |
                ((bytes[1] as u128) << 112) |
                ((bytes[2] as u128) << 104) |
                ((bytes[3] as u128) << 96) |
                ((bytes[4] as u128) << 88) |
                ((bytes[5] as u128) << 80) |
                ((bytes[6] as u128) << 72) |
                ((bytes[7] as u128) << 64) |
                ((bytes[8] as u128) << 56) |
                ((bytes[9] as u128) << 48) |
                ((bytes[10] as u128) << 40) |
                ((bytes[11] as u128) << 32) |
                ((bytes[12] as u128) << 24) |
                ((bytes[13] as u128) << 16) |
                ((bytes[14] as u128) << 8) |
                (bytes[15] as u128)
        }))
    }

    fn write_ulong(v: u64) -> [u8; 8] {
        [
            ((v >> 56) as u8),
            ((v >> 48) as u8),
            ((v >> 40) as u8),
            ((v >> 32) as u8),
            ((v >> 24) as u8),
            ((v >> 16) as u8),
            ((v >> 8) as u8),
            (v as u8),
        ]
    }

    fn read_ulong(data: &[u8]) -> DeserializeResult<'_, u64> {
        Ok(take(8, data)?.map(move |bytes| {
            ((bytes[0] as u64) << 56) |
                ((bytes[1] as u64) << 48) |
                ((bytes[2] as u64) << 40) |
                ((bytes[3] as u64) << 32) |
                ((bytes[4] as u64) << 24) |
                ((bytes[5] as u64) << 16) |
                ((bytes[6] as u64) << 8) |
                (bytes[7] as u64)
        }))
    }

    fn write_uint(v: u32) -> [u8; 4] {
        [
            ((v >> 24) as u8),
            ((v >> 16) as u8),
            ((v >> 8) as u8),
            (v as u8),
        ]
    }

    fn read_uint(data: &[u8]) -> DeserializeResult<'_, u32> {
        Ok(take(4, data)?.map(move |bytes| {
            ((bytes[0] as u32) << 24) |
                ((bytes[1] as u32) << 16) |
                ((bytes[2] as u32) << 8) |
                (bytes[3] as u32)
        }))
    }

    fn write_ushort(v: u16) -> [u8; 2] {
        [
            ((v >> 8) as u8),
            (v as u8),
        ]
    }

    fn read_ushort(data: &[u8]) -> DeserializeResult<'_, u16> {
        Ok(take(2, data)?
            .map(move |bytes| ((bytes[0] as u16) << 8) | (bytes[1] as u16)))
    }

    fn write_float(v: f32) -> [u8; 4] {
        v.to_be_bytes()
    }

    fn read_float(data: &[u8]) -> DeserializeResult<'_, f32> {
        Ok(take(4, data)?.map(move |bytes| {
            f32::from_be_bytes(bytes.try_into().expect("is exactly 4 long"))
        }))

    }

    fn write_double(v: f64) -> [u8; 8] {
        v.to_be_bytes()
    }

    fn read_double(data: &[u8]) -> DeserializeResult<'_, f64> {
        Ok(take(8, data)?.map(move |bytes| {
            f64::from_be_bytes(bytes.try_into().expect("is exactly 8 long"))
        }))
    }
}

pub struct LittleEndian;

impl ByteOrder for LittleEndian {
    fn write_u2long(v: u128) -> [u8; 16] {
        [
            v as u8,
            (v >> 8) as u8,
            (v >> 16) as u8,
            (v >> 24) as u8,
            (v >> 32) as u8,
            (v >> 40) as u8,
            (v >> 48) as u8,
            (v >> 56) as u8,
            (v >> 64) as u8,
            (v >> 72) as u8,
            (v >> 80) as u8,
            (v >> 88) as u8,
            (v >> 96) as u8,
            (v >> 104) as u8,
            (v >> 112) as u8,
            (v >> 120) as u8,
        ]
    }

    fn read_u2long(data: &[u8]) -> DeserializeResult<'_, u128> {
        Ok(take(16, data)?.map(move |bytes| {
            (bytes[0] as u128) |
                ((bytes[1] as u128) << 8) |
                ((bytes[2] as u128) << 16) |
                ((bytes[3] as u128) << 24) |
                ((bytes[4] as u128) << 32) |
                ((bytes[5] as u128) << 40) |
                ((bytes[6] as u128) << 48) |
                ((bytes[7] as u128) << 56) |
                ((bytes[8] as u128) << 64) |
                ((bytes[9] as u128) << 72) |
                ((bytes[10] as u128) << 80) |
                ((bytes[11] as u128) << 88) |
                ((bytes[12] as u128) << 96) |
                ((bytes[13] as u128) << 104) |
                ((bytes[14] as u128) << 112) |
                ((bytes[15] as u128) << 120)
        }))
    }

    fn write_ulong(v: u64) -> [u8; 8] {
        [
            v as u8,
            (v >> 8) as u8,
            (v >> 16) as u8,
            (v >> 24) as u8,
            (v >> 32) as u8,
            (v >> 40) as u8,
            (v >> 48) as u8,
            (v >> 56) as u8
        ]
    }

    fn read_ulong(data: &[u8]) -> DeserializeResult<'_, u64> {
        Ok(take(8, data)?.map(move |bytes| {
            (bytes[0] as u64) |
                ((bytes[1] as u64) << 8) |
                ((bytes[2] as u64) << 16) |
                ((bytes[3] as u64) << 24) |
                ((bytes[4] as u64) << 32) |
                ((bytes[5] as u64) << 40) |
                ((bytes[6] as u64) << 48) |
                ((bytes[7] as u64) << 56)
        }))
    }

    fn write_uint(v: u32) -> [u8; 4] {
        [
            v as u8,
            (v >> 8) as u8,
            (v >> 16) as u8,
            (v >> 24) as u8,
        ]
    }

    fn read_uint(data: &[u8]) -> DeserializeResult<'_, u32> {
        Ok(take(4, data)?.map(move |bytes| {
            (bytes[0] as u32) |
                ((bytes[1] as u32) << 8) |
                ((bytes[2] as u32) << 16) |
                ((bytes[3] as u32) << 24)
        }))
    }

    fn write_ushort(v: u16) -> [u8; 2] {
        [
            v as u8,
            (v >> 8) as u8,
        ]
    }

    fn read_ushort(data: &[u8]) -> DeserializeResult<'_, u16> {
        Ok(take(4, data)?
            .map(move |bytes| (bytes[0] as u16) | ((bytes[1] as u16) << 8)))
    }

    fn write_float(v: f32) -> [u8; 4] {
        v.to_le_bytes()
    }

    fn read_float(data: &[u8]) -> DeserializeResult<'_, f32> {
        Ok(take(4, data)?.map(move |bytes| {
            f32::from_le_bytes(bytes.try_into().expect("is exactly 4 long"))
        }))
    }

    fn write_double(v: f64) -> [u8; 8] {
        v.to_le_bytes()
    }

    fn read_double(data: &[u8]) -> DeserializeResult<'_, f64> {
        Ok(take(8, data)?.map(move |bytes| {
            f64::from_le_bytes(bytes.try_into().expect("is exactly 8 long"))
        }))
    }
}