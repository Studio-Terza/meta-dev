use lint_layout::Name;

use crate::boundary::ItemId;
use crate::region::Region;
use crate::type_ref::TypeRef;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Held {
    StructField,
    EnumField,
    UnionField,
    NewtypeField,
    StructParameter,
    Alias,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Holding {
    name: Name,
    held: Held,
    region: Region,
    declares: Option<ItemId>,
    ty: TypeRef,
}

impl Holding {
    #[must_use]
    pub const fn new(name: Name, held: Held, ty: TypeRef) -> Self {
        Self { name, held, region: Region::SHIPPED, declares: None, ty }
    }

    #[must_use]
    pub fn placed(self, region: Region, declares: Option<ItemId>) -> Self {
        Self { name: self.name, held: self.held, region, declares, ty: self.ty }
    }

    #[must_use]
    pub const fn held(&self) -> Held {
        self.held
    }

    #[must_use]
    pub const fn declares(&self) -> Option<ItemId> {
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
