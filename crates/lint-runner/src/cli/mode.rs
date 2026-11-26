mod lints;

use std::ffi::OsString;
use std::io::Write;
use std::path::PathBuf;
use std::thread::Scope;

use file_ports::Files;
use lint_core::{Readout, SourceFile};
use lint_test_budget::TestBudget;

use timer_ports::Timer;

use super::boundary::Workers;
use super::timings::{Measured, Worker};
use super::{Outcome, Sources, Verdict};
use lints::{Crew, Lints};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Mode {
    Gate,
    Report,
}

#[derive(Debug)]
pub(crate) enum Run<'scope> {
    Gate(Crew<'scope>),
    Report,
}

impl Mode {
    const REPORT_FLAG: &'static str = "--report";

    pub(crate) fn of_arguments(arguments: &[OsString]) -> Mode {
        arguments.iter().find(|argument| argument.as_os_str() == Self::REPORT_FLAG).map_or(Mode::Gate, |_| Mode::Report)
    }

    pub(crate) fn roots(arguments: &[OsString]) -> Vec<PathBuf> {
        arguments.iter().filter(|argument| argument.as_os_str() != Self::REPORT_FLAG).map(PathBuf::from).collect()
    }

    pub(crate) fn helpers(self) -> Vec<Worker> {
        match self {
            Mode::Gate => Workers::available().helpers().map(Worker::from).collect(),
            Mode::Report => Vec::new(),
        }
    }

    pub(crate) fn started<'scope, 'run: 'scope, F: Files + Sync, T: Timer + Sync>(
        self,
        scope: &'scope Scope<'scope, '_>,
        sources: &'scope Sources<F, T>,
        readers: Vec<lint_reader::Reader<'run>>,
    ) -> Run<'scope> {
        match self {
            Mode::Gate => Run::Gate(Lints::started(scope, sources, readers)),
            Mode::Report => Run::Report,
        }
    }
}

impl Run<'_> {
    pub(crate) fn verdict_of(
        self,
        readout: &Measured<Readout>,
        sources: &Sources<impl Files, impl Timer + Sync>,
        out: &mut impl Write,
        err: &mut impl Write,
    ) -> Verdict {
        match self {
            Run::Gate(crew) => crew.gated(readout, sources, out, err),
            Run::Report => Self::reported(readout.done().files(), out, err),
        }
    }

    fn reported(files: &[SourceFile], out: &mut impl Write, err: &mut impl Write) -> Verdict {
        Outcome::Clean.surveyed(&[TestBudget::measured(files)], out, err)
    }
}
