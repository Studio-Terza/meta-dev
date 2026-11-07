use std::str::FromStr;

use super::super::test_name::TestName;
use super::super::verdict::Verdict;
use super::binary::Binary;
use super::unmatched::Unmatched;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TestResult {
    name: String,
    verdict: Verdict,
}

impl TestResult {
    const TEST: &'static str = "test ";
    const SEPARATOR: &'static str = " ... ";
    const PASSED: &'static str = "ok";
    const FAILED: &'static str = "FAILED";
    const JOIN: &'static str = " :: ";

    pub fn key(&self, binary: Option<&Binary>) -> TestName {
        TestName::new(format!("{}{}{}", binary.map(Binary::label).unwrap_or_default(), Self::JOIN, self.name))
    }

    pub fn verdict(&self) -> Verdict {
        self.verdict
    }

    fn verdict_of(outcome: &str) -> Option<Verdict> {
        match outcome {
            Self::PASSED => Some(Verdict::Passed),
            Self::FAILED => Some(Verdict::Failed),
            _ => None,
        }
    }
}

impl FromStr for TestResult {
    type Err = Unmatched;

    fn from_str(line: &str) -> Result<TestResult, Unmatched> {
        let parsed = || {
            let padded =
                line.get(..Self::TEST.len()).filter(|start| *start == Self::TEST).and(line.get(Self::TEST.len()..))?;
            let trailing: usize =
                padded.chars().rev().take_while(|letter| letter.is_whitespace()).map(char::len_utf8).sum();
            let body = padded.get(..padded.len() - trailing)?;
            let (at, separator) = body.match_indices(Self::SEPARATOR).last()?;
            let name = body.get(..at).filter(|_| at > 0)?;
            let verdict = Self::verdict_of(body.get(at + separator.len()..)?)?;
            Some(TestResult { name: name.to_owned(), verdict })
        };
        parsed().ok_or(Unmatched)
    }
}
