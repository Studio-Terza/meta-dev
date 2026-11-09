use serde::Deserialize;
use text_lines::OneLine;

use super::number::Number;
use super::title::Title;
use newtype::Wrapping;

#[derive(Debug, Deserialize)]
pub struct Listed {
    number: Number,
    title: OneLine,
}

impl Listed {
    pub(super) fn numbered(&self, title: Title<'_>) -> Option<Number> {
        (self.title.as_ref() == title.into_inner()).then_some(self.number)
    }
}
