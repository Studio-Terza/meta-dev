#[doc(hidden)]
pub trait Stored {
    type Kind;
}

#[doc(hidden)]
#[derive(Debug)]
pub enum Inner {}

#[doc(hidden)]
#[derive(Debug)]
pub enum Spelled {}

#[doc(hidden)]
#[derive(Debug)]
pub enum Optional {}

impl<Value: Stored> Stored for Option<Value> {
    type Kind = Optional;
}
