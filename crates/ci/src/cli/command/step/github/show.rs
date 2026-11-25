use crate::cli::command::Telling;
use console_ports::Shown;
use std::ffi::OsString;
use std::fmt;
use std::iter;
use std::path::PathBuf;

use file_ports::Files;

use super::super::journal::Journal;
use super::Step;
use crate::cli::command::failure::Failure;
use crate::cli::command::step::{Arguments, Ended, Journaled, Word};
use crate::cli::ports::{Exit, Ports, Text};
use grammar_output::Record;

#[derive(Debug)]
pub(super) struct Shows<'journal>(Vec<Entry<'journal>>);

#[derive(Debug)]
struct Entry<'journal> {
    journal: &'journal Journal,
    text: Text,
}

#[derive(Debug)]
pub struct Show {
    journals: Vec<Journal>,
}

impl Show {
    const ABSENT: &'static str = "not produced\n";

    pub(super) fn shown<'journal>(
        files: &impl Files,
        journals: &'journal [Journal],
    ) -> Result<Shows<'journal>, Failure<'journal>> {
        let entries = journals
            .iter()
            .zip(Journal::read_present(files, journals.iter().collect()))
            .map(|(journal, read)| {
                let text = read.unwrap_or_else(|| Ok(Text::new(Self::ABSENT.to_owned())))?;
                Ok(Entry { journal, text })
            })
            .collect::<Result<_, Failure<'journal>>>()?;
        Ok(Shows(entries))
    }
}

#[derive(Debug, Record)]
#[record(source = Arguments)]
struct ShownWords {
    #[field(Journaled)]
    first: Journal,
    #[repeated(Word)]
    rest: Vec<OsString>,
}

impl From<ShownWords> for Show {
    fn from(words: ShownWords) -> Show {
        let rest = words.rest.into_iter().map(|file| Journal::from(PathBuf::from(file)));
        Show { journals: iter::once(words.first).chain(rest).collect() }
    }
}

impl Step for Show {
    fn from_arguments(given: Vec<OsString>) -> Result<Show, Failure<'static>> {
        ShownWords::read(Arguments::from(given))?.ended().map(Show::from)
    }

    fn carry_out<'step>(&'step self, ports: &'step impl Ports) -> Result<Telling<'step>, Failure<'step>> {
        let shows = Self::shown(ports.files(), &self.journals)?;
        Ok(Telling::reported(vec![Shown::said(shows)], Ok(Exit::Success)))
    }
}

impl fmt::Display for Shows<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.iter().try_for_each(|entry| write!(formatter, "{}:\n{}", entry.journal, entry.text))
    }
}
