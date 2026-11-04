use std::collections::{BTreeMap, BTreeSet};

use syn::visit::{self, Visit};
use syn::{File, ItemMod, ItemUse, UseTree};

use lint_layout::{Name, Whole, Word, words};
use lint_placement::CrateName;

use crate::endpoint::Endpoint;
use crate::resolved::Resolved;

#[derive(Debug)]
pub(crate) struct Origins<'file> {
    own: &'file CrateName,
    modules: BTreeSet<Name>,
    imported: BTreeMap<Name, Name>,
}

impl<'file> Origins<'file> {
    const LOCAL: [Word<'static>; 3] = words!["crate", "self", "super"];

    pub(crate) fn of_file(own: &'file CrateName, syntax: &File) -> Origins<'file> {
        let mut origins = Origins { own, modules: BTreeSet::new(), imported: BTreeMap::new() };
        origins.visit_file(syntax);
        origins
    }

    pub(crate) fn own(&self) -> &'file CrateName {
        self.own
    }

    pub(crate) fn located(&self, resolved: Resolved) -> Resolved {
        resolved.map_found(|endpoint| self.endpoint(endpoint))
    }

    pub(crate) fn endpoint(&self, endpoint: Endpoint) -> Endpoint {
        let origin =
            endpoint.spelling().and_then(|spelling| spelling.head(&self.imported)).map(|head| self.crate_of(head));
        endpoint.originating(origin)
    }

    fn crate_of(&self, head: &Name) -> CrateName {
        head.matches::<Whole>(&Self::LOCAL)
            .map(|_| ())
            .or_else(|| self.modules.get(head).map(|_| ()))
            .map_or_else(|| CrateName::from(head), |()| self.own.clone())
    }

    fn headed(head: Option<&Name>, found: Name) -> Name {
        head.cloned().unwrap_or(found)
    }

    fn imports(head: Option<&Name>, tree: &UseTree) -> Vec<Import> {
        match tree {
            UseTree::Path(path) => Self::imports(Some(&Self::headed(head, Name::from(&path.ident))), &path.tree),
            UseTree::Name(name) => {
                vec![Import { name: Name::from(&name.ident), head: Self::headed(head, Name::from(&name.ident)) }]
            }
            UseTree::Rename(rename) => {
                vec![Import { name: Name::from(&rename.rename), head: Self::headed(head, Name::from(&rename.ident)) }]
            }
            UseTree::Glob(_) => Vec::new(),
            UseTree::Group(group) => group.items.iter().flat_map(|item| Self::imports(head, item)).collect(),
        }
    }
}

#[derive(Debug)]
struct Import {
    name: Name,
    head: Name,
}

impl<'ast> Visit<'ast> for Origins<'_> {
    fn visit_item_use(&mut self, found: &'ast ItemUse) {
        self.imported.extend(Self::imports(None, &found.tree).into_iter().map(|import| (import.name, import.head)));
    }

    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        self.modules.insert(Name::from(&module.ident));
        visit::visit_item_mod(self, module);
    }
}
