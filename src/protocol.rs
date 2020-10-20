use crate::{Deserialize, DeserializeErr, Serialize, Serializer, SerializeResult};
use alloc::{string::String, fmt, vec::Vec};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PacketDirection {
    ClientBound,
    ServerBound,
}

impl PacketDirection {
    pub fn opposite(&self) -> Self {
        use PacketDirection::*;
        match self {
            ClientBound => ServerBound,
            ServerBound => ClientBound,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum State {
    Handshaking,
    Status,
    Login,
    Play,
}

impl State {
    pub fn name(&self) -> String {
        use State::*;
        match self {
            Handshaking => "Handshaking",
            Status => "Status",
            Login => "Login",
            Play => "Play",
        }
            .to_owned()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Id {
    pub id: i32,
    pub state: State,
    pub direction: PacketDirection,
}

impl Serialize for Id {
    fn mc_serialize<S: Serializer>(&self, to: &mut S) -> SerializeResult {
        to.serialize_other(&self.id)
    }
}

impl From<(i32, State, PacketDirection)> for Id {
    fn from(other: (i32, State, PacketDirection)) -> Self {
        let (id, state, direction) = other;
        Self { id, state, direction }
    }
}

impl From<Id> for (i32, State, PacketDirection) {
    fn from(id: Id) -> Self {
        let Id { id, state, direction } = id;
        (id, state, direction)
    }
}

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

pub trait Packet: Sized {
    fn id(&self) -> Id;

    fn version() -> crate::types::VarInt;

    fn mc_deserialize(raw: RawPacket<'_>) -> Result<Self, PacketErr>;

    fn mc_serialize_body<S>(&self, to: &mut S) -> SerializeResult where S: Serializer;
}

pub enum PacketErr {
    UnknownId(Id),
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

#[cfg(feature = "std")]
impl std::error::Error for PacketErr {}

#[derive(Debug, Clone, PartialEq)]
pub struct RawPacket<'a> {
    pub id: Id,
    pub data: &'a [u8],
}

pub trait ProtocolType: Serialize + Deserialize {}

impl<T: Serialize + Deserialize> ProtocolType for T {}

#[cfg(all(test, feature = "std"))]
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
macro_rules! proto_struct {
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

        #[cfg(all(test, feature = "std"))]
        impl TestRandom for $bodyt {
            fn test_gen_random() -> Self {
                Self::default()
            }
        }
    };
    ($bodyt: ident $(<$($g: ident),*>)? {
        $($fname: ident: $ftyp: ty ),+
    }) => {
        $crate::as_item! {
            #[derive(Debug, Clone, PartialEq)]
            pub struct $bodyt$(<$($g),*> where $($g: alloc::fmt::Debug + Clone + PartialEq),*)? {
               $(pub $fname: $ftyp),+
            }
        }

        impl$(<$($g),*>)? Serialize for $bodyt$(<$($g),*> where $($g: Serialize + alloc::fmt::Debug + Clone + PartialEq),*)? {
            fn mc_serialize<S: Serializer>(&self, to: &mut S) -> SerializeResult {
                $(
                    to.serialize_other(&self.$fname)?;
                )+
                Ok(())
            }
        }

        impl$(<$($g),*>)? Deserialize for $bodyt$(<$($g),*> where $($g: Deserialize + alloc::fmt::Debug + Clone + PartialEq),*)? {
            fn mc_deserialize(_rest: &[u8]) -> DeserializeResult<'_, Self> {
                $(let Deserialized{ value: $fname, data: _rest } = <$ftyp>::mc_deserialize(_rest)?;)+

                Deserialized::ok(Self{ $($fname),+ }, _rest)
            }
        }

        #[allow(unused_parens)]
        impl$(<$($g),*>)? From<($($ftyp),+)> for $bodyt$(<$($g),*>)? $(where $($g: alloc::fmt::Debug + Clone + PartialEq),*)? {
            fn from(other: ($($ftyp),+)) -> Self {
                let ($($fname),+) = other;
                Self { $($fname),+ }
            }
        }

        #[allow(unused_parens)]
        impl$(<$($g),*>)? From<$bodyt$(<$($g),*>)?> for ($($ftyp),+) $(where $($g: alloc::fmt::Debug + Clone + PartialEq),*)? {
            fn from(other: $bodyt$(<$($g),*>)?) -> Self {
                ($(other.$fname),+)
            }
        }

        #[cfg(all(test, feature = "std"))]
        impl$(<$($g),*>)? TestRandom for $bodyt$(<$($g),*> where $($g: TestRandom + alloc::fmt::Debug + Clone + PartialEq),*)? {
            fn test_gen_random() -> Self {
                Self{ $($fname: <$ftyp>::test_gen_random()),+ }
            }
        }
    }
}

#[macro_export]
macro_rules! define_protocol {
    ($version: literal, $packett: ident, $rawpackett: ident, $rawdt: ident => {
        $($nam: ident, $id: literal, $state: ident, $direction: ident => $body: ident {
            $($fnam: ident: $ftyp: ty),* }),*
        }
    ) => {
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

        impl crate::protocol::Packet for $packett {
            fn version() -> crate::types::VarInt {
                crate::types::VarInt($version)
            }

            fn id(&self) -> crate::protocol::Id {
                use self::$packett::*;
                use crate::protocol::State::*;
                use crate::protocol::PacketDirection::*;

                match self {
                    $($nam(_) => ($id, $state, $direction)),*
                }.into()
            }

            fn mc_deserialize(raw: crate::protocol::RawPacket) ->
                Result<Self, crate::protocol::PacketErr>
            {
                use self::$packett::*;
                use crate::protocol::State::*;
                use crate::protocol::PacketDirection::*;
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
                    other => Err(UnknownId(other.into())),
                }
            }

            fn mc_serialize_body<S>(&self, to: &mut S) -> crate::SerializeResult where S: crate::Serializer {
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
                    packets: alloc::vec!(
                        $(crate::protocol::ProtocolPacketSpec{
                            state: stringify!($state).to_owned(),
                            direction: stringify!($direction).to_owned(),
                            id: $id,
                            name: stringify!($nam).to_owned(),
                            body_struct: stringify!($body).to_owned(),
                            fields: alloc::vec!(
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

        #[cfg(feature = "std")]
        impl<'a> std::convert::TryFrom<crate::protocol::RawPacket<'a>> for $rawpackett<'a> {
            type Error = crate::protocol::PacketErr;

            fn try_from(value: crate::protocol::RawPacket<'a>) -> Result<Self, Self::Error> {
                use self::$rawpackett::*;
                use crate::protocol::State::*;
                use crate::protocol::PacketDirection::*;
                use crate::protocol::PacketErr::*;
                #[cfg(feature = "std")]
                use std::marker;
                #[cfg(not(feature = "std"))]
                use no_std_compat::marker;

                match (value.id.id, value.id.state, value.id.direction) {
                    $(($id, $state, $direction) => Ok($nam($rawdt {
                        data: value.data,
                        _typ: marker::PhantomData,
                    }))),*,
                    other => Err(UnknownId(other.into()))
                }
            }
        }

        #[cfg(feature = "std")]
        impl<'a> std::convert::Into<crate::protocol::RawPacket<'a>> for $rawpackett<'a> {
            fn into(self) -> crate::protocol::RawPacket<'a> {
                self.into_raw_packet()
            }
        }

        impl<'a> $rawpackett<'a> {
            fn into_raw_packet(self) -> crate::protocol::RawPacket<'a> {
                crate::protocol::RawPacket {
                    id: self.id(),
                    data: self.bytes(),
                }
            }
        }

        #[cfg(feature = "std")]
        impl<'a> std::convert::Into<&'a [u8]> for $rawpackett<'a> {
            fn into(self) -> &'a [u8] {
                use self::$rawpackett::*;

                match self {
                    $($nam(bod) => bod.data),*
                }
            }
        }

        impl<'a> $rawpackett<'a> {
            pub fn id(&self) -> crate::protocol::Id {
                use self::$rawpackett::*;
                use crate::protocol::State::*;
                use crate::protocol::PacketDirection::*;

                match self {
                    $($nam(_) => ($id, $state, $direction)),*
                }.into()
            }

            pub fn deserialize(self) -> Result<$packett, crate::protocol::PacketErr> {
                use crate::protocol::Packet;
                $packett::mc_deserialize(self.into_raw_packet())
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
            _typ: core::marker::PhantomData<T>
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

        $($crate::proto_struct!($body { $($fnam: $ftyp),* });)*
    };
}

#[macro_export]
macro_rules! strip_plus {
    (+ $($rest: tt)*) => {
        $($rest)*
    }
}

#[macro_export]
macro_rules! proto_enum_deserialize_variant {
    ($data: ident, $ty: ident :: $nam: ident ($bod: ty)) => {
        Ok(<$bod>::mc_deserialize($data)?.map(move |body| $ty::$nam(body)))
    };
    ($data: ident, $ty: ident :: $nam: ident) => {
        Deserialized::ok($ty::$nam, $data)
    };
}

#[macro_export]
macro_rules! instead_of_ident {
    ($ident: tt, $replacement: tt) => {
        $replacement
    }
}

#[macro_export]
macro_rules! proto_enum_with_type {
    ($typ: ty, $typname: ident, $(($bval: literal, $nam: ident $(($bod: ty))?)),*) => {
        $crate::as_item! {
            #[derive(PartialEq, Clone, Debug)]
            pub enum $typname {
                $($nam $(($bod))?),*
            }
        }

        impl Serialize for $typname {
            fn mc_serialize<S: Serializer>(&self, to: &mut S) -> SerializeResult {
                let id_to_serialize: $typ = match self {
                    $($typname::$nam$((instead_of_ident!($bod, _)))? => $bval),*
                }.into();
                to.serialize_other(&id_to_serialize)?;
                self.serialize_body(to)
            }
        }

        impl Deserialize for $typname {
            fn mc_deserialize(data: &[u8]) -> DeserializeResult<'_, Self> {
                <$typ>::mc_deserialize(data)?.and_then(move |id, rest| {
                    Self::deserialize_with_id(id, rest)
                })
            }
        }

        impl $typname {
            pub const fn variant_count() -> usize {
                crate::strip_plus!($(+ crate::instead_of_ident!($bval, 1))+)
            }

            pub fn deserialize_with_id<'a>(id: $typ, data: &'a[u8]) -> DeserializeResult<'a, Self> {
                match id.into() {
                    $($bval => proto_enum_deserialize_variant!(data, $typname::$nam $(($bod))?)),*,
                    other => {
                        return Err(DeserializeErr::CannotUnderstandValue(alloc::format!("invalid {} {:?}", stringify!($typname), other)))
                    }
                }
            }

            pub fn name(&self) -> &str {
                match self {
                    $($typname::$nam$((instead_of_ident!($bod, _)))? => stringify!($nam)),*
                }
            }

            pub fn id(&self) -> $typ {
                match self {
                    $($typname::$nam$((instead_of_ident!($bod, _)))? => $bval.into()),*
                }
            }

            #[allow(unused_variables)]
            pub fn serialize_body<S: Serializer>(&self, to: &mut S) -> SerializeResult {
                match &self {
                    $($typname::$nam$((instead_of_ident!($bod, bod)))? => {
                        $(to.serialize_other(instead_of_ident!($bod, bod))?;)?
                        Ok(())
                    }),*
                }
            }
        }

        #[cfg(all(test, feature = "std"))]
        impl TestRandom for $typname {
            fn test_gen_random() -> Self {
                let mut rng = rand::thread_rng();
                use rand::distributions::Distribution;
                let distr = rand::distributions::Uniform::new(1, Self::variant_count() + 1);
                let mut idx: usize = distr.sample(&mut rng);
                $(
                    idx -= 1;
                    if idx == 0 {
                        return $typname::$nam$((<$bod>::test_gen_random()))?;
                    }
                )+
                panic!("cannot generate random {}", stringify!($typname));
            }
        }
    }
}

#[macro_export]
macro_rules! proto_byte_enum {
    ($typname: ident, $($bval: literal :: $nam: ident $(($bod: ty))?),*) => {
        proto_enum_with_type!(u8, $typname, $(($bval, $nam $(($bod))?)),*);
    }
}

#[macro_export]
macro_rules! proto_varint_enum {
    ($typname: ident, $($bval: literal :: $nam: ident $(($bod: ty))?),*) => {
        proto_enum_with_type!(VarInt, $typname, $(($bval, $nam $(($bod))?)),*);
    }
}

#[macro_export]
macro_rules! proto_int_enum {
    ($typname: ident, $($bval: literal :: $nam: ident $(($bod: ty))?),*) => {
        proto_enum_with_type!(i32, $typname, $(($bval, $nam $(($bod))?)),*);
    }
}

#[macro_export]
macro_rules! proto_str_enum {
    ($typname: ident, $($sval: literal :: $nam: ident $(($bod: ident))?),*) => {
        crate::as_item! {
            #[derive(PartialEq, Clone, Debug)]
            pub enum $typname {
                $($nam $(($bod))?),*
            }
        }

        impl Serialize for $typname {
            fn mc_serialize<S: Serializer>(&self, to: &mut S) -> SerializeResult {
                let name = self.name().to_owned();
                to.serialize_other(&name)?;
                self.serialize_body(to)
            }
        }

        impl Deserialize for $typname {
            fn mc_deserialize(data: &[u8]) -> DeserializeResult<'_, Self> {
                String::mc_deserialize(data)?.and_then(move |name, rest| {
                    Self::deserialize_with_id(name.as_str(), rest)
                })
            }
        }

        impl $typname {
            pub const fn variant_count() -> usize {
                crate::strip_plus!($(+ crate::instead_of_ident!($sval, 1))+)
            }

            pub fn name(&self) -> &str {
                match self {
                    $($typname::$nam$((instead_of_ident!($bod, _)))? => $sval),+,
                }
            }

            pub fn id(&self) -> String {
                self.name().to_owned()
            }

            pub fn deserialize_with_id<'a>(name: &str, data: &'a[u8]) -> DeserializeResult<'a, Self> {
                match name {
                    $($sval => proto_enum_deserialize_variant!(data, $typname::$nam $(($bod))?)),*,
                    other => Err(DeserializeErr::CannotUnderstandValue(alloc::format!("invalid {} ident '{}'", stringify!($typname), other)))
                }
            }

            #[allow(unused_variables)]
            pub fn serialize_body<S: Serializer>(&self, to: &mut S) -> SerializeResult {
                match &self {
                    $($typname::$nam$((instead_of_ident!($bod, bod)))? => {
                        $(to.serialize_other(instead_of_ident!($bod, bod))?;)?
                        Ok(())
                    }),*
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

        #[cfg(all(test, feature = "std"))]
        impl TestRandom for $typname {
            fn test_gen_random() -> Self {
                let mut rng = rand::thread_rng();
                use rand::distributions::Distribution;
                let distr = rand::distributions::Uniform::new(1, Self::variant_count() + 1);
                let mut idx: usize = distr.sample(&mut rng);
                $(
                    idx -= 1;
                    if idx == 0 {
                        return $typname::$nam$(($bod::test_gen_random()))?;
                    }
                )+
                panic!("cannot generate random {}", stringify!($typname));
            }
        }
    }
}

#[macro_export]
macro_rules! proto_byte_flag {
    ($typname: ident, $($bval: literal :: $isnam: ident $setnam: ident),*) => {
        #[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
        pub struct $typname(pub u8);

        impl $typname {
            $(pub fn $isnam(&self) -> bool {
                self.0 & $bval != 0
            }

            pub fn $setnam(&mut self, value: bool) {
                if value {
                    self.0 |= $bval;
                } else {
                    self.0 ^= $bval;
                }
            })+
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

        #[cfg(all(test, feature = "std"))]
        impl TestRandom for $typname {
            fn test_gen_random() -> Self {
                let mut out = <$typname>::default();
                $(
                    out.$setnam(rand::random::<bool>());
                )+
                out
            }
        }
    }
}