use crate::cli::command::Telling;
use console_ports::Shown;
mod count;
mod mutant;
pub(super) mod outcome;
mod report;
mod tally;

use std::ffi::OsString;
use std::path::PathBuf;

use file_ports::Files;
use text_lines::Lines;

use super::super::journal::Journal;
use super::Step;
use crate::cli::command::failure::Failure;
use crate::cli::command::step::{Arguments, Ended, Journaled, Location};
use crate::cli::ports::{Exit, Ports, Text};
use count::Count;
use grammar_output::Record;
use mutant::Mutant;
use outcome::Outcome;
use report::Report;
use strum::VariantArray;
use tally::Tally;

#[derive(Debug, Record)]
#[record(source = Arguments)]
pub struct ShardSummary {
    #[field(Location)]
    shards: PathBuf,
    #[field(Journaled)]
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
        let (tags, journals): (Vec<Outcome>, Vec<Journal>) = Outcome::VARIANTS
            .iter()
            .copied()
            .flat_map(|outcome| directories.iter().map(move |directory| (outcome, directory.join(outcome.file_name()))))
            .map(|(outcome, path)| (outcome, Journal::from(path)))
            .unzip();
        let read: Vec<Option<Text>> =
            Journal::read_present(files, journals).into_iter().map(Option::transpose).collect::<Result<_, _>>()?;
        Ok(Outcome::VARIANTS
            .iter()
            .copied()
            .map(|outcome| {
                let texts = tags.iter().zip(&read).filter(|(tag, _)| **tag == outcome).flat_map(|(_, text)| text);
                Text::new(texts.map(AsRef::as_ref).collect())
            })
            .collect())
    }

    fn report(&self, files: &impl Files) -> Result<Report, Failure<'_>> {
        let texts = self.texts(files)?;
        let tallies = Outcome::VARIANTS
            .iter()
            .copied()
            .zip(&texts)
            .map(|(outcome, text)| {
                Tally::new(outcome, Count::new(text.as_ref().parse::<Lines>().into_iter().flatten().count()))
            })
            .collect();
        let listed: String = Outcome::VARIANTS
            .iter()
            .zip(&texts)
            .filter(|(outcome, _)| Outcome::LISTED.contains(outcome))
            .map(|(_, text)| text.as_ref())
            .collect();
        let mut names: Vec<Mutant> = listed.parse::<Lines>().into_iter().flatten().map(Mutant::new).collect();
        names.sort();
        Ok(Report::new(tallies, names))
    }
}

impl Step for ShardSummary {
    fn from_arguments(given: Vec<OsString>) -> Result<ShardSummary, Failure<'static>> {
        ShardSummary::read(Arguments::from(given))?.ended()
    }

    fn carry_out<'step>(&'step self, ports: &'step impl Ports) -> Result<Telling<'step>, Failure<'step>> {
        let report = self.report(ports.files())?;
        let said = vec![Shown::said(format_args!("{report}"))];
        Ok(Telling::reported(said, self.summary.append(ports.files(), report).map(|()| Exit::Success)))
    }
}
