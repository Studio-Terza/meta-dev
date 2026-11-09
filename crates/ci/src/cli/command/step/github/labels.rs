mod change;
mod check;
mod fault;
mod git;
mod hub;
mod place;
mod scope;

use std::ffi::OsString;
use std::fmt;
use std::io::Write;
use std::num::NonZeroU8;
use std::path::Path;

use super::super::Step;
use super::super::branch::Branch;
use super::super::changed::Changed;
use super::super::journal::Journal;
use super::pull_number::PullNumber;
use crate::cli::command::failure::Failure;
use crate::cli::command::step::Arguments;
use crate::cli::ports::{Exit, Ports};
use check::Check;
use fault::Owed;
use git::Git;
use hub::Hub;
use scope::Scope;

#[derive(Debug)]
pub struct Labels {
    base: Branch,
    pull_request: PullNumber,
    metadata: Journal,
}

impl fmt::Display for Labels {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "labels of pull request {} against the merge base with {}", self.pull_request, self.base)
    }
}

impl Step for Labels {
    fn from_arguments(given: Vec<OsString>) -> Result<Labels, Failure<'static>> {
        let mut arguments = Arguments::from(given);
        let base = &arguments.taken()?;
        let pull_request = &arguments.taken()?;
        let metadata = &arguments.taken()?;
        arguments.ended()?;
        Ok(Labels {
            base: Branch::new(base.to_str().ok_or(Failure::Usage)?.to_owned()),
            pull_request: PullNumber::new(pull_request.to_str().ok_or(Failure::Usage)?.to_owned()),
            metadata: Journal::file(Path::new(metadata)),
        })
    }

    fn carry_out(&self, ports: &impl Ports, out: &mut impl Write) -> Result<Exit, Failure<'_>> {
        let changes = Git::with_ports(ports).changes_since_fork(&self.base)?;
        let pull_request = Hub::with_ports(ports).pull_request(&self.pull_request)?;
        Changed::stored_metadata(ports, &self.metadata)?;
        writeln!(out, "{self}")?;
        let scope = Scope::reached_by(&self.metadata, &changes);
        let checks: Vec<Check> =
            Check::ALL.into_iter().filter(|check| pull_request.carrying(*check).is_none()).collect();
        let judged: Vec<Owed> = scope.judged(ports, &checks, pull_request.refactor(), &changes)?;
        let faults: Vec<&Owed> = judged.iter().filter_map(Owed::outstanding).collect();
        faults.iter().try_for_each(|owed| owed.complained(out))?;
        Ok(faults.first().map_or(Exit::Success, |_| Exit::Failure(NonZeroU8::MIN)))
    }
}
