use lint_layout::Name;

use crate::boundary::ItemId;
use crate::region::Region;
use crate::type_ref::TypeRef;

#[derive(newtype::Arbitrary, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Held {
    StructField,
    EnumField,
    UnionField,
    NewtypeField,
    StructParameter,
    Alias,
    Value,
    Bound,
    Variant,
}

#[derive(newtype::Arbitrary, Clone, Debug, PartialEq, Eq)]
pub struct Holding {
    name: Name,
    held: Held,
    region: Region,
    declares: Option<ItemId>,
    ty: TypeRef,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unsited {
    name: Name,
    held: Held,
    ty: TypeRef,
}

impl Unsited {
    #[must_use]
    pub fn new(name: Name, held: Held, ty: TypeRef) -> Unsited {
        Unsited { name, held, ty }
    }

    #[must_use]
    pub fn placed(self, region: Region, declares: Option<ItemId>) -> Holding {
        Holding { name: self.name, held: self.held, region, declares, ty: self.ty }
    }
}

impl Holding {
    #[must_use]
    pub fn held(&self) -> Held {
        self.held
    }

    #[must_use]
    pub fn declares(&self) -> Option<ItemId> {
        self.declares
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
