use serde::Deserialize;

use crate::cli::command::step::seconds::Seconds;
use text_lines::OneLine;

#[derive(Debug, Deserialize)]
pub struct Unit {
    name: OneLine,
    duration: Seconds,
}

impl Unit {
    pub(super) fn label(&self) -> &OneLine {
        &self.name
    }

    pub(super) fn took(&self) -> Seconds {
        self.duration
    }
}
