use std::fmt;

use crate::seconds::Seconds;
use crate::stage::Stage;

#[derive(Debug, PartialEq)]
pub struct Entry {
    stage: Stage,
    seconds: Seconds,
}

impl Entry {
    pub fn new(stage: Stage, seconds: Seconds) -> Entry {
        Entry { stage, seconds }
    }
}

impl fmt::Display for Entry {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "{} = {}", self.stage, self.seconds.recorded())
    }
}
