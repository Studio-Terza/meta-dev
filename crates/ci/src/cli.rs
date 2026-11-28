use std::ffi::OsString;
use std::path::PathBuf;

use console_ports::Answer;
use dispatch_ports::ShellFiller;
use file_ports::Files;
use port_core::Run;
use timer_ports::ClockFiller;

use crate::failure::Failure;
use crate::kills::Kills;
use crate::mutants::Plan;
use crate::ports::Ports;
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

    pub fn answer(self, files: &impl Files, clock: impl ClockFiller, shell: impl ShellFiller) -> Answer {
        let run = Run::root();
        let wall = run.port(clock);
        let programs = run.port(shell);
        let ports = Ports::new(files, &wall, &programs);
        Answer::answered(match self {
            Cli::Timed(timed) => timed.answered(&ports),
            Cli::Plan(plan) => plan.answered(&ports),
            Cli::Kills(kills) => kills.answered(&ports),
            Cli::Usage => Err(Failure::Usage),
        })
    }
}
