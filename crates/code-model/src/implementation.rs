use lint_layout::Name;

use crate::trait_path::TraitPath;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Origin {
    Written,
    Derived,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Implementation {
    subject: Name,
    implemented: Option<TraitPath>,
    origin: Origin,
}

impl Implementation {
    #[must_use]
    pub fn new(subject: Name, implemented: Option<TraitPath>, origin: Origin) -> Implementation {
        Implementation { subject, implemented, origin }
    }

    #[must_use]
    pub fn written(&self) -> Option<&Implementation> {
        match self.origin {
            Origin::Written => Some(self),
            Origin::Derived => None,
        }
    }

    #[must_use]
    pub fn implemented(&self) -> Option<&TraitPath> {
        self.implemented.as_ref()
    }

    #[must_use]
    pub fn subject(&self) -> &Name {
        &self.subject
    }
}
