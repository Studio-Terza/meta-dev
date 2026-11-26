mod boundary;
mod entry;
mod heading;
mod section;
mod unviable;

use std::borrow::Cow;
use std::collections::BTreeSet;
use std::fmt;
use std::io;
use std::path::Path;

use boundary::Count;
use entry::Entry;
use file_ports::{Bytes, Files};
use heading::Heading;
use section::Section;
use serde::Serialize;
use unviable::Unviable;

use super::failure::Failure;
use super::recorded_mutant::{RecordedMutant, Verdict};
use super::shard_matrix::ShardMatrix;
use super::test_name::TestName;

#[derive(Debug, Serialize)]
pub struct Summary {
    mutants: Count,
    tests: Count,
    surviving: Vec<Entry>,
    timed_out: Vec<Entry>,
    tests_that_kill_nothing: Vec<TestName>,
    unviable: Vec<Entry>,
}

impl Summary {
    pub(crate) fn from_results(matrices: &[ShardMatrix]) -> Summary {
        let baseline: BTreeSet<TestName> = matrices
            .iter()
            .flat_map(|matrix| matrix.baseline().iter().map(ToString::to_string).map(TestName::new))
            .collect();
        let mutants: Vec<RecordedMutant> =
            matrices.iter().flat_map(|matrix| matrix.mutants().iter().cloned()).collect();
        let killers: BTreeSet<TestName> =
            mutants.iter().flat_map(RecordedMutant::failed).map(ToString::to_string).map(TestName::new).collect();
        Summary {
            mutants: Count::items_in(&mutants),
            tests: Count::new(baseline.len()),
            surviving: Entry::named_with(&mutants, Verdict::MissedMutant),
            timed_out: Entry::named_with(&mutants, Verdict::Timeout),
            tests_that_kill_nothing: baseline.difference(&killers).cloned().collect(),
            unviable: Unviable::by_crate(&mutants),
        }
    }

    pub(crate) fn written_as_json<'path>(&self, files: &impl Files, path: &'path Path) -> Result<(), Failure<'path>> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|source| Failure::Parse { path: Cow::Borrowed(path), source })?;
        files
            .write(path, Bytes::from(json.as_str()))
            .map_err(|failure| Failure::Io { path: Cow::Borrowed(path), source: io::Error::from(failure) })
    }

}

impl fmt::Display for Summary {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "## Mutation matrix\n")?;
        writeln!(formatter, "{} mutants, {} tests.", self.mutants, self.tests)?;
        write!(formatter, "{}", Section::new(Heading::Surviving, &self.surviving))?;
        write!(formatter, "{}", Section::new(Heading::TimedOut, &self.timed_out))?;
        write!(formatter, "{}", Section::new(Heading::KillingNothing, &self.tests_that_kill_nothing))?;
        write!(formatter, "{}", Section::new(Heading::Unviable, &self.unviable))
    }
}
