use syn::visit::Visit;
use syn::{File, Ident, ItemUse, UseRename, UseTree};

use lint_layout::Name;

#[derive(Debug, Default)]
pub(super) struct Renames<'ast> {
    found: Vec<&'ast UseRename>,
}

impl<'ast> Renames<'ast> {
    pub(super) fn in_file(file: &'ast File) -> Renames<'ast> {
        let mut renames = Renames::default();
        renames.visit_file(file);
        renames
    }

    pub(super) fn resolved(&self, ident: &Ident) -> Name {
        self.found
            .iter()
            .find(|rename| rename.rename == *ident)
            .map_or_else(|| Name::from(ident), |rename| Name::from(&rename.ident))
    }

    fn collected(tree: &'ast UseTree) -> Vec<&'ast UseRename> {
        match tree {
            UseTree::Path(path) => Self::collected(&path.tree),
            UseTree::Rename(rename) => vec![rename],
            UseTree::Group(group) => group.items.iter().flat_map(Self::collected).collect(),
            UseTree::Name(_) | UseTree::Glob(_) => Vec::new(),
        }
    }
}

impl<'ast> Visit<'ast> for Renames<'ast> {
    fn visit_item_use(&mut self, item: &'ast ItemUse) {
        self.found.extend(Self::collected(&item.tree));
    }
}
