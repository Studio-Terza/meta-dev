use serde::Deserialize;
use text_lines::OneLine;

use super::number::Number;

#[derive(Debug, Deserialize)]
pub struct Listed {
    number: Number,
    title: OneLine,
}

impl Listed {
    pub(super) fn numbered(&self, title: &str) -> Option<Number> {
        (self.title.as_ref() == title).then_some(self.number)
    }
}
