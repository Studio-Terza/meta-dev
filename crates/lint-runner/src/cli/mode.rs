mod elegance;
mod lints;

use std::ffi::OsString;
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

use super::boundary::{Dumped, Workers};
use super::timings::{Measured, Worker};
use super::{Outcome, Sources, Verdict};
use crate::cli::ignored::Ignored;
use elegance::Elegance;
use invariants::Ignoring;
use lints::{Crew, Lints};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Mode {
    Gate,
    Report,
    Query(Result<Query, QueryFailure>),
    Laws(Vec<PathBuf>),
    Dump(Vec<PathBuf>),
}

#[derive(Debug)]
pub(crate) enum Run<'scope, 'mode> {
    Gate(Crew<'scope>),
    Report,
    Query(Option<&'mode Query>),
    Laws,
    Dump,
}

impl Mode {
    const REPORT_FLAG: &'static str = "--report";
    const QUERY_WORD: &'static str = "query";
    const LAWS_WORD: &'static str = "laws";
    const DUMP_WORD: &'static str = "graph-dump";

    pub(crate) fn of_arguments(arguments: &[OsString]) -> Mode {
        let first = arguments.split_first();
        let lawed = first
            .filter(|(word, _)| word.as_os_str() == Self::LAWS_WORD)
            .map(|(_, rest)| Mode::Laws(rest.iter().map(PathBuf::from).collect()));
        let dumped = first
            .filter(|(word, _)| word.as_os_str() == Self::DUMP_WORD)
            .map(|(_, rest)| Mode::Dump(rest.iter().map(PathBuf::from).collect()));
        let queried = first
            .filter(|(word, _)| word.as_os_str() == Self::QUERY_WORD)
            .map(|(_, rest)| Mode::Query(Query::read(rest)));
        let reported =
            arguments.iter().find(|argument| argument.as_os_str() == Self::REPORT_FLAG).map(|_| Mode::Report);
        lawed.or(dumped).or(queried).or(reported).unwrap_or(Mode::Gate)
    }

    pub(crate) fn roots(&self, arguments: &[OsString]) -> Vec<PathBuf> {
        match self {
            Mode::Query(Ok(query)) => query.roots().to_vec(),
            Mode::Query(Err(_)) => Vec::new(),
            Mode::Laws(roots) | Mode::Dump(roots) => roots.clone(),
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
            Mode::Gate | Mode::Report | Mode::Query(Ok(_)) | Mode::Laws(_) | Mode::Dump(_) => None,
        }
    }

    pub(crate) fn helpers(&self) -> Vec<Worker> {
        match self {
            Mode::Gate => Workers::available().helpers().map(Worker::from).collect(),
            Mode::Report | Mode::Query(_) | Mode::Laws(_) | Mode::Dump(_) => Vec::new(),
        }
    }

    pub(crate) fn started<'scope, 'run: 'scope, F: Files + Sync, C: Cargo + Sync, T: Timer + Sync>(
        &self,
        scope: &'scope Scope<'scope, '_>,
        sources: &'scope Sources<F, C, T>,
        readers: Vec<lint_reader::Reader<'run>>,
    ) -> Run<'scope, '_> {
        match self {
            Mode::Gate => Run::Gate(Lints::started(scope, sources, readers)),
            Mode::Report => Run::Report,
            Mode::Query(found) => Run::Query(found.as_ref().ignoring(Ignored::Refused)),
            Mode::Laws(_) => Run::Laws,
            Mode::Dump(_) => Run::Dump,
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
            Run::Dump => {
                Outcome::Clean.surveyed(&[Dumped::from(&Reader::read_files(readout.done().files()))], out, err)
            }
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
        sources.located().done().as_ref().ignoring(Ignored::Unlocated).map_or(&[][..], Described::workspaces)
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
