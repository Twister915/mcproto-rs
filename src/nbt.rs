use crate::utils::{
    read_int, read_long, read_one_byte, read_short, take, write_int, write_long, write_short,
};
use crate::{DeserializeErr, DeserializeResult, Deserialized};
use std::fmt;

#[cfg(test)]
use crate::protocol::TestRandom;

#[derive(Clone, Debug, PartialEq)]
pub struct NamedTag {
    pub name: String,
    pub payload: Tag,
}

impl NamedTag {
    pub fn root_compound_tag_from_bytes(data: &[u8]) -> DeserializeResult<NamedTag> {
        read_nbt_data(data)
    }

    pub fn is_end(&self) -> bool {
        match self.payload {
            Tag::End => true,
            _ => false,
        }
    }
}

#[cfg(test)]
impl TestRandom for NamedTag {
    fn test_gen_random() -> Self {
        Self {
            name: "".to_owned(),
            payload: Tag::Compound(vec!(Self{
                name: String::test_gen_random(),
                payload: Tag::test_gen_random(),
            })),
        }
    }
}

impl fmt::Display for NamedTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "TAG_{}('{}'): ",
            self.payload.tag_type_name(),
            self.name
        ))?;
        self.payload.write_contents(f)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Tag {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(Vec<u8>),
    String(String),
    List(Vec<Tag>),
    Compound(Vec<NamedTag>),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
    End,
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("TAG_{}: ", self.tag_type_name()))?;
        self.write_contents(f)
    }
}

impl Tag {
    pub fn with_name(self, name: &str) -> NamedTag {
        NamedTag {
            name: name.into(),
            payload: self,
        }
    }

    pub fn tag_type_name(&self) -> &str {
        match self {
            Tag::Byte(_) => "Byte",
            Tag::Short(_) => "Short",
            Tag::Int(_) => "Int",
            Tag::Long(_) => "Long",
            Tag::Float(_) => "Float",
            Tag::Double(_) => "Double",
            Tag::ByteArray(_) => "Byte_Array",
            Tag::String(_) => "String",
            Tag::List(_) => "List",
            Tag::Compound(_) => "Compound",
            Tag::IntArray(_) => "Int_Array",
            Tag::LongArray(_) => "Long_Array",
            Tag::End => "END",
        }
    }

    fn write_contents(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Tag::Byte(v) => f.write_fmt(format_args!("{}", *v)),
            Tag::Short(v) => f.write_fmt(format_args!("{}", *v)),
            Tag::Int(v) => f.write_fmt(format_args!("{}", *v)),
            Tag::Long(v) => f.write_fmt(format_args!("{}L", *v)),
            Tag::Float(v) => f.write_fmt(format_args!("{}", *v)),
            Tag::Double(v) => f.write_fmt(format_args!("{}", *v)),
            Tag::ByteArray(v) => f.write_fmt(format_args!("[{} bytes]", v.len())),
            Tag::String(v) => f.write_fmt(format_args!("\"{}\"", v)),
            Tag::List(v) => {
                let out = write_contents(v);
                f.write_str(out.as_str())
            }
            Tag::Compound(v) => {
                let out = write_contents(v);
                f.write_str(out.as_str())
            }
            Tag::IntArray(v) => f.write_fmt(format_args!("[{} ints]", v.len())),
            Tag::LongArray(v) => f.write_fmt(format_args!("[{} longs]", v.len())),
            Tag::End => f.write_str("END"),
        }
    }
}

#[cfg(test)]
impl TestRandom for Tag {
    fn test_gen_random() -> Self {
        let random_idx = rand::random::<usize>() % 8;
        match random_idx {
            0 => Tag::Byte(i8::test_gen_random()),
            1 => Tag::Short(i16::test_gen_random()),
            2 => Tag::Int(i32::test_gen_random()),
            3 => Tag::Long(i64::test_gen_random()),
            4 => Tag::Float(f32::test_gen_random()),
            5 => Tag::Double(f64::test_gen_random()),
            6 => Tag::String(String::test_gen_random()),
            7 => Tag::List({
                let count = rand::random::<usize>() % 256;
                let mut out = Vec::with_capacity(count);
                let random_idx = rand::random::<usize>() % 6;
                for _ in 0..count {
                    out.push(match random_idx {
                        0 => Tag::Byte(i8::test_gen_random()),
                        1 => Tag::Short(i16::test_gen_random()),
                        2 => Tag::Int(i32::test_gen_random()),
                        3 => Tag::Long(i64::test_gen_random()),
                        4 => Tag::Float(f32::test_gen_random()),
                        5 => Tag::Double(f64::test_gen_random()),
                        6 => Tag::String(String::test_gen_random()),
                        other => panic!("impossible {}", other),
                    });
                }

                out
            }),
            8 => Tag::Compound({
                let count = rand::random::<usize>() % 256;
                let mut out = Vec::with_capacity(count);
                for _ in 0..count {
                    out.push(NamedTag::test_gen_random());
                }
                out
            }),
            other => panic!("impossible {}", other),
        }
    }
}

#[inline]
fn write_contents<F>(contents: &Vec<F>) -> String
where
    F: fmt::Display,
{
    format!(
        "{} entries\n{{\n{}\n}}",
        contents.len(),
        contents
            .iter()
            .flat_map(move |elem| elem
                .to_string()
                .split("\n")
                .map(String::from)
                .collect::<Vec<String>>())
            .map(move |line| "  ".to_owned() + line.as_str())
            .collect::<Vec<String>>()
            .join("\n")
    )
}

// deserialization first

// reads from the root level
fn read_nbt_data(data: &[u8]) -> DeserializeResult<NamedTag> {
    let Deserialized {
        value: tag_type_id,
        data: _,
    } = read_one_byte(data)?;
    match tag_type_id {
        0x0A => read_named_tag(data),
        other => Err(DeserializeErr::NbtInvalidStartTag(other)),
    }
}

// reads any named tag: read id -> read name -> read tag with id -> name tag with name
#[inline]
pub fn read_named_tag(data: &[u8]) -> DeserializeResult<NamedTag> {
    let Deserialized {
        value: tag_type_id,
        data,
    } = read_one_byte(data)?;
    if tag_type_id == 0x00 {
        // tag end
        Deserialized::ok(Tag::End.with_name(""), data)
    } else {
        let Deserialized { value: name, data } = read_string(data)?;
        Ok(read_tag(tag_type_id, data)?.map(move |payload| NamedTag { name, payload }))
    }
}

// reads any tag (given it's id)
#[inline]
pub fn read_tag(tag_type_id: u8, data: &[u8]) -> DeserializeResult<Tag> {
    match tag_type_id {
        0x00 => Deserialized::ok(Tag::End, data),
        0x01 => read_tag_byte(data),
        0x02 => read_tag_short(data),
        0x03 => read_tag_int(data),
        0x04 => read_tag_long(data),
        0x05 => read_tag_float(data),
        0x06 => read_tag_double(data),
        0x07 => read_tag_byte_array(data),
        0x08 => read_tag_string(data),
        0x09 => read_tag_list(data),
        0x0A => read_tag_compound(data),
        0x0B => read_tag_int_array(data),
        0x0C => read_tag_long_array(data),
        other => Err(DeserializeErr::NbtUnknownTagType(other)),
    }
}

#[inline]
fn read_tag_byte(data: &[u8]) -> DeserializeResult<Tag> {
    Ok(read_one_byte(data)?.map(move |byte| Tag::Byte(byte as i8)))
}

#[inline]
fn read_tag_short(data: &[u8]) -> DeserializeResult<Tag> {
    Ok(read_short(data)?.map(move |i| Tag::Short(i as i16)))
}

#[inline]
fn read_tag_int(data: &[u8]) -> DeserializeResult<Tag> {
    Ok(read_int(data)?.map(move |i| Tag::Int(i as i32)))
}

#[inline]
fn read_tag_long(data: &[u8]) -> DeserializeResult<Tag> {
    Ok(read_long(data)?.map(move |i| Tag::Long(i as i64)))
}

#[inline]
fn read_tag_float(data: &[u8]) -> DeserializeResult<Tag> {
    Ok(read_int(data)?.map(move |i| Tag::Float(f32::from_bits(i as u32))))
}

#[inline]
fn read_tag_double(data: &[u8]) -> DeserializeResult<Tag> {
    Ok(read_long(data)?.map(move |i| Tag::Double(f64::from_bits(i as u64))))
}

#[inline]
fn read_tag_byte_array(data: &[u8]) -> DeserializeResult<Tag> {
    Ok(read_int(data)?
        .and_then(move |size, rest| take(size as usize)(rest))?
        .map(move |arr| Tag::ByteArray(Vec::from(arr))))
}

#[inline]
fn read_tag_string(data: &[u8]) -> DeserializeResult<Tag> {
    Ok(read_string(data)?.map(move |str| Tag::String(str)))
}

fn read_tag_list(data: &[u8]) -> DeserializeResult<Tag> {
    let Deserialized {
        value: contents_tag_type_id,
        data,
    } = read_one_byte(data)?;
    let Deserialized {
        value: list_length,
        data,
    } = read_int(data)?;
    if list_length <= 0 {
        if contents_tag_type_id != 0x00 {
            Err(DeserializeErr::NbtBadLength(list_length as isize))
        } else {
            Deserialized::ok(Tag::List(vec![]), data)
        }
    } else {
        let mut out_vec = Vec::with_capacity(list_length as usize);
        let mut remaining_data = data;
        for _ in 0..list_length {
            let Deserialized {
                value: element,
                data: rest,
            } = read_tag(contents_tag_type_id, &remaining_data)?;
            out_vec.push(element);
            remaining_data = rest;
        }

        Deserialized::ok(Tag::List(out_vec), remaining_data)
    }
}

fn read_tag_compound(data: &[u8]) -> DeserializeResult<Tag> {
    let mut out = Vec::new();
    let mut remaining_data = data;
    loop {
        let Deserialized {
            value: elem,
            data: rest,
        } = read_named_tag(remaining_data)?;
        remaining_data = rest;
        if elem.is_end() {
            break;
        }
        out.push(elem);
    }

    Deserialized::ok(Tag::Compound(out), remaining_data)
}

#[inline]
fn read_tag_int_array(data: &[u8]) -> DeserializeResult<Tag> {
    read_array_tag(
        data,
        move |data| Ok(read_int(data)?.map(move |r| r as i32)),
        Tag::IntArray,
    )
}

#[inline]
fn read_tag_long_array(data: &[u8]) -> DeserializeResult<Tag> {
    read_array_tag(
        data,
        move |data| Ok(read_long(data)?.map(move |r| r as i64)),
        Tag::LongArray,
    )
}

#[inline]
fn read_array_tag<'a, R, F, M>(
    data: &'a [u8],
    parser: F,
    finalizer: M,
) -> DeserializeResult<'a, Tag>
where
    F: Fn(&'a [u8]) -> DeserializeResult<'a, R>,
    M: Fn(Vec<R>) -> Tag,
{
    let Deserialized { value: count, data } = read_int(data)?.map(move |v| v as i32);
    if count < 0 {
        Err(DeserializeErr::NbtBadLength(count as isize))
    } else {
        let mut out = Vec::with_capacity(count as usize);
        let mut data_remaining = data;
        for _ in 0..count {
            let Deserialized {
                value: elem,
                data: rest,
            } = parser(data_remaining)?;
            data_remaining = rest;
            out.push(elem);
        }

        Deserialized::ok(finalizer(out), data_remaining)
    }
}

#[inline]
fn read_string(data: &[u8]) -> DeserializeResult<String> {
    read_short(data)?
        .and_then(move |length, data| take(length as usize)(data))?
        .try_map(move |bytes| {
            String::from_utf8(Vec::from(bytes))
                .map_err(move |err| DeserializeErr::BadStringEncoding(err))
        })
}

// serialize
impl NamedTag {
    pub fn bytes(&self) -> Vec<u8> {
        let type_id = self.payload.id();
        if type_id == 0x00 {
            vec![0x00]
        } else {
            let payload_bytes = self.payload.bytes();
            let name_len = self.name.len();
            let name_len_bytes = write_short(name_len as u16);
            let mut out =
                Vec::with_capacity(1 + name_len_bytes.len() + name_len + payload_bytes.len());
            out.push(type_id);
            out.extend_from_slice(&name_len_bytes);
            out.extend(self.name.bytes());
            out.extend(payload_bytes);
            out
        }
    }
}

impl Tag {
    pub fn id(&self) -> u8 {
        match self {
            Tag::Byte(_) => 0x01,
            Tag::Short(_) => 0x02,
            Tag::Int(_) => 0x03,
            Tag::Long(_) => 0x04,
            Tag::Float(_) => 0x05,
            Tag::Double(_) => 0x06,
            Tag::ByteArray(_) => 0x07,
            Tag::String(_) => 0x08,
            Tag::List(_) => 0x09,
            Tag::Compound(_) => 0x0A,
            Tag::IntArray(_) => 0x0B,
            Tag::LongArray(_) => 0x0C,
            Tag::End => 0x00,
        }
    }

    pub fn bytes(&self) -> Vec<u8> {
        match self {
            Tag::Byte(b) => vec![*b as u8],
            Tag::Short(v) => Vec::from(write_short(*v as u16)),
            Tag::Int(v) => Vec::from(write_int(*v as u32)),
            Tag::Long(v) => Vec::from(write_long(*v as u64)),
            Tag::Float(v) => Vec::from(write_int(v.to_bits())),
            Tag::Double(v) => Vec::from(write_long(v.to_bits())),
            Tag::ByteArray(v) => {
                let n = v.len();
                let mut out = Vec::with_capacity(n + 4);
                let size_bytes = write_int(n as u32);
                out.extend_from_slice(&size_bytes);
                out.extend(v);
                out
            }
            Tag::String(v) => {
                let n = v.len();
                let mut out = Vec::with_capacity(n + 2);
                let size_bytes = write_short(n as u16);
                out.extend_from_slice(&size_bytes);
                out.extend(v.bytes());
                out
            }
            Tag::List(v) => {
                let count = v.len();
                let elem_id = {
                    if count == 0 {
                        0x00
                    } else {
                        let mut id = None;
                        for elem in v {
                            let elem_id = elem.id();
                            if let Some(old_id) = id.replace(elem_id) {
                                if old_id != elem_id {
                                    panic!(
                                        "list contains tags of different types, cannot serialize"
                                    );
                                }
                            }
                        }

                        id.expect("there must be some elements in the list")
                    }
                };

                let mut out = Vec::new();
                out.push(elem_id);
                let count_bytes = write_int(count as u32);
                out.extend_from_slice(&count_bytes);
                out.extend(v.iter().flat_map(move |elem| elem.bytes().into_iter()));
                out
            }
            Tag::Compound(v) => {
                let mut out = Vec::new();
                for elem in v {
                    out.extend(elem.bytes());
                }
                out.extend(Tag::End.with_name("").bytes());
                out
            }
            Tag::IntArray(v) => {
                let n = v.len();
                let mut out = Vec::with_capacity(4 + (4 * n));
                let n_bytes = write_int(n as u32);
                out.extend_from_slice(&n_bytes);
                for value in v {
                    let bytes = write_int(*value as u32);
                    out.extend_from_slice(&bytes);
                }
                out
            }
            Tag::LongArray(v) => {
                let n = v.len();
                let mut out = Vec::with_capacity(4 + (8 * n));
                let n_bytes = write_int(n as u32);
                out.extend_from_slice(&n_bytes);
                for value in v {
                    let bytes = write_long(*value as u64);
                    out.extend_from_slice(&bytes);
                }
                out
            }
            Tag::End => Vec::default(),
        }
    }
}

// test
#[cfg(test)]
mod tests {
    use super::*;
    use flate2::read::GzDecoder;
    use std::fs::File;
    use std::io::Read;

    #[test]
    fn test_read_bignbt_example() {
        let actual = read_bigtest();

        let expected = Tag::Compound(vec!(
            Tag::Long(9223372036854775807).with_name("longTest"),
            Tag::Short(32767).with_name("shortTest"),
            Tag::String("HELLO WORLD THIS IS A TEST STRING ÅÄÖ!".into()).with_name("stringTest"),
            Tag::Float(0.49823147).with_name("floatTest"),
            Tag::Int(2147483647).with_name("intTest"),
            Tag::Compound(vec!(
                Tag::Compound(vec!(
                    Tag::String("Hampus".into()).with_name("name"),
                    Tag::Float(0.75).with_name("value"),
                )).with_name("ham"),
                Tag::Compound(vec!(
                    Tag::String("Eggbert".into()).with_name("name"),
                    Tag::Float(0.5).with_name("value"),
                )).with_name("egg")
            )).with_name("nested compound test"),
            Tag::List(vec!(
                Tag::Long(11),
                Tag::Long(12),
                Tag::Long(13),
                Tag::Long(14),
                Tag::Long(15),
            )).with_name("listTest (long)"),
            Tag::List(vec!(
                Tag::Compound(vec!(
                    Tag::String("Compound tag #0".into()).with_name("name"),
                    Tag::Long(1264099775885).with_name("created-on"),
                )),
                Tag::Compound(vec!(
                    Tag::String("Compound tag #1".into()).with_name("name"),
                    Tag::Long(1264099775885).with_name("created-on"),
                ))
            )).with_name("listTest (compound)"),
            Tag::Byte(127).with_name("byteTest"),
            Tag::ByteArray(bigtest_generate_byte_array()).with_name("byteArrayTest (the first 1000 values of (n*n*255+n*7)%100, starting with n=0 (0, 62, 34, 16, 8, ...))"),
            Tag::Double(0.4931287132182315).with_name("doubleTest")
        )).with_name("Level");

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_serialize_bigtest() {
        let (unzipped, result) = read_bigtest_with_bytes();
        let serialized = result.bytes();
        assert_eq!(unzipped, serialized);
        let Deserialized {
            value: unserialized,
            data: _,
        } = NamedTag::root_compound_tag_from_bytes(serialized.as_slice())
            .expect("deserialize serialized nbt");
        assert_eq!(unserialized, result);
    }

    #[test]
    fn test_int_array() {
        let original = Tag::Compound(vec![Tag::IntArray(vec![
            1, 2, -5, 123127, -12373, 0, 0, 4, 2,
        ])
        .with_name("test ints")])
        .with_name("test");

        let bytes = original.bytes();
        let Deserialized {
            value: unserialized,
            data: _,
        } = NamedTag::root_compound_tag_from_bytes(bytes.as_slice())
            .expect("deserialize int array");
        assert_eq!(original, unserialized);
    }

    #[test]
    fn test_long_array() {
        let original = Tag::Compound(vec![Tag::LongArray(vec![
            1,
            2,
            -5,
            123127999999,
            -1237399999,
            0,
            0,
            4,
            2,
        ])
        .with_name("test ints")])
        .with_name("test");

        let bytes = original.bytes();
        let Deserialized {
            value: unserialized,
            data: _,
        } = NamedTag::root_compound_tag_from_bytes(bytes.as_slice())
            .expect("deserialize int array");
        assert_eq!(original, unserialized);
    }

    #[test]
    fn test_display() {
        println!("{}", read_bigtest());
    }

    #[test]
    fn test_debug() {
        println!("{:?}", read_bigtest());
    }

    fn read_bigtest_with_bytes() -> (Vec<u8>, NamedTag) {
        let unzipped = read_compressed_file("src/testdata/bigtest.nbt").expect("read nbt data");
        let Deserialized {
            value: result,
            data: rest,
        } = NamedTag::root_compound_tag_from_bytes(unzipped.as_slice()).expect("deserialize nbt");
        assert_eq!(rest.len(), 0);

        (unzipped, result)
    }

    fn read_bigtest() -> NamedTag {
        let (_, result) = read_bigtest_with_bytes();
        result
    }

    fn bigtest_generate_byte_array() -> Vec<u8> {
        const COUNT: usize = 1000;
        let mut out = Vec::with_capacity(COUNT);
        for i in 0..COUNT {
            out.push((((i * i * 255) + (i * 7)) % 100) as u8);
        }
        out
    }

    fn read_compressed_file(at: &str) -> std::io::Result<Vec<u8>> {
        let file = File::open(at)?;
        let mut gz = GzDecoder::new(file);
        let mut out = Vec::new();
        gz.read_to_end(&mut out)?;
        Ok(out)
    }
}
