use std::ffi::OsString;
use std::io::Write;

use super::Step;
use super::issue::{Issue, Title};
use crate::cli::command::failure::Failure;
use crate::cli::command::step::Arguments;
use crate::cli::ports::{Exit, Invocation, Ports};

#[derive(Debug)]
pub struct StageIssues {
    limits: gate_limits::Cli,
}

impl Step for StageIssues {
    fn from_arguments(arguments: Vec<OsString>) -> Result<Self, Failure<'static>> {
        let mut arguments = Arguments::from(arguments);
        let limits = arguments.taken()?;
        let measured = arguments.taken()?;
        Ok(Self { limits: gate_limits::Cli::from_args([limits, measured].into_iter().chain(arguments.rest())) })
    }

    fn carry_out(&self, ports: &impl Ports, out: &mut impl Write) -> Result<Exit, Failure<'_>> {
        let findings = self.limits.past_warn(ports.files())?;
        let titles: Vec<Title<'_>> = findings.iter().map(|finding| Title::new(finding.stage().as_ref())).collect();
        let issues = Issue::titled(&titles, ports)?;
        let filings = findings
            .iter()
            .zip(&issues)
            .map(|(finding, issue)| {
                writeln!(out, "{} is past its warn level", finding.stage())?;
                issue.filing(finding, out)
            })
            .collect::<Result<Vec<Invocation>, Failure<'static>>>()?;
        Issue::filed(&filings, ports)
    }
}
