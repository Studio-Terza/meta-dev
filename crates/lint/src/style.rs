use crate::finding::{Found, Rule};
use crate::source::{Place, Source};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Reads {
    Every,
    Shipped,
}

impl Reads {
    pub(crate) fn admits(self, source: &Source) -> Option<()> {
        match self {
            Reads::Every => Some(()),
            Reads::Shipped => (source.place() == Place::Shipped).then_some(()),
        }
    }
}

pub(crate) trait Style<'file>: Sized {
    const RULE: Rule;
    const READS: Reads;

    fn new(source: &'file Source) -> Self;

    fn found(self) -> Vec<Found>;
}
