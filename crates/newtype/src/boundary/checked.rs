use std::fmt::Display;

use super::plain::Wrapping;
use super::plain::kind::{self, Validity};

pub trait Checked: Wrapping {
    type Error: Display;

    fn checked(inner: &Self::Inner) -> Result<&Self::Inner, Self::Error>;
}

pub trait Guarded: Checked + Validity<Kind = kind::Checked> {
    fn try_new(inner: Self::Inner) -> Result<Self, Self::Error>;
}
