use newtype::{Arbitrary, Display, Newtype};
use serde::Serialize;

use crate::cli::report::recorded_mutant::{RecordedMutant, Verdict};

#[derive(Newtype, Arbitrary, Display, Serialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[serde(transparent)]
pub struct Entry(String);

impl Entry {
    pub(crate) fn named_with(mutants: &[RecordedMutant], summary: Verdict) -> Vec<Self> {
        let mut names: Vec<Self> = mutants
            .iter()
            .filter(|mutant| mutant.summary() == summary)
            .map(RecordedMutant::name)
            .map(ToString::to_string)
            .map(Entry)
            .collect();
        names.sort();
        names
    }
}
