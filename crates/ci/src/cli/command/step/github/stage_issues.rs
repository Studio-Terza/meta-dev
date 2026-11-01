mod number;
mod search;
mod ticket;

use std::ffi::OsString;
use std::io::Write;

use super::Step;
use crate::cli::command::failure::Failure;
use crate::cli::ports::{Exit, Ports};
use search::Search;
use ticket::Ticket;

#[derive(Debug)]
pub struct StageIssues {
    limits: gate_limits::Cli,
}

impl Step for StageIssues {
    fn from_arguments(arguments: &[OsString]) -> Result<StageIssues, Failure> {
        match arguments {
            [_, _] => Ok(StageIssues { limits: gate_limits::Cli::from_args(arguments.iter().cloned()) }),
            _ => Err(Failure::Usage),
        }
    }

    fn carry_out(&self, ports: &impl Ports, out: &mut impl Write) -> Result<Exit, Failure> {
        self.limits.past_warn()?.iter().try_fold(Exit::Success, |exit, finding| {
            exit.followed_by(|| {
                Ticket::open_for(finding, Search::for_stage(finding.stage()).number_among(ports)?).filed(ports, out)
            })
        })
    }
}
