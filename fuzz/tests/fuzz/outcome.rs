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
    fn from(err: Vec<u8>) -> Outcome {
        match err.first() {
            None => Outcome::Silent,
            Some(_) => Outcome::Explained(Explanation(err)),
        }
    }
}

impl Outcome {
    pub fn checked(self, verdict: Verdict) -> Check {
        match self {
            Outcome::Silent => Check::silent(verdict),
            Outcome::Explained(explanation) => Check::explained(verdict, explanation),
        }
    }
}

impl Check {
    fn silent(verdict: Verdict) -> Check {
        match verdict {
            Verdict::Clean => Check::Held,
            Verdict::Violations | Verdict::Refused => Check::FailingRunSilent(verdict),
        }
    }

    fn explained(verdict: Verdict, explanation: Explanation) -> Check {
        match verdict {
            Verdict::Clean => Check::CleanRunExplained(explanation),
            Verdict::Violations | Verdict::Refused => Check::Held,
        }
    }
}
