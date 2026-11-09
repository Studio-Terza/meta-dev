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
use std::io::Write;
use std::path::Path;

use super::Step;
use super::compile_times::CompileTimes;
use super::journal::Journal;
use super::measured::Measured;
use super::stage::Stage;
use crate::cli::command::failure::Failure;
use crate::cli::command::step::Arguments;
use crate::cli::ports::{Exit, Ports};
use file_ports::Text;
use grammar_timing_page::Build;
use host::Host;
use partial_timings::PartialTimings;
use timed_unit::TimedUnit;
use unit_dag::UnitDag;
use work::Work;

#[derive(Debug)]
pub struct GraphPath {
    page: Journal,
    measured: Measured,
    report: Journal,
}

impl GraphPath {
    const PATH: &'static str = "graph-path";

    fn recorded(
        &self,
        build: Build,
        ports: &impl Ports,
        out: &mut impl Write,
        page: &Text,
    ) -> Result<Exit, Failure<'_>> {
        match build {
            Build::Finished => self.measured(ports, out, page),
            Build::Failed => self.skipped(ports, out),
        }
    }

    fn skipped(&self, ports: &impl Ports, out: &mut impl Write) -> Result<Exit, Failure<'_>> {
        write!(out, "{PartialTimings}")?;
        self.report.emptied(ports.files())?.append(ports.files(), PartialTimings).map(|()| Exit::Success)
    }

    fn measured(&self, ports: &impl Ports, out: &mut impl Write, page: &Text) -> Result<Exit, Failure<'_>> {
        let units: Vec<TimedUnit> = CompileTimes::units(&self.page, page)?;
        let crates = Host::asked(ports)?.crates(ports)?;
        let dag = UnitDag::weighed(&crates, units);
        let critical = dag.critical();
        let work = Work::from(dag.work());
        write!(out, "{critical}{work}")?;
        let report = self.report.emptied(ports.files())?;
        report.append(ports.files(), &critical)?;
        report.append(ports.files(), work)?;
        self.measured
            .append(ports.files(), &Stage::new(Self::PATH.to_owned()), *critical.as_ref())
            .map(|()| Exit::Success)
    }
}

impl Step for GraphPath {
    fn from_arguments(given: Vec<OsString>) -> Result<GraphPath, Failure<'static>> {
        let mut arguments = Arguments::from(given);
        let page = &arguments.taken()?;
        let measured = &arguments.taken()?;
        let report = &arguments.taken()?;
        arguments.ended()?;
        Ok(GraphPath {
            page: Journal::file(Path::new(page)),
            measured: Measured::file(Path::new(measured)),
            report: Journal::file(Path::new(report)),
        })
    }

    fn carry_out(&self, ports: &impl Ports, out: &mut impl Write) -> Result<Exit, Failure<'_>> {
        self.page.read(ports.files()).and_then(|page| self.recorded(Build::recorded(page.as_ref()), ports, out, &page))
    }
}
