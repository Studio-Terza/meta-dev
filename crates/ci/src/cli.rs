use std::ffi::OsString;
use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;

use invariants::Ignoring;

use crate::failure::Failure;
use crate::ignored::Ignored;
use crate::mutants::Plan;
use crate::queries::{Mode, Queries};
use crate::timed::Timed;

#[derive(Debug)]
pub enum Cli {
    Timed(Timed),
    Plan(Plan),
    Queries(Queries),
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
    const SINCE: &'static str = "--since";
    const QUERIES: &'static str = "queries";
    const RECORD: &'static str = "--record";

    #[must_use]
    pub fn from_args(arguments: &[OsString]) -> Cli {
        match arguments {
            [name, limits, stage, flag, mark, program, rest @ ..] if name == Cli::TIMED && flag == Cli::SINCE => {
                Cli::Timed(Timed::new([limits, stage], Some(mark), program, rest))
            }
            [name, limits, stage, program, rest @ ..] if name == Cli::TIMED => {
                Cli::Timed(Timed::new([limits, stage], None, program, rest))
            }
            [name, workspace, diff, changed, unselected, output] if name == Cli::PLAN => {
                Cli::Plan(Plan::new([workspace, diff, changed, unselected, output].map(PathBuf::from)))
            }
            [name, workspace] if name == Cli::QUERIES => {
                Cli::Queries(Queries::new(PathBuf::from(workspace), Mode::Checked))
            }
            [name, workspace, flag] if name == Cli::QUERIES && flag == Cli::RECORD => {
                Cli::Queries(Queries::new(PathBuf::from(workspace), Mode::Recorded))
            }
            _ => Cli::Usage,
        }
    }

    pub fn exit(self, out: &mut impl Write, err: &mut impl Write) -> ExitCode {
        let ran = match self {
            Cli::Timed(timed) => timed.run(out),
            Cli::Plan(plan) => plan.run(out),
            Cli::Queries(queries) => queries.run(out),
            Cli::Usage => Err(Failure::Usage),
        };
        let exit = ran.or_else(|failure| writeln!(err, "{failure}").map(|()| Exit::Failed));
        match exit.ignoring(Ignored::Unwritable) {
            Some(Exit::Passed) => ExitCode::SUCCESS,
            Some(Exit::Failed) | None => ExitCode::FAILURE,
        }
    }
}
