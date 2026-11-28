use std::ffi::OsString;
use std::path::PathBuf;

use console_ports::Answer;

use crate::failure::Failure;
use crate::kills::Kills;
use crate::mutants::Plan;
use crate::timed::Timed;

#[derive(Debug)]
pub enum Cli {
    Timed(Timed),
    Plan(Plan),
    Kills(Kills),
    Usage,
}

impl Cli {
    const TIMED: &'static str = "timed";
    const PLAN: &'static str = "mutants-plan";
    const KILLS: &'static str = "kill-matrix";
    const SINCE: &'static str = "--since";

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
            [name, report, shards @ ..] if name == Cli::KILLS && !shards.is_empty() => {
                Cli::Kills(Kills::new(report, shards))
            }
            _ => Cli::Usage,
        }
    }

    #[must_use]
    pub fn answer(self) -> Answer {
        Answer::answered(match self {
            Cli::Timed(timed) => timed.answered(),
            Cli::Plan(plan) => plan.answered(),
            Cli::Kills(kills) => kills.answered(),
            Cli::Usage => Err(Failure::Usage),
        })
    }
}
