use super::plain::Wrapping;

pub trait Checked: Wrapping {
    type Error;

    fn checked(inner: &Self::Inner) -> Result<&Self::Inner, Self::Error>;
}
