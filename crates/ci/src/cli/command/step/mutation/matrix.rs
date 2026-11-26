use crate::cli::command::Telling;
use console_ports::Shown;
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::ExitCode;

use console_ports::Complaint;
use file_ports::Files;

use super::super::journal::Journal;
use super::Step;
use crate::cli::command::failure::{Failure, Refusal};
use crate::cli::command::step::{Arguments, Ended, Journaled, Location};
use crate::cli::ports::{Exit, Ports, Text};
use grammar_output::Record;

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

#[derive(Debug, Record)]
#[record(source = Arguments)]
struct MatrixWords {
    #[field(Location)]
    shards: PathBuf,
    #[field(Location)]
    sources: PathBuf,
    #[field(Location)]
    directory: PathBuf,
    #[field(Journaled)]
    summary: Journal,
}

impl From<MatrixWords> for Matrix {
    fn from(words: MatrixWords) -> Matrix {
        Matrix {
            shards: words.shards,
            sources: words.sources,
            markdown: Journal::from(words.directory.join(Matrix::MARKDOWN)),
            json: words.directory.join(Matrix::JSON),
            directory: words.directory,
            summary: words.summary,
        }
    }
}

impl Step for Matrix {
    fn from_arguments(given: Vec<OsString>) -> Result<Matrix, Failure<'static>> {
        MatrixWords::read(Arguments::from(given))?.ended().map(Matrix::from)
    }

    fn carry_out<'step>(&'step self, ports: &'step impl Ports) -> Result<Telling<'step>, Failure<'step>> {
        let markdown = self.written(ports.files())?.read(ports.files())?;
        let said = vec![Shown::said(format_args!("{markdown}"))];
        Ok(Telling::reported(said, self.summary.append(ports.files(), &markdown).map(|()| Exit::Success)))
    }
}
