use std::fmt;

use design_model::JobId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sequel {
    WentOn,
    NoMoreJobs,
    Halted { job: JobId },
}

impl fmt::Display for Sequel {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Sequel::WentOn => write!(formatter, "structure went on with the next job"),
            Sequel::NoMoreJobs => write!(formatter, "there were no more jobs"),
            Sequel::Halted { job } => {
                write!(formatter, "structure went on, then halted when job {job} stalled on a transient cause")
            }
        }
    }
}
