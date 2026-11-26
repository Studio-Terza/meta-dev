use crate::cli::command::Telling;
use console_ports::Shown;
mod branch;
mod change;
mod check;
mod fault;
mod git;
mod hub;
mod place;
mod scope;

use std::ffi::OsString;
use std::fmt;
use std::num::NonZeroU8;

use super::super::Step;
use super::super::changed::Changed;
use super::super::journal::Journal;
use super::pull_number::PullNumber;
use crate::cli::command::failure::Failure;
use crate::cli::command::step::{Arguments, Ended, Journaled, Utf8};
use crate::cli::ports::{Exit, Ports};
use branch::Branch;
use check::Check;
use fault::Owed;
use git::Git;
use grammar_output::Record;
use hub::Hub;
use scope::Scope;
use strum::VariantArray;

#[derive(Debug, Record)]
#[record(source = Arguments)]
pub struct Labels {
    #[field(Utf8, via = Branch::new)]
    base: Branch,
    #[field(Utf8, via = PullNumber::new)]
    pull_request: PullNumber,
    #[field(Journaled)]
    metadata: Journal,
}

impl fmt::Display for Labels {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "labels of pull request {} against the merge base with {}", self.pull_request, self.base)
    }
}

impl Step for Labels {
    fn from_arguments(given: Vec<OsString>) -> Result<Labels, Failure<'static>> {
        Labels::read(Arguments::from(given))?.ended()
    }

    fn carry_out<'step>(&'step self, ports: &'step impl Ports) -> Result<Telling<'step>, Failure<'step>> {
        let changes = Git::with_ports(ports).changes_since_fork(&self.base)?;
        let pull_request = Hub::with_ports(ports).pull_request(&self.pull_request)?;
        Changed::stored_metadata(ports, &self.metadata)?;
        let scope = Scope::reached_by(&self.metadata, &changes);
        let carried = pull_request.carried();
        let checks: Vec<Check> =
            Check::VARIANTS.iter().copied().filter(|check| carried.iter().all(|label| label != check)).collect();
        let judged: Vec<Owed> = scope.judged(ports, &checks, pull_request.purpose(), &changes)?;
        let faults: Vec<&Owed> = judged.iter().filter(|owed| owed.outstanding().is_some()).collect();
        let said = [Shown::said(format_args!("{self}\n"))].into_iter().chain(faults.iter().map(Shown::said)).collect();
        Ok(Telling::reported(said, Ok(faults.first().map_or(Exit::Success, |_| Exit::Failure(NonZeroU8::MIN)))))
    }
}
