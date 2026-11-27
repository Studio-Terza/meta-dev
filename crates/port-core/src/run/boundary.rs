use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub(crate) struct Told(String);

impl<Failure: Error> From<&Failure> for Told {
    fn from(failure: &Failure) -> Told {
        Told(failure.to_string())
    }
}

impl fmt::Display for Told {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
