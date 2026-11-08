use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sequel {
    WentOn,
    NoMoreJobs,
}

impl fmt::Display for Sequel {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Sequel::WentOn => write!(formatter, "structure went on with the next job"),
            Sequel::NoMoreJobs => write!(formatter, "there were no more jobs"),
        }
    }
}
