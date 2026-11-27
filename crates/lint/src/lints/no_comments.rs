mod boundary;

use crate::finding::{Found, Rule};
use crate::source::Source;
use crate::style::{Reads, Style};
use crate::terms::Fix;

#[derive(Debug)]
pub(crate) struct NoComments<'file> {
    source: &'file Source,
}

impl NoComments<'_> {
    const FIX: Fix = Fix("delete it and say the same in the names of the code");
}

impl<'file> Style<'file> for NoComments<'file> {
    const RULE: Rule = Rule::NoComments;
    const READS: Reads = Reads::Every;

    fn new(source: &'file Source) -> NoComments<'file> {
        NoComments { source }
    }

    fn found(self) -> Vec<Found> {
        NoComments::commented(self.source.text())
            .into_iter()
            .map(|line| Found::at(line, "comment", NoComments::FIX))
            .collect()
    }
}
