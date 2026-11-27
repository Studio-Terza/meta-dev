use proc_macro2::Ident;
use syn::{UseName, UseRename, UseTree};

#[derive(Clone, Copy, Debug)]
pub(crate) enum UseEnd<'tree> {
    Named(&'tree UseName),
    Renamed(&'tree UseRename),
    Glob,
}

#[derive(Debug)]
pub(crate) struct UseLeaf<'tree> {
    path: Vec<&'tree Ident>,
    end: UseEnd<'tree>,
}

impl<'tree> UseLeaf<'tree> {
    pub(crate) fn of_tree(tree: &'tree UseTree) -> Vec<UseLeaf<'tree>> {
        UseLeaf::under(&[], tree)
    }

    pub(crate) fn path(&self) -> &[&'tree Ident] {
        &self.path
    }

    pub(crate) fn end(&self) -> UseEnd<'tree> {
        self.end
    }

    fn under(path: &[&'tree Ident], tree: &'tree UseTree) -> Vec<UseLeaf<'tree>> {
        let leaf = |end| vec![UseLeaf { path: path.to_vec(), end }];
        match tree {
            UseTree::Path(segment) => {
                let longer: Vec<&'tree Ident> = path.iter().copied().chain([&segment.ident]).collect();
                UseLeaf::under(&longer, &segment.tree)
            }
            UseTree::Group(group) => group.items.iter().flat_map(|item| UseLeaf::under(path, item)).collect(),
            UseTree::Name(name) => leaf(UseEnd::Named(name)),
            UseTree::Rename(rename) => leaf(UseEnd::Renamed(rename)),
            UseTree::Glob(_) => leaf(UseEnd::Glob),
        }
    }
}

impl<'tree> UseEnd<'tree> {
    pub(crate) fn original(self) -> Option<&'tree Ident> {
        match self {
            UseEnd::Named(named) => Some(&named.ident),
            UseEnd::Renamed(renamed) => Some(&renamed.ident),
            UseEnd::Glob => None,
        }
    }

    pub(crate) fn bound(self) -> Option<&'tree Ident> {
        match self {
            UseEnd::Named(named) => Some(&named.ident),
            UseEnd::Renamed(renamed) => Some(&renamed.rename),
            UseEnd::Glob => None,
        }
    }
}
