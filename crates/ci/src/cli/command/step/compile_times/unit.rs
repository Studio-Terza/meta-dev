use std::collections::BTreeMap;

use serde::Deserialize;

use crate::cli::command::step::seconds::Seconds;
use one_line::OneLine;

#[derive(Debug, Deserialize)]
pub struct Unit {
    name: OneLine,
    duration: Seconds,
}

impl Unit {
    pub(super) fn tallied(self, mut totals: BTreeMap<OneLine, Seconds>) -> BTreeMap<OneLine, Seconds> {
        let total = totals.entry(self.name).or_default();
        *total = [*total, self.duration].into_iter().sum();
        totals
    }
}
