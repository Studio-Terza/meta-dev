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

    fn from_str(name: &str) -> Result<Command, Failure> {
        Self::NAMED.into_iter().find(|(known, _)| *known == name).map(|(_, command)| command).ok_or(Failure::Usage)
    }
}

impl Command {
    const NAMED: [(&'static str, Command); 5] = [
        ("affected", Command::Affected),
        ("widened", Command::Widened),
        ("history", Command::History),
        ("sources", Command::Sources),
        ("beyond", Command::Beyond),
    ];
}
