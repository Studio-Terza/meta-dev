use lint_runner::Verdict;

#[derive(Debug, PartialEq, Eq)]
pub struct Explanation(Vec<u8>);

#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    Silent,
    Explained(Explanation),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Check {
    Held,
    CleanRunExplained(Explanation),
    FailingRunSilent(Verdict),
}

impl From<Vec<u8>> for Outcome {
    fn from(err: Vec<u8>) -> Self {
        match err.first() {
            None => Self::Silent,
            Some(_) => Self::Explained(Explanation(err)),
        }
    }
}

impl Outcome {
    pub fn checked(self, verdict: Verdict) -> Check {
        match self {
            Self::Silent => Check::silent(verdict),
            Self::Explained(explanation) => Check::explained(verdict, explanation),
        }
    }
}

impl Check {
    fn silent(verdict: Verdict) -> Self {
        match verdict {
            Verdict::Clean => Self::Held,
            Verdict::Violations | Verdict::Refused => Self::FailingRunSilent(verdict),
        }
    }

    fn explained(verdict: Verdict, explanation: Explanation) -> Self {
        match verdict {
            Verdict::Clean => Self::CleanRunExplained(explanation),
            Verdict::Violations | Verdict::Refused => Self::Held,
        }
    }
}
