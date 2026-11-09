use serde::Deserialize;

use super::crates::Crates;
use super::package::Package;
use super::role::Role;
use super::unit_key::UnitKey;
use super::weighed::Weighed;
use crate::cli::command::step::seconds::Seconds;
use text_lines::OneLine;

#[derive(Debug, Deserialize)]
pub(super) struct TimedUnit {
    name: OneLine,
    version: OneLine,
    #[serde(rename = "target")]
    role: Role,
    duration: Seconds,
}

impl TimedUnit {
    pub(super) fn weighed(self, crates: &Crates) -> Weighed {
        let package = Package::new(self.name, self.version);
        let origin = crates.origin(&package);
        let seconds = self
            .role
            .uncached()
            .iter()
            .find(|uncached| **uncached == origin)
            .map_or_else(Seconds::default, |_| self.duration);
        Weighed::new(seconds, UnitKey::new(package, self.role))
    }
}
