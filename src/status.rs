use crate::types::Chat;
use crate::uuid::UUID4;
use crate::{
    Deserialize as McDeserialize, DeserializeErr, DeserializeResult, Serialize as McSerialize,
    SerializeErr, SerializeResult,
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use alloc::{string::String, fmt, vec::Vec, borrow::ToOwned};
use alloc::format;

#[cfg(all(test, feature = "std"))]
use crate::protocol::TestRandom;

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

#[cfg(all(test, feature = "std"))]
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
        let content = alloc::format!("data:{};base64,{}", self.content_type, data_base64);
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
                // favicon syntax data:{content-type};base64,{}
                let v = str_tag(v, "data:", &self)?;
                let (content_type, v) = str_until_pat(v, ";", &self)?;
                let rest = str_tag(v, "base64,", &self)?;
                match base64::decode(rest) {
                    Ok(data) => {
                        Ok(StatusFaviconSpec{
                            data,
                            content_type: content_type.to_owned(),
                        })
                    },
                    Err(err) => {
                        Err(E::custom(format_args!("failed base64 decode {:?}", err)))
                    }
                }
            }
        }

        deserializer.deserialize_str(Visitor {})
    }
}

fn str_tag<'a, E, V>(
    target: &'a str,
    expected: &str,
    v: &V,
) -> Result<&'a str, E> where
    E: serde::de::Error,
    V: serde::de::Visitor<'a>
{
    let (front, back) = str_take(target, expected.len(), v)?;
    if front != expected {
        Err(E::invalid_value(serde::de::Unexpected::Str(target), v))
    } else {
        Ok(back)
    }
}

fn str_until_pat<'a, E, V>(
    target: &'a str,
    pat: &str,
    v: &V,
) -> Result<(&'a str, &'a str), E> where
    E: serde::de::Error,
    V: serde::de::Visitor<'a>
{
    let n_pat = pat.len();
    if target.len() < n_pat {
        return Err(E::invalid_value(serde::de::Unexpected::Str(target), v));
    }

    for i in 0..=(target.len()-n_pat) {
        let v = &target[i..i+n_pat];
        if v == pat {
            return Ok((&target[..i], &target[i+1..]));
        }
    }

    Err(E::invalid_value(serde::de::Unexpected::Str(target), v))
}

fn str_take<'a, E, V>(
    target: &'a str,
    n: usize,
    v: &V,
) -> Result<(&'a str, &'a str), E> where
    E: serde::de::Error,
    V: serde::de::Visitor<'a>
{
    if target.len() < n {
        Err(E::invalid_value(serde::de::Unexpected::Str(target), v))
    } else {
        Ok(target.split_at(n))
    }
}