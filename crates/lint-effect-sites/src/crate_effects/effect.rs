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
        formatter.write_str(match self {
            Effect::Files => "files",
            Effect::Environment => "environment",
            Effect::Clock => "clock",
            Effect::Network => "network",
            Effect::Processes => "processes",
            Effect::Randomness => "randomness",
            Effect::Stdio => "stdio",
        })
    }
}
