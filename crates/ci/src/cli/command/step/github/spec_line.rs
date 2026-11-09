mod verdict;

use std::ffi::OsString;
use std::io::Write;

use super::Step;
use super::pull_number::PullNumber;
use crate::cli::command::failure::Failure;
use crate::cli::command::step::Arguments;
use crate::cli::ports::{Exit, Invocation, Ports};
use verdict::Verdict;

#[derive(Debug)]
pub struct SpecLine {
    pull_request: PullNumber,
}

impl SpecLine {
    const BODY: &'static str = "body";
    const RAW_BODY: &'static str = ".body";
}

impl Step for SpecLine {
    fn from_arguments(given: Vec<OsString>) -> Result<SpecLine, Failure<'static>> {
        let mut arguments = Arguments::from(given);
        let pull_request = &arguments.taken()?;
        arguments.ended()?;
        Ok(SpecLine { pull_request: PullNumber::new(pull_request.to_str().ok_or(Failure::Usage)?.to_owned()) })
    }

    fn carry_out(&self, ports: &impl Ports, out: &mut impl Write) -> Result<Exit, Failure<'_>> {
        let invocation = Invocation::new(
            "gh",
            ["pr", "view", self.pull_request.as_ref(), "--json", Self::BODY, "--jq", Self::RAW_BODY],
        );
        let body = ports.output(&invocation).map_err(Failure::starting(&invocation))?;
        let verdict = Verdict::judging(&body);
        verdict.reported(out)?;
        Ok(verdict.exit())
    }
}
