use crate::protocol::TestRandom;
use crate::types::Chat;
use crate::uuid::UUID4;
use crate::{
    Deserialize as McDeserialize, DeserializeErr, DeserializeResult, Serialize as McSerialize,
    SerializeErr, SerializeResult,
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct StatusSpec {
    pub version: StatusVersionSpec,
    pub players: StatusPlayersSpec,
    pub description: Chat,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub favicon: Option<StatusFaviconSpec>,
}

impl McSerialize for StatusSpec {
    fn mc_serialize<S: crate::Serializer>(&self, to: &mut S) -> SerializeResult {
        serde_json::to_string(self)
            .map_err(move |err| {
                SerializeErr::FailedJsonEncode(format!("failed to serialize json status {}", err))
            })?
            .mc_serialize(to)
    }
}

impl McDeserialize for StatusSpec {
    fn mc_deserialize(data: &[u8]) -> DeserializeResult<'_, Self> {
        String::mc_deserialize(data)?.try_map(move |v| {
            serde_json::from_str(v.as_str()).map_err(move |err| {
                DeserializeErr::CannotUnderstandValue(format!(
                    "failed to deserialize json status {}",
                    err
                ))
            })
        })
    }
}

#[cfg(test)]
impl TestRandom for StatusSpec {
    fn test_gen_random() -> Self {
        Self {
            version: StatusVersionSpec {
                protocol: rand::random(),
                name: String::test_gen_random(),
            },
            players: StatusPlayersSpec {
                sample: Vec::default(),
                max: rand::random(),
                online: rand::random(),
            },
            favicon: None,
            description: Chat::test_gen_random(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct StatusVersionSpec {
    pub name: String,
    pub protocol: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct StatusPlayersSpec {
    pub max: i32,
    pub online: i32,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default = "Vec::default")]
    pub sample: Vec<StatusPlayerSampleSpec>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct StatusPlayerSampleSpec {
    pub name: String,
    pub id: UUID4,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StatusFaviconSpec {
    pub content_type: String,
    pub data: Vec<u8>,
}

impl Serialize for StatusFaviconSpec {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        let data_base64 = base64::encode(self.data.as_slice());
        let content = format!("data:{};base64,{}", self.content_type, data_base64);
        serializer.serialize_str(content.as_str())
    }
}

impl<'de> Deserialize<'de> for StatusFaviconSpec {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        struct Visitor;
        impl serde::de::Visitor<'_> for Visitor {
            type Value = StatusFaviconSpec;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(formatter, "a string with base64 data for favicon")
            }

            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
                use lazy_static::lazy_static;
                use regex::Regex;
                // regex to parse valid base64 content
                const PATTERN: &str = r"^data:([A-Za-z/]+);base64,([-A-Za-z0-9+/]*={0,3})$";
                lazy_static! {
                    static ref RE: Regex = Regex::new(PATTERN).expect("regex is valid");
                }

                // try to use regex on the input
                // RE.captures_iter(v).next() means "try to get the first capture iterator"
                // then we take that iterator, get(1), and if 1 exists, get(2), and if both exist,
                // then we try to parse the base64, and drop the error if one occurs. We then
                // wrap the content_type and parsed data in StatusFaviconSpec
                // then we convert the option to a result using map and unwrap_or_else
                let mut captures: regex::CaptureMatches<'_, '_> = RE.captures_iter(v);
                captures
                    .next()
                    .and_then(move |captures| {
                        captures.get(1).and_then(move |content_type| {
                            captures.get(2).and_then(move |raw_base64| {
                                base64::decode(raw_base64.as_str().as_bytes())
                                    .map(move |data| StatusFaviconSpec {
                                        content_type: content_type.as_str().to_owned(),
                                        data,
                                    })
                                    .ok()
                            })
                        })
                    })
                    .map(move |result| Ok(result))
                    .unwrap_or_else(|| {
                        Err(serde::de::Error::invalid_value(
                            serde::de::Unexpected::Str(v),
                            &self,
                        ))
                    })
            }
        }

        deserializer.deserialize_str(Visitor {})
    }
}
