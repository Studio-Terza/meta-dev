use std::collections::BTreeMap;

use syn::{Item, Stmt, Type, UsePath, UseTree};

use lint_layout::Name;

#[derive(Clone, Debug)]
pub(crate) struct Import {
    name: Name,
    root: Option<Name>,
}

impl Import {
    pub(crate) fn root(&self) -> Option<&Name> {
        self.root.as_ref()
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Scope {
    imports: BTreeMap<Name, Import>,
    self_type: Option<Name>,
}

impl Scope {
    const SELF: &'static str = "Self";

    pub(crate) fn within<'item>(&self, items: impl Iterator<Item = &'item Item>) -> Scope {
        let mut inner = self.clone();
        inner.imports.extend(items.flat_map(Self::item_imports));
        inner
    }

    pub(crate) fn within_statements(&self, statements: &[Stmt]) -> Scope {
        self.within(statements.iter().filter_map(Self::statement_item))
    }

    pub(crate) fn implementing(&self, self_type: &Type) -> Scope {
        Scope { imports: self.imports.clone(), self_type: self.type_named(self_type) }
    }

    pub(crate) fn named(&self, written: &Name) -> Option<Name> {
        Some(written)
            .filter(|name| **name != Self::SELF)
            .map(|name| self.import(name).map_or(name, |import| &import.name))
            .or(self.self_type.as_ref())
            .cloned()
    }

    pub(crate) fn import(&self, written: &Name) -> Option<&Import> {
        self.imports.get(written)
    }

    pub(crate) fn type_named(&self, found: &Type) -> Option<Name> {
        match found {
            Type::Reference(reference) => self.type_named(&reference.elem),
            Type::Paren(inner) => self.type_named(&inner.elem),
            Type::Path(_) => Name::of_type(found).and_then(|written| self.named(&written)),
            _ => None,
        }
    }

    fn statement_item(statement: &Stmt) -> Option<&Item> {
        if let Stmt::Item(item) = statement { Some(item) } else { None }
    }

    fn item_imports(item: &Item) -> BTreeMap<Name, Import> {
        match item {
            Item::Use(used) => Self::imports(&used.tree, None),
            _ => BTreeMap::new(),
        }
    }

    fn imports(tree: &UseTree, root: Option<&Name>) -> BTreeMap<Name, Import> {
        match tree {
            UseTree::Path(path) => Self::path_imports(path, root),
            UseTree::Rename(rename) => BTreeMap::from([(
                Name::from(&rename.rename),
                Import { name: Name::from(&rename.ident), root: root.cloned() },
            )]),
            UseTree::Name(named) => BTreeMap::from([(
                Name::from(&named.ident),
                Import { name: Name::from(&named.ident), root: root.cloned() },
            )]),
            UseTree::Group(group) => group.items.iter().flat_map(|item| Self::imports(item, root)).collect(),
            UseTree::Glob(_) => BTreeMap::new(),
        }
    }

    fn path_imports(path: &UsePath, root: Option<&Name>) -> BTreeMap<Name, Import> {
        let first = Name::from(&path.ident);
        Self::imports(&path.tree, Some(root.unwrap_or(&first)))
    }
}
