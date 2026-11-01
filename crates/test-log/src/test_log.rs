mod binary;
mod line;
mod reading;
mod test_name;
mod test_result;
mod unmatched;
mod verdict;

use std::collections::{BTreeMap, BTreeSet};

use line::Line;
use reading::Reading;
pub use test_name::TestName;
use text_lines::Lines;
use verdict::Verdict;

#[derive(Debug, Default)]
pub struct TestLog(BTreeMap<TestName, Verdict>);

impl TestLog {
    #[must_use]
    pub fn names(&self) -> BTreeSet<TestName> {
        self.0.keys().cloned().collect()
    }

    #[must_use]
    pub fn failed(&self) -> Vec<TestName> {
        self.0.iter().filter(|(_, verdict)| **verdict == Verdict::Failed).map(|(name, _)| name.clone()).collect()
    }
}

impl From<&str> for TestLog {
    fn from(text: &str) -> TestLog {
        TestLog(
            Lines::from(text)
                .map(Line::from)
                .fold(Reading::default(), |reading, line| line.continuing(reading))
                .into_results(),
        )
    }
}
