// ... PRIMITIVE TYPES ...

use alloc::{string::String, vec::Vec, fmt};
use crate::utils::*;
use crate::uuid::UUID4;
use crate::*;

pub use super::chat::*;

#[cfg(all(test, feature = "std"))]
use crate::protocol::TestRandom;
use crate::byte_order::{ProtoByteOrder, ByteOrder};
use std::ops::Deref;

// bool
impl Serialize for bool {
    fn mc_serialize<S: Serializer>(&self, to: &mut S) -> SerializeResult {
        to.serialize_byte(if *self { 1 } else { 0 })
    }
}

impl Deserialize for bool {
    fn mc_deserialize(data: &[u8]) -> DeserializeResult<'_, Self> {
        ProtoByteOrder::read_ubyte(data)?.try_map(move |b| match b {
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

macro_rules! def_primitive {
    ($nam: ty, $read: ident, $write: ident) => {
        impl Serialize for $nam {
            fn mc_serialize<S: Serializer>(&self, to: &mut S) -> SerializeResult {
                let data = ProtoByteOrder::$write(*self);
                to.serialize_bytes(&data)
            }
        }

        impl Deserialize for $nam {
            fn mc_deserialize(data: &[u8]) -> DeserializeResult<'_, Self> {
                ProtoByteOrder::$read(data)
            }
        }

        #[cfg(all(test, feature = "std"))]
        impl TestRandom for $nam {
            fn test_gen_random() -> Self {
                rand::random()
            }
        }
    };
}

def_primitive!(u8, read_ubyte, write_ubyte);
def_primitive!(i8, read_byte, write_byte);
def_primitive!(u16, read_ushort, write_ushort);
def_primitive!(i16, read_short, write_short);
def_primitive!(u32, read_uint, write_uint);
def_primitive!(i32, read_int, write_int);
def_primitive!(u64, read_ulong, write_ulong);
def_primitive!(i64, read_long, write_long);
def_primitive!(u128, read_u2long, write_u2long);
def_primitive!(i128, read_2long, write_2long);
def_primitive!(f32, read_float, write_float);
def_primitive!(f64, read_double, write_double);

// VAR INT AND VAR LONG
macro_rules! def_varnum {
    ($nam: ident, $data_type: ty, $working_type: ty, $max_bytes: literal) => {
        #[derive(Copy, Clone, PartialOrd, PartialEq, Default, Hash, Ord, Eq)]
        pub struct $nam(pub $data_type);

        impl Serialize for $nam {
            fn mc_serialize<S: Serializer>(&self, to: &mut S) -> SerializeResult {
                let mut out = [0u8; $max_bytes];
                let mut v: $working_type = self.0 as $working_type;
                let mut byte_idx = 0;
                let mut has_more = true;
                while has_more {
                    if byte_idx == out.len() {
                        panic!("tried to write too much data for {}", stringify!($nam));
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

                to.serialize_bytes(&out[..byte_idx])
            }
        }

        impl Deserialize for $nam {
            fn mc_deserialize(orig_data: &[u8]) -> DeserializeResult<Self> {
                let mut data = orig_data;
                let mut v: $working_type = 0;
                let mut bit_place: usize = 0;
                let mut i: usize = 0;
                let mut has_more = true;

                while has_more {
                    if i == $max_bytes {
                        return DeserializeErr::VarNumTooLong(Vec::from(&orig_data[..i])).into();
                    }
                    let Deserialized { value: byte, data: rest } = ProtoByteOrder::read_ubyte(data)?;
                    data = rest;
                    has_more = byte & 0x80 != 0;
                    v |= ((byte as $working_type) & 0x7F) << bit_place;
                    bit_place += 7;
                    i += 1;
                }

                Deserialized::ok(Self(v as $data_type), data)
            }
        }

        impl From<$data_type> for $nam {
            fn from(other: $data_type) -> Self {
                Self(other)
            }
        }

        impl From<$nam> for $data_type {
            fn from(other: $nam) -> Self {
                other.0
            }
        }

        impl core::ops::Deref for $nam {
            type Target = $data_type;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl fmt::Display for $nam {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl fmt::Debug for $nam {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(stringify!($nam))?;
                f.write_str("(")?;
                write!(f, "{}", self.0)?;
                f.write_str(")")?;
                Ok(())
            }
        }

        #[cfg(all(test, feature = "std"))]
        impl TestRandom for $nam {
            fn test_gen_random() -> Self {
                let out: $data_type = rand::random();
                Self(out)
            }
        }
    }
}

def_varnum!(VarInt, i32, u32, 5);
def_varnum!(VarLong, i64, u64, 10);

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
                take(length.0 as usize, rest)?.try_map(move |taken| {
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

        let data = ProtoByteOrder::write_ulong(((x_raw << 38) | (z_raw << 12) | y_raw) as u64);
        to.serialize_bytes(&data)
    }
}

impl Deserialize for IntPosition {
    fn mc_deserialize(data: &[u8]) -> DeserializeResult<'_, Self> {
        let Deserialized { value: raw, data } = ProtoByteOrder::read_ulong(data)?;
        let mut x = ((raw >> 38) as u32) & 0x3FFFFFF;
        let mut z = ((raw >> 12) & 0x3FFFFFF) as u32;
        let mut y = ((raw & 0xFFF) as u16) & 0xFFF;

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
        Ok(ProtoByteOrder::read_ubyte(data)?.map(move |b| Angle { value: b }))
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
        let bytes = ProtoByteOrder::write_u2long(self.to_u128());
        to.serialize_bytes(&bytes[..])
    }
}

impl Deserialize for UUID4 {
    fn mc_deserialize(data: &[u8]) -> DeserializeResult<'_, Self> {
        Ok(ProtoByteOrder::read_u2long(data)?.map(move |raw| UUID4::from(raw)))
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
pub struct ItemStack {
    pub item_id: VarInt,
    pub item_count: i8,
    pub nbt: Option<nbt::NamedTag>,
}

impl Serialize for ItemStack {
    fn mc_serialize<S: Serializer>(&self, to: &mut S) -> SerializeResult {
        to.serialize_other(&self.item_id)?;
        to.serialize_other(&self.item_count)?;
        match self.nbt.as_ref() {
            Some(nbt) => to.serialize_bytes(nbt.bytes().as_slice()),
            None => to.serialize_byte(nbt::Tag::End.id()),
        }
    }
}

impl Deserialize for ItemStack {
    fn mc_deserialize(data: &[u8]) -> DeserializeResult<'_, Self> {
        let Deserialized { value: item_id, data } = VarInt::mc_deserialize(data)?;
        let Deserialized { value: item_count, data } = i8::mc_deserialize(data)?;
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
        }.map(move |nbt| Self {
            item_id,
            item_count,
            nbt,
        }))
    }
}

#[cfg(all(test, feature = "std"))]
impl TestRandom for ItemStack {
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

pub type Slot = Option<ItemStack>;

macro_rules! def_vector_type {
    ($name: ident, $($fnam: ident),+) => {
        crate::as_item! {
            pub struct $name<T> {
                $(pub $fnam: T),+
            }
        }

        impl<T> Serialize for $name<T> where T: Serialize {
            fn mc_serialize<S: Serializer>(&self, to: &mut S) -> SerializeResult {
                $(
                    to.serialize_other(&self.$fnam)?;
                )+
                Ok(())
            }
        }

        impl<T> Deserialize for $name<T> where T: Deserialize {
            fn mc_deserialize(data: &[u8]) -> DeserializeResult<'_, Self> {
                $(let Deserialized { value: $fnam, data } = T::mc_deserialize(data)?;)+
                Deserialized::ok(Self { $($fnam),+ }, data)
            }
        }

        #[cfg(all(test, feature = "std"))]
        impl<T> TestRandom for $name<T> where T: TestRandom {
            fn test_gen_random() -> Self {
                Self {
                    $($fnam: T::test_gen_random(),)+
                }
            }
        }

        impl<T> Clone for $name<T> where T: Clone {
            fn clone(&self) -> Self {
                Self {
                    $($fnam: self.$fnam.clone(),)+
                }
            }
        }

        impl<T> Copy for $name<T> where T: Copy {}

        impl<T, Rhs> PartialEq<$name<Rhs>> for $name<T> where T: PartialEq<Rhs> {
            fn eq(&self, other: &$name<Rhs>) -> bool {
                $(self.$fnam.eq(&other.$fnam)) && +
            }

            fn ne(&self, other: &$name<Rhs>) -> bool {
                $(self.$fnam.ne(&other.$fnam)) || +
            }
        }

        impl<T> core::hash::Hash for $name<T> where T: core::hash::Hash {
            fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
                $(self.$fnam.hash(state);)+
            }
        }

        impl<T> fmt::Debug for $name<T> where T: fmt::Debug {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(stringify!($name))?;
                f.write_str("( ")?;
                $(
                    f.write_str(stringify!($fnam))?;
                    f.write_str("=")?;
                    self.$fnam.fmt(f)?;
                    f.write_str(" ")?;
                )+
                f.write_str(")")?;
                Ok(())
            }
        }

        impl<T> fmt::Display for $name<T> where T: fmt::Display {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(stringify!($name))?;
                f.write_str("( ")?;
                $(
                    f.write_str(stringify!($fnam))?;
                    f.write_str("=")?;
                    self.$fnam.fmt(f)?;
                    f.write_str(" ")?;
                )+
                f.write_str(")")?;
                Ok(())
            }
        }

        impl<T> From<$name<T>> for ($(crate::instead_of_ident!($fnam, T)),+) {
            fn from(other: $name<T>) -> Self {
                ($(other.$fnam),+)
            }
        }

        impl<'a, T> From<&'a $name<T>> for ($(&'a crate::instead_of_ident!($fnam, T)),+) {
            fn from(other: &'a $name<T>) -> Self {
                ($(&other.$fnam),+)
            }
        }

        impl<'a, T> From<&'a $name<T>> for ($(crate::instead_of_ident!($fnam, T)),+) where T: Clone {
            fn from(other: &'a $name<T>) -> Self {
                ($(other.$fnam.clone()),+)
            }
        }

        impl<T> From<($(crate::instead_of_ident!($fnam, T)),+)> for $name<T> {
            fn from(other: ($(crate::instead_of_ident!($fnam, T)),+)) -> Self {
                let ($($fnam),+) = other;
                Self { $($fnam),+ }
            }
        }

        impl<'a, T> From<&'a ($(crate::instead_of_ident!($fnam, T)),+)> for $name<T> where T: Clone {
            fn from(other: &'a ($(crate::instead_of_ident!($fnam, T)),+)) -> Self {
                let ($($fnam),+) = other;
                $(let $fnam = $fnam.clone();)+
                Self { $($fnam),+ }
            }
        }

        impl<'a, T> From<($(&'a crate::instead_of_ident!($fnam, T)),+)> for $name<T> where T: Clone {
            fn from(other: ($(&'a crate::instead_of_ident!($fnam, T)),+)) -> Self {
                let ($($fnam),+) = other;
                $(let $fnam = $fnam.clone();)+
                Self { $($fnam),+ }
            }
        }

        impl<T> $name<T> {
            pub fn from_other<O>(other: O) -> Self where O: Into<($(crate::instead_of_ident!($fnam, T)),+)> {
                let ($($fnam),+) = other.into();
                Self { $($fnam,)+ }
            }

            pub fn as_other<O>(&self) -> O where O: From<($(crate::instead_of_ident!($fnam, T)),+)>, T: Clone {
                O::from(self.into())
            }

            pub fn into_other<O>(self) -> O where O: From<($(crate::instead_of_ident!($fnam, T)),+)> {
                O::from(self.into())
            }
        }
    };
}

def_vector_type!(Vec3, x, y, z);
def_vector_type!(Vec2, x, y);
def_vector_type!(ChunkPosition, x, z);
pub type TopDownPosition<T> = ChunkPosition<T>;
def_vector_type!(EntityRotation, yaw, pitch);

proto_struct!(EntityLocation<P, R> {
    position: Vec3<P>,
    rotation: EntityRotation<R>
});

#[derive(Clone, Debug, PartialEq)]
pub struct CountedArray<E, C> {
    data: Vec<E>,
    _counter_type: core::marker::PhantomData<C>,
}

pub trait ArrayCounter: Serialize + Deserialize {

    fn as_count(&self) -> usize;

    fn from_count(count: usize) -> Self;
}

impl<E, C> Serialize for CountedArray<E, C> where E: Serialize, C: ArrayCounter {
    fn mc_serialize<S: Serializer>(&self, to: &mut S) -> SerializeResult {
        let count = C::from_count(self.data.len());
        to.serialize_other(&count)?;
        for elem in &self.data {
            to.serialize_other(elem)?;
        }
        Ok(())
    }
}

impl<E, C> Deserialize for CountedArray<E, C> where E: Deserialize, C: ArrayCounter {
    fn mc_deserialize(data: &[u8]) -> DeserializeResult<'_, Self> {
        let Deserialized { value: count, mut data } = C::mc_deserialize(data)?;
        let count = count.as_count();
        let mut elems = Vec::with_capacity(count);
        for _ in 0..count {
            let Deserialized { value: elem, data: rest } = E::mc_deserialize(data)?;
            data = rest;
            elems.push(elem);
        }

        Deserialized::ok(Self {
            data: elems,
            _counter_type: core::marker::PhantomData,
        }, data)
    }
}

impl<E, C> core::ops::Deref for CountedArray<E, C> where C: ArrayCounter {
    type Target = Vec<E>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<E, C> core::ops::DerefMut for CountedArray<E, C> where C: ArrayCounter {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl<E, C> From<CountedArray<E, C>> for Vec<E> where C: ArrayCounter {
    fn from(other: CountedArray<E, C>) -> Self {
        other.data
    }
}

impl<E, C> From<Vec<E>> for CountedArray<E, C> where C: ArrayCounter {
    fn from(data: Vec<E>) -> Self {
        Self {
            data,
            _counter_type: core::marker::PhantomData,
        }
    }
}

#[cfg(all(test, feature = "std"))]
impl<E, C> TestRandom for CountedArray<E, C>
    where E: TestRandom, C: ArrayCounter
{
    fn test_gen_random() -> Self {
        let elem_count: usize = rand::random::<usize>() % 32;
        let mut out = Vec::with_capacity(elem_count);
        for _ in 0..elem_count {
            out.push(E::test_gen_random());
        }

        out.into()
    }
}

impl ArrayCounter for VarInt {
    fn as_count(&self) -> usize {
        self.0 as usize
    }

    fn from_count(count: usize) -> Self {
        Self(count as i32)
    }
}

impl ArrayCounter for i16 {
    fn as_count(&self) -> usize {
        (*self) as usize
    }

    fn from_count(count: usize) -> Self {
        count as i16
    }
}

impl ArrayCounter for i32 {
    fn as_count(&self) -> usize {
        (*self) as usize
    }

    fn from_count(count: usize) -> Self {
        count as i32
    }
}

impl ArrayCounter for i8 {
    fn as_count(&self) -> usize {
        (*self) as usize
    }

    fn from_count(count: usize) -> Self {
        count as i8
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RemainingBytes {
    pub data: Vec<u8>,
}

impl Serialize for RemainingBytes {
    fn mc_serialize<S: Serializer>(&self, to: &mut S) -> SerializeResult {
        to.serialize_bytes(self.data.as_slice())
    }
}

impl Deserialize for RemainingBytes {
    fn mc_deserialize(data: &[u8]) -> DeserializeResult<'_, Self> {
        Deserialized::ok(
            RemainingBytes {
                data: Vec::from(data),
            },
            &[],
        )
    }
}

impl Into<Vec<u8>> for RemainingBytes {
    fn into(self) -> Vec<u8> {
        self.data
    }
}

impl From<Vec<u8>> for RemainingBytes {
    fn from(data: Vec<u8>) -> Self {
        Self { data }
    }
}

impl core::ops::Deref for RemainingBytes {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl core::ops::DerefMut for RemainingBytes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

#[cfg(all(test, feature = "std"))]
impl TestRandom for RemainingBytes {
    fn test_gen_random() -> Self {
        let size: usize = rand::random::<usize>() % 256;
        let mut out = Vec::with_capacity(size);
        for _ in 0..size {
            out.push(rand::random());
        }

        Self { data: out }
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
