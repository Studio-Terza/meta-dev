pub trait Validity {
    type Kind;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Checked {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unchecked {}
