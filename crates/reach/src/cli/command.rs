use std::str::FromStr;

use crate::cli::failure::Failure;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    Affected,
    Widened,
    History,
    Sources,
    Beyond,
}

impl FromStr for Command {
    type Err = Failure;

    fn from_str(name: &str) -> Result<Self, Failure> {
        Self::NAMED.into_iter().find(|(known, _)| *known == name).map(|(_, command)| command).ok_or(Failure::Usage)
    }
}

impl Command {
    const NAMED: [(&'static str, Self); 5] = [
        ("affected", Self::Affected),
        ("widened", Self::Widened),
        ("history", Self::History),
        ("sources", Self::Sources),
        ("beyond", Self::Beyond),
    ];
}
