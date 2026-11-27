use std::ops::Not;

use serde::de::{DeserializeOwned, Error};
use serde_json::Value;

#[derive(Clone, Copy, Debug)]
pub struct Object<'value>(&'value Value);

impl<'value> Object<'value> {
    pub const TYPE: &'static str = "type";
    const ABSENT: Value = Value::Null;

    pub fn new(value: &'value Value) -> Object<'value> {
        Object(value)
    }

    pub fn tag(self, key: &str) -> Option<&'value str> {
        self.0.get(key).and_then(Value::as_str)
    }

    pub fn present(self, name: &str) -> Option<Object<'value>> {
        self.0.get(name).filter(|value| value.is_null().not()).map(Object)
    }

    pub fn required(self, name: &'static str) -> Result<Object<'value>, serde_json::Error> {
        self.0.get(name).map(Object).ok_or_else(|| serde_json::Error::missing_field(name))
    }

    pub fn field<T: DeserializeOwned>(self, name: &str) -> Result<T, serde_json::Error> {
        T::deserialize(self.0.get(name).unwrap_or(&Self::ABSENT))
    }

    pub fn whole<T: DeserializeOwned>(self) -> Result<T, serde_json::Error> {
        T::deserialize(self.0)
    }

    pub fn each(self) -> Result<Vec<Object<'value>>, serde_json::Error> {
        self.0
            .as_array()
            .map(|values| values.iter().map(Object).collect())
            .ok_or_else(|| serde_json::Error::custom("expected an array"))
    }
}
