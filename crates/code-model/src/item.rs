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
pub struct AliasParameter {
    name: Name,
    default: Option<TypeRef>,
}

impl AliasParameter {
    #[must_use]
    pub fn new(name: Name, default: Option<TypeRef>) -> AliasParameter {
        AliasParameter { name, default }
    }

    pub(crate) fn default(&self) -> Option<&TypeRef> {
        self.default.as_ref()
    }
}

impl AsRef<Name> for AliasParameter {
    fn as_ref(&self) -> &Name {
        &self.name
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Alias {
    parameters: Vec<AliasParameter>,
    target: TypeRef,
}

impl Alias {
    #[must_use]
    pub fn new(parameters: Vec<AliasParameter>, target: TypeRef) -> Alias {
        Alias { parameters, target }
    }

    pub(crate) fn expansion(&self, given: &[TypeRef]) -> TypeRef {
        let arguments = self.parameters.iter().skip(given.len()).map_while(AliasParameter::default).fold(
            given.to_vec(),
            |mut bound, default| {
                let argument = default.substituted(&self.parameters, &bound);
                bound.push(argument);
                bound
            },
        );
        self.target.substituted(&self.parameters, &arguments)
    }
}
