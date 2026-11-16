mod boundary;

pub use boundary::{Column, ColumnFailure};

pub trait Columns: Sized {
    type Raw;

    fn from_raw(raw: Self::Raw) -> Result<Self, ColumnFailure>;
}
