use std::path::Path;

use syn::Ident;

use super::owner::Owner;
use crate::finding::Finding;
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

    pub fn outside(&self, doors: &[Owner]) -> bool {
        self.owner == Owner::Free || !doors.contains(&self.owner)
    }

    pub fn into_violation(self, path: &Path) -> Violation {
        Violation::new(
            Site::new(path, LineNumber::from(self.handler.span())),
            Finding::TextOutsideDoor { handler: Name::from(&self.handler) },
        )
    }
}
