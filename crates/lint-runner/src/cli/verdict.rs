use std::fmt;
use std::io::Write;
use std::process::ExitCode;

use invariants::Ignoring;

use crate::cli::ignored::Ignored;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    Clean,
    Violations,
    Refused,
}

impl Verdict {
    pub(crate) fn refused(failure: &impl fmt::Display, err: &mut impl Write) -> Verdict {
        writeln!(err, "{failure}").ignoring(Ignored::Unwritable);
        Verdict::Refused
    }

    pub(crate) fn ran(ran: Result<Verdict, impl fmt::Display>, err: &mut impl Write) -> Verdict {
        match ran {
            Ok(verdict) => verdict,
            Err(failure) => Verdict::refused(&failure, err),
        }
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
