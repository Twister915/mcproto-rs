use crate::utils::*;
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserializer, Serializer};
use std::fmt::{Debug, Display, Formatter};

#[derive(Copy, Clone, PartialEq, Hash, Eq)]
pub struct UUID4 {
    raw: u128,
}

impl Display for UUID4 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.hex().as_str())
    }
}

impl Debug for UUID4 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("UUID4{")?;
        f.write_str(self.hex().as_str())?;
        f.write_str("}")
    }
}

impl From<u128> for UUID4 {
    fn from(raw: u128) -> Self {
        UUID4 { raw }
    }
}

impl serde::Serialize for UUID4 {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}

impl<'de> serde::Deserialize<'de> for UUID4 {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        struct Visitor;
        impl serde::de::Visitor<'_> for Visitor {
            type Value = UUID4;

            fn expecting(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "a string representing the UUID")
            }

            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
                if let Some(id) = UUID4::parse(v) {
                    Ok(id)
                } else {
                    Err(serde::de::Error::invalid_value(
                        serde::de::Unexpected::Str(v),
                        &self,
                    ))
                }
            }
        }

        deserializer.deserialize_str(Visitor {})
    }
}

impl UUID4 {
    pub fn parse(from: &str) -> Option<UUID4> {
        RawUUID::from_str(from).and_then(move |raw| raw.parse4())
    }

    pub fn random() -> Self {
        UUID4 {
            raw: rand::random(),
        }
    }

    pub fn to_u128(self) -> u128 {
        self.raw
    }

    pub fn hex(self) -> String {
        let bytes = self.raw.to_be_bytes();
        let parts = [
            hex(&bytes[..4]),
            hex(&bytes[4..6]),
            hex(&bytes[6..8]),
            hex(&bytes[8..10]),
            hex(&bytes[10..16]),
        ];
        parts.join("-")
    }
}

struct RawUUID<'a> {
    parts: [&'a str; 5],
}

impl<'a> RawUUID<'a> {
    fn from_str(from: &'a str) -> Option<RawUUID<'a>> {
        const PATTERN: &str = r"^([A-Fa-f0-9]{8})-?([A-Fa-f0-9]{4})-?([A-Fa-f0-9]{4})-?([A-Fa-f0-9]{4})-?([A-Fa-f0-9]{12})$";
        // let re = Regex::new(PATTERN).expect("regex is valid");
        lazy_static! {
            static ref RE: Regex = Regex::new(PATTERN).expect("regex is valid");
        }

        RE.captures_iter(from)
            .filter_map(move |c| {
                c.get(1).map(move |g| g.as_str()).and_then(move |g0| {
                    c.get(2).map(move |g| g.as_str()).and_then(move |g1| {
                        c.get(3).map(move |g| g.as_str()).and_then(move |g2| {
                            c.get(4).map(move |g| g.as_str()).and_then(move |g3| {
                                c.get(5).map(move |g4| RawUUID {
                                    parts: [g0, g1, g2, g3, g4.as_str()],
                                })
                            })
                        })
                    })
                })
            })
            .nth(0)
    }

    fn parse4(self) -> Option<UUID4> {
        let mut bit_index: usize = 0;
        let mut raw: u128 = 0;

        for part in &self.parts {
            for char in part.chars() {
                if let Some(parsed) = parse_hex_char(char as u8) {
                    raw |= (parsed as u128) << (124 - bit_index);
                    bit_index += 4;
                } else {
                    return None;
                }
            }
        }

        Some(UUID4 { raw })
    }
    //
    // fn parse3(self) -> Option<UUID3> {
    //     self.parse4().map(move |id| UUID3 { raw: id.raw })
    // }
}

// #[derive(Clone, Copy, Debug, PartialEq, Hash, Eq)]
// pub struct UUID3 {
//     raw: u128,
// }
//
// impl UUID3 {
//     pub fn parse(from: &str) -> Option<UUID3> {
//         RawUUID::from_str(from).and_then(move |id| id.parse3())
//     }
//
//     pub fn from(namespace: UUID4, data: &str) -> UUID3 {
//         namespace.raw
//     }
// }

#[cfg(test)]
mod tests {
    use super::UUID4;

    #[test]
    fn test_random_uuid4() {
        UUID4::random();
    }

    const VALID_UUID: &str = "e1cde35a-0758-47f6-adf8-9dcb44884e5d";

    #[test]
    fn test_uuid4_parse() {
        UUID4::parse(VALID_UUID).expect("should parse valid uuid correctly");
    }

    #[test]
    fn test_parsed_uuid4_to_hex() {
        let uuid_hex = UUID4::parse(VALID_UUID)
            .expect("should parse valid uuid correctly")
            .hex();

        assert_eq!(uuid_hex.as_str(), VALID_UUID)
    }

    #[test]
    fn test_uuid4_equal() {
        let uuid_a = UUID4::parse(VALID_UUID).expect("should parse valid uuid correctly");
        let uuid_b = UUID4::parse(VALID_UUID).expect("should parse valid uuid correctly");
        assert_eq!(uuid_a, uuid_b);
    }

    #[test]
    fn test_random_uuid4_hex() {
        let src_uuid = UUID4::random();
        let uuid_hex = src_uuid.hex();
        let uuid_parsed =
            UUID4::parse(uuid_hex.as_str()).expect("should parse generated uuid correctly");
        assert_eq!(src_uuid, uuid_parsed);
        let uuid_parsed_hex = uuid_parsed.hex();
        assert_eq!(uuid_hex, uuid_parsed_hex);
    }

    #[test]
    fn test_display_uuid() {
        println!("got uuid {}", UUID4::random());
    }

    #[test]
    fn test_debug_uuid() {
        println!("got uuid {:?}", UUID4::random());
    }

    #[test]
    fn test_to_json() {
        let id = UUID4::random();
        let str = serde_json::to_string(&id).expect("should serialize fine");
        assert_eq!(str, format!("\"{}\"", id.to_string()))
    }

    #[test]
    fn test_from_json() {
        let id = UUID4::random();
        let json = format!("\"{}\"", id.to_string());
        let deserialized: UUID4 = serde_json::from_str(json.as_str()).expect("should read fine");
        assert_eq!(deserialized, id);
    }
}
