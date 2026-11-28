use invariants::Ignoring;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Map, Value};

use crate::Decode;

invariants::ignored! {
    enum Ignored {
        Unlenient => "a value that does not decode as the lenient type reads as absent",
    }
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Opaque(Value);

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Fields(Map<String, Value>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lenient<Read>(Option<Read>);

impl<Read> Lenient<Read> {
    pub fn read(&self) -> Option<&Read> {
        self.0.as_ref()
    }
}

impl<'arriving, Read: Decode> Deserialize<'arriving> for Lenient<Read> {
    fn deserialize<Arriving: Deserializer<'arriving>>(arriving: Arriving) -> Result<Lenient<Read>, Arriving::Error> {
        let value = Value::deserialize(arriving)?;
        Ok(Lenient(Read::deserialize(value).ignoring(Ignored::Unlenient)))
    }
}
