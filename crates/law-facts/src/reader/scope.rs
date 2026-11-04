use std::collections::BTreeMap;

use syn::{Item, Type, UseTree};

use lint_layout::Name;

#[derive(Clone, Debug, Default)]
pub(crate) struct Scope {
    aliases: BTreeMap<Name, Name>,
    self_type: Option<Name>,
}

impl Scope {
    const SELF: &'static str = "Self";

    pub(crate) fn within(&self, items: &[Item]) -> Scope {
        let mut inner = self.clone();
        inner.aliases.extend(items.iter().flat_map(Self::item_renames));
        inner
    }

    pub(crate) fn implementing(&self, self_type: &Type) -> Scope {
        Scope { aliases: self.aliases.clone(), self_type: self.type_named(self_type) }
    }

    pub(crate) fn named(&self, written: Name) -> Option<Name> {
        let aliased = self.aliases.get(&written).cloned();
        Some(written)
            .filter(|name| *name != Self::SELF)
            .map(|name| aliased.unwrap_or(name))
            .or_else(|| self.self_type.clone())
    }

    pub(crate) fn type_named(&self, found: &Type) -> Option<Name> {
        match found {
            Type::Reference(reference) => self.type_named(&reference.elem),
            Type::Paren(inner) => self.type_named(&inner.elem),
            Type::Group(inner) => self.type_named(&inner.elem),
            Type::Path(_) => Name::of_type(found).and_then(|written| self.named(written)),
            _ => None,
        }
    }

    fn item_renames(item: &Item) -> BTreeMap<Name, Name> {
        match item {
            Item::Use(used) => Self::renames(&used.tree),
            _ => BTreeMap::new(),
        }
    }

    fn renames(tree: &UseTree) -> BTreeMap<Name, Name> {
        match tree {
            UseTree::Path(path) => Self::renames(&path.tree),
            UseTree::Rename(rename) => BTreeMap::from([(Name::from(&rename.rename), Name::from(&rename.ident))]),
            UseTree::Group(group) => group.items.iter().flat_map(Self::renames).collect(),
            UseTree::Name(_) | UseTree::Glob(_) => BTreeMap::new(),
        }
    }
}
