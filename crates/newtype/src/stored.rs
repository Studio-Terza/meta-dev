pub trait Stored {
    type Kind;
}

#[derive(Debug)]
pub enum Inner {}

#[derive(Debug)]
pub enum Validated {}

#[derive(Debug)]
pub enum Spelled {}
