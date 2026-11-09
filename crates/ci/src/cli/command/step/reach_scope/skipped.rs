use std::fmt;
use std::str::FromStr;

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

impl FromStr for Skipped {
    type Err = NotSkipped;

    fn from_str(line: &str) -> Result<Skipped, NotSkipped> {
        line.char_indices()
            .last()
            .filter(|(_, last)| *last == Self::EMPTY)
            .and_then(|(at, _)| line.get(..at))
            .map(|name| Skipped(name.to_owned()))
            .ok_or(NotSkipped)
    }
}

impl From<&str> for Skips {
    fn from(report: &str) -> Skips {
        Skips(report.parse::<Lines>().into_iter().flatten().filter_map(|line| line.parse::<Skipped>().ok()).collect())
    }
}

impl fmt::Display for Skips {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.iter().try_for_each(|Skipped(scope)| {
            writeln!(formatter, "skipped: {scope} is empty, so the steps that use it do not run")
        })
    }
}
