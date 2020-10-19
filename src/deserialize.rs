use crate::types::VarInt;
use alloc::{vec::Vec, string::{FromUtf8Error, String}, fmt};

pub enum DeserializeErr {
    Eof,
    VarNumTooLong(Vec<u8>),
    NegativeLength(VarInt),
    BadStringEncoding(FromUtf8Error),
    InvalidBool(u8),
    NbtUnknownTagType(u8),
    NbtBadLength(isize),
    NbtInvalidStartTag(u8),
    CannotUnderstandValue(String),
    FailedJsonDeserialize(String),
}

impl fmt::Display for DeserializeErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use DeserializeErr::*;
        match self {
            Eof => f.write_str("unexpected eof"),
            VarNumTooLong(data) => {
                f.write_fmt(format_args!("var num is too long: data={:?}", data))
            }
            NegativeLength(data) => {
                f.write_fmt(format_args!("negative length encountered {:?}", data))
            }
            BadStringEncoding(data) => f.write_fmt(format_args!(
                "failed to decode string, utf error: {:?}",
                data
            )),
            InvalidBool(value) => f.write_fmt(format_args!(
                "could not decode boolean, unexpected byte: {:?}",
                value
            )),
            NbtUnknownTagType(data) => f.write_fmt(format_args!("nbt: bad tag type {}", data)),
            NbtBadLength(data) => f.write_fmt(format_args!("nbt: bad length {:?}", data)),
            NbtInvalidStartTag(data) => {
                f.write_fmt(format_args!("nbt: unexpected start tag id: {:?}", data))
            }
            CannotUnderstandValue(data) => {
                f.write_fmt(format_args!("cannot understand value: {:?}", data))
            }
            FailedJsonDeserialize(data) => {
                f.write_fmt(format_args!("failed to deserialize json: {:?}", data))
            }
        }
    }
}
impl fmt::Debug for DeserializeErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <dyn fmt::Display>::fmt(self, f)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for DeserializeErr {}

impl<'b, R> Into<DeserializeResult<'b, R>> for DeserializeErr {
    fn into(self) -> DeserializeResult<'b, R> {
        Err(self)
    }
}

pub struct Deserialized<'b, R> {
    pub value: R,
    pub data: &'b [u8],
}

impl<'b, R> Into<DeserializeResult<'b, R>> for Deserialized<'b, R> {
    fn into(self) -> DeserializeResult<'b, R> {
        Ok(self)
    }
}

impl<'b, R> Deserialized<'b, R> {
    pub fn create(value: R, data: &'b [u8]) -> Self {
        Deserialized { value, data }
    }

    pub fn ok(value: R, rest: &'b [u8]) -> DeserializeResult<'b, R> {
        Self::create(value, rest).into()
    }

    pub fn replace<T>(self, other: T) -> Deserialized<'b, T> {
        Deserialized {
            value: other,
            data: self.data,
        }
    }

    pub fn map<F, T>(self, f: F) -> Deserialized<'b, T>
    where
        F: FnOnce(R) -> T,
    {
        Deserialized {
            value: f(self.value),
            data: self.data,
        }
    }

    pub fn try_map<F, T>(self, f: F) -> DeserializeResult<'b, T>
    where
        F: FnOnce(R) -> Result<T, DeserializeErr>,
    {
        match f(self.value) {
            Ok(new_value) => Ok(Deserialized {
                value: new_value,
                data: self.data,
            }),
            Err(err) => Err(err),
        }
    }

    pub fn and_then<F, T>(self, f: F) -> DeserializeResult<'b, T>
    where
        F: FnOnce(R, &'b [u8]) -> DeserializeResult<'b, T>,
    {
        f(self.value, self.data)
    }
}

impl<'b, R> From<(R, &'b [u8])> for Deserialized<'b, R> {
    fn from(v: (R, &'b [u8])) -> Self {
        let (value, data) = v;
        Deserialized { value, data }
    }
}

pub type DeserializeResult<'b, R> = Result<Deserialized<'b, R>, DeserializeErr>;

pub trait Deserialize: Sized {
    fn mc_deserialize(data: &[u8]) -> DeserializeResult<Self>;
}
