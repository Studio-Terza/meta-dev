use std::fmt::Display;

use super::plain::Wrapping;

pub trait Checked: Wrapping {
    type Error: Display;

    fn checked(inner: &Self::Inner) -> Result<&Self::Inner, Self::Error>;
}
