use crate::{ColumnFailure, StoreFailure};

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Null,
    Integer(i64),
    Real(f64),
    Text(String),
    Blob(Vec<u8>),
}

pub trait Bind {
    fn null(&mut self) -> Result<(), StoreFailure>;

    fn integer(&mut self, integer: i64) -> Result<(), StoreFailure>;

    fn real(&mut self, real: f64) -> Result<(), StoreFailure>;

    fn text(&mut self, text: String) -> Result<(), StoreFailure>;

    fn blob(&mut self, blob: Vec<u8>) -> Result<(), StoreFailure>;
}

pub trait IntoValue {
    fn into_value(self) -> Value;
}

pub trait FromValue: Sized {
    fn from_value(value: Value) -> Result<Self, ColumnFailure>;
}

impl Value {
    pub fn bound(self, bind: &mut impl Bind) -> Result<(), StoreFailure> {
        match self {
            Value::Null => bind.null(),
            Value::Integer(integer) => bind.integer(integer),
            Value::Real(real) => bind.real(real),
            Value::Text(text) => bind.text(text),
            Value::Blob(blob) => bind.blob(blob),
        }
    }
}

impl IntoValue for i64 {
    fn into_value(self) -> Value {
        Value::Integer(self)
    }
}

impl IntoValue for f64 {
    fn into_value(self) -> Value {
        Value::Real(self)
    }
}

impl IntoValue for String {
    fn into_value(self) -> Value {
        Value::Text(self)
    }
}

impl IntoValue for Vec<u8> {
    fn into_value(self) -> Value {
        Value::Blob(self)
    }
}

impl<Inner: IntoValue> IntoValue for Option<Inner> {
    fn into_value(self) -> Value {
        self.map_or(Value::Null, IntoValue::into_value)
    }
}

impl FromValue for i64 {
    fn from_value(value: Value) -> Result<i64, ColumnFailure> {
        match value {
            Value::Integer(integer) => Ok(integer),
            Value::Null => Err(ColumnFailure::Absent),
            Value::Real(_) | Value::Text(_) | Value::Blob(_) => Err(ColumnFailure::Mistyped),
        }
    }
}

impl FromValue for f64 {
    fn from_value(value: Value) -> Result<f64, ColumnFailure> {
        match value {
            Value::Real(real) => Ok(real),
            Value::Null => Err(ColumnFailure::Absent),
            Value::Integer(_) | Value::Text(_) | Value::Blob(_) => Err(ColumnFailure::Mistyped),
        }
    }
}

impl FromValue for String {
    fn from_value(value: Value) -> Result<String, ColumnFailure> {
        match value {
            Value::Text(text) => Ok(text),
            Value::Null => Err(ColumnFailure::Absent),
            Value::Integer(_) | Value::Real(_) | Value::Blob(_) => Err(ColumnFailure::Mistyped),
        }
    }
}

impl FromValue for Vec<u8> {
    fn from_value(value: Value) -> Result<Vec<u8>, ColumnFailure> {
        match value {
            Value::Blob(blob) => Ok(blob),
            Value::Null => Err(ColumnFailure::Absent),
            Value::Integer(_) | Value::Real(_) | Value::Text(_) => Err(ColumnFailure::Mistyped),
        }
    }
}

impl<Inner: FromValue> FromValue for Option<Inner> {
    fn from_value(value: Value) -> Result<Option<Inner>, ColumnFailure> {
        match value {
            Value::Null => Ok(None),
            Value::Integer(_) | Value::Real(_) | Value::Text(_) | Value::Blob(_) => Inner::from_value(value).map(Some),
        }
    }
}
