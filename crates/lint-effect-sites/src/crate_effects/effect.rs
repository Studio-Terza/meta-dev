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
            Effect::Files => formatter.write_str("files"),
            Effect::Environment => formatter.write_str("environment"),
            Effect::Clock => formatter.write_str("clock"),
            Effect::Network => formatter.write_str("network"),
            Effect::Processes => formatter.write_str("processes"),
            Effect::Randomness => formatter.write_str("randomness"),
            Effect::Stdio => formatter.write_str("stdio"),
        }
    }
}
