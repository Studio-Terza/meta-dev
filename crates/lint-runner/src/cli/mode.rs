mod elegance;
mod lints;
mod revisions;

use std::ffi::OsString;
use std::fmt;
use std::io::Write;
use std::path::PathBuf;
use std::thread::Scope;

use file_ports::Files;
use graph_query::{Query, QueryFailure};
use lint_clones::Clones;
use lint_core::{Lint, Readout, SourceFile};
use lint_paired_apis::PairedApis;
use lint_test_budget::TestBudget;
use lint_transition_graph::{Extraction, TypeSystem};

use super::boundary::Helpers;
use super::{Outcome, Sources};
use elegance::Elegance;
use lints::Lints;
use revisions::{Revisions, Usage};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Mode {
    Gate,
    Report,
    Query(Result<Query, QueryFailure>),
    Strength(Result<Revisions, Usage>),
}

impl Mode {
    const REPORT_FLAG: &'static str = "--report";
    const QUERY_WORD: &'static str = "query";
    const STRENGTH_WORD: &'static str = "strength";

    pub(crate) fn of_arguments(arguments: &[OsString]) -> Mode {
        let queried = arguments
            .split_first()
            .filter(|(first, _)| first.as_os_str() == Self::QUERY_WORD)
            .map(|(_, rest)| Mode::Query(Query::try_from(rest)));
        let compared = arguments
            .split_first()
            .filter(|(first, _)| first.as_os_str() == Self::STRENGTH_WORD)
            .map(|(_, rest)| Mode::Strength(Revisions::try_from(rest)));
        queried.or(compared).unwrap_or_else(|| {
            arguments
                .iter()
                .find(|argument| argument.as_os_str() == Self::REPORT_FLAG)
                .map_or(Mode::Gate, |_| Mode::Report)
        })
    }

    pub(crate) fn roots(&self, arguments: &[OsString]) -> Vec<PathBuf> {
        match self {
            Mode::Query(Ok(query)) => query.roots().to_vec(),
            Mode::Query(Err(_)) | Mode::Strength(Err(_)) => Vec::new(),
            Mode::Strength(Ok(revisions)) => revisions.roots(),
            Mode::Gate | Mode::Report => arguments
                .iter()
                .filter(|argument| argument.as_os_str() != Self::REPORT_FLAG)
                .map(PathBuf::from)
                .collect(),
        }
    }

    pub(crate) fn refusal(&self) -> Option<&dyn fmt::Display> {
        match self {
            Mode::Query(Err(failure)) => Some(failure),
            Mode::Strength(Err(usage)) => Some(usage),
            Mode::Gate | Mode::Report | Mode::Query(Ok(_)) | Mode::Strength(Ok(_)) => None,
        }
    }

    pub(crate) fn helpers<'scope, F: Files + Sync>(
        &self,
        scope: &'scope Scope<'scope, '_>,
        sources: &'scope Sources<F>,
    ) -> Helpers<'scope> {
        match self {
            Mode::Gate => Lints::started(scope, sources),
            Mode::Report | Mode::Query(_) | Mode::Strength(_) => Vec::new(),
        }
    }

    pub(crate) fn outcome_of<'file>(
        &self,
        readout: &'file Readout,
        helpers: Helpers<'_>,
        files: &impl Files,
        out: &mut impl Write,
    ) -> Outcome<'file> {
        match self {
            Mode::Gate => Self::gated(readout, helpers, out),
            Mode::Report => Self::reported(readout.files(), out),
            Mode::Query(found) => Self::queried(found.as_ref().ok(), readout.files(), out),
            Mode::Strength(Ok(revisions)) => revisions.outcome(readout, files),
            Mode::Strength(Err(_)) => Outcome::Clean,
        }
    }

    fn gated<'file>(readout: &'file Readout, helpers: Helpers<'_>, out: &mut impl Write) -> Outcome<'file> {
        let files = readout.files();
        write!(out, "{}{}{}", TestBudget::measured(files), PairedApis::report(files), Clones::report(files)).ok();
        Lints::checked(readout, helpers)
    }

    fn reported(files: &[SourceFile], out: &mut impl Write) -> Outcome<'static> {
        Elegance::reports(files).iter().try_for_each(|report| write!(out, "{report}")).ok();
        Outcome::Clean
    }

    fn queried(found: Option<&Query>, files: &[SourceFile], out: &mut impl Write) -> Outcome<'static> {
        found
            .iter()
            .try_for_each(|query| {
                let extraction = Extraction::from(files);
                write!(out, "{}", query.reply(&TypeSystem::built_from(files, &extraction)))
            })
            .ok();
        Outcome::Clean
    }
}
