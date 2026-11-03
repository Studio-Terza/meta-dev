use std::collections::{BTreeMap, BTreeSet};

use syn::visit::{self, Visit};
use syn::{File, ItemMod, ItemUse, Path, UseTree};

use lint_layout::{Name, Whole, Word, words};
use lint_placement::CrateName;

#[derive(Debug)]
pub(super) struct Origins {
    own: CrateName,
    modules: BTreeSet<Name>,
    imported: BTreeMap<Name, Name>,
}

impl Origins {
    const LOCAL: [Word<'static>; 3] = words!["crate", "self", "super"];

    pub(super) fn of_file(own: CrateName, syntax: &File) -> Origins {
        let mut origins = Origins { own, modules: BTreeSet::new(), imported: BTreeMap::new() };
        origins.visit_file(syntax);
        origins
    }

    pub(super) fn own(&self) -> &CrateName {
        &self.own
    }

    pub(super) fn of_path(&self, path: &Path) -> Option<CrateName> {
        let first = Name::from(&path.segments.first()?.ident);
        match path.segments.len() {
            1 => self.imported.get(&first).map(|head| self.crate_of(head)),
            _ => Some(self.crate_of(&first)),
        }
    }

    fn crate_of(&self, head: &Name) -> CrateName {
        match (head.matches::<Whole>(&Self::LOCAL), self.modules.contains(head)) {
            (Some(_), _) | (None, true) => self.own.clone(),
            (None, false) => CrateName::from(head),
        }
    }

    fn imports(head: Option<&Name>, tree: &UseTree) -> Vec<Import> {
        let from = |found: Name| head.cloned().unwrap_or(found);
        match tree {
            UseTree::Path(path) => Self::imports(Some(&from(Name::from(&path.ident))), &path.tree),
            UseTree::Name(name) => vec![Import { name: Name::from(&name.ident), head: from(Name::from(&name.ident)) }],
            UseTree::Rename(rename) => {
                vec![Import { name: Name::from(&rename.rename), head: from(Name::from(&rename.ident)) }]
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

impl<'ast> Visit<'ast> for Origins {
    fn visit_item_use(&mut self, found: &'ast ItemUse) {
        self.imported.extend(Self::imports(None, &found.tree).into_iter().map(|import| (import.name, import.head)));
    }

    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        self.modules.insert(Name::from(&module.ident));
        visit::visit_item_mod(self, module);
    }
}
