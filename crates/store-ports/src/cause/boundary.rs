use std::fmt;

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct Cause(String);

impl Cause {
    #[must_use]
    pub fn told(failure: &impl fmt::Display) -> Cause {
        Cause(failure.to_string())
    }
}
