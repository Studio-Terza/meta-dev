use std::ffi::OsString;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use file_ports::Files;

use super::super::journal::Journal;
use super::Step;
use crate::cli::command::failure::{Failure, Refusal};
use crate::cli::command::step::Arguments;
use crate::cli::ports::{Exit, Ports};

#[derive(Debug)]
pub struct Matrix {
    shards: PathBuf,
    sources: PathBuf,
    directory: PathBuf,
    markdown: Journal,
    json: PathBuf,
    summary: Journal,
}

impl Matrix {
    const MARKDOWN: &'static str = "report.md";
    const JSON: &'static str = "report.json";

    fn written(&self, files: &impl Files) -> Result<&Journal, Failure<'_>> {
        files.create_directories(&self.directory).map_err(|failure| Failure::unusable(&self.directory, failure))?;
        let arguments = [&self.shards, &self.sources, self.markdown.as_ref(), &self.json];
        let mut refusal = Vec::new();
        let answered = mutation_report::Cli::answer(&arguments, files, &mut refusal);
        Some(answered)
            .filter(|answered| *answered == ExitCode::SUCCESS)
            .map(|_| &self.markdown)
            .ok_or_else(|| Failure::Matrix(Refusal::from(refusal)))
    }
}

impl Step for Matrix {
    fn from_arguments(arguments: Vec<OsString>) -> Result<Self, Failure<'static>> {
        let mut arguments = Arguments::from(arguments);
        let shards = &arguments.taken()?;
        let sources = &arguments.taken()?;
        let directory = &arguments.taken()?;
        let summary = &arguments.taken()?;
        arguments.ended()?;
        Ok(Self {
            shards: PathBuf::from(shards),
            sources: PathBuf::from(sources),
            markdown: Journal::from(Path::new(directory).join(Self::MARKDOWN)),
            json: Path::new(directory).join(Self::JSON),
            directory: PathBuf::from(directory),
            summary: Journal::file(Path::new(summary)),
        })
    }

    fn carry_out(&self, ports: &impl Ports, out: &mut impl Write) -> Result<Exit, Failure<'_>> {
        let markdown = self.written(ports.files())?.read(ports.files())?;
        write!(out, "{markdown}")?;
        self.summary.append(ports.files(), &markdown)?;
        Ok(Exit::Success)
    }
}
