use lint_layout::Name;

use crate::boundary::ItemId;
use crate::region::Region;
use crate::sited::Sited;
use crate::type_ref::TypeRef;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Held {
    StructField,
    EnumField,
    UnionField,
    NewtypeField,
    StructParameter,
    EnumParameter,
    UnionParameter,
    Const,
    Static,
    Alias,
    AssociatedType,
    AssociatedConst,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Slot {
    subject: TypeRef,
    interface: Option<Sited>,
}

impl Slot {
    #[must_use]
    pub fn new(subject: TypeRef, interface: Option<Sited>) -> Slot {
        Slot { subject, interface }
    }

    #[must_use]
    pub fn interface(&self) -> Option<&Sited> {
        self.interface.as_ref()
    }
}

impl AsRef<TypeRef> for Slot {
    fn as_ref(&self) -> &TypeRef {
        &self.subject
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Holding {
    name: Name,
    held: Held,
    region: Region,
    declares: Option<ItemId>,
    slot: Option<Slot>,
    ty: TypeRef,
}

impl Holding {
    #[must_use]
    pub fn new(name: Name, held: Held, ty: TypeRef) -> Holding {
        Holding { name, held, region: Region::SHIPPED, declares: None, slot: None, ty }
    }

    #[must_use]
    pub fn placed(self, region: Region, declares: Option<ItemId>, slot: Option<Slot>) -> Holding {
        Holding { name: self.name, held: self.held, region, declares, slot, ty: self.ty }
    }

    #[must_use]
    pub fn held(&self) -> Held {
        self.held
    }

    #[must_use]
    pub fn declares(&self) -> Option<ItemId> {
        self.declares
    }

    #[must_use]
    pub fn slot(&self) -> Option<&Slot> {
        self.slot.as_ref()
    }
}

impl AsRef<Name> for Holding {
    fn as_ref(&self) -> &Name {
        &self.name
    }
}

impl AsRef<TypeRef> for Holding {
    fn as_ref(&self) -> &TypeRef {
        &self.ty
    }
}

impl AsRef<Region> for Holding {
    fn as_ref(&self) -> &Region {
        &self.region
    }
}
