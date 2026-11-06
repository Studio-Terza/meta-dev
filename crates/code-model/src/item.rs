use lint_layout::Name;

use crate::type_ref::TypeRef;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ItemKind {
    Struct,
    Enum,
    Union,
    Trait,
    Alias,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item {
    name: Name,
    kind: ItemKind,
}

impl Item {
    #[must_use]
    pub fn new(name: Name, kind: ItemKind) -> Item {
        Item { name, kind }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Alias {
    parameters: Vec<Name>,
    target: TypeRef,
}

impl Alias {
    #[must_use]
    pub fn new(parameters: Vec<Name>, target: TypeRef) -> Alias {
        Alias { parameters, target }
    }

    pub(crate) fn parameters(&self) -> &[Name] {
        &self.parameters
    }

    pub(crate) fn target(&self) -> &TypeRef {
        &self.target
    }
}
