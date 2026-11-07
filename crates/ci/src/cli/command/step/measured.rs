use std::path::Path;

use text_lines::Lines;

use file_ports::Files;

use super::journal::Journal;
use super::seconds::Seconds;
use super::stage::Stage;
use crate::cli::command::failure::Failure;
use crate::cli::ports::Text;

#[derive(Debug)]
pub struct Measured(Journal);

impl Measured {
    const ASSIGN: &'static str = " = ";
    const TAB: &'static str = "\t";

    pub fn file(path: &Path) -> Self {
        Self(Journal::file(path))
    }

    pub(crate) fn append(&self, files: &impl Files, stage: &Stage, seconds: Seconds) -> Result<(), Failure<'_>> {
        self.0.append(files, format_args!("{stage}{}{seconds}\n", Self::ASSIGN))
    }

    pub(crate) fn row(&self, files: &impl Files) -> Result<Text, Failure<'_>> {
        let text = self.0.read(files)?;
        Ok(Text::new(Lines::from(text.as_ref()).collect::<Vec<&str>>().join(Self::TAB)))
    }
}
