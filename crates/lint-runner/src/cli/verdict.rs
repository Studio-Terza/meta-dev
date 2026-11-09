use std::fmt;
use std::io::Write;
use std::process::ExitCode;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    Clean,
    Violations,
    Refused,
}

impl Verdict {
    pub(crate) fn refused(failure: &impl fmt::Display, err: &mut impl Write) -> Verdict {
        writeln!(err, "{failure}").ok();
        Verdict::Refused
    }
}

impl From<Verdict> for ExitCode {
    fn from(verdict: Verdict) -> ExitCode {
        match verdict {
            Verdict::Clean => ExitCode::SUCCESS,
            Verdict::Violations | Verdict::Refused => ExitCode::FAILURE,
        }
    }
}
