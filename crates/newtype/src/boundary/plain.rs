pub trait Wrapping: Sized {
    type Inner;

    fn into_inner(self) -> Self::Inner;
}

pub trait Newtype: Wrapping {
    fn new(inner: Self::Inner) -> Self;
}
