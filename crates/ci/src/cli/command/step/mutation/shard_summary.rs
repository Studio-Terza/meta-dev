mod count;
mod mutant;
mod outcome;
mod report;
mod tally;

use std::ffi::OsString;
use std::io::Write;
use std::path::{Path, PathBuf};

use file_ports::Files;
use text_lines::Lines;

use super::super::journal::Journal;
use super::Step;
use crate::cli::command::failure::Failure;
use crate::cli::command::step::Arguments;
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
    fn directories(&self, files: &impl Files) -> Result<Vec<PathBuf>, Failure<'_>> {
        let mut found = files.entries(&self.shards).map_err(|failure| Failure::unusable(&self.shards, failure))?;
        found.sort();
        Ok(found)
    }

    fn texts(&self, files: &impl Files) -> Result<Vec<Text>, Failure<'_>> {
        let directories = self.directories(files)?;
        let (tags, journals): (Vec<Outcome>, Vec<Journal>) = Outcome::ALL
            .into_iter()
            .flat_map(|outcome| directories.iter().map(move |directory| (outcome, directory.join(outcome.file_name()))))
            .map(|(outcome, path)| (outcome, Journal::from(path)))
            .unzip();
        let read: Vec<Option<Text>> =
            Journal::read_present(files, journals).into_iter().map(Option::transpose).collect::<Result<_, _>>()?;
        Ok(Outcome::ALL
            .into_iter()
            .map(|outcome| {
                let texts = tags.iter().zip(&read).filter(|(tag, _)| **tag == outcome).flat_map(|(_, text)| text);
                Text::new(texts.map(AsRef::as_ref).collect())
            })
            .collect())
    }

    fn report(&self, files: &impl Files) -> Result<Report, Failure<'_>> {
        let texts = self.texts(files)?;
        let tallies = Outcome::ALL
            .into_iter()
            .zip(&texts)
            .map(|(outcome, text)| Tally::new(outcome, Count::new(Lines::from(text.as_ref()).count())))
            .collect();
        let listed: String = Outcome::ALL
            .iter()
            .zip(&texts)
            .filter(|(outcome, _)| Outcome::LISTED.contains(outcome))
            .map(|(_, text)| text.as_ref())
            .collect();
        let mut names: Vec<Mutant> = Lines::from(listed.as_str()).map(|name| Mutant::new(name.to_owned())).collect();
        names.sort();
        Ok(Report::new(tallies, names))
    }
}

impl Step for ShardSummary {
    fn from_arguments(arguments: Vec<OsString>) -> Result<ShardSummary, Failure<'static>> {
        let mut arguments = Arguments::from(arguments);
        let shards = &arguments.taken()?;
        let summary = &arguments.taken()?;
        arguments.ended()?;
        Ok(ShardSummary { shards: PathBuf::from(shards), summary: Journal::file(Path::new(summary)) })
    }

    fn carry_out(&self, ports: &impl Ports, out: &mut impl Write) -> Result<Exit, Failure<'_>> {
        let report = self.report(ports.files())?;
        write!(out, "{report}")?;
        self.summary.append(ports.files(), report)?;
        Ok(Exit::Success)
    }
}
