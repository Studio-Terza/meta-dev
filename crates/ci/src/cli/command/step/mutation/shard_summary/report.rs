use std::fmt;

use super::tally::Tally;

#[derive(Debug)]
pub struct Report {
    tallies: Vec<Tally>,
    names: Vec<String>,
}

impl Report {
    const FENCE: &'static str = "```";

    pub(crate) fn new(tallies: Vec<Tally>, names: Vec<String>) -> Report {
        Report { tallies, names }
    }
}

impl fmt::Display for Report {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.tallies.iter().try_for_each(|tally| writeln!(formatter, "{tally}"))?;
        writeln!(formatter, "{}", Self::FENCE)?;
        self.names.iter().try_for_each(|name| writeln!(formatter, "{name}"))?;
        writeln!(formatter, "{}", Self::FENCE)
    }
}
