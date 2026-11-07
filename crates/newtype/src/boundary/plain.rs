use super::checked::Checked;

pub trait Wrapping: Sized {
    type Inner;

    fn into_inner(self) -> Self::Inner;

    fn admitted(inner: Self::Inner, built: impl FnOnce(Self::Inner) -> Self) -> Result<Self, Self::Error>
    where
        Self: Checked,
    {
        Self::checked(&inner)?;
        Ok(built(inner))
    }
}

pub trait Newtype: Wrapping {
    fn new(inner: Self::Inner) -> Self;
}
