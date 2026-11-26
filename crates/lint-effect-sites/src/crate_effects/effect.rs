use strum::Display;

#[derive(Display, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[strum(serialize_all = "snake_case")]
pub(super) enum Effect {
    Files,
    Environment,
    Clock,
    Network,
    Processes,
    Randomness,
    Stdio,
}
