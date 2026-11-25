use std::ffi::OsString;

use text_lines::Lines;

use super::pass::Survivors;
use crate::cli::command::failure::Failure;
use crate::cli::command::step::journal::Journal;
use crate::cli::ports::{Ports, Text};

#[derive(Debug)]
pub(super) struct Selected(Option<Text>);

impl Selected {
    pub(super) fn read<'journal>(
        ports: &impl Ports,
        journal: Option<&'journal Journal>,
    ) -> Result<Selected, Failure<'journal>> {
        let read = Journal::read_present(ports.files(), journal.into_iter().collect());
        read.into_iter().flatten().next().transpose().map(Selected)
    }

    pub(super) fn examined(&self) -> Option<Vec<OsString>> {
        self.0.as_ref().map_or(Some(Vec::new()), |_| {
            Some(Survivors::from(self.named()).examined()).filter(|examined| examined.iter().next().is_some())
        })
    }

    pub(super) fn named(&self) -> Vec<Text> {
        self.0
            .as_ref()
            .map_or_default(|names| names.as_ref().parse::<Lines>().into_iter().flatten().map(Text::new).collect())
    }
}
