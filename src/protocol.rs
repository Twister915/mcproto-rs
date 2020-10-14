use crate::{Deserialize, DeserializeErr, Serialize};
use std::fmt;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ProtocolSpec {
    pub name: String,
    pub packets: Vec<ProtocolPacketSpec>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ProtocolPacketSpec {
    pub state: String,
    pub direction: String,
    pub id: i32,
    pub name: String,
    pub body_struct: String,
    pub fields: Vec<ProtocolPacketField>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ProtocolPacketField {
    pub name: String,
    pub kind: String,
}

pub trait PacketIdentifier: Clone + fmt::Debug + PartialEq + Serialize {}

impl<T: Clone + fmt::Debug + PartialEq + Serialize> PacketIdentifier for T {}

pub trait Packet<I: PacketIdentifier>: Serialize {
    fn id(&self) -> I;

    fn mc_deserialize(raw: RawPacket<'_, I>) -> Result<Self, PacketErr>;
}

pub enum PacketErr {
    UnknownId(i32),
    DeserializeFailed(DeserializeErr),
    ExtraData(Vec<u8>),
}

impl fmt::Display for PacketErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use PacketErr::*;
        match self {
            UnknownId(id) => f.write_fmt(format_args!("unknown packet id {:?}", id)),
            DeserializeFailed(err) => {
                f.write_fmt(format_args!("failed to deserialize packet: {:?}", err))
            }
            ExtraData(data) => f.write_fmt(format_args!(
                "extra data unparsed at end of packet: {:?}",
                data
            )),
        }
    }
}

impl fmt::Debug for PacketErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <dyn fmt::Display>::fmt(self, f)
    }
}

impl std::error::Error for PacketErr {}

#[derive(Debug, Clone, PartialEq)]
pub struct RawPacket<'a, I> {
    pub id: I,
    pub data: &'a [u8],
}

pub trait ProtocolType: Serialize + Deserialize {}

impl<T: Serialize + Deserialize> ProtocolType for T {}

#[cfg(test)]
pub trait TestRandom {
    fn test_gen_random() -> Self;
}

#[macro_export]
macro_rules! as_item {
    ($i:item) => {
        $i
    };
}

#[macro_export]
macro_rules! __protocol_body_serialize_def_helper {
    ($to: ident, $slf: ident, $fieldn: ident, $($field_rest: ident),+) => {
        $to.serialize_other(&$slf.$fieldn)?;
        $crate::__protocol_body_serialize_def_helper!($to, $slf, $($field_rest),+);
    };

    ( $to: ident, $slf: ident, $fieldn: ident ) => {
        $to.serialize_other(&$slf.$fieldn)
    };
}

#[macro_export]
macro_rules! __protocol_body_def_helper {
    ($bodyt: ident { }) => {
        #[derive(Debug, Clone, PartialEq, Default)]
        pub struct $bodyt;

        impl Serialize for $bodyt {
            fn mc_serialize<S: Serializer>(&self, _: &mut S) -> SerializeResult {
                Ok(())
            }
        }

        impl Deserialize for $bodyt {
            fn mc_deserialize(data: &[u8]) -> DeserializeResult<'_, Self> {
                Deserialized::ok(Self::default(), data)
            }
        }

        #[cfg(test)]
        impl TestRandom for $bodyt {
            fn test_gen_random() -> Self {
                Self::default()
            }
        }
    };
    ($bodyt: ident { $($fname: ident: $ftyp: ty ),+ }) => {
        $crate::as_item! {
            #[derive(Debug, Clone, PartialEq)]
            pub struct $bodyt {
               $(pub $fname: $ftyp),+
            }
        }

        impl Serialize for $bodyt {
            fn mc_serialize<S: Serializer>(&self, to: &mut S) -> SerializeResult {
                $(
                    to.serialize_other(&self.$fname)?;
                )+
                Ok(())
            }
        }

        impl Deserialize for $bodyt {
            fn mc_deserialize(_rest: &[u8]) -> DeserializeResult<'_, Self> {
                $(let Deserialized{ value: $fname, data: _rest } = <$ftyp>::mc_deserialize(_rest)?;)+

                Deserialized::ok(Self{ $($fname),+ }, _rest)
            }
        }

        #[cfg(test)]
        impl TestRandom for $bodyt {
            fn test_gen_random() -> Self {
                Self{ $($fname: <$ftyp>::test_gen_random()),+ }
            }
        }
    }
}

#[macro_export]
macro_rules! define_protocol {
    ($packett: ident, $rawpackett: ident, $rawdt: ident, $directiont: ident, $statet: ident, $idt: ident, $idi: ident => { $($nam: ident, $id: literal, $state: ident, $direction: ident => $body: ident { $($fnam: ident: $ftyp: ty),* }),*}) => {
        #[derive(Debug, PartialEq, Eq, Clone, Copy)]
        pub struct $idi {
            pub id: $idt,
            pub state: $statet,
            pub direction: $directiont
        }

        impl crate::Serialize for $idi {
            fn mc_serialize<S: Serializer>(&self, to: &mut S) -> SerializeResult {
                VarInt(self.id).mc_serialize(to)
            }
        }

        impl From<($idt, $statet, $directiont)> for $idi {
            fn from(tuple: ($idt, $statet, $directiont)) -> Self {
                let (id, state, direction) = tuple;
                Id { id, state, direction }
            }
        }

        $crate::as_item! {
            #[derive(Debug, PartialEq, Clone)]
            pub enum $packett {
                $($nam($body)),*,
            }
        }

        $crate::as_item! {
            #[derive(Debug, PartialEq)]
            pub enum $rawpackett<'a> {
                $($nam($rawdt<'a, $body>)),*,
            }
        }

        impl crate::protocol::Packet<$idi> for $packett {
            fn id(&self) -> $idi {
                use self::$packett::*;
                use self::$statet::*;
                use self::$directiont::*;

                match self {
                    $($nam(_) => ($id, $state, $direction)),*
                }.into()
            }

            fn mc_deserialize(raw: crate::protocol::RawPacket<$idi>) ->
                Result<Self, crate::protocol::PacketErr>
            {
                use self::$packett::*;
                use self::$statet::*;
                use self::$directiont::*;
                use crate::protocol::PacketErr::*;
                use crate::{Deserialize, Deserialized};

                let id = raw.id;
                let data = raw.data;

                match (id.id, id.state, id.direction) {
                    $(($id, $state, $direction) => {
                        let Deserialized { value: body, data: rest } = $body::mc_deserialize(data).map_err(DeserializeFailed)?;
                        if !rest.is_empty() {
                            Err(ExtraData(rest.to_vec()))
                        } else {
                            Ok($nam(body))
                        }
                    }),*,
                    other => Err(UnknownId(other.0)),
                }
            }
        }

        impl crate::Serialize for $packett {
            fn mc_serialize<S: crate::Serializer>(&self, to: &mut S) -> crate::SerializeResult {
                use self::$packett::*;
                match self {
                    $($nam(body) => to.serialize_other(body)),+
                }
            }
        }

        impl $packett {
            pub fn describe() -> crate::protocol::ProtocolSpec {
                crate::protocol::ProtocolSpec {
                    name: stringify!($packett).to_owned(),
                    packets: vec!(
                        $(crate::protocol::ProtocolPacketSpec{
                            state: stringify!($state).to_owned(),
                            direction: stringify!($direction).to_owned(),
                            id: $id,
                            name: stringify!($nam).to_owned(),
                            body_struct: stringify!($body).to_owned(),
                            fields: vec!(
                                $(crate::protocol::ProtocolPacketField{
                                    name: stringify!($fnam).to_owned(),
                                    kind: stringify!($ftyp).to_owned(),
                                }),*
                            )
                        }),*,
                    )
                }
            }
        }

        impl<'a> std::convert::TryFrom<crate::protocol::RawPacket<'a, $idi>> for $rawpackett<'a> {
            type Error = crate::protocol::PacketErr;

            fn try_from(value: crate::protocol::RawPacket<'a, $idi>) -> Result<Self, Self::Error> {
                use self::$rawpackett::*;
                use self::$statet::*;
                use self::$directiont::*;
                use crate::protocol::PacketErr::*;

                match (value.id.id, value.id.state, value.id.direction) {
                    $(($id, $state, $direction) => Ok($nam($rawdt {
                        data: value.data,
                        _typ: std::marker::PhantomData,
                    }))),*,
                    other => Err(UnknownId(other.0))
                }
            }
        }

        impl<'a> std::convert::Into<crate::protocol::RawPacket<'a, $idi>> for $rawpackett<'a> {
            fn into(self) -> crate::protocol::RawPacket<'a, $idi> {
                crate::protocol::RawPacket {
                    id: self.id(),
                    data: self.bytes(),
                }
            }
        }

        impl<'a> std::convert::Into<&'a [u8]> for $rawpackett<'a> {
            fn into(self) -> &'a [u8] {
                use self::$rawpackett::*;

                match self {
                    $($nam(bod) => bod.data),*
                }
            }
        }

        impl<'a> $rawpackett<'a> {
            pub fn id(&self) -> $idi {
                use self::$rawpackett::*;
                use self::$statet::*;
                use self::$directiont::*;

                match self {
                    $($nam(_) => ($id, $state, $direction)),*
                }.into()
            }

            pub fn deserialize(self) -> Result<$packett, crate::protocol::PacketErr> {
                use crate::protocol::Packet;
                $packett::mc_deserialize(self.into())
            }

            pub fn bytes(&self) -> &'a [u8] {
                use self::$rawpackett::*;

                match self {
                    $($nam(bod) => bod.data),*
                }
            }
        }

        #[derive(PartialEq, Debug)]
        pub struct $rawdt<'a, T> {
            pub data: &'a [u8],
            _typ: std::marker::PhantomData<T>
        }

        impl<'a, T> $rawdt<'a, T> where T: crate::Deserialize {
            pub fn deserialize(&self) -> Result<T, crate::protocol::PacketErr> {
                use crate::protocol::PacketErr::*;

                let Deserialized { value: body, data: rest } = T::mc_deserialize(self.data).map_err(DeserializeFailed)?;
                if !rest.is_empty() {
                    Err(ExtraData(rest.to_vec()))
                } else {
                    Ok(body)
                }
            }
        }

        $($crate::__protocol_body_def_helper!($body { $($fnam: $ftyp),* });)*
    };
}

#[macro_export]
macro_rules! count_num {
    () => { 0 };
    ($item: tt) => { 1 };
    ($item: tt, $($rest: tt),+) => { 1 + count_num!($($rest),+) }
}

#[macro_export]
macro_rules! proto_enum_with_type {
    ($typ: ty, $from_nam: ident, $as_nam: ident, $fmt: literal, $typname: ident, $(($bval: literal, $nam: ident)),*) => {
        $crate::as_item! {
            #[derive(PartialEq, Clone, Copy)]
            pub enum $typname {
                $($nam),*
            }
        }

        impl Serialize for $typname {
            fn mc_serialize<S: Serializer>(&self, to: &mut S) -> SerializeResult {
                to.serialize_other(&self.$as_nam())
            }
        }

        impl Deserialize for $typname {
            fn mc_deserialize(data: &[u8]) -> DeserializeResult<'_, Self> {
                <$typ>::mc_deserialize(data)?.and_then(move |id, rest| {
                    Self::$from_nam(id).map(move |val| {
                        Deserialized::ok(val, rest)
                    }).unwrap_or_else(|| Err(DeserializeErr::CannotUnderstandValue(format!("invalid {} {}", stringify!($typname), id))))
                })
            }
        }

        impl Into<$typ> for $typname {
            fn into(self) -> $typ {
                use $typname::*;
                match self {
                    $($nam => $bval.into()),*,
                }
            }
        }

        impl std::fmt::Display for $typname {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, $fmt, self.name(), self.$as_nam())?;
                Ok(())
            }
        }

        impl std::fmt::Debug for $typname {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, $fmt, self.name(), self.$as_nam())?;
                Ok(())
            }
        }

        impl $typname {
            pub fn $from_nam(b: $typ) -> Option<Self> {
                use $typname::*;
                match b.into() {
                    $($bval => Some($nam)),*,
                    _ => None
                }
            }

            pub fn name(&self) -> &str {
                use $typname::*;
                match self {
                    $($nam => stringify!($nam)),+,
                }
            }

            pub fn $as_nam(&self) -> $typ {
                (*self).into()
            }
        }

        #[cfg(test)]
        impl TestRandom for $typname {
            fn test_gen_random() -> Self {
                let mut idx: usize = (rand::random::<usize>() % (count_num!($($bval),+))) + 1;
                $(
                    idx -= 1;
                    if idx == 0 {
                        return $typname::$nam;
                    }
                )+
                panic!("cannot generate random {}", stringify!($typname));
            }
        }
    }
}

#[macro_export]
macro_rules! proto_byte_enum {
    ($typname: ident, $($bval: literal :: $nam: ident),*) => {
        proto_enum_with_type!(u8, from_byte, as_byte, "{}(0x{:02x})", $typname, $(($bval, $nam)),*);
    }
}

#[macro_export]
macro_rules! proto_varint_enum {
    ($typname: ident, $($bval: literal :: $nam: ident),*) => {
        proto_enum_with_type!(VarInt, from_varint, as_varint, "{}({:?})", $typname, $(($bval, $nam)),*);
    }
}

#[macro_export]
macro_rules! proto_int_enum {
    ($typname: ident, $($bval: literal :: $nam: ident),*) => {
        proto_enum_with_type!(i32, from_int, as_int, "{}(0x{:02x})", $typname, $(($bval, $nam)),*);
    }
}

#[macro_export]
macro_rules! proto_str_enum {
    ($typname: ident, $($sval: literal :: $nam: ident),*) => {
        crate::as_item! {
            #[derive(PartialEq, Clone, Copy)]
            pub enum $typname {
                $($nam),+,
            }
        }

        impl Serialize for $typname {
            fn mc_serialize<S: Serializer>(&self, to: &mut S) -> SerializeResult {
                let name = self.name().to_owned();
                to.serialize_other(&name)
            }
        }

        impl Deserialize for $typname {
            fn mc_deserialize(data: &[u8]) -> DeserializeResult<'_, Self> {
                String::mc_deserialize(data)?.and_then(move |name, rest| {
                    if let Some(v) = Self::from_string(&name) {
                        Deserialized::ok(v, rest)
                    } else {
                        Err(DeserializeErr::CannotUnderstandValue(format!("invalid {} ident '{}'", stringify!($typname), name)))
                    }
                })
            }
        }

        impl $typname {
            pub fn from_str(arg: &str) -> Option<Self> {
                use $typname::*;

                match arg {
                    $($sval => Some($nam)),+,
                    _ => None
                }
            }

            pub fn from_string(arg: &String) -> Option<Self> {
                Self::from_str(arg.as_str())
            }

            pub fn name(&self) -> &str {
                use $typname::*;

                match self {
                    $($nam => $sval),+,
                }
            }
        }

        impl From<&$typname> for String {
            fn from(arg: &$typname) -> Self {
                arg.name().to_owned()
            }
        }

        impl From<$typname> for String {
            fn from(arg: $typname) -> Self {
                arg.name().to_owned()
            }
        }

        impl std::fmt::Display for $typname {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(self.name())
            }
        }

        impl std::fmt::Debug for $typname {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                <dyn std::fmt::Display>::fmt(self, f)
            }
        }

        #[cfg(test)]
        impl TestRandom for $typname {
            fn test_gen_random() -> Self {
                let mut idx: usize = (rand::random::<usize>() % (count_num!($($nam),+))) + 1;
                $(
                    idx -= 1;
                    if idx == 0 {
                        return $typname::$nam;
                    }
                )+
                panic!("cannot generate random {}", stringify!($typname));
            }
        }
    }
}

#[macro_export]
macro_rules! proto_byte_flag {
    ($typname: ident, $($bval: literal :: $nam: ident),*) => {
        #[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
        pub struct $typname(pub u8);

        impl $typname {
            $(paste::paste! {
                pub fn [<is_ $nam>](&self) -> bool {
                    self.0 & $bval != 0
                }
            })*

            $(paste::paste! {
                pub fn [<set_ $nam>](&mut self, value: bool) {
                    if value {
                        self.0 |= $bval;
                    } else {
                        self.0 ^= $bval;
                    }
                }
            })*

            $(paste::paste! {
                pub fn [<with_ $nam>](mut self, value: bool) -> Self {
                    if value {
                        self.0 |= $bval;
                    } else {
                        self.0 ^= $bval;
                    }

                    self
                }
            })*
        }

        impl Serialize for $typname {
            fn mc_serialize<S: Serializer>(&self, to: &mut S) -> SerializeResult {
                to.serialize_byte(self.0)
            }
        }

        impl Deserialize for $typname {
            fn mc_deserialize(data: &[u8]) -> DeserializeResult<'_, Self> {
                Ok(u8::mc_deserialize(data)?.map(move |b| $typname(b)))
            }
        }

        #[cfg(test)]
        impl TestRandom for $typname {
            fn test_gen_random() -> Self {
                let mut out = <$typname>::default();
                $(paste::paste! {
                    out.[<set_ $nam>](rand::random::<bool>());
                })+
                out
            }
        }
    }
}

#[macro_export]
macro_rules! counted_array_type {
    ($name: ident, $countert: ty, $tousize_fn: ident, $fromusize_fn: ident) => {
        #[derive(Debug, Clone, PartialEq)]
        pub struct $name<T>
        where
            T: Debug + Clone + PartialEq,
        {
            pub data: Vec<T>,
        }

        impl<T> Serialize for $name<T>
        where
            T: Serialize + Debug + Clone + PartialEq,
        {
            fn mc_serialize<S: Serializer>(&self, to: &mut S) -> SerializeResult {
                let count: $countert = $fromusize_fn(self.data.len());
                to.serialize_other(&count)?;

                for entry in &self.data {
                    to.serialize_other(entry)?;
                }

                Ok(())
            }
        }

        impl<T> Deserialize for $name<T>
        where
            T: Deserialize + Debug + Clone + PartialEq,
        {
            fn mc_deserialize(data: &[u8]) -> DeserializeResult<'_, Self> {
                let Deserialized {
                    value: raw_count,
                    mut data,
                } = <$countert>::mc_deserialize(data)?;
                let count: usize = $tousize_fn(raw_count);

                let mut out = Vec::with_capacity(count);
                for _ in 0..count {
                    let Deserialized {
                        value: next,
                        data: rest,
                    } = T::mc_deserialize(data)?;
                    data = rest;
                    out.push(next);
                }

                Deserialized::ok(Self { data: out }, data)
            }
        }

        impl<T> Into<Vec<T>> for $name<T>
        where
            T: Debug + Clone + PartialEq,
        {
            fn into(self) -> Vec<T> {
                self.data
            }
        }

        impl<T> From<Vec<T>> for $name<T>
        where
            T: Debug + Clone + PartialEq,
        {
            fn from(data: Vec<T>) -> Self {
                Self { data }
            }
        }

        impl<'a, T> IntoIterator for &'a mut $name<T>
        where
            T: Debug + Clone + PartialEq,
        {
            type Item = &'a mut T;
            type IntoIter = std::slice::IterMut<'a, T>;

            fn into_iter(self) -> Self::IntoIter {
                let data = &mut self.data;
                data.iter_mut()
            }
        }

        impl<'a, T> IntoIterator for &'a $name<T>
        where
            T: Debug + Clone + PartialEq,
        {
            type Item = &'a T;
            type IntoIter = std::slice::Iter<'a, T>;

            fn into_iter(self) -> Self::IntoIter {
                let data = &self.data;
                data.iter()
            }
        }

        impl<T> IntoIterator for $name<T>
        where
            T: Debug + Clone + PartialEq,
        {
            type Item = T;
            type IntoIter = std::vec::IntoIter<T>;

            fn into_iter(self) -> Self::IntoIter {
                self.data.into_iter()
            }
        }

        #[cfg(test)]
        impl<T> TestRandom for $name<T>
        where
            T: TestRandom + Debug + Clone + PartialEq,
        {
            fn test_gen_random() -> Self {
                let elem_count: usize = rand::random::<usize>() % 32;
                let mut out = Vec::with_capacity(elem_count);
                for _ in 0..elem_count {
                    out.push(T::test_gen_random());
                }

                Self { data: out }
            }
        }
    };
}
