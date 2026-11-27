use crate::cli::command::Telling;
use console_ports::Shown;
use std::ffi::OsString;

use super::Step;
use super::issue::{Issue, Title};
use crate::cli::command::failure::Failure;
use crate::cli::command::step::{Arguments, Ended, Word};
use crate::cli::ports::{Invocation, Ports};
use grammar_output::Record;

#[derive(Debug)]
pub(crate) struct StageIssues {
    limits: gate_limits::Cli,
}

#[derive(Debug, Record)]
#[record(source = Arguments)]
struct StagedWords {
    #[field(Word)]
    limits: OsString,
    #[field(Word)]
    measured: OsString,
    #[repeated(Word)]
    rest: Vec<OsString>,
}

impl From<StagedWords> for StageIssues {
    fn from(words: StagedWords) -> StageIssues {
        let given = [words.limits, words.measured].into_iter().chain(words.rest);
        StageIssues { limits: gate_limits::Cli::from_args(given) }
    }
}

impl Step for StageIssues {
    fn from_arguments(given: Vec<OsString>) -> Result<StageIssues, Failure<'static>> {
        StagedWords::read(Arguments::from(given))?.ended().map(StageIssues::from)
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
