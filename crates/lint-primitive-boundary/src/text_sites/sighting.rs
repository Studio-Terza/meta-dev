use std::path::Path;

use syn::Ident;

use crate::finding::Finding;
use crate::owner::Owner;
use crate::reach::{Reach, Reached};
use lint_core::Site;
use lint_core::Violation;
use lint_layout::LineNumber;
use lint_layout::Name;

pub struct Sighting {
    owner: Owner,
    reach: Reach,
    handler: Ident,
}

impl Sighting {
    pub fn new(owner: Owner, reach: Reach, handler: Ident) -> Sighting {
        Sighting { owner, reach, handler }
    }

    pub fn outside(self, doors: &[Owner]) -> Option<Sighting> {
        let free = (self.owner == Owner::Free).then_some(());
        let undoored = doors.iter().all(|door| *door != self.owner).then_some(());
        free.or(undoored).map(|()| self)
    }

    pub fn into_reached(self, path: &Path) -> Reached<'_> {
        Reached::new(
            self.reach,
            Violation::new(
                Site::new(path, LineNumber::from(self.handler.span())),
                Finding::TextOutsideDoor { handler: Name::from(&self.handler) },
            ),
        )
    }
}
