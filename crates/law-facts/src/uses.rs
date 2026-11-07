mod reach;

use std::collections::{BTreeMap, BTreeSet};
use std::iter::once;

use syn::{Ident, UsePath, UseTree};

use lint_layout::Name;
use lint_placement::CrateName;
use reach::Reach;

#[derive(Debug)]
pub(crate) struct Place<'uses> {
    reach: Reach,
    name: &'uses Name,
}

impl<'uses> Place<'uses> {
    pub(crate) fn routed<T>(
        &self,
        home: impl FnOnce(&'uses Name) -> T,
        owned: impl FnOnce(&CrateName, &'uses Name) -> T,
    ) -> T {
        self.reach.routed(|| home(self.name), |owner| owned(owner, self.name))
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Import {
    name: Name,
    root: Option<Name>,
}

impl Import {
    pub(crate) fn name(&self) -> &Name {
        &self.name
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Uses {
    named: BTreeMap<Name, Import>,
    globs: BTreeSet<Name>,
}

impl Uses {
    pub(crate) fn of_tree(tree: &UseTree) -> Self {
        Self::tree(tree, None)
    }

    pub(crate) fn merged(mut self, other: Self) -> Self {
        self.absorb(other);
        self
    }

    pub(crate) fn absorb(&mut self, other: Self) {
        self.named.extend(other.named);
        self.globs.extend(other.globs);
    }

    pub(crate) fn import(&self, written: &Name) -> Option<&Import> {
        self.named.get(written)
    }

    pub(crate) fn qualified<'uses>(&'uses self, first: &'uses Name, last: &'uses Name) -> Place<'uses> {
        let root = self.named.get(first).map_or(first, |import| import.root.as_ref().unwrap_or(&import.name));
        Place { reach: Reach::of_root(root), name: last }
    }

    pub(crate) fn sources<'uses>(&'uses self, written: &'uses Name) -> Vec<Place<'uses>> {
        self.named.get(written).map_or_else(
            || {
                once(Reach::Home)
                    .chain(self.globs.iter().map(Reach::of_root))
                    .map(|reach| Place { reach, name: written })
                    .collect()
            },
            |import| {
                Vec::from([Place {
                    reach: import.root.as_ref().map_or(Reach::Home, Reach::of_root),
                    name: &import.name,
                }])
            },
        )
    }

    fn tree(tree: &UseTree, root: Option<&Name>) -> Self {
        match tree {
            UseTree::Path(path) => Self::path(path, root),
            UseTree::Rename(rename) => Self::single(&rename.rename, &rename.ident, root),
            UseTree::Name(named) => Self::single(&named.ident, &named.ident, root),
            UseTree::Group(group) => {
                group.items.iter().map(|item| Self::tree(item, root)).fold(Self::default(), Self::merged)
            }
            UseTree::Glob(_) => Self { named: BTreeMap::new(), globs: root.into_iter().cloned().collect() },
        }
    }

    fn single(written: &Ident, name: &Ident, root: Option<&Name>) -> Self {
        let import = Import { name: Name::from(name), root: root.cloned() };
        Self { named: BTreeMap::from([(Name::from(written), import)]), globs: BTreeSet::new() }
    }

    fn path(path: &UsePath, root: Option<&Name>) -> Self {
        let first = Name::from(&path.ident);
        Self::tree(&path.tree, Some(root.unwrap_or(&first)))
    }
}
