mod elegance;
mod lints;

use std::ffi::{OsStr, OsString};
use std::fmt;
use std::io::Write;
use std::path::PathBuf;
use std::thread::Scope;

use cargo_ports::Cargo;
use file_ports::Files;
use graph::Model;
use graph_query::{Answered, Query, QueryFailure};
use graph_reader::Reader;
use lint_core::{Readout, Report, SourceFile};
use lint_laws::LawRunners;
use lint_reader::Described;
use lint_test_budget::TestBudget;

use timer_ports::Timer;
use workspace_metadata::Metadata;

use super::timings::Measured;
use super::{Outcome, Sources, Verdict};
use elegance::Elegance;
use lints::{Crew, Lints};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Mode {
    Gate,
    Report,
    Query(Result<Query, QueryFailure>),
    Laws,
}

#[derive(Debug)]
pub(crate) enum Run<'scope, 'mode> {
    Gate(Crew<'scope>),
    Report,
    Query(Option<&'mode Query>),
    Laws,
}

impl Mode {
    const REPORT_FLAG: &'static str = "--report";
    const QUERY_WORD: &'static str = "query";
    const LAWS_WORD: &'static str = "laws";

    pub(crate) fn of_arguments(arguments: &[OsString]) -> Mode {
        arguments.split_first().and_then(|(first, rest)| Self::worded(first, rest)).unwrap_or_else(|| {
            arguments
                .iter()
                .find(|argument| argument.as_os_str() == Self::REPORT_FLAG)
                .map_or(Mode::Gate, |_| Mode::Report)
        })
    }

    fn worded(first: &OsStr, rest: &[OsString]) -> Option<Mode> {
        (first == Self::QUERY_WORD)
            .then(|| Mode::Query(Query::read(rest)))
            .or_else(|| (first == Self::LAWS_WORD).then_some(Mode::Laws))
    }

    pub(crate) fn roots(&self, arguments: &[OsString]) -> Vec<PathBuf> {
        match self {
            Mode::Query(Ok(query)) => query.roots().to_vec(),
            Mode::Query(Err(_)) => Vec::new(),
            Mode::Laws => arguments.iter().skip(1).map(PathBuf::from).collect(),
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
            Mode::Gate | Mode::Report | Mode::Query(Ok(_)) | Mode::Laws => None,
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
            Mode::Laws => Run::Laws,
        }
    }
}

impl Run<'_, '_> {
    pub(crate) fn verdict_of(
        self,
        readout: &Measured<Readout>,
        sources: &Sources<impl Files, impl Cargo, impl Timer + Sync>,
        out: &mut impl Write,
        err: &mut impl Write,
    ) -> Verdict {
        match self {
            Run::Gate(crew) => crew.gated(readout, sources, out, err),
            Run::Report => Self::reported(readout.done().files(), out, err),
            Run::Query(found) => Self::queried(
                found.map(|query| query.answered(readout.done().files(), Self::workspaces(sources))).as_ref(),
                out,
                err,
            ),
            Run::Laws => Self::lawful(readout.done().files(), sources.wired().files(), err),
        }
    }

    fn lawful(files: &[SourceFile], disk: &impl Files, err: &mut impl Write) -> Verdict {
        let model = Reader::read_files(files);
        disk.write_each(LawRunners::from(&model).written())
            .map_or_else(|failure| Verdict::refused(&failure, err), |()| Verdict::Clean)
    }

    fn reported(files: &[SourceFile], out: &mut impl Write, err: &mut impl Write) -> Verdict {
        let model = Reader::read_files(files);
        Outcome::Clean.surveyed(&[Survey::measured(files, &model)], out, err)
    }

    fn queried(answered: Option<&Answered>, out: &mut impl Write, err: &mut impl Write) -> Verdict {
        Outcome::Clean.surveyed(answered.as_slice(), out, err)
    }

    fn workspaces<'run>(sources: &'run Sources<impl Files, impl Cargo, impl Timer>) -> &'run [Metadata] {
        sources.located().done().as_ref().ok().map_or(&[][..], Described::workspaces)
    }
}

#[derive(Debug)]
struct Survey<'files> {
    budget: TestBudget,
    reports: Vec<Report<'files>>,
}

impl<'files> Survey<'files> {
    fn measured(files: &'files [SourceFile], model: &'files Model) -> Survey<'files> {
        Survey { budget: TestBudget::measured(files).started(model), reports: Elegance::reports(files, model) }
    }
}

impl fmt::Display for Survey<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.budget)?;
        self.reports.iter().try_for_each(|report| write!(formatter, "{report}"))
    }
}
