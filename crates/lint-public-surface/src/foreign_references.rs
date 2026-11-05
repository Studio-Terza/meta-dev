mod absolute_path;
mod crate_reach;

use std::collections::BTreeMap;

use syn::visit::Visit;
use syn::{Ident, ItemUse, Macro, Path, UsePath, UseTree};

use super::export::Export;
use lint_core::BinaryFiles;
use lint_core::Macros;
use lint_core::SourceFile;
use lint_layout::Name;
use lint_placement::CrateName;

use absolute_path::AbsolutePath;
use crate_reach::CrateReach;

#[derive(Default)]
pub struct ForeignReferences {
    own_crate: Option<CrateName>,
    crates: BTreeMap<CrateName, CrateReach>,
}

#[derive(Clone, Copy)]
enum Leaf<'tree> {
    Named(&'tree Ident),
    Glob,
}

impl Leaf<'_> {
    fn reached(self, reach: &mut CrateReach) {
        match self {
            Leaf::Named(name) => reach.named(Name::from(name)),
            Leaf::Glob => reach.globbed(),
        }
    }
}

impl ForeignReferences {
    const TESTS: &'static str = "tests";

    pub fn in_file(file: &SourceFile, binaries: &BinaryFiles) -> ForeignReferences {
        let own_crate = binaries
            .holding(file.path())
            .map_or_else(|| Some(CrateName::from(file.placement().crate_root())), |_| None);
        let mut references = ForeignReferences { own_crate, crates: BTreeMap::new() };
        references.visit_file(file.syntax());
        references
    }

    pub fn in_own_tests(file: &SourceFile) -> Option<ForeignReferences> {
        let package = file.placement().package().root();
        file.placement().crate_root().directory().starts_with(package.directory().join(Self::TESTS)).then(|| {
            let tested = CrateName::from(package);
            let mut references = ForeignReferences::default();
            references.visit_file(file.syntax());
            ForeignReferences {
                crates: references
                    .crates
                    .remove_entry(&tested)
                    .map(|(name, reach)| (name, reach.names_only()))
                    .into_iter()
                    .collect(),
                ..ForeignReferences::default()
            }
        })
    }

    pub fn absorb(mut self, other: ForeignReferences) -> ForeignReferences {
        other.crates.into_iter().for_each(|(crate_name, reach)| {
            self.crates.entry(crate_name).or_default().absorb(reach);
        });
        self
    }

    pub fn unnamed<'export>(&self, crate_name: &CrateName, export: &'export Export) -> Option<&'export Export> {
        self.crates.get(crate_name).and_then(|reach| reach.unnamed(export.name())).map(|_| export)
    }

    fn walk(&mut self, first: Option<&Ident>, tree: &UseTree) {
        match tree {
            UseTree::Path(path) => self.walk_path(first, path),
            UseTree::Group(group) => group.items.iter().for_each(|item| self.walk(first, item)),
            UseTree::Name(name) => self.record(first, Leaf::Named(&name.ident)),
            UseTree::Rename(rename) => self.record(first, Leaf::Named(&rename.ident)),
            UseTree::Glob(_) => self.record(first, Leaf::Glob),
        }
    }

    fn record_modules(&mut self, first: &Ident, tree: &UseTree) {
        match tree {
            UseTree::Path(path) => self.record(Some(first), Leaf::Named(&path.ident)),
            UseTree::Group(group) => group.items.iter().for_each(|item| self.record_modules(first, item)),
            UseTree::Name(_) | UseTree::Rename(_) | UseTree::Glob(_) => {}
        }
    }

    fn walk_path(&mut self, first: Option<&Ident>, path: &UsePath) {
        first
            .is_none()
            .then_some(&path.ident)
            .into_iter()
            .for_each(|crate_ident| self.record_modules(crate_ident, &path.tree));
        self.walk(Some(first.unwrap_or(&path.ident)), &path.tree);
    }

    fn record(&mut self, first: Option<&Ident>, leaf: Leaf<'_>) {
        first
            .map(CrateName::from)
            .filter(|crate_name| Some(crate_name) != self.own_crate.as_ref())
            .into_iter()
            .for_each(|crate_name| leaf.reached(self.crates.entry(crate_name).or_default()));
    }
}

impl<'ast> Visit<'ast> for ForeignReferences {
    fn visit_item_use(&mut self, item: &'ast ItemUse) {
        self.walk(None, &item.tree);
    }

    fn visit_macro(&mut self, invocation: &'ast Macro) {
        Macros::walk(invocation, |statement| self.visit_stmt(statement));
        AbsolutePath::walk(&invocation.tokens, |path| {
            path.reach(|crate_ident, item| self.record(Some(crate_ident), Leaf::Named(item)));
        });
        syn::visit::visit_macro(self, invocation);
    }

    fn visit_path(&mut self, path: &'ast Path) {
        path.segments.first().zip(path.segments.iter().nth(1)).into_iter().for_each(|(first, second)| {
            self.record(Some(&first.ident), Leaf::Named(&second.ident));
        });
        syn::visit::visit_path(self, path);
    }
}
