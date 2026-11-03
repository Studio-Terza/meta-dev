use std::path::Path;

use syn::Ident;

use crate::finding::Finding;
use crate::owner::Owner;
use lint_core::Site;
use lint_core::Violation;
use lint_layout::LineNumber;
use lint_layout::Name;

pub struct Sighting {
    owner: Owner,
    handler: Ident,
}

impl Sighting {
    pub fn new(owner: Owner, handler: Ident) -> Sighting {
        Sighting { owner, handler }
    }

    pub fn outside(self, doors: &[Owner]) -> Option<Sighting> {
        Some(self).filter(|sighting| sighting.owner == Owner::Free || !doors.contains(&sighting.owner))
    }

    pub fn into_violation(self, path: &Path) -> Violation<'_> {
        Violation::new(
            Site::new(path, LineNumber::from(self.handler.span())),
            Finding::TextOutsideDoor { handler: Name::from(&self.handler) },
        )
    }
}
