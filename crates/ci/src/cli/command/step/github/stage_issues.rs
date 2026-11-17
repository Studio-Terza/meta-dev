use crate::cli::command::Telling;
use console_ports::Shown;
use std::ffi::OsString;

use super::Step;
use super::issue::{Issue, Title};
use crate::cli::command::failure::Failure;
use crate::cli::command::step::Arguments;
use crate::cli::ports::{Invocation, Ports};

#[derive(Debug)]
pub struct StageIssues {
    limits: gate_limits::Cli,
}

impl Step for StageIssues {
    fn from_arguments(given: Vec<OsString>) -> Result<StageIssues, Failure<'static>> {
        let mut arguments = Arguments::from(given);
        let limits = arguments.taken()?;
        let measured = arguments.taken()?;
        Ok(StageIssues { limits: gate_limits::Cli::from_args([limits, measured].into_iter().chain(arguments.rest())) })
    }

    fn carry_out<'step>(&'step self, ports: &'step impl Ports) -> Result<Telling<'step>, Failure<'step>> {
        let findings = self.limits.past_warn(ports.files())?;
        let titles: Vec<Title<'_>> = findings.iter().map(|finding| Title::new(finding.stage().as_ref())).collect();
        let issues = Issue::titled(&titles, ports)?;
        let said = findings
            .iter()
            .zip(&issues)
            .map(|(finding, issue)| {
                Shown::said(format_args!("{} is past its warn level\n{}", finding.stage(), issue.announcement()))
            })
            .collect();
        let filings: Vec<Invocation> =
            findings.iter().zip(&issues).map(|(finding, issue)| issue.filing(finding)).collect();
        Ok(Telling::reported(said, Issue::filed(&filings, ports)))
    }
}
