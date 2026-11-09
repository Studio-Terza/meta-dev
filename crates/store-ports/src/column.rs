mod boundary;

use newtype::Stored;

pub use boundary::ColumnFailure;
use boundary::Encoding;

pub trait Column: Sized {
    type Raw;

    fn raw(&self) -> Self::Raw;

    fn from_raw(raw: Self::Raw) -> Result<Self, ColumnFailure>;
}

pub trait Columns: Sized {
    type Raw;

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
