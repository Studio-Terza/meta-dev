use std::path::Path;

use super::journal::Journal;
use super::seconds::Seconds;
use crate::cli::command::failure::Failure;

#[derive(Clone, Debug)]
pub struct Measured(Journal);

impl Measured {
    pub fn file(path: &Path) -> Measured {
        Measured(Journal::file(path))
    }

    pub(crate) fn append(&self, stage: &str, seconds: Seconds) -> Result<(), Failure> {
        self.0.append(format_args!("{stage} = {seconds}\n"))
    }
}
