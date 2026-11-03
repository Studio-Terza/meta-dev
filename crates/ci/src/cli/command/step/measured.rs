use std::path::Path;

use text_lines::Lines;

use super::journal::Journal;
use super::seconds::Seconds;
use super::stage::Stage;
use crate::cli::command::failure::Failure;
use crate::cli::ports::Text;

#[derive(Clone, Debug)]
pub struct Measured(Journal);

impl Measured {
    const ASSIGN: &'static str = " = ";
    const TAB: &'static str = "\t";

    pub fn file(path: &Path) -> Measured {
        Measured(Journal::file(path))
    }

    pub(crate) fn append(&self, stage: &Stage, seconds: Seconds) -> Result<(), Failure> {
        self.0.append(format_args!("{stage}{}{seconds}\n", Self::ASSIGN))
    }

    pub(crate) fn row(&self) -> Result<Text, Failure> {
        let text = self.0.read()?;
        Ok(Text::new(Lines::from(text.as_ref()).collect::<Vec<&str>>().join(Self::TAB)))
    }
}
