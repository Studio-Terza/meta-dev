use std::fmt;

use super::super::Toward;
use super::super::member::Member;
use super::listing::{Listed, Listing};
use super::named::Named;

#[derive(Debug)]
pub(super) struct Neighbours<'view, 'found, 'file> {
    named: Named<'view, 'found, 'file>,
    toward: Toward,
    listed: Listed,
}

impl<'view, 'found, 'file> Neighbours<'view, 'found, 'file> {
    pub(super) const fn new(named: Named<'view, 'found, 'file>, toward: Toward, listed: Listed) -> Self {
        Neighbours { named, toward, listed }
    }

    fn listing(&self, member: Member<'found>) -> Listing<'view, 'found, 'file> {
        let view = self.named.view();
        Listing::new(view, view.adjacent(member, self.toward), view.outside(member, self.toward), self.listed)
    }
}

impl fmt::Display for Neighbours<'_, '_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.named.found() {
            Err(missing) => write!(formatter, "{missing}"),
            Ok(found) => found.iter().try_for_each(|member| {
                write!(formatter, "  {}\n{}", self.named.placed(*member), self.listing(*member))
            }),
        }
    }
}
