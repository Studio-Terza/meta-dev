pub mod kind;

use kind::Validity;

pub trait Wrapping: Sized {
    type Inner;

    fn into_inner(self) -> Self::Inner;
}

pub trait Newtype: Wrapping + Validity<Kind = kind::Unchecked> {
    fn new(inner: Self::Inner) -> Self;
}
