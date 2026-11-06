use lint_layout::Name;

use crate::resolved::Resolved;
use crate::type_ref::TypeRef;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Origin {
    Written,
    Derived,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Subject {
    written: Name,
    ty: TypeRef,
}

impl Subject {
    #[must_use]
    pub fn new(written: Name, ty: TypeRef) -> Subject {
        Subject { written, ty }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Implementation {
    subject: Subject,
    implemented: Option<Resolved>,
    origin: Origin,
}

impl Implementation {
    #[must_use]
    pub fn new(subject: Subject, implemented: Option<Resolved>, origin: Origin) -> Implementation {
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
        &self.subject.written
    }

    #[must_use]
    pub fn subject_type(&self) -> &TypeRef {
        &self.subject.ty
    }
}
