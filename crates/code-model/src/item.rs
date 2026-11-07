use std::convert::Infallible;

use lint_layout::Name;

use crate::resolved::Resolved;
use crate::sited::Sited;
use crate::type_ref::{Named, Rebuild, TypeRef};

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
    pub const fn new(name: Name, kind: ItemKind) -> Self {
        Self { name, kind }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AliasParameter {
    name: Name,
    default: Option<TypeRef>,
}

impl AliasParameter {
    #[must_use]
    pub const fn new(name: Name, default: Option<TypeRef>) -> Self {
        Self { name, default }
    }

    pub(crate) const fn default(&self) -> Option<&TypeRef> {
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
    pub const fn new(parameters: Vec<AliasParameter>, target: TypeRef) -> Self {
        Self { parameters, target }
    }

    pub(crate) fn expansion(&self, given: &[TypeRef]) -> TypeRef {
        let arguments = self.parameters.iter().skip(given.len()).map_while(AliasParameter::default).fold(
            given.to_vec(),
            |mut bound, default| {
                let argument = Substituting { parameters: &self.parameters, arguments: &bound }.substituted(default);
                bound.push(argument);
                bound
            },
        );
        Substituting { parameters: &self.parameters, arguments: &arguments }.substituted(&self.target)
    }
}

#[derive(Clone, Copy, Debug)]
struct Substituting<'given> {
    parameters: &'given [AliasParameter],
    arguments: &'given [TypeRef],
}

impl<'given> Substituting<'given> {
    fn substituted(mut self, ty: &TypeRef) -> TypeRef {
        let Ok(substituted) = self.child(ty);
        substituted
    }

    fn bound(self, named: &Named<Sited>) -> Option<&'given TypeRef> {
        let resolved: &Resolved = named.as_ref();
        let declared = resolved.parameter().and_then(|name| {
            self.parameters.iter().enumerate().find(|(_, parameter)| AsRef::<Name>::as_ref(*parameter) == name)
        });
        declared.and_then(|(at, _)| self.arguments.get(at))
    }
}

impl Rebuild<Sited> for Substituting<'_> {
    type Into = Sited;
    type Fail = Infallible;

    fn path(&mut self, path: &Sited) -> Sited {
        path.clone()
    }

    fn child(&mut self, ty: &TypeRef) -> Result<TypeRef, Infallible> {
        match ty.named().and_then(|named| self.bound(named)) {
            Some(bound) => Ok(bound.clone()),
            None => ty.rebuilt(self),
        }
    }
}
