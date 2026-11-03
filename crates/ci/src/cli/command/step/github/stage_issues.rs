use std::ffi::OsString;
use std::io::Write;

use super::Step;
use super::issue::{Issue, Title};
use crate::cli::command::failure::Failure;
use crate::cli::ports::{Exit, Invocation, Ports};

#[derive(Debug)]
pub struct StageIssues {
    limits: gate_limits::Cli,
}

impl Step for StageIssues {
    fn from_arguments(arguments: &[OsString]) -> Result<StageIssues, Failure> {
        match arguments {
            [_, _, ..] => Ok(StageIssues { limits: gate_limits::Cli::from_args(arguments.iter().cloned()) }),
            _ => Err(Failure::Usage),
        }
    }

    fn carry_out(&self, ports: &impl Ports, out: &mut impl Write) -> Result<Exit, Failure> {
        let findings = self.limits.past_warn()?;
        let titles: Vec<Title<'_>> = findings.iter().map(|finding| Title::new(finding.stage().as_ref())).collect();
        let issues = Issue::titled(&titles, ports)?;
        let filings = findings
            .iter()
            .zip(&issues)
            .map(|(finding, issue)| {
                writeln!(out, "{} is past its warn level", finding.stage())?;
                issue.filing(finding, out)
            })
            .collect::<Result<Vec<Invocation>, Failure>>()?;
        Issue::filed(&filings, ports)
    }
}
