use lint_layout::Name;

use crate::region::Region;
use crate::resolved::Resolved;
use crate::sited::Sited;
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
    implemented: Option<Sited>,
    origin: Origin,
    region: Region,
}

impl Implementation {
    #[must_use]
    pub fn new(subject: Subject, implemented: Option<Sited>, origin: Origin, region: Region) -> Implementation {
        Implementation { subject, implemented, origin, region }
    }

    #[must_use]
    pub fn written(&self) -> Option<&Implementation> {
        match self.origin {
            Origin::Written => Some(self),
            Origin::Derived => None,
        }
    }

    #[must_use]
    pub fn origin(&self) -> Origin {
        self.origin
    }

    #[must_use]
    pub fn implemented(&self) -> Option<&Resolved> {
        self.implemented.as_ref().map(AsRef::as_ref)
    }

    #[must_use]
    pub fn interface(&self) -> Option<&Sited> {
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

impl AsRef<Region> for Implementation {
    fn as_ref(&self) -> &Region {
        &self.region
    }
}
