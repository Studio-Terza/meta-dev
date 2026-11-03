use std::fmt;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Caught,
    Missed,
    Timeout,
    Unviable,
}

impl Outcome {
    pub(crate) const ALL: [Outcome; 4] = [Outcome::Caught, Outcome::Missed, Outcome::Timeout, Outcome::Unviable];
    pub(crate) const LISTED: [Outcome; 2] = [Outcome::Missed, Outcome::Timeout];

    const EXTENSION: &'static str = ".txt";

    pub(crate) fn file_name(self) -> PathBuf {
        PathBuf::from([self.to_string().as_str(), Self::EXTENSION].concat())
    }
}

impl fmt::Display for Outcome {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Outcome::Caught => formatter.write_str("caught"),
            Outcome::Missed => formatter.write_str("missed"),
            Outcome::Timeout => formatter.write_str("timeout"),
            Outcome::Unviable => formatter.write_str("unviable"),
        }
    }
}
