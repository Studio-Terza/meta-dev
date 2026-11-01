mod bits;
mod section;

use std::collections::BTreeSet;
use std::fmt;

use bits::Count;
use section::Section;
use serde::Serialize;

use super::recorded_mutant::RecordedMutant;

#[derive(Debug, Serialize)]
pub struct Summary {
    mutants: Count,
    tests: Count,
    golden_tests: Count,
    surviving: Vec<String>,
    timed_out: Vec<String>,
    tests_that_kill_nothing: Vec<String>,
    killed_with_no_golden_failing: Vec<String>,
}

impl Summary {
    const MISSED: &'static str = "MissedMutant";
    const TIMEOUT: &'static str = "Timeout";
    const SCOPE: &'static str = "::";

    pub(crate) fn from_results(
        baseline: &BTreeSet<String>,
        mutants: &[RecordedMutant],
        golden: &BTreeSet<String>,
    ) -> Summary {
        let killers: BTreeSet<&String> = mutants.iter().flat_map(RecordedMutant::failed).collect();
        Summary {
            mutants: Count::items_in(mutants),
            tests: Count::new(baseline.len()),
            golden_tests: Count::new(golden.len()),
            surviving: Self::named_with(mutants, Self::MISSED),
            timed_out: Self::named_with(mutants, Self::TIMEOUT),
            tests_that_kill_nothing: baseline.iter().filter(|test| !killers.contains(test)).cloned().collect(),
            killed_with_no_golden_failing: Self::killed_without_golden(mutants, golden),
        }
    }

    pub(crate) fn json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    fn named_with(mutants: &[RecordedMutant], summary: &str) -> Vec<String> {
        let mut names: Vec<String> = mutants
            .iter()
            .filter(|mutant| mutant.has_summary(summary))
            .map(|mutant| mutant.name().to_owned())
            .collect();
        names.sort();
        names
    }

    fn killed_without_golden(mutants: &[RecordedMutant], golden: &BTreeSet<String>) -> Vec<String> {
        let mut ordered: Vec<&RecordedMutant> = mutants.iter().collect();
        ordered.sort_by(|left, right| left.name().cmp(right.name()));
        ordered
            .into_iter()
            .filter(|mutant| {
                !mutant.failed().is_empty() && !mutant.failed().iter().any(|test| Self::is_golden(test, golden))
            })
            .map(ToString::to_string)
            .collect()
    }

    fn is_golden(test: &str, golden: &BTreeSet<String>) -> bool {
        let name =
            test.match_indices(Self::SCOPE).last().and_then(|(at, mark)| test.get(at + mark.len()..)).unwrap_or(test);
        let spelled: String = name.chars().skip_while(|letter| letter.is_whitespace()).collect();
        golden.contains(&spelled)
    }
}

impl fmt::Display for Summary {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "## Mutation matrix\n")?;
        writeln!(formatter, "{} mutants, {} tests, {} golden tests.", self.mutants, self.tests, self.golden_tests)?;
        [
            Section::new("Surviving mutants", &self.surviving),
            Section::new("Timed out mutants", &self.timed_out),
            Section::new("Tests that kill no mutant", &self.tests_that_kill_nothing),
            Section::new("Mutants killed while no golden test fails", &self.killed_with_no_golden_failing),
        ]
        .iter()
        .try_for_each(|section| write!(formatter, "{section}"))
    }
}
