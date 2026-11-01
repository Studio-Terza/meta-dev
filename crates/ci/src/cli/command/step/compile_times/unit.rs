use serde::Deserialize;

use super::label::Label;
use crate::cli::command::step::seconds::Seconds;

#[derive(Debug, Deserialize)]
pub struct Unit {
    name: Label,
    duration: Seconds,
}

impl Unit {
    pub(super) fn label(&self) -> &Label {
        &self.name
    }

    pub(super) fn took(&self) -> Seconds {
        self.duration
    }
}
