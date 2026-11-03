mod boundary;
mod line;
mod reading;
mod test_name;
mod verdict;

use std::collections::BTreeMap;

use line::Line;
use reading::Reading;
pub use test_name::TestName;
use text_lines::Lines;
use verdict::Verdict;

#[derive(Debug, Default)]
pub struct TestLog(BTreeMap<TestName, Verdict>);

impl TestLog {
    pub fn names(&self) -> impl Iterator<Item = &TestName> {
        self.0.keys()
    }

    #[must_use]
    pub fn failed(&self) -> Vec<&TestName> {
        self.0.iter().filter(|(_, verdict)| **verdict == Verdict::Failed).map(|(name, _)| name).collect()
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
