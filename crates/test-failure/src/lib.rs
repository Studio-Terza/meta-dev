use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub struct Failed(String);

impl<Failure: fmt::Display> From<Failure> for Failed {
    fn from(failure: Failure) -> Failed {
        Failed(failure.to_string())
    }
}
