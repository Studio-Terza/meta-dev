use std::ffi::OsString;
use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;

use invariants::Ignoring;

use crate::failure::Failure;
use crate::ignored::Ignored;
use crate::mutants::{Plan, Prior};
use crate::timed::Timed;

#[derive(Debug)]
pub enum Cli {
    Timed(Timed),
    Plan(Plan),
    Usage,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Exit {
    Passed,
    Failed,
}

impl Cli {
    const TIMED: &'static str = "timed";
    const PLAN: &'static str = "mutants-plan";

    #[must_use]
    pub fn from_args(arguments: &[OsString]) -> Cli {
        match arguments {
            [name, limits, stage, program, rest @ ..] if name == Cli::TIMED => {
                Cli::Timed(Timed::new(PathBuf::from(limits), stage.to_string_lossy().into_owned(), program, rest))
            }
            [name, listed, reused, caught, output, prior @ ..] if name == Cli::PLAN => {
                Cli::Plan(Plan::new([listed, reused, caught, output].map(PathBuf::from), Prior::given(prior)))
            }
            _ => Cli::Usage,
        }
    }

    pub fn exit(self, out: &mut impl Write, err: &mut impl Write) -> ExitCode {
        let ran = match self {
            Cli::Timed(timed) => timed.run(out),
            Cli::Plan(plan) => plan.run(out),
            Cli::Usage => Err(Failure::Usage),
        };
        let exit = ran.or_else(|failure| writeln!(err, "{failure}").map(|()| Exit::Failed));
        match exit.ignoring(Ignored::Unwritable) {
            Some(Exit::Passed) => ExitCode::SUCCESS,
            Some(Exit::Failed) | None => ExitCode::FAILURE,
        }
    }
}
