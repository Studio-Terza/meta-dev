use std::fmt;
use std::str::FromStr;

use grammar::assignment::Emptied;
use newtype::Wrapping;
use text_lines::Lines;

#[derive(Debug, PartialEq, Eq)]
pub struct Skipped(String);

#[derive(Debug, PartialEq, Eq)]
pub struct NotSkipped;

#[derive(Debug, PartialEq, Eq)]
pub struct Skips(Vec<Skipped>);

impl FromStr for Skipped {
    type Err = NotSkipped;

    fn from_str(line: &str) -> Result<Skipped, NotSkipped> {
        Emptied::try_from(line)
            .map(|emptied| Skipped(emptied.into_inner().to_owned()))
            .map_err(|_unassigned| NotSkipped)
    }
}

impl From<&str> for Skips {
    fn from(report: &str) -> Skips {
        Skips(Lines::from(report).filter_map(|line| line.parse::<Skipped>().ok()).collect())
    }
}

impl fmt::Display for Skips {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.iter().try_for_each(|Skipped(scope)| {
            writeln!(formatter, "skipped: {scope} is empty, so the steps that use it do not run")
        })
    }
}
