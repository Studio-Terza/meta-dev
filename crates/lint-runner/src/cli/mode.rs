mod elegance;
mod lints;

use std::ffi::OsString;
use std::io::Write;
use std::path::PathBuf;
use std::thread::Scope;

use cargo_ports::Cargo;
use code_reader::Reader;
use file_ports::Files;
use graph_query::{Query, QueryFailure};
use lint_clones::Clones;
use lint_core::{Lint, Readout, SourceFile};
use lint_paired_apis::PairedApis;
use lint_test_budget::TestBudget;
use lint_transition_graph::{Extraction, TypeSystem};

use super::boundary::Queue;
use super::{Outcome, Sources};
use elegance::Elegance;
use lints::{Crew, Lints};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Mode {
    Gate,
    Report,
    Query(Result<Query, QueryFailure>),
}

#[derive(Debug)]
pub(crate) enum Run<'scope, 'mode> {
    Gate(Crew<'scope>),
    Report,
    Query(Option<&'mode Query>),
}

impl Mode {
    const REPORT_FLAG: &'static str = "--report";
    const QUERY_WORD: &'static str = "query";

    pub(crate) fn of_arguments(arguments: &[OsString]) -> Mode {
        arguments.split_first().filter(|(first, _)| first.as_os_str() == Self::QUERY_WORD).map_or_else(
            || {
                arguments
                    .iter()
                    .find(|argument| argument.as_os_str() == Self::REPORT_FLAG)
                    .map_or(Mode::Gate, |_| Mode::Report)
            },
            |(_, rest)| Mode::Query(Query::read(rest)),
        )
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

    pub(crate) fn started<'scope, F: Files + Sync, C: Cargo + Sync>(
        &self,
        scope: &'scope Scope<'scope, '_>,
        sources: &'scope Sources<F, C>,
    ) -> Run<'scope, '_> {
        match self {
            Mode::Gate => Run::Gate(Lints::started(scope, sources)),
            Mode::Report => Run::Report,
            Mode::Query(found) => Run::Query(found.as_ref().ok()),
        }
    }
}

impl Run<'_, '_> {
    pub(crate) fn outcome_of<'file>(
        self,
        readout: &'file Readout,
        queue: &Queue,
        out: &mut impl Write,
    ) -> Outcome<'file> {
        match self {
            Run::Gate(crew) => Self::gated(readout, queue, crew, out),
            Run::Report => Self::reported(readout.files(), out),
            Run::Query(found) => Self::queried(found, readout.files(), out),
        }
    }

    fn gated<'file>(readout: &'file Readout, queue: &Queue, crew: Crew<'_>, out: &mut impl Write) -> Outcome<'file> {
        let files = readout.files();
        write!(out, "{}{}{}", TestBudget::measured(files), PairedApis::report(files), Clones::report(files)).ok();
        crew.checked(readout, queue)
    }
    fn reported(files: &[SourceFile], out: &mut impl Write) -> Outcome<'static> {
        let model = Reader::read_files(files);
        Elegance::reports(files, &model).iter().try_for_each(|report| write!(out, "{report}")).ok();
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
