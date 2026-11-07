use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) enum Effect {
    Files,
    Environment,
    Clock,
    Network,
    Processes,
    Randomness,
    Stdio,
}

impl fmt::Display for Effect {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Files => formatter.write_str("files"),
            Self::Environment => formatter.write_str("environment"),
            Self::Clock => formatter.write_str("clock"),
            Self::Network => formatter.write_str("network"),
            Self::Processes => formatter.write_str("processes"),
            Self::Randomness => formatter.write_str("randomness"),
            Self::Stdio => formatter.write_str("stdio"),
        }
    }
}
