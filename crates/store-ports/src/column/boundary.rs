use std::error::Error;
use std::fmt::Display;
use std::num::TryFromIntError;
use std::str::FromStr;

use newtype::{Inner, Newtype, Optional, Spelled};

use super::Column;

#[derive(Debug, thiserror::Error)]
pub enum ColumnFailure {
    #[error("a stored number does not fit its column's type: {0}")]
    Unfit(#[source] TryFromIntError),
    #[error("a stored spelling names nothing of its column's type: {0}")]
    Misspelled(#[source] Box<dyn Error + Send + Sync>),
    #[error("a stored column its row needs is null")]
    Absent,
}

pub trait Encoding<Value> {
    type Raw;

    fn raw(value: &Value) -> Self::Raw;

    fn from_raw(raw: Self::Raw) -> Result<Value, ColumnFailure>;
}

pub trait Primitive: Sized {
    type Raw;

    fn into_raw(self) -> Self::Raw;

    fn from_raw(raw: Self::Raw) -> Result<Self, ColumnFailure>;
}

impl<Value: Newtype<Inner: Primitive> + Clone> Encoding<Value> for Inner {
    type Raw = <Value::Inner as Primitive>::Raw;

    fn raw(value: &Value) -> Self::Raw {
        value.clone().into_inner().into_raw()
    }

    fn from_raw(raw: Self::Raw) -> Result<Value, ColumnFailure> {
        Value::Inner::from_raw(raw).map(Value::new)
    }
}

impl<Value: Display + FromStr<Err: Error + Send + Sync + 'static>> Encoding<Value> for Spelled {
    type Raw = String;

    fn raw(value: &Value) -> String {
        value.to_string()
    }

    fn from_raw(raw: String) -> Result<Value, ColumnFailure> {
        raw.parse().map_err(|unknown| ColumnFailure::Misspelled(Box::new(unknown)))
    }
}

impl<Value: Column> Encoding<Option<Value>> for Optional {
    type Raw = Option<Value::Raw>;

    fn raw(value: &Option<Value>) -> Option<Value::Raw> {
        value.as_ref().map(Value::raw)
    }

    fn from_raw(raw: Option<Value::Raw>) -> Result<Option<Value>, ColumnFailure> {
        raw.map(Value::from_raw).transpose()
    }
}

impl Primitive for i64 {
    type Raw = i64;

    fn into_raw(self) -> i64 {
        self
    }

    fn from_raw(raw: i64) -> Result<i64, ColumnFailure> {
        Ok(raw)
    }
}

impl Primitive for f64 {
    type Raw = f64;

    fn into_raw(self) -> f64 {
        self
    }

    fn from_raw(raw: f64) -> Result<f64, ColumnFailure> {
        Ok(raw)
    }
}

impl Primitive for String {
    type Raw = String;

    fn into_raw(self) -> String {
        self
    }

    fn from_raw(raw: String) -> Result<String, ColumnFailure> {
        Ok(raw)
    }
}

impl Primitive for Vec<u8> {
    type Raw = Vec<u8>;

    fn into_raw(self) -> Vec<u8> {
        self
    }

    fn from_raw(raw: Vec<u8>) -> Result<Vec<u8>, ColumnFailure> {
        Ok(raw)
    }
}

impl Primitive for usize {
    type Raw = i64;

    fn into_raw(self) -> i64 {
        i64::try_from(self).unwrap_or(i64::MAX)
    }

    fn from_raw(raw: i64) -> Result<usize, ColumnFailure> {
        usize::try_from(raw).map_err(ColumnFailure::Unfit)
    }
}

impl Primitive for u64 {
    type Raw = i64;

    fn into_raw(self) -> i64 {
        i64::try_from(self).unwrap_or(i64::MAX)
    }

    fn from_raw(raw: i64) -> Result<u64, ColumnFailure> {
        u64::try_from(raw).map_err(ColumnFailure::Unfit)
    }
}

impl Primitive for u32 {
    type Raw = i64;

    fn into_raw(self) -> i64 {
        i64::from(self)
    }

    fn from_raw(raw: i64) -> Result<u32, ColumnFailure> {
        u32::try_from(raw).map_err(ColumnFailure::Unfit)
    }
}
