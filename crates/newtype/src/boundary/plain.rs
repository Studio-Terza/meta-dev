use super::checked::Checked;
use super::validity::Validity;
use crate::kind;

pub trait Wrapping: Sized {
    type Inner;

    fn into_inner(self) -> Self::Inner;

    fn admitted(inner: Self::Inner, built: impl FnOnce(Self::Inner) -> Self) -> Result<Self, Self::Error>
    where
        Self: Checked + Validity<Kind = kind::Checked>,
    {
        Self::checked(&inner)?;
        Ok(built(inner))
    }
}

pub trait Newtype: Wrapping + Validity<Kind = kind::Unchecked> {
    fn new(inner: Self::Inner) -> Self;
}
