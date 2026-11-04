mod elegance;
mod lints;

use std::ffi::OsString;
use std::io::Write;
use std::path::PathBuf;

use file_ports::Files;
use graph_query::{Query, QueryFailure};
use lint_clones::Clones;
use lint_core::{Lint, Readout, SourceFile};
use lint_paired_apis::PairedApis;
use lint_test_budget::TestBudget;
use transition_facts::Transitions;
use type_facts::Facts;

use super::{Outcome, Sources};
use elegance::Elegance;
use lints::Lints;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Mode {
    Gate,
    Report,
    Query(Result<Query, QueryFailure>),
}

impl Mode {
    const REPORT_FLAG: &'static str = "--report";
    const QUERY_WORD: &'static str = "query";

    pub(crate) fn of_arguments(arguments: &[OsString]) -> Mode {
        match arguments.split_first() {
            Some((first, rest)) if first.as_os_str() == Self::QUERY_WORD => Mode::Query(Query::try_from(rest)),
            _ => arguments
                .iter()
                .find(|argument| argument.as_os_str() == Self::REPORT_FLAG)
                .map_or(Mode::Gate, |_| Mode::Report),
        }
    }

    pub(crate) fn roots(&self, arguments: &[OsString]) -> Vec<PathBuf> {
        match self {
            Mode::Query(Ok(query)) => query.roots().to_vec(),
            Mode::Query(Err(_)) => Vec::new(),
            Mode::Gate | Mode::Report => arguments
                .iter()
                .filter(|argument| argument.as_os_str() != Self::REPORT_FLAG)
                .map(PathBuf::from)
                .collect(),
        }
    }

    pub(crate) fn refusal(&self) -> Option<&QueryFailure> {
        match self {
            Mode::Query(Err(failure)) => Some(failure),
            Mode::Gate | Mode::Report | Mode::Query(Ok(_)) => None,
        }
    }

    pub(crate) fn outcome_of<'file>(
        &self,
        readout: &'file Readout,
        sources: &Sources<impl Files + Sync>,
        out: &mut impl Write,
    ) -> Outcome<'file> {
        match self {
            Mode::Gate => Self::gated(readout, sources, out),
            Mode::Report => Self::reported(readout.files(), out),
            Mode::Query(found) => Self::queried(found, readout.files(), out),
        }
    }

    fn gated<'file>(
        readout: &'file Readout,
        sources: &Sources<impl Files + Sync>,
        out: &mut impl Write,
    ) -> Outcome<'file> {
        let files = readout.files();
        write!(out, "{}{}{}", TestBudget::measured(files), PairedApis::report(files), Clones::report(files)).ok();
        Lints::checked(readout, sources)
    }

    fn reported(files: &[SourceFile], out: &mut impl Write) -> Outcome<'static> {
        Elegance::reports(files).iter().try_for_each(|report| write!(out, "{report}")).ok();
        Outcome::Clean
    }

    fn queried(found: &Result<Query, QueryFailure>, files: &[SourceFile], out: &mut impl Write) -> Outcome<'static> {
        found
            .iter()
            .try_for_each(|query| {
                write!(out, "{}", query.matches(&Facts::of_files(files), &Transitions::of_files(files)))
            })
            .ok();
        Outcome::Clean
    }
}
