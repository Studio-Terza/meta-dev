pub trait Newtype: Sized {
    type Inner;

    fn new(inner: Self::Inner) -> Self;

    fn into_inner(self) -> Self::Inner;
}
