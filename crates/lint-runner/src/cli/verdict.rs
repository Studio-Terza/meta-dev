use std::process::ExitCode;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    Clean,
    Violations,
    Refused,
}

impl From<Verdict> for ExitCode {
    fn from(verdict: Verdict) -> Self {
        match verdict {
            Verdict::Clean => Self::SUCCESS,
            Verdict::Violations | Verdict::Refused => Self::FAILURE,
        }
    }
}
