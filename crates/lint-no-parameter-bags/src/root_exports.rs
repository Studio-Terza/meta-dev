use std::collections::{BTreeMap, BTreeSet};

use syn::{UseTree, Visibility};

use lint_core::{Parsed, SourceFile, Syntax};
use lint_layout::Name;
use lint_placement::CrateName;
use lint_placement::Role;

#[derive(Debug, Default)]
pub struct RootExports(BTreeMap<CrateName, BTreeSet<Name>>);

impl RootExports {
    pub fn exported(&self, file: &SourceFile, name: &Name) -> Option<&Name> {
        self.0.get(&CrateName::from(file.head().placement().crate_root()))?.get(name)
    }

    fn record(&mut self, file: &SourceFile) {
        let names = file
            .syntax()
            .items
            .iter()
            .filter_map(Syntax::item_use)
            .filter(|used| matches!(used.vis, Visibility::Public(_)))
            .flat_map(|used| Self::leaves(&used.tree));
        self.0.entry(CrateName::from(file.head().placement().crate_root())).or_default().extend(names);
    }

    fn leaves(tree: &UseTree) -> Vec<Name> {
        match tree {
            UseTree::Path(path) => Self::leaves(&path.tree),
            UseTree::Group(group) => group.items.iter().flat_map(Self::leaves).collect(),
            UseTree::Name(name) => vec![Name::from(&name.ident)],
            UseTree::Rename(rename) => vec![Name::from(&rename.rename)],
            UseTree::Glob(_) => Vec::new(),
        }
    }
}

impl From<Parsed<'_, '_>> for RootExports {
    fn from(parsed: Parsed<'_, '_>) -> RootExports {
        let exports = RootExports::default();
        let root = Some(parsed.file()).filter(|file| file.head().placement().role() == Role::LibraryRoot);
        root.into_iter().fold(exports, |mut found, file| {
            found.record(file);
            found
        })
    }
}

impl FromIterator<RootExports> for RootExports {
    fn from_iter<Parts: IntoIterator<Item = RootExports>>(parts: Parts) -> RootExports {
        RootExports(parts.into_iter().flat_map(|part| part.0).fold(
            BTreeMap::new(),
            |mut merged: BTreeMap<CrateName, BTreeSet<Name>>, (crate_name, names)| {
                merged.entry(crate_name).or_default().extend(names);
                merged
            },
        ))
    }
}
