use std::ops::Not;

use serde::de::Error;
use serde_json::Value;

use crate::{Decode, JsonFailure};

#[derive(Clone, Debug)]
pub struct Document(Value);

#[derive(Clone, Copy, Debug)]
pub struct Object<'value>(&'value Value);

impl Document {
    pub fn read(text: &str) -> Result<Document, JsonFailure> {
        Ok(Document(serde_json::from_str(text)?))
    }

    #[must_use]
    pub fn object(&self) -> Object<'_> {
        Object(&self.0)
    }
}

impl<'value> Object<'value> {
    pub const TYPE: &'static str = "type";
    const ABSENT: Value = Value::Null;

    #[must_use]
    pub fn tag(self, key: &str) -> Option<&'value str> {
        self.0.get(key).and_then(Value::as_str)
    }

    #[must_use]
    pub fn present(self, name: &str) -> Option<Object<'value>> {
        self.0.get(name).filter(|value| value.is_null().not()).map(Object)
    }

    pub fn required(self, name: &'static str) -> Result<Object<'value>, JsonFailure> {
        self.0.get(name).map(Object).ok_or_else(|| JsonFailure::from(serde_json::Error::missing_field(name)))
    }

    pub fn field<Read: Decode>(self, name: &str) -> Result<Read, JsonFailure> {
        Ok(Read::deserialize(self.0.get(name).unwrap_or(&Self::ABSENT))?)
    }

    pub fn whole<Read: Decode>(self) -> Result<Read, JsonFailure> {
        Ok(Read::deserialize(self.0)?)
    }

    pub fn each(self) -> Result<Vec<Object<'value>>, JsonFailure> {
        self.0
            .as_array()
            .map(|values| values.iter().map(Object).collect())
            .ok_or_else(|| JsonFailure::from(serde_json::Error::custom("expected an array")))
    }
}
