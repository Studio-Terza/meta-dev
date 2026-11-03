mod count;
mod mutant;
mod outcome;
mod report;
mod tally;

use std::ffi::OsString;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use file_ports::Files;
use text_lines::Lines;

use super::super::journal::Journal;
use super::Step;
use crate::cli::command::failure::Failure;
use crate::cli::ports::{Exit, Ports, Text};
use count::Count;
use mutant::Mutant;
use outcome::Outcome;
use report::Report;
use tally::Tally;

#[derive(Debug)]
pub struct ShardSummary {
    shards: PathBuf,
    summary: Journal,
}

impl ShardSummary {
    fn directories(&self, files: &impl Files) -> Result<Vec<PathBuf>, Failure> {
        let mut found = files
            .entries(&self.shards)
            .map_err(|failure| Failure::Io { path: self.shards.clone(), source: io::Error::from(failure) })?;
        found.sort();
        Ok(found)
    }

    fn text_of(&self, files: &impl Files, outcome: Outcome) -> Result<Text, Failure> {
        self.directories(files)?
            .iter()
            .map(|directory| Journal::file(&directory.join(outcome.file_name())))
            .filter(|journal| journal.existing(files).is_some())
            .try_fold(String::new(), |text, file| Ok(text + file.read(files)?.as_ref()))
            .map(Text::new)
    }

    fn report(&self, files: &impl Files) -> Result<Report, Failure> {
        let tallies = Outcome::ALL
            .into_iter()
            .map(|outcome| {
                Ok(Tally::new(outcome, Count::new(Lines::from(self.text_of(files, outcome)?.as_ref()).count())))
            })
            .collect::<Result<_, Failure>>()?;
        let listed = Outcome::LISTED.into_iter().try_fold(String::new(), |text, outcome| {
            Ok::<String, Failure>(text + self.text_of(files, outcome)?.as_ref())
        })?;
        let mut names: Vec<Mutant> = Lines::from(listed.as_str()).map(|name| Mutant::new(name.to_owned())).collect();
        names.sort();
        Ok(Report::new(tallies, names))
    }
}

impl Step for ShardSummary {
    fn from_arguments(arguments: &[OsString]) -> Result<ShardSummary, Failure> {
        match arguments {
            [shards, summary] => {
                Ok(ShardSummary { shards: PathBuf::from(shards), summary: Journal::file(Path::new(summary)) })
            }
            _ => Err(Failure::Usage),
        }
    }

    fn carry_out(&self, ports: &impl Ports, out: &mut impl Write) -> Result<Exit, Failure> {
        let report = self.report(ports.files())?;
        write!(out, "{report}")?;
        self.summary.append(ports.files(), report)?;
        Ok(Exit::Success)
    }
}
