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
    pub(crate) const ALL: [Self; 4] = [Self::Caught, Self::Missed, Self::Timeout, Self::Unviable];
    pub(crate) const LISTED: [Self; 2] = [Self::Missed, Self::Timeout];

    const EXTENSION: &'static str = ".txt";

    pub(crate) fn file_name(self) -> PathBuf {
        PathBuf::from([self.to_string().as_str(), Self::EXTENSION].concat())
    }
}

impl fmt::Display for Outcome {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Caught => formatter.write_str("caught"),
            Self::Missed => formatter.write_str("missed"),
            Self::Timeout => formatter.write_str("timeout"),
            Self::Unviable => formatter.write_str("unviable"),
        }
    }
}
