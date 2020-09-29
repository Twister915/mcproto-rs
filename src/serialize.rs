#[derive(Debug)]
pub enum SerializeErr {
    FailedJsonEncode(String),
    InconsistentPlayerActions(String)
}

pub type SerializeResult = Result<(), SerializeErr>;

pub trait Serialize: Sized {
    fn mc_serialize<S: Serializer>(&self, to: &mut S) -> SerializeResult;
}

pub trait Serializer: Sized {

    fn serialize_bytes(&mut self, data: &[u8]) -> SerializeResult;

    fn serialize_byte(&mut self, byte: u8) -> SerializeResult {
        self.serialize_bytes(vec!(byte).as_slice())
    }

    fn serialize_other<S: Serialize>(&mut self, other: &S) -> SerializeResult {
        other.mc_serialize(self)
    }
}