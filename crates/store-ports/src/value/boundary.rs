use crate::{ColumnFailure, StoreFailure};

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Null,
    Integer(i64),
    Real(f64),
    Text(String),
    Blob(Vec<u8>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Argument {
    Null,
    Integer(i64),
    Text(String),
    Blob(Vec<u8>),
}

pub trait Bind {
    fn null(&mut self) -> Result<(), StoreFailure>;

    fn integer(&mut self, integer: i64) -> Result<(), StoreFailure>;

    fn text(&mut self, text: String) -> Result<(), StoreFailure>;

    fn blob(&mut self, blob: Vec<u8>) -> Result<(), StoreFailure>;
}

pub trait IntoArgument {
    fn into_argument(self) -> Argument;
}

pub trait FromValue: Sized {
    fn from_value(value: Value) -> Result<Self, ColumnFailure>;
}

impl Argument {
    pub fn bound(self, bind: &mut impl Bind) -> Result<(), StoreFailure> {
        match self {
            Argument::Null => bind.null(),
            Argument::Integer(integer) => bind.integer(integer),
            Argument::Text(text) => bind.text(text),
            Argument::Blob(blob) => bind.blob(blob),
        }
    }
}

impl IntoArgument for i64 {
    fn into_argument(self) -> Argument {
        Argument::Integer(self)
    }
}

impl IntoArgument for String {
    fn into_argument(self) -> Argument {
        Argument::Text(self)
    }
}

impl IntoArgument for Vec<u8> {
    fn into_argument(self) -> Argument {
        Argument::Blob(self)
    }
}

impl<Inner: IntoArgument> IntoArgument for Option<Inner> {
    fn into_argument(self) -> Argument {
        self.map_or(Argument::Null, IntoArgument::into_argument)
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
