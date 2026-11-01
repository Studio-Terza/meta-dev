use std::ffi::OsString;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use super::super::journal::Journal;
use super::Step;
use crate::cli::command::failure::{Failure, Refusal};
use crate::cli::ports::{Exit, Ports};

#[derive(Debug)]
pub struct Matrix {
    shards: PathBuf,
    sources: PathBuf,
    directory: PathBuf,
    summary: Journal,
}

impl Matrix {
    const MARKDOWN: &'static str = "report.md";
    const JSON: &'static str = "report.json";

    fn written(&self) -> Result<Journal, Failure> {
        fs::create_dir_all(&self.directory).map_err(|source| Failure::Io { path: self.directory.clone(), source })?;
        let markdown = self.directory.join(Self::MARKDOWN);
        let arguments = [
            self.shards.clone().into_os_string(),
            self.sources.clone().into_os_string(),
            markdown.clone().into_os_string(),
            self.directory.join(Self::JSON).into_os_string(),
        ];
        let mut refusal = Vec::new();
        match mutation_report::Cli::answer(arguments.into_iter(), &mut refusal) {
            code if code == ExitCode::SUCCESS => Ok(Journal::file(&markdown)),
            _ => Err(Failure::Matrix(Refusal::from(refusal))),
        }
    }
}

impl Step for Matrix {
    fn from_arguments(arguments: &[OsString]) -> Result<Matrix, Failure> {
        match arguments {
            [shards, sources, directory, summary] => Ok(Matrix {
                shards: PathBuf::from(shards),
                sources: PathBuf::from(sources),
                directory: PathBuf::from(directory),
                summary: Journal::file(Path::new(summary)),
            }),
            _ => Err(Failure::Usage),
        }
    }

    fn carry_out(&self, _ports: &impl Ports, out: &mut impl Write) -> Result<Exit, Failure> {
        let markdown = self.written()?.read()?;
        write!(out, "{markdown}")?;
        self.summary.append(&markdown)?;
        Ok(Exit::Success)
    }
}
