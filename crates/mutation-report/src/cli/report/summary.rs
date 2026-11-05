mod boundary;
mod entry;
mod heading;
mod section;

use std::borrow::Cow;
use std::collections::BTreeSet;
use std::fmt;
use std::io;
use std::path::Path;

use boundary::Count;
use entry::Entry;
use file_ports::{Bytes, Files};
use heading::Heading;
use one_line::OneLine;
use section::Section;
use serde::Serialize;

use super::failure::Failure;
use super::golden_tests::GoldenTests;
use super::recorded_mutant::{RecordedMutant, Verdict};
use super::shard_matrix::ShardMatrix;
use super::test_name::TestName;

#[derive(Debug, Serialize)]
pub struct Summary {
    mutants: Count,
    tests: Count,
    golden_tests: Count,
    surviving: Vec<Entry>,
    timed_out: Vec<Entry>,
    tests_that_kill_nothing: Vec<TestName>,
    killed_with_no_golden_failing: Vec<Entry>,
}

impl Summary {
    const SCOPE: &'static str = "::";

    pub(crate) fn from_results(matrices: &[ShardMatrix], golden_tests: &GoldenTests) -> Summary {
        let baseline: BTreeSet<TestName> = matrices
            .iter()
            .flat_map(|matrix| matrix.baseline().iter().map(ToString::to_string).map(TestName::new))
            .collect();
        let mutants: Vec<RecordedMutant> =
            matrices.iter().flat_map(|matrix| matrix.mutants().iter().cloned()).collect();
        let golden = golden_tests.names();
        let killers: BTreeSet<TestName> =
            mutants.iter().flat_map(RecordedMutant::failed).map(ToString::to_string).map(TestName::new).collect();
        Summary {
            mutants: Count::items_in(&mutants),
            tests: Count::new(baseline.len()),
            golden_tests: Count::new(golden.len()),
            surviving: Entry::named_with(&mutants, Verdict::MissedMutant),
            timed_out: Entry::named_with(&mutants, Verdict::Timeout),
            tests_that_kill_nothing: baseline.difference(&killers).cloned().collect(),
            killed_with_no_golden_failing: Self::killed_without_golden(&mutants, golden),
        }
    }

    pub(crate) fn written_as_json<'path>(&self, files: &impl Files, path: &'path Path) -> Result<(), Failure<'path>> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|source| Failure::Parse { path: Cow::Borrowed(path), source })?;
        files
            .write(path, Bytes::from(json.as_str()))
            .map_err(|failure| Failure::Io { path: Cow::Borrowed(path), source: io::Error::from(failure) })
    }

    fn killed_without_golden(mutants: &[RecordedMutant], golden: &BTreeSet<TestName>) -> Vec<Entry> {
        let mut ordered: Vec<&RecordedMutant> = mutants.iter().collect();
        ordered.sort_by(|left, right| left.name().cmp(right.name()));
        ordered
            .into_iter()
            .filter_map(|mutant| mutant.failed().first().map(|_| mutant))
            .filter_map(|mutant| {
                mutant.failed().iter().find_map(|test| Self::golden(test, golden)).map_or(Some(mutant), |_| None)
            })
            .map(ToString::to_string)
            .map(Entry::new)
            .collect()
    }

    fn golden<'set>(test: &OneLine, golden: &'set BTreeSet<TestName>) -> Option<&'set TestName> {
        let test = test.as_ref();
        let name =
            test.match_indices(Self::SCOPE).last().and_then(|(at, mark)| test.get(at + mark.len()..)).unwrap_or(test);
        let spelled: String = name.chars().skip_while(|letter| letter.is_whitespace()).collect();
        golden.get(&TestName::new(spelled))
    }
}

impl fmt::Display for Summary {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "## Mutation matrix\n")?;
        writeln!(formatter, "{} mutants, {} tests, {} golden tests.", self.mutants, self.tests, self.golden_tests)?;
        write!(formatter, "{}", Section::new(Heading::Surviving, &self.surviving))?;
        write!(formatter, "{}", Section::new(Heading::TimedOut, &self.timed_out))?;
        write!(formatter, "{}", Section::new(Heading::KillingNothing, &self.tests_that_kill_nothing))?;
        write!(formatter, "{}", Section::new(Heading::KilledWithoutGolden, &self.killed_with_no_golden_failing))
    }
}
