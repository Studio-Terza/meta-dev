pub trait Validated: Sized {
    type Inner;
    type Error;

    fn try_new(inner: Self::Inner) -> Result<Self, Self::Error>;

    fn into_inner(self) -> Self::Inner;
}
