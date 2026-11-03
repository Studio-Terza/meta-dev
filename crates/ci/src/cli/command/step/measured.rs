use std::path::Path;

use text_lines::Lines;

use super::journal::Journal;
use super::seconds::Seconds;
use crate::cli::command::failure::Failure;

#[derive(Clone, Debug)]
pub struct Measured(Journal);

impl Measured {
    const ASSIGN: &'static str = " = ";
    const TAB: &'static str = "\t";

    pub fn file(path: &Path) -> Measured {
        Measured(Journal::file(path))
    }

    pub(crate) fn append(&self, stage: &str, seconds: Seconds) -> Result<(), Failure> {
        self.0.append(format_args!("{stage}{}{seconds}\n", Self::ASSIGN))
    }

    pub(crate) fn row(&self) -> Result<String, Failure> {
        let text = self.0.read()?;
        Ok(Lines::from(text.as_str()).collect::<Vec<&str>>().join(Self::TAB))
    }
}
