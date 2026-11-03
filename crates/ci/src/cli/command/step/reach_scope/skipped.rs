use std::fmt;

use text_lines::Lines;

#[derive(Debug, PartialEq, Eq)]
pub struct Skipped(String);

#[derive(Debug, PartialEq, Eq)]
pub struct NotSkipped;

#[derive(Debug, PartialEq, Eq)]
pub struct Skips(Vec<Skipped>);

impl Skipped {
    const EMPTY: char = '=';
}

impl TryFrom<&str> for Skipped {
    type Error = NotSkipped;

    fn try_from(line: &str) -> Result<Skipped, NotSkipped> {
        line.char_indices()
            .last()
            .filter(|(_, last)| *last == Self::EMPTY)
            .and_then(|(at, _)| line.get(..at))
            .map(|name| Skipped(name.to_owned()))
            .ok_or(NotSkipped)
    }
}

impl fmt::Display for Skipped {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "skipped: {} is empty, so the steps that use it do not run", self.0)
    }
}

impl From<&str> for Skips {
    fn from(report: &str) -> Skips {
        Skips(Lines::from(report).filter_map(|line| Skipped::try_from(line).ok()).collect())
    }
}

impl fmt::Display for Skips {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.iter().try_for_each(|skipped| writeln!(formatter, "{skipped}"))
    }
}
