use syn::{Ident, UseName, UseRename, UseTree};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UseEnd<'tree> {
    Named(&'tree UseName),
    Renamed(&'tree UseRename),
    Glob,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UseLeaf<'tree> {
    path: Vec<&'tree Ident>,
    end: UseEnd<'tree>,
}

impl<'tree> UseLeaf<'tree> {
    #[must_use]
    pub fn of_tree(tree: &'tree UseTree) -> Vec<UseLeaf<'tree>> {
        Self::under(Vec::new(), tree)
    }

    #[must_use]
    pub fn path(&self) -> &[&'tree Ident] {
        &self.path
    }

    #[must_use]
    pub fn end(&self) -> UseEnd<'tree> {
        self.end
    }

    fn under(path: Vec<&'tree Ident>, tree: &'tree UseTree) -> Vec<UseLeaf<'tree>> {
        match tree {
            UseTree::Path(segment) => Self::under(path.into_iter().chain([&segment.ident]).collect(), &segment.tree),
            UseTree::Group(group) => group.items.iter().flat_map(|item| Self::under(path.clone(), item)).collect(),
            UseTree::Name(name) => vec![UseLeaf { path, end: UseEnd::Named(name) }],
            UseTree::Rename(rename) => {
                vec![UseLeaf { path, end: UseEnd::Renamed(rename) }]
            }
            UseTree::Glob(_) => vec![UseLeaf { path, end: UseEnd::Glob }],
        }
    }
}

impl<'tree> UseEnd<'tree> {
    #[must_use]
    pub fn bound(self) -> Option<&'tree Ident> {
        match self {
            UseEnd::Named(named) => Some(&named.ident),
            UseEnd::Renamed(renamed) => Some(&renamed.rename),
            UseEnd::Glob => None,
        }
    }

    #[must_use]
    pub fn named(self) -> Option<&'tree UseName> {
        match self {
            UseEnd::Named(named) => Some(named),
            UseEnd::Renamed(_) | UseEnd::Glob => None,
        }
    }

    #[must_use]
    pub fn renamed(self) -> Option<&'tree UseRename> {
        match self {
            UseEnd::Renamed(renamed) => Some(renamed),
            UseEnd::Named(_) | UseEnd::Glob => None,
        }
    }
}
