// ... PRIMITIVE TYPES ...

use alloc::{string::String, vec::Vec, fmt};
use crate::utils::*;
use crate::uuid::UUID4;
use crate::*;

pub use super::chat::*;

#[cfg(all(test, feature = "std"))]
use crate::protocol::TestRandom;

// bool
impl Serialize for bool {
    fn mc_serialize<S: Serializer>(&self, to: &mut S) -> SerializeResult {
        to.serialize_byte(if *self { 1 } else { 0 })
    }
}

impl Deserialize for bool {
    fn mc_deserialize(data: &[u8]) -> DeserializeResult<'_, Self> {
        read_one_byte(data)?.try_map(move |b| match b {
            0x00 => Ok(false),
            0x01 => Ok(true),
            other => Err(DeserializeErr::InvalidBool(other)),
        })
    }
}

#[cfg(all(test, feature = "std"))]
impl TestRandom for bool {
    fn test_gen_random() -> Self {
        rand::random()
    }
}

// u8
impl Serialize for u8 {
    fn mc_serialize<S: Serializer>(&self, to: &mut S) -> SerializeResult {
        to.serialize_byte(*self)
    }
}

impl Deserialize for u8 {
    fn mc_deserialize(data: &[u8]) -> DeserializeResult<'_, Self> {
        read_one_byte(data)
    }
}

#[cfg(all(test, feature = "std"))]
impl TestRandom for u8 {
    fn test_gen_random() -> Self {
        rand::random()
    }
}

// i8
impl Serialize for i8 {
    fn mc_serialize<S: Serializer>(&self, to: &mut S) -> SerializeResult {
        to.serialize_byte(*self as u8)
    }
}

impl Deserialize for i8 {
    fn mc_deserialize(data: &[u8]) -> DeserializeResult<'_, Self> {
        Ok(read_one_byte(data)?.map(move |byte| byte as i8))
    }
}

#[cfg(all(test, feature = "std"))]
impl TestRandom for i8 {
    fn test_gen_random() -> Self {
        rand::random()
    }
}

// u16
impl Serialize for u16 {
    fn mc_serialize<S: Serializer>(&self, to: &mut S) -> SerializeResult {
        let data = write_short(*self);
        to.serialize_bytes(&data[..])
    }
}

impl Deserialize for u16 {
    fn mc_deserialize(data: &[u8]) -> DeserializeResult<'_, Self> {
        read_short(data)
    }
}

#[cfg(all(test, feature = "std"))]
impl TestRandom for u16 {
    fn test_gen_random() -> Self {
        rand::random()
    }
}

// i16
impl Serialize for i16 {
    fn mc_serialize<S: Serializer>(&self, to: &mut S) -> SerializeResult {
        (*self as u16).mc_serialize(to)
    }
}

impl Deserialize for i16 {
    fn mc_deserialize(data: &[u8]) -> DeserializeResult<'_, Self> {
        u16::mc_deserialize(data)?
            .map(move |other| other as i16)
            .into()
    }
}

#[cfg(all(test, feature = "std"))]
impl TestRandom for i16 {
    fn test_gen_random() -> Self {
        rand::random()
    }
}

// int
impl Serialize for i32 {
    fn mc_serialize<S: Serializer>(&self, to: &mut S) -> SerializeResult {
        let data = write_int(*self as u32);
        to.serialize_bytes(&data[..])
    }
}

impl Deserialize for i32 {
    fn mc_deserialize(data: &[u8]) -> DeserializeResult<'_, Self> {
        Ok(read_int(data)?.map(move |v| v as i32))
    }
}

#[cfg(all(test, feature = "std"))]
impl TestRandom for i32 {
    fn test_gen_random() -> Self {
        rand::random()
    }
}

// long
impl Serialize for i64 {
    fn mc_serialize<S: Serializer>(&self, to: &mut S) -> SerializeResult {
        let data = write_long(*self as u64);
        to.serialize_bytes(&data[..])
    }
}

impl Deserialize for i64 {
    fn mc_deserialize(data: &[u8]) -> DeserializeResult<'_, Self> {
        Ok(read_long(data)?.map(move |v| v as i64))
    }
}

#[cfg(all(test, feature = "std"))]
impl TestRandom for i64 {
    fn test_gen_random() -> Self {
        rand::random()
    }
}

// float
impl Serialize for f32 {
    //noinspection ALL
    fn mc_serialize<S: Serializer>(&self, to: &mut S) -> SerializeResult {
        let data = (*self).to_be_bytes();
        to.serialize_bytes(&data[..])
    }
}

impl Deserialize for f32 {
    fn mc_deserialize(data: &[u8]) -> DeserializeResult<'_, Self> {
        i32::mc_deserialize(data)?
            .map(move |r| f32::from_bits(r as u32))
            .into()
    }
}

#[cfg(all(test, feature = "std"))]
impl TestRandom for f32 {
    fn test_gen_random() -> Self {
        rand::random()
    }
}

// double
impl Serialize for f64 {
    //noinspection ALL
    fn mc_serialize<S: Serializer>(&self, to: &mut S) -> SerializeResult {
        let data = (*self).to_be_bytes();
        to.serialize_bytes(&data[..])
    }
}

impl Deserialize for f64 {
    fn mc_deserialize(data: &[u8]) -> DeserializeResult<'_, Self> {
        i64::mc_deserialize(data)?
            .map(move |r| f64::from_bits(r as u64))
            .into()
    }
}

#[cfg(all(test, feature = "std"))]
impl TestRandom for f64 {
    fn test_gen_random() -> Self {
        rand::random()
    }
}

// VAR INT AND VAR LONG
const VAR_INT_BYTES: usize = 5;
const VAR_LONG_BYTES: usize = 10;

#[derive(Copy, Clone, PartialOrd, PartialEq, Debug, Default, Hash, Ord, Eq)]
pub struct VarInt(pub i32);

impl Serialize for VarInt {
    fn mc_serialize<S: Serializer>(&self, to: &mut S) -> SerializeResult {
        let mut data = [0u8; VAR_INT_BYTES];
        to.serialize_bytes(serialize_var_num((self.0 as u32) as u64, &mut data))
    }
}

impl Deserialize for VarInt {
    fn mc_deserialize(orig_data: &[u8]) -> DeserializeResult<Self> {
        Ok(deserialize_var_num(orig_data, VAR_INT_BYTES)?.map(move |v| VarInt(v as i32)))
    }
}

impl Into<i32> for VarInt {
    fn into(self) -> i32 {
        self.0
    }
}

impl From<i32> for VarInt {
    fn from(v: i32) -> Self {
        Self(v)
    }
}

impl Into<usize> for VarInt {
    fn into(self) -> usize {
        self.0 as usize
    }
}

impl From<usize> for VarInt {
    fn from(v: usize) -> Self {
        Self(v as i32)
    }
}

impl fmt::Display for VarInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "VarInt({})", self.0)
    }
}

#[cfg(all(test, feature = "std"))]
impl TestRandom for VarInt {
    fn test_gen_random() -> Self {
        let out: i32 = rand::random();
        Self(out)
    }
}

#[derive(Copy, Clone, PartialOrd, PartialEq, Debug, Default, Hash, Ord, Eq)]
pub struct VarLong(pub i64);

impl Serialize for VarLong {
    fn mc_serialize<S: Serializer>(&self, to: &mut S) -> SerializeResult {
        let mut data = [0u8; VAR_LONG_BYTES];
        to.serialize_bytes(serialize_var_num(self.0 as u64, &mut data))
    }
}

impl Deserialize for VarLong {
    fn mc_deserialize(orig_data: &[u8]) -> DeserializeResult<'_, Self> {
        Ok(deserialize_var_num(orig_data, VAR_LONG_BYTES)?.map(move |v| VarLong(v as i64)))
    }
}

#[cfg(all(test, feature = "std"))]
impl TestRandom for VarLong {
    fn test_gen_random() -> Self {
        let out: i64 = rand::random();
        Self(out)
    }
}

fn serialize_var_num(data: u64, out: &mut [u8]) -> &[u8] {
    let mut v: u64 = data;
    let mut byte_idx = 0;
    let mut has_more = true;
    while has_more {
        if byte_idx == out.len() {
            panic!("tried to write too much data for Var num");
        }

        let mut v_byte = (v & 0x7F) as u8;
        v >>= 7;
        has_more = v != 0;
        if has_more {
            v_byte |= 0x80;
        }

        out[byte_idx] = v_byte;
        byte_idx += 1;
    }

    &out[..byte_idx]
}

fn deserialize_var_num(orig_data: &[u8], max_bytes: usize) -> DeserializeResult<u64> {
    let mut data = orig_data;
    let mut v: u64 = 0;
    let mut bit_place: usize = 0;
    let mut i: usize = 0;
    let mut has_more = true;

    while has_more {
        if i == max_bytes {
            return DeserializeErr::VarNumTooLong(Vec::from(&orig_data[..i])).into();
        }
        let Deserialized {
            value: byte,
            data: rest,
        } = read_one_byte(data)?;
        data = rest;
        has_more = byte & 0x80 != 0;
        v |= ((byte as u64) & 0x7F) << bit_place;
        bit_place += 7;
        i += 1;
    }

    Deserialized::ok(v, data)
}

// STRING
impl Serialize for String {
    fn mc_serialize<S: Serializer>(&self, to: &mut S) -> SerializeResult {
        to.serialize_other(&VarInt(self.len() as i32))?;
        to.serialize_bytes(self.as_bytes())
    }
}

impl Deserialize for String {
    fn mc_deserialize(data: &[u8]) -> DeserializeResult<'_, Self> {
        VarInt::mc_deserialize(data)?.and_then(move |length, rest| {
            if length.0 < 0 {
                Err(DeserializeErr::NegativeLength(length))
            } else {
                take(length.0 as usize)(rest)?.try_map(move |taken| {
                    String::from_utf8(taken.to_vec()).map_err(DeserializeErr::BadStringEncoding)
                })
            }
        })
    }
}

#[cfg(all(test, feature = "std"))]
impl TestRandom for String {
    fn test_gen_random() -> Self {
        let raw_len: u8 = rand::random();
        let len = raw_len as usize;
        let mut out = String::with_capacity(len);
        for _ in 0..len {
            let c_idx: u8 = rand::random::<u8>() % 36;

            let c = if c_idx <= 10 {
                (48 + c_idx) as char
            } else {
                ((c_idx - 10) + 65) as char
            };

            out.push(c)
        }

        out
    }
}

// position
#[derive(Clone, Copy, PartialEq, Hash, Debug)]
pub struct IntPosition {
    pub x: i32,
    pub y: i16,
    pub z: i32,
}

impl Serialize for IntPosition {
    fn mc_serialize<S: Serializer>(&self, to: &mut S) -> SerializeResult {
        let x_raw = if self.x < 0 {
            (self.x + 0x2000000) as u64 | 0x2000000
        } else {
            self.x as u64
        } & 0x3FFFFFF;
        let z_raw = if self.z < 0 {
            (self.z + 0x2000000) as u64 | 0x2000000
        } else {
            self.z as u64
        } & 0x3FFFFFF;
        let y_raw = if self.y < 0 {
            (self.y + 0x800) as u64 | 0x800
        } else {
            self.y as u64
        } & 0xFFF;

        let data_raw = ((x_raw << 38) | (z_raw << 12) | y_raw) as u64;
        let data_i64 = data_raw as i64;
        to.serialize_other(&data_i64)
    }
}

impl Deserialize for IntPosition {
    fn mc_deserialize(data: &[u8]) -> DeserializeResult<'_, Self> {
        let Deserialized { value: raw, data } = i64::mc_deserialize(data)?;
        let raw_unsigned = raw as u64;
        let mut x = ((raw_unsigned >> 38) as u32) & 0x3FFFFFF;
        let mut z = ((raw_unsigned >> 12) & 0x3FFFFFF) as u32;
        let mut y = ((raw_unsigned & 0xFFF) as u16) & 0xFFF;

        if (x & 0x2000000) != 0 {
            // is the 26th bit set
            // if so, treat the rest as a positive integer, and treat 26th bit as -2^25
            // 2^25 == 0x2000000
            // 0x1FFFFFF == 2^26 - 1 (all places set to 1 except 26th place)
            x = (((x & 0x1FFFFFF) as i32) - 0x2000000) as u32;
        }
        if (y & 0x800) != 0 {
            y = (((y & 0x7FF) as i16) - 0x800) as u16;
        }
        if (z & 0x2000000) != 0 {
            z = (((z & 0x1FFFFFF) as i32) - 0x2000000) as u32;
        }

        Deserialized::ok(
            IntPosition {
                x: x as i32,
                y: y as i16,
                z: z as i32,
            },
            data,
        )
    }
}

#[cfg(all(test, feature = "std"))]
impl TestRandom for IntPosition {
    fn test_gen_random() -> Self {
        let x: i32 = ((rand::random::<u32>() % (1 << 26)) as i32) - (1 << 25);
        let z: i32 = ((rand::random::<u32>() % (1 << 26)) as i32) - (1 << 25);
        let y: i16 = ((rand::random::<u16>() % (1 << 12)) as i16) - (1 << 11);
        Self { x, y, z }
    }
}

// angle
#[derive(Copy, Clone, PartialEq, Hash, Debug)]
pub struct Angle {
    pub value: u8,
}

impl Serialize for Angle {
    fn mc_serialize<S: Serializer>(&self, to: &mut S) -> SerializeResult {
        to.serialize_byte(self.value)
    }
}

impl Deserialize for Angle {
    fn mc_deserialize(data: &[u8]) -> DeserializeResult<'_, Self> {
        Ok(read_one_byte(data)?.map(move |b| Angle { value: b }))
    }
}

#[cfg(all(test, feature = "std"))]
impl TestRandom for Angle {
    fn test_gen_random() -> Self {
        Self {
            value: rand::random(),
        }
    }
}

// UUID

impl Serialize for UUID4 {
    fn mc_serialize<S: Serializer>(&self, to: &mut S) -> SerializeResult {
        let bytes = self.to_u128().to_be_bytes();
        to.serialize_bytes(&bytes[..])
    }
}

impl Deserialize for UUID4 {
    fn mc_deserialize(data: &[u8]) -> DeserializeResult<'_, Self> {
        take(16)(data)?
            .map(move |bytes| {
                let raw = (bytes[0] as u128) << 120
                    | (bytes[1] as u128) << 112
                    | (bytes[2] as u128) << 104
                    | (bytes[3] as u128) << 96
                    | (bytes[4] as u128) << 88
                    | (bytes[5] as u128) << 80
                    | (bytes[6] as u128) << 72
                    | (bytes[7] as u128) << 64
                    | (bytes[8] as u128) << 56
                    | (bytes[9] as u128) << 48
                    | (bytes[10] as u128) << 40
                    | (bytes[11] as u128) << 32
                    | (bytes[12] as u128) << 24
                    | (bytes[13] as u128) << 16
                    | (bytes[14] as u128) << 8
                    | bytes[15] as u128;
                UUID4::from(raw)
            })
            .into()
    }
}

#[cfg(all(test, feature = "std"))]
impl TestRandom for UUID4 {
    fn test_gen_random() -> Self {
        UUID4::random()
    }
}

// NBT

#[derive(Clone, PartialEq, Debug)]
pub struct NamedNbtTag {
    pub root: nbt::NamedTag,
}

impl Serialize for NamedNbtTag {
    fn mc_serialize<S: Serializer>(&self, to: &mut S) -> SerializeResult {
        let bytes = self.root.bytes();
        to.serialize_bytes(bytes.as_slice())
    }
}

impl Deserialize for NamedNbtTag {
    fn mc_deserialize(data: &[u8]) -> DeserializeResult<'_, Self> {
        Ok(
            nbt::NamedTag::root_compound_tag_from_bytes(data)?
                .map(move |root| NamedNbtTag { root }),
        )
    }
}

impl From<nbt::NamedTag> for NamedNbtTag {
    fn from(root: nbt::NamedTag) -> Self {
        Self { root }
    }
}

impl Into<nbt::NamedTag> for NamedNbtTag {
    fn into(self) -> nbt::NamedTag {
        self.root
    }
}

#[cfg(all(test, feature = "std"))]
impl TestRandom for NamedNbtTag {
    fn test_gen_random() -> Self {
        Self {
            root: nbt::NamedTag::test_gen_random(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FixedInt {
    raw: i32,
}

impl Serialize for FixedInt {
    fn mc_serialize<S: Serializer>(&self, to: &mut S) -> SerializeResult {
        to.serialize_other(&self.raw)
    }
}

impl Deserialize for FixedInt {
    fn mc_deserialize(data: &[u8]) -> DeserializeResult<'_, Self> {
        Ok(i32::mc_deserialize(data)?.map(move |raw| FixedInt { raw }))
    }
}

impl FixedInt {
    pub fn new(data: f64, fractional_bytes: usize) -> Self {
        Self {
            raw: (data * ((1 << fractional_bytes) as f64)) as i32,
        }
    }

    pub fn into_float(self, fractional_bytes: usize) -> f64 {
        (self.raw as f64) / ((1 << fractional_bytes) as f64)
    }
}

#[cfg(all(test, feature = "std"))]
impl TestRandom for FixedInt {
    fn test_gen_random() -> Self {
        FixedInt::new(f64::test_gen_random(), 16)
    }
}

#[derive(Default)]
pub struct BytesSerializer {
    data: Vec<u8>,
}

impl Serializer for BytesSerializer {
    fn serialize_bytes(&mut self, data: &[u8]) -> SerializeResult {
        self.data.extend_from_slice(data);
        Ok(())
    }
}

impl BytesSerializer {
    pub fn with_capacity(cap: usize) -> Self {
        BytesSerializer {
            data: Vec::with_capacity(cap),
        }
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.data
    }
}

impl<T> Serialize for Option<T>
where
    T: Serialize,
{
    fn mc_serialize<S: Serializer>(&self, to: &mut S) -> SerializeResult {
        match self {
            Some(value) => {
                to.serialize_other(&true)?;
                to.serialize_other(value)
            }
            None => to.serialize_other(&false),
        }
    }
}

impl<T> Deserialize for Option<T>
where
    T: Deserialize,
{
    fn mc_deserialize(data: &[u8]) -> DeserializeResult<'_, Self> {
        bool::mc_deserialize(data)?.and_then(move |is_present, data| {
            if is_present {
                Ok(T::mc_deserialize(data)?.map(move |component| Some(component)))
            } else {
                Deserialized::ok(None, data)
            }
        })
    }
}

#[cfg(all(test, feature = "std"))]
impl<T> TestRandom for Option<T>
where
    T: TestRandom,
{
    fn test_gen_random() -> Self {
        let is_present: bool = rand::random();
        if is_present {
            Some(T::test_gen_random())
        } else {
            None
        }
    }
}

// SLOT
#[derive(Debug, PartialEq, Clone)]
pub struct Slot {
    pub item_id: VarInt,
    pub item_count: i8,
    pub nbt: Option<nbt::NamedTag>,
}

impl Serialize for Slot {
    fn mc_serialize<S: Serializer>(&self, to: &mut S) -> SerializeResult {
        to.serialize_other(&self.item_id)?;
        to.serialize_other(&self.item_count)?;
        match self.nbt.as_ref() {
            Some(nbt) => to.serialize_bytes(nbt.bytes().as_slice()),
            None => to.serialize_byte(nbt::Tag::End.id()),
        }
    }
}

impl Deserialize for Slot {
    fn mc_deserialize(data: &[u8]) -> DeserializeResult<'_, Self> {
        let Deserialized {
            value: item_id,
            data,
        } = VarInt::mc_deserialize(data)?;
        let Deserialized {
            value: item_count,
            data,
        } = i8::mc_deserialize(data)?;
        if data.is_empty() {
            return Err(DeserializeErr::Eof);
        }

        let id = data[0];
        let rest = &data[1..];
        Ok(match id {
            0x00 => Deserialized {
                value: None,
                data: rest,
            },
            _ => nbt::read_named_tag(data)?.map(move |tag| Some(tag)),
        }
        .map(move |nbt| Slot {
            item_id,
            item_count,
            nbt,
        }))
    }
}

#[cfg(all(test, feature = "std"))]
impl TestRandom for Slot {
    fn test_gen_random() -> Self {
        let item_id = VarInt::test_gen_random();
        let item_count = i8::test_gen_random() % 65;
        let nbt = <Option<nbt::NamedTag>>::test_gen_random();

        Self {
            item_id,
            item_count,
            nbt,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::fmt::Debug;
    use alloc::borrow::ToOwned;

    #[test]
    fn test_bool() {
        test_type(true);
        test_type(false);
    }

    #[test]
    fn test_signed_byte() {
        test_type(0i8);
        test_type(127i8);
        test_type(-15i8);
    }

    #[test]
    fn test_unsigned_byte() {
        test_type(0u8);
        test_type(128u8);
        test_type(255u8);
    }

    #[test]
    fn test_signed_short() {
        test_type(0i16);
        test_type(-88i16);
        test_type(25521i16);
    }

    #[test]
    fn test_unsigned_short() {
        test_type(0u16);
        test_type(1723u16);
        test_type(65534u16);
    }

    #[test]
    fn test_signed_int() {
        test_type(0i32);
        test_type(123127i32);
        test_type(-171238i32);
        test_type(2147483647i32);
    }

    #[test]
    fn test_signed_long() {
        test_type(0i64);
        test_type(123127i64);
        test_type(-12123127i64);
        test_type(2147483647i64);
        test_type(-10170482028482i64);
    }

    #[test]
    fn test_float() {
        test_type(0.2313f32);
        test_type(0f32);
        test_type(123123213f32);
        test_type(-123123f32);
    }

    #[test]
    fn test_double() {
        test_type(0.2313f64);
        test_type(0f64);
        test_type(123123213f64);
        test_type(-123123f64);
    }

    #[test]
    fn test_var_int() {
        test_type(VarInt(0));
        test_type(VarInt(1231231));
        test_type(VarInt(2147483647));
        test_type(VarInt(-2147483648));
        test_type(VarInt(-1));
        test_type(VarInt(-1001237));
    }

    #[test]
    fn test_var_long() {
        test_type(VarLong(0));
        test_type(VarLong(1231231));
        test_type(VarLong(12312319123));
        test_type(VarLong(9223372036854775807));
        test_type(VarLong(-1));
        test_type(VarLong(-12312319123));
        test_type(VarLong(-9223372036854775808));
        test_type(VarLong(-1001237));
    }

    #[test]
    fn test_string() {
        test_type(String::from("hello my name is joey 123"));
        test_type(String::from(""));
        test_type(String::from("AAAA"));
        test_type(String::from("hello my name is joey 123").repeat(1000));
    }

    #[test]
    fn test_nbt() {
        test_type(NamedNbtTag {
            root: nbt::Tag::Compound(alloc::vec![
                nbt::Tag::String("test 123".to_owned()).with_name("abc 123")
            ])
            .with_name("root"),
        })
    }

    #[test]
    fn test_int_position() {
        test_type(IntPosition {
            x: 12312,
            y: -32,
            z: 321312,
        });

        test_type(IntPosition {
            x: 12312,
            y: -32,
            z: -321312,
        });

        test_type(IntPosition {
            x: -12312,
            y: -32,
            z: -321312,
        });

        test_type(IntPosition {
            x: -12312,
            y: 32,
            z: 321312,
        });

        test_type(IntPosition { x: 0, y: 0, z: 0 });

        test_type(IntPosition {
            x: 48,
            y: 232,
            z: 12,
        });

        test_type(IntPosition {
            x: 33554431,
            y: 2047,
            z: 33554431,
        });

        test_type(IntPosition {
            x: -33554432,
            y: -2048,
            z: -33554432,
        });

        test_type(IntPosition {
            x: 3,
            y: 0,
            z: 110655,
        });
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_uuid() {
        for _ in 0..5 {
            test_type(UUID4::random());
        }
    }

    #[test]
    fn test_angle() {
        test_type(Angle { value: 0 });
        test_type(Angle { value: 24 });
        test_type(Angle { value: 255 });
        test_type(Angle { value: 8 });
    }

    fn test_type<S: Serialize + Deserialize + PartialEq + Debug>(value: S) {
        let bytes = {
            let mut test = BytesSerializer::default();
            value
                .mc_serialize(&mut test)
                .expect("serialization should succeed");
            test.into_bytes()
        };
        let deserialized =
            S::mc_deserialize(bytes.as_slice()).expect("deserialization should succeed");
        assert!(deserialized.data.is_empty());
        assert_eq!(
            deserialized.value, value,
            "deserialized value == serialized value"
        );
        let re_serialized = {
            let mut test = BytesSerializer::default();
            deserialized
                .value
                .mc_serialize(&mut test)
                .expect("serialization should succeed");
            test.into_bytes()
        };
        assert_eq!(
            re_serialized, bytes,
            "serialized value == original serialized bytes"
        );
    }
}
