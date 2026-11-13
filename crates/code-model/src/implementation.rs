use lint_layout::Name;

use crate::function::Parameter;
use crate::region::Region;
use crate::resolved::Resolved;
use crate::sited::{Sited, Spelling};
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
    parameters: Vec<Parameter>,
    assigned: Vec<Assigned>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Assigned {
    name: Name,
    ty: TypeRef,
}

impl Assigned {
    #[must_use]
    pub fn new(name: Name, ty: TypeRef) -> Assigned {
        Assigned { name, ty }
    }
}

impl AsRef<Name> for Assigned {
    fn as_ref(&self) -> &Name {
        &self.name
    }
}

impl AsRef<TypeRef> for Assigned {
    fn as_ref(&self) -> &TypeRef {
        &self.ty
    }
}

impl Implementation {
    #[must_use]
    pub fn new(subject: Subject, implemented: Option<Sited>, origin: Origin, region: Region) -> Implementation {
        Implementation { subject, implemented, origin, region, parameters: Vec::new(), assigned: Vec::new() }
    }

    #[must_use]
    pub fn parameterised(self, parameters: Vec<Parameter>, assigned: Vec<Assigned>) -> Implementation {
        Implementation { parameters, assigned, ..self }
    }

    #[must_use]
    pub fn assigned(&self) -> &[Assigned] {
        &self.assigned
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
    pub fn interface_name(&self) -> Option<&Name> {
        AsRef::<Spelling>::as_ref(self.implemented.as_ref()?).written()?.last()
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

impl AsRef<[Parameter]> for Implementation {
    fn as_ref(&self) -> &[Parameter] {
        &self.parameters
    }
}
