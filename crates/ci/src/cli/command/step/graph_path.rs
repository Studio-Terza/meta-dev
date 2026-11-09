mod crates;
mod manifest;
mod package;
mod role;
mod timed_unit;
mod unit_dag;
mod unit_key;
mod weighed;

use std::ffi::OsString;
use std::io::Write;
use std::path::Path;

use super::Step;
use super::changed::Changed;
use super::compile_times::CompileTimes;
use super::journal::Journal;
use super::measured::Measured;
use super::stage::Stage;
use crate::cli::command::failure::Failure;
use crate::cli::command::step::Arguments;
use crate::cli::ports::{Exit, Invocation, Ports};
use crates::Crates;
use manifest::Manifest;
use timed_unit::TimedUnit;
use unit_dag::UnitDag;

#[derive(Debug)]
pub struct GraphPath {
    page: Journal,
    measured: Measured,
    report: Journal,
}

impl GraphPath {
    const METADATA: [&'static str; 4] = ["metadata", "--locked", "--format-version", "1"];
    const PATH: &'static str = "graph-path";
    const WORK: &'static str = "graph-work";

    fn crates(ports: &impl Ports) -> Result<Crates, Failure<'static>> {
        let listed = Changed::asked(ports, &Invocation::new("cargo", Self::METADATA))?;
        serde_json::from_str::<Manifest>(listed.as_ref()).map(Manifest::crates).map_err(Failure::Metadata)
    }
}

impl Step for GraphPath {
    fn from_arguments(arguments: Vec<OsString>) -> Result<GraphPath, Failure<'static>> {
        let mut arguments = Arguments::from(arguments);
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
        let units: Vec<TimedUnit> = CompileTimes::units(&self.page, &self.page.read(ports.files())?)?;
        let crates = Self::crates(ports)?;
        let dag = UnitDag::weighed(&crates, units);
        let critical = dag.critical();
        write!(out, "{critical}")?;
        self.report.emptied(ports.files())?.append(ports.files(), &critical)?;
        self.measured.append(ports.files(), &Stage::new(Self::PATH.to_owned()), *critical.as_ref())?;
        self.measured.append(ports.files(), &Stage::new(Self::WORK.to_owned()), dag.work()).map(|()| Exit::Success)
    }
}
