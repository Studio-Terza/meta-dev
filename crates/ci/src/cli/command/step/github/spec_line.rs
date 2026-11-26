use crate::cli::command::Telling;
use console_ports::Shown;
mod verdict;

use std::ffi::OsString;

use super::Step;
use super::pull_number::PullNumber;
use crate::cli::command::failure::Failure;
use crate::cli::command::step::{Arguments, Ended, Utf8};
use crate::cli::ports::{Invocation, Ports};
use grammar_output::Record;
use verdict::Verdict;

#[derive(Debug, Record)]
#[record(source = Arguments)]
pub struct SpecLine {
    #[field(Utf8, via = PullNumber::new)]
    pull_request: PullNumber,
}

impl SpecLine {
    const BODY: &'static str = "body";
    const RAW_BODY: &'static str = ".body";
}

impl Step for SpecLine {
    fn from_arguments(given: Vec<OsString>) -> Result<SpecLine, Failure<'static>> {
        SpecLine::read(Arguments::from(given))?.ended()
    }

    fn carry_out<'step>(&'step self, ports: &'step impl Ports) -> Result<Telling<'step>, Failure<'step>> {
        let invocation = Invocation::new(
            "gh",
            ["pr", "view", self.pull_request.as_ref(), "--json", Self::BODY, "--jq", Self::RAW_BODY],
        );
        let body = ports.output(&invocation).map_err(Failure::starting(&invocation))?;
        let verdict = Verdict::judging(&body);
        Ok(Telling::reported(vec![Shown::said(&verdict)], Ok(verdict.exit())))
    }
}
