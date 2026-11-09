mod crates;
mod manifest;
mod package;
mod role;
mod skipped;
mod timed_unit;
mod unit_dag;
mod unit_key;
mod weighed;

use std::ffi::OsString;
use std::io::Write;
use std::iter;
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
use file_ports::Text;
use grammar::timing_page::Build;
use manifest::Manifest;
use skipped::Skipped;
use text_lines::Lines;
use timed_unit::TimedUnit;
use unit_dag::UnitDag;

#[derive(Debug)]
pub struct GraphPath {
    page: Journal,
    measured: Measured,
    report: Journal,
}

impl GraphPath {
    const METADATA: [&'static str; 5] = ["metadata", "--locked", "--format-version", "1", "--filter-platform"];
    const HOST: [&'static str; 2] = ["--print", "host-tuple"];
    const PATH: &'static str = "graph-path";
    const WORK: &'static str = "graph-work";

    fn crates(ports: &impl Ports) -> Result<Crates, Failure<'static>> {
        let host = Self::host(ports)?;
        let arguments = Self::METADATA.into_iter().map(OsString::from).chain(iter::once(host));
        let listed = Changed::asked(ports, &Invocation::new("cargo", arguments))?;
        serde_json::from_str::<Manifest>(listed.as_ref()).map(Manifest::crates).map_err(Failure::Metadata)
    }

    fn host(ports: &impl Ports) -> Result<OsString, Failure<'static>> {
        let named = Changed::asked(ports, &Invocation::new("rustc", Self::HOST))?;
        let lines: Vec<String> = named.as_ref().parse::<Lines>().into_iter().flatten().collect();
        <[String; 1]>::try_from(lines).map(|[host]| OsString::from(host)).map_err(Failure::NoHost)
    }

    fn skipped(&self, ports: &impl Ports, out: &mut impl Write) -> Result<Exit, Failure<'_>> {
        write!(out, "{Skipped}")?;
        self.report.emptied(ports.files())?.append(ports.files(), Skipped).map(|()| Exit::Success)
    }

    fn measured(&self, ports: &impl Ports, out: &mut impl Write, page: &Text) -> Result<Exit, Failure<'_>> {
        let units: Vec<TimedUnit> = CompileTimes::units(&self.page, page)?;
        let crates = Self::crates(ports)?;
        let dag = UnitDag::weighed(&crates, units);
        let critical = dag.critical();
        write!(out, "{critical}")?;
        self.report.emptied(ports.files())?.append(ports.files(), &critical)?;
        self.measured.append(ports.files(), &Stage::new(Self::PATH.to_owned()), *critical.as_ref())?;
        self.measured.append(ports.files(), &Stage::new(Self::WORK.to_owned()), dag.work()).map(|()| Exit::Success)
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
        let page = self.page.read(ports.files())?;
        match Build::recorded(page.as_ref()) {
            Build::Finished => self.measured(ports, out, &page),
            Build::Failed => self.skipped(ports, out),
        }
    }
}
