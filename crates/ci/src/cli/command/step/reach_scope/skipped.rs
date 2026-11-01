use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub struct Skipped(String);

#[derive(Debug, PartialEq, Eq)]
pub struct NotSkipped;

impl Skipped {
    const EMPTY: char = '=';
}

impl TryFrom<&str> for Skipped {
    type Error = NotSkipped;

    fn try_from(line: &str) -> Result<Skipped, NotSkipped> {
        line.char_indices()
            .last()
            .filter(|(at, last)| *last == Self::EMPTY && *at > 0)
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
