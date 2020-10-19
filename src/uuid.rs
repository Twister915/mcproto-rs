use crate::utils::*;
use serde::{Deserializer, Serializer};
use alloc::{fmt, string::{ToString, String}};
use fmt::{Display, Debug, Formatter};

#[derive(Copy, Clone, PartialEq, Hash, Eq)]
pub struct UUID4 {
    raw: u128,
}

impl Display for UUID4 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.hex().as_str())
    }
}

impl Debug for UUID4 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
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

    #[cfg(feature = "std")]
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
        // 8-4-4-4-12
        let (s0, from) = str_split(from, 8)?;
        str_check_hex(s0)?;
        let from = str_tag(from, "-")?;
        let (s1, from) = str_split(from, 4)?;
        str_check_hex(s1)?;
        let from = str_tag(from, "-")?;
        let (s2, from) = str_split(from, 4)?;
        str_check_hex(s2)?;
        let from = str_tag(from, "-")?;
        let (s3, from) = str_split(from, 4)?;
        str_check_hex(s3)?;
        let from = str_tag(from, "-")?;
        let (s4, from) = str_split(from, 12)?;
        str_check_hex(s4)?;
        str_check_eof(from)?;

        Some(Self{
            parts: [s0, s1, s2, s3, s4],
        })
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
}

fn str_tag<'a>(source: &'a str, tag: &str) -> Option<&'a str> {
    let (front, back) = str_split(source, tag.len())?;
    if front != tag {
        None
    } else {
        Some(back)
    }
}

fn str_check_eof(source: &str) -> Option<()> {
    if source.is_empty() {
        Some(())
    } else {
        None
    }
}

fn str_check_hex(mut source: &str) -> Option<()> {
    if source.is_empty() {
        return None
    }

    loop {
        let (part, rest) = str_split(source, 2)?;
        for c in part.chars() {
            parse_hex_char(c as u8)?;
        }

        source = rest;
        if source.is_empty() {
            return Some(());
        }
    }
}

fn str_split(source: &str, n: usize) -> Option<(&str, &str)> {
    if source.len() < n {
        None
    } else {
        Some(source.split_at(n))
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::UUID4;
    use alloc::string::ToString;

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

    #[bench]
    fn bench_parse_uuid4(b: &mut test::Bencher) {
        let rand = UUID4::random();
        let str = rand.to_string();
        b.bytes = str.bytes().len() as u64;
        b.iter(|| {
            UUID4::parse(str.as_str()).expect("should parse fine")
        })
    }

    #[bench]
    fn bench_uuid4_to_str(b: &mut test::Bencher) {
        let rand = UUID4::random();
        b.bytes = 128;
        b.iter(|| {
            rand.to_string()
        })
    }
}
