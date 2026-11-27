use std::error::Error;
use std::fmt::Display;
use std::num::{NonZeroU32, TryFromIntError};
use std::str::FromStr;

use newtype::{Guarded, Inner, Listed, Newtype, Optional, Spelled, Stored, Validated};

use crate::store::Cause;

#[derive(Debug, thiserror::Error)]
pub enum ColumnFailure {
    #[error("a stored number does not fit its column's type: {0}")]
    Unfit(#[source] TryFromIntError),
    #[error("a stored spelling names nothing of its column's type: {0}")]
    Misspelled(#[source] Cause),
    #[error("a stored column its row needs is null")]
    Absent,
    #[error("a stored value is not of its column's storage type")]
    Mistyped,
    #[error("a stored value fails its column type's check: {0}")]
    Refused(#[source] Cause),
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

impl<Value> Encoding<Value> for Validated
where
    Value: Guarded<Inner: Primitive, Error: Error + Send + Sync + 'static> + Clone,
{
    type Raw = <Value::Inner as Primitive>::Raw;

    fn raw(value: &Value) -> Self::Raw {
        value.clone().into_inner().into_raw()
    }

    fn from_raw(raw: Self::Raw) -> Result<Value, ColumnFailure> {
        Value::Inner::from_raw(raw)
            .and_then(|inner| Value::try_new(inner).map_err(|refused| ColumnFailure::Refused(Cause::told(&refused))))
    }
}

impl<Value: Display + FromStr<Err: Error + Send + Sync + 'static>> Encoding<Value> for Spelled {
    type Raw = String;

    fn raw(value: &Value) -> String {
        value.to_string()
    }

    fn from_raw(raw: String) -> Result<Value, ColumnFailure> {
        raw.parse().map_err(|unknown| ColumnFailure::Misspelled(Cause::told(&unknown)))
    }
}

impl<Value: Display + FromStr<Err: Error + Send + Sync + 'static>> Encoding<Value> for Listed {
    type Raw = String;

    fn raw(value: &Value) -> String {
        <Spelled as Encoding<Value>>::raw(value)
    }

    fn from_raw(raw: String) -> Result<Value, ColumnFailure> {
        <Spelled as Encoding<Value>>::from_raw(raw)
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

impl Primitive for String {
    type Raw = String;

    fn into_raw(self) -> String {
        self
    }

    fn from_raw(raw: String) -> Result<String, ColumnFailure> {
        Ok(raw)
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

impl Primitive for NonZeroU32 {
    type Raw = i64;

    fn into_raw(self) -> i64 {
        i64::from(self.get())
    }

    fn from_raw(raw: i64) -> Result<NonZeroU32, ColumnFailure> {
        u32::try_from(raw).and_then(NonZeroU32::try_from).map_err(ColumnFailure::Unfit)
    }
}

pub trait Column: Sized {
    type Raw;

    fn raw(&self) -> Self::Raw;

    fn from_raw(raw: Self::Raw) -> Result<Self, ColumnFailure>;
}

impl<Value: Stored<Kind: Encoding<Value>>> Column for Value {
    type Raw = <Value::Kind as Encoding<Value>>::Raw;

    fn raw(&self) -> Self::Raw {
        Value::Kind::raw(self)
    }

    fn from_raw(raw: Self::Raw) -> Result<Value, ColumnFailure> {
        Value::Kind::from_raw(raw)
    }
}
