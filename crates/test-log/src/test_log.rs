mod binary;
mod line;
mod reading;
mod test_result;
mod unmatched;
mod verdict;

use std::collections::{BTreeMap, BTreeSet};

use line::Line;
use reading::Reading;
use text_lines::Lines;
use verdict::Verdict;

#[derive(Debug, Default)]
pub struct TestLog(BTreeMap<String, Verdict>);

impl TestLog {
    #[must_use]
    pub fn names(&self) -> BTreeSet<String> {
        self.0.keys().cloned().collect()
    }

    #[must_use]
    pub fn failed(&self) -> Vec<String> {
        self.0.iter().filter(|(_, verdict)| **verdict == Verdict::Failed).map(|(name, _)| name.clone()).collect()
    }
}

impl From<&str> for TestLog {
    fn from(text: &str) -> TestLog {
        TestLog(
            Lines::over(text)
                .map(Line::from)
                .fold(Reading::default(), |reading, line| line.continuing(reading))
                .into_results(),
        )
    }
}
