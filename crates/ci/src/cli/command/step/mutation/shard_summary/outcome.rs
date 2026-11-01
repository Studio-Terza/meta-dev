use std::fmt;

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

    pub(crate) fn file_name(self) -> String {
        [self.to_string().as_str(), Self::EXTENSION].concat()
    }
}

impl fmt::Display for Outcome {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Outcome::Caught => "caught",
            Outcome::Missed => "missed",
            Outcome::Timeout => "timeout",
            Outcome::Unviable => "unviable",
        })
    }
}
