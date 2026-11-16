use crate::cli::command::Telling;
use console_ports::Shown;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use console_ports::Complaint;
use file_ports::Files;

use super::super::journal::Journal;
use super::Step;
use crate::cli::command::failure::{Failure, Refusal};
use crate::cli::command::step::Arguments;
use crate::cli::ports::{Exit, Ports, Text};

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
        let answered = mutation_report::Cli::answer(&arguments, files);
        let refusal: &Complaint = answered.as_ref();
        Some(answered.code())
            .filter(|code| *code == ExitCode::SUCCESS)
            .map(|_| &self.markdown)
            .ok_or_else(|| Failure::Matrix(Refusal::from(Text::new(AsRef::<str>::as_ref(refusal).to_owned()))))
    }
}

impl Step for Matrix {
    fn from_arguments(given: Vec<OsString>) -> Result<Matrix, Failure<'static>> {
        let mut arguments = Arguments::from(given);
        let shards = &arguments.taken()?;
        let sources = &arguments.taken()?;
        let directory = &arguments.taken()?;
        let summary = &arguments.taken()?;
        arguments.ended()?;
        Ok(Matrix {
            shards: PathBuf::from(shards),
            sources: PathBuf::from(sources),
            markdown: Journal::from(Path::new(directory).join(Self::MARKDOWN)),
            json: Path::new(directory).join(Self::JSON),
            directory: PathBuf::from(directory),
            summary: Journal::file(Path::new(summary)),
        })
    }

    fn carry_out<'step>(&'step self, ports: &'step impl Ports) -> Result<Telling<'step>, Failure<'step>> {
        let markdown = self.written(ports.files())?.read(ports.files())?;
        let said = vec![Shown::said(format_args!("{markdown}"))];
        Ok(Telling::reported(said, self.summary.append(ports.files(), &markdown).map(|()| Exit::Success)))
    }
}
