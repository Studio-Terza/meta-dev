mod elegance;
mod lints;

use std::ffi::OsString;
use std::fmt;
use std::io::Write;
use std::path::PathBuf;
use std::thread::Scope;

use cargo_ports::Cargo;
use code_reader::Reader;
use file_ports::Files;
use graph_query::{Query, QueryFailure};
use law_facts::Laws;
use lint_clones::Clones;
use lint_core::{Lint, Readout, Report, SourceFile};
use lint_paired_apis::PairedApis;
use lint_test_budget::TestBudget;
use lint_transition_graph::{Extraction, TypeSystem};
use timer_ports::Timer;

use super::timings::Measured;
use super::{Outcome, Sources, Verdict};
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

    pub(crate) fn started<'scope, F: Files + Sync, C: Cargo + Sync, T: Timer + Sync>(
        &self,
        scope: &'scope Scope<'scope, '_>,
        sources: &'scope Sources<F, C, T>,
    ) -> Run<'scope, '_> {
        match self {
            Mode::Gate => Run::Gate(Lints::started(scope, sources)),
            Mode::Report => Run::Report,
            Mode::Query(found) => Run::Query(found.as_ref().ok()),
        }
    }
}

impl Run<'_, '_> {
    pub(crate) fn verdict_of(
        self,
        readout: &Measured<Readout>,
        sources: &Sources<impl Files, impl Cargo, impl Timer>,
        out: &mut impl Write,
        err: &mut impl Write,
    ) -> Verdict {
        match self {
            Run::Gate(crew) => crew.gated(readout, sources, out, err),
            Run::Report => Self::reported(readout.done().files(), out, err),
            Run::Query(found) => Self::queried(found, readout.done().files(), out, err),
        }
    }

    fn reported(files: &[SourceFile], out: &mut impl Write, err: &mut impl Write) -> Verdict {
        let model = Reader::read_files(files);
        Outcome::Clean.surveyed(&Elegance::reports(files, &model), out, err)
    }

    fn queried(found: Option<&Query>, files: &[SourceFile], out: &mut impl Write, err: &mut impl Write) -> Verdict {
        let extraction = Extraction::from(files);
        let laws = Laws::from(files);
        let system = TypeSystem::built_from(&extraction, &laws);
        Outcome::Clean.surveyed(found.map(|query| query.reply(&system)).as_slice(), out, err)
    }
}

#[derive(Debug)]
pub(crate) struct Survey<'files> {
    budget: TestBudget,
    paired: Report<'files>,
    clones: Report<'files>,
}

impl<'files> Survey<'files> {
    pub(crate) fn measured(files: &'files [SourceFile]) -> Survey<'files> {
        Survey { budget: TestBudget::measured(files), paired: PairedApis::report(files), clones: Clones::report(files) }
    }
}

impl fmt::Display for Survey<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}{}{}", self.budget, self.paired, self.clones)
    }
}
