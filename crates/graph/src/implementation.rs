use lint_layout::Name;

use crate::boundary::ItemId;
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

impl Origin {
    pub fn sorted<Sorted>(self, written: Sorted, derived: Sorted) -> Sorted {
        match self {
            Origin::Written => written,
            Origin::Derived => derived,
        }
    }
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
pub struct Interface {
    sited: Sited,
    arguments: Vec<TypeRef>,
}

impl Interface {
    #[must_use]
    pub fn new(sited: Sited, arguments: Vec<TypeRef>) -> Interface {
        Interface { sited, arguments }
    }

    #[must_use]
    pub fn arguments(&self) -> &[TypeRef] {
        &self.arguments
    }
}

impl AsRef<Sited> for Interface {
    fn as_ref(&self) -> &Sited {
        &self.sited
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Implementation {
    subject: Subject,
    implemented: Option<Interface>,
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unparameterised {
    subject: Subject,
    implemented: Option<Interface>,
    origin: Origin,
    region: Region,
}

impl Unparameterised {
    #[must_use]
    pub fn new(subject: Subject, implemented: Option<Interface>, origin: Origin, region: Region) -> Unparameterised {
        Unparameterised { subject, implemented, origin, region }
    }

    #[must_use]
    pub fn parameterised(self, parameters: Vec<Parameter>, assigned: Vec<Assigned>) -> Implementation {
        let Unparameterised { subject, implemented, origin, region } = self;
        Implementation { subject, implemented, origin, region, parameters, assigned }
    }
}

impl Implementation {
    #[must_use]
    pub fn assigned(&self) -> &[Assigned] {
        &self.assigned
    }

    #[must_use]
    pub fn origin(&self) -> Origin {
        self.origin
    }

    #[must_use]
    pub fn implemented(&self) -> Option<&Resolved> {
        self.interface().map(AsRef::as_ref)
    }

    #[must_use]
    pub fn interface_name(&self) -> Option<&Name> {
        AsRef::<Spelling>::as_ref(self.interface()?).written()?.last()
    }

    #[must_use]
    pub fn interface(&self) -> Option<&Sited> {
        self.implemented.as_ref().map(AsRef::as_ref)
    }

    #[must_use]
    pub fn interface_arguments(&self) -> &[TypeRef] {
        self.implemented.as_ref().map_or(&[], Interface::arguments)
    }

    #[must_use]
    pub fn subject(&self) -> &Name {
        &self.subject.written
    }

    #[must_use]
    pub fn subject_type(&self) -> &TypeRef {
        &self.subject.ty
    }

    #[must_use]
    pub fn subject_item(&self) -> Option<ItemId> {
        let named = self.subject.ty.named()?;
        AsRef::<Resolved>::as_ref(named).crate_item()
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
