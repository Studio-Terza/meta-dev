pub trait Stored {
    type Kind;
}

#[derive(Debug)]
pub enum Inner {}

#[derive(Debug)]
pub enum Validated {}

#[derive(Debug)]
pub enum Spelled {}

#[derive(Debug)]
pub enum Optional {}

impl<Value: Stored> Stored for Option<Value> {
    type Kind = Optional;
}
