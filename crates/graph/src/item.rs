use std::convert::Infallible;
use std::ops::Range;

use lint_layout::Name;
use newtype::proptest::arbitrary::any;
use newtype::proptest::collection::vec;
use newtype::proptest::strategy::Strategy;

use crate::item_path::ItemPath;
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
    Derive,
    Function,
    Const,
    Static,
    Macro,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Exposure {
    Public,
    Restricted,
    Hidden,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item {
    name: Name,
    kind: ItemKind,
    marks: Vec<ItemPath>,
    krate: Option<Name>,
    exposure: Exposure,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unplaced {
    name: Name,
    kind: ItemKind,
}

impl Unplaced {
    #[must_use]
    pub fn new(name: Name, kind: ItemKind) -> Unplaced {
        Unplaced { name, kind }
    }

    #[must_use]
    pub fn placed(self, marks: Vec<ItemPath>, krate: Option<Name>, exposure: Exposure) -> Item {
        Item { name: self.name, kind: self.kind, marks, krate, exposure }
    }
}

impl Item {
    #[must_use]
    pub fn exposure(&self) -> Exposure {
        self.exposure
    }

    #[must_use]
    pub fn krate(&self) -> Option<&Name> {
        self.krate.as_ref()
    }

    #[must_use]
    pub fn marks(&self) -> &[ItemPath] {
        &self.marks
    }

    #[must_use]
    pub fn kind(&self) -> ItemKind {
        self.kind
    }
}

impl AsRef<Name> for Item {
    fn as_ref(&self) -> &Name {
        &self.name
    }
}

#[derive(newtype::Arbitrary, Clone, Debug, PartialEq, Eq)]
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

#[derive(newtype::Arbitrary, Clone, Debug, PartialEq, Eq)]
#[newtype(generate = Alias::sampled)]
pub struct Alias {
    parameters: Vec<AliasParameter>,
    target: TypeRef,
}

impl Alias {
    const SAMPLED: Range<usize> = 0..3;

    fn sampled() -> impl Strategy<Value = Alias> {
        Strategy::prop_map((vec(any::<AliasParameter>(), Alias::SAMPLED), any::<TypeRef>()), |(parameters, target)| {
            Alias { parameters, target }
        })
    }

    #[must_use]
    pub fn new(parameters: Vec<AliasParameter>, target: TypeRef) -> Alias {
        Alias { parameters, target }
    }

    pub(crate) fn target(&self) -> &TypeRef {
        &self.target
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
    fn substituted(self, ty: &TypeRef) -> TypeRef {
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

    fn path(&self, path: &Sited) -> Sited {
        path.clone()
    }

    fn child(&self, ty: &TypeRef) -> Result<TypeRef, Infallible> {
        match ty.named().and_then(|named| self.bound(named)) {
            Some(bound) => Ok(bound.clone()),
            None => ty.rebuilt(self),
        }
    }
}
