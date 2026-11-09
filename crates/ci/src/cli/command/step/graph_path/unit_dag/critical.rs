use std::fmt;

use super::super::weighed::Weighed;
use crate::cli::command::step::seconds::Seconds;

#[derive(Debug, Default)]
pub(in super::super) struct Critical {
    seconds: Seconds,
    units: Vec<Weighed>,
}

impl Critical {
    pub(super) fn new(seconds: Seconds, units: Vec<Weighed>) -> Critical {
        Critical { seconds, units }
    }
}

impl AsRef<Seconds> for Critical {
    fn as_ref(&self) -> &Seconds {
        &self.seconds
    }
}

impl fmt::Display for Critical {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            formatter,
            "graph-path {} seconds, the longest chain of clippy units, counting only units sccache never caches:",
            self.seconds
        )?;
        self.units.iter().try_for_each(|unit| writeln!(formatter, "{unit}"))
    }
}
