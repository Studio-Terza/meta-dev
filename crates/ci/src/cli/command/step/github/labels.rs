mod change;
mod check;
mod git;
mod hub;
mod place;

use std::ffi::OsString;
use std::fmt;
use std::io::Write;
use std::num::NonZeroU8;

use super::Step;
use crate::cli::command::failure::Failure;
use crate::cli::ports::{Exit, Ports};
use change::Change;
use check::Check;
use git::Git;
use hub::Hub;

#[derive(Debug)]
pub struct Labels {
    base: String,
    pull_request: String,
}

impl Labels {
    fn complained(check: Check, changes: &[Change], out: &mut impl Write) -> Result<(), Failure> {
        check.flagged(changes).iter().try_for_each(|change| writeln!(out, "{}", change.now_at().display()))?;
        Ok(writeln!(out, "::error::{check}")?)
    }
}

impl fmt::Display for Labels {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "labels of pull request {} against the merge base with {}", self.pull_request, self.base)
    }
}

impl Step for Labels {
    fn from_arguments(arguments: &[OsString]) -> Result<Labels, Failure> {
        match arguments {
            [base, pull_request] => Ok(Labels {
                base: base.to_str().ok_or(Failure::Usage)?.to_owned(),
                pull_request: pull_request.to_str().ok_or(Failure::Usage)?.to_owned(),
            }),
            _ => Err(Failure::Usage),
        }
    }

    fn carry_out(&self, ports: &impl Ports, out: &mut impl Write) -> Result<Exit, Failure> {
        let changes = Git::with_ports(ports).changes_since_fork(&self.base)?;
        let labels = Hub::with_ports(ports).labels_of(&self.pull_request)?;
        writeln!(out, "{self}")?;
        let owed: Vec<Check> = Check::ALL
            .into_iter()
            .filter(|check| !check.flagged(&changes).is_empty() && labels.carrying(check.label()).is_none())
            .collect();
        owed.iter().try_for_each(|check| Self::complained(*check, &changes, out))?;
        Ok(if owed.is_empty() { Exit::Success } else { Exit::Failure(NonZeroU8::MIN) })
    }
}
