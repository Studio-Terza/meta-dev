use crate::cli::command::Telling;
use console_ports::Shown;
mod crates;
mod host;
mod manifest;
mod package;
mod partial_timings;
mod timed_unit;
mod unit_dag;
mod unit_key;
mod unit_role;
mod weighed;
mod work;

use std::ffi::OsString;

use super::Step;
use super::compile_times::CompileTimes;
use super::journal::Journal;
use super::measured::Measured;
use super::stage::Stage;
use crate::cli::command::failure::Failure;
use crate::cli::command::step::{Arguments, Ended, Journaled, Recording};
use crate::cli::ports::{Exit, Ports};
use file_ports::Text;
use grammar_output::Record;
use grammar_timing_page::Build;
use host::Host;
use partial_timings::PartialTimings;
use timed_unit::TimedUnit;
use unit_dag::UnitDag;
use work::Work;

#[derive(Debug, Record)]
#[record(source = Arguments)]
pub(crate) struct GraphPath {
    #[field(Journaled)]
    page: Journal,
    #[field(Recording)]
    measured: Measured,
    #[field(Journaled)]
    report: Journal,
}

impl GraphPath {
    const PATH: &'static str = "graph-path";

    fn recorded(&self, build: Build, ports: &impl Ports, page: &Text) -> Result<Telling<'_>, Failure<'_>> {
        match build {
            Build::Finished => self.measured(ports, page),
            Build::Failed => self.skipped(ports),
        }
    }

    fn skipped(&self, ports: &impl Ports) -> Result<Telling<'_>, Failure<'_>> {
        let said = vec![Shown::said(PartialTimings)];
        self.report.replace(ports.files(), &Text::default())?;
        let ended = self.report.append(ports.files(), PartialTimings).map(|()| Exit::Success);
        Ok(Telling::reported(said, ended))
    }

    fn measured(&self, ports: &impl Ports, page: &Text) -> Result<Telling<'_>, Failure<'_>> {
        let units: Vec<TimedUnit> = CompileTimes::units(&self.page, page)?;
        let crates = Host::asked(ports)?.crates(ports)?;
        let dag = UnitDag::weighed(&crates, units);
        let critical = dag.critical();
        let work = Work::from(dag.work());
        let said = vec![Shown::said(format_args!("{critical}{work}"))];
        self.report.replace(ports.files(), &Text::default())?;
        self.report.append(ports.files(), &critical)?;
        self.report.append(ports.files(), work)?;
        let ended = self
            .measured
            .append(ports.files(), &Stage::new(Self::PATH.to_owned()), *critical.as_ref())
            .map(|()| Exit::Success);
        Ok(Telling::reported(said, ended))
    }
}

impl Step for GraphPath {
    fn from_arguments(given: Vec<OsString>) -> Result<GraphPath, Failure<'static>> {
        GraphPath::read(Arguments::from(given))?.ended()
    }

    fn carry_out<'step>(&'step self, ports: &'step impl Ports) -> Result<Telling<'step>, Failure<'step>> {
        self.page.read(ports.files()).and_then(|page| self.recorded(Build::recorded(page.as_ref()), ports, &page))
    }
}
