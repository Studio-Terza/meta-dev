use lint_layout::Name;

use crate::resolved::Resolved;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Origin {
    Written,
    Derived,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Implementation {
    subject: Name,
    implemented: Option<Resolved>,
    origin: Origin,
}

impl Implementation {
    #[must_use]
    pub fn new(subject: Name, implemented: Option<Resolved>, origin: Origin) -> Implementation {
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
    pub fn implemented(&self) -> Option<&Resolved> {
        self.implemented.as_ref()
    }

    #[must_use]
    pub fn subject(&self) -> &Name {
        &self.subject
    }
}
