use std::collections::BTreeMap;

use syn::visit::Visit;
use syn::{ItemType, ItemUse, Path, Type, UseTree};

use lint_layout::Name;
use lint_layout::UsePath;

#[derive(Default)]
pub struct UseAliases(BTreeMap<Name, UsePath>);

impl UseAliases {
    pub fn in_file(file: &syn::File) -> UseAliases {
        let mut aliases = UseAliases::default();
        aliases.visit_file(file);
        aliases
    }

    pub fn resolve(&self, path: &Path) -> UsePath {
        let spelled = UsePath::from(path);
        match spelled.first().and_then(|first| self.0.get(first)) {
            Some(full) => spelled.rerooted(full),
            None => spelled,
        }
    }

    fn walk(&mut self, prefix: &UsePath, tree: &UseTree) {
        match tree {
            UseTree::Path(path) => {
                self.walk(&prefix.extended(Name::from(&path.ident)), &path.tree);
            }
            UseTree::Group(group) => group.items.iter().for_each(|item| self.walk(prefix, item)),
            UseTree::Name(name) => {
                self.0.insert(Name::from(&name.ident), prefix.extended(Name::from(&name.ident)));
            }
            UseTree::Rename(rename) => {
                self.0.insert(Name::from(&rename.rename), prefix.extended(Name::from(&rename.ident)));
            }
            UseTree::Glob(_) => {}
        }
    }
}

impl<'ast> Visit<'ast> for UseAliases {
    fn visit_item_use(&mut self, item: &'ast ItemUse) {
        self.walk(&UsePath::default(), &item.tree);
    }

    fn visit_item_type(&mut self, item: &'ast ItemType) {
        self.aliased(item);
    }
}

impl UseAliases {
    fn aliased(&mut self, item: &ItemType) {
        let target = match &*item.ty {
            Type::Path(target) if target.qself.is_none() => Some(target),
            _ => None,
        };
        self.0.extend(target.map(|target| (Name::from(&item.ident), UsePath::from(&target.path))));
    }
}
