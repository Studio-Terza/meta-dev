use std::collections::BTreeSet;
use std::path::Path;

use super::super::entry::Entry;
use crate::cli::report::recorded_mutant::{RecordedMutant, Verdict};

#[derive(Debug)]
pub struct Unviable;

impl Unviable {
    const ALL: &'static str = "all crates";

    pub(crate) fn by_crate(mutants: &[RecordedMutant]) -> Vec<Entry> {
        let crates: BTreeSet<&str> = mutants.iter().filter_map(Unviable::crate_of).collect();
        let each = crates.into_iter().map(|krate| {
            Unviable::counted(krate, mutants.iter().filter(|mutant| Unviable::crate_of(mutant) == Some(krate)))
        });
        [Unviable::counted(Self::ALL, mutants.iter())].into_iter().chain(each).collect()
    }

    fn crate_of(mutant: &RecordedMutant) -> Option<&str> {
        Path::new(mutant.name().as_ref()).components().nth(1).and_then(|component| component.as_os_str().to_str())
    }

    fn counted<'mutants>(krate: &str, mutants: impl Iterator<Item = &'mutants RecordedMutant>) -> Entry {
        let verdicts: Vec<Verdict> = mutants.map(RecordedMutant::summary).collect();
        let unviable = verdicts.iter().filter(|verdict| **verdict == Verdict::Unviable).count();
        Entry::new(format!("{krate}: {unviable} of {} unviable", verdicts.len()))
    }
}
