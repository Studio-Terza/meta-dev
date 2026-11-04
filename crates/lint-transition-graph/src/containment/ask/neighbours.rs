use std::fmt;

use super::super::Toward;
use super::listing::{Listed, Listing};
use super::named::Named;

#[derive(Debug)]
pub(super) struct Neighbours<'view, 'found, 'file> {
    named: Named<'view, 'found, 'file>,
    toward: Toward,
    listed: Listed,
}

impl<'view, 'found, 'file> Neighbours<'view, 'found, 'file> {
    pub(super) fn new(
        named: Named<'view, 'found, 'file>,
        toward: Toward,
        listed: Listed,
    ) -> Neighbours<'view, 'found, 'file> {
        Neighbours { named, toward, listed }
    }
}

impl fmt::Display for Neighbours<'_, '_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let view = self.named.view();
        self.named.each(formatter, |formatter, member| {
            write!(formatter, "{}", Listing::new(view, view.adjacent(member, self.toward), self.listed))
        })
    }
}
