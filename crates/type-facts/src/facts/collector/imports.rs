use syn::visit::{self, Visit};
use syn::{ItemMod, ItemUse, UsePath, UseTree};

use lint_core::SourceFile;
use lint_layout::Name;
use lint_placement::{CrateName, ModulePath};

use crate::facts::import::Import;

#[derive(Clone, Debug)]
enum Step {
    Root,
    Same,
    Up,
    Down(Name),
}

impl Step {
    const KEYWORDS: [(&'static str, Step); 3] = [("crate", Step::Root), ("self", Step::Same), ("super", Step::Up)];

    fn applied(self, from: ModulePath) -> ModulePath {
        match self {
            Step::Root => ModulePath::ROOT,
            Step::Same => from,
            Step::Up => from.parent(),
            Step::Down(name) => from.child(name),
        }
    }
}

impl From<&Name> for Step {
    fn from(name: &Name) -> Step {
        Step::KEYWORDS
            .iter()
            .find(|(word, _)| *name == *word)
            .map_or_else(|| Step::Down(name.clone()), |(_, step)| step.clone())
    }
}

struct Imported {
    path: Vec<Name>,
    named: Option<Name>,
}

impl Imported {
    fn alone(path: &[Name], named: Option<Name>) -> Vec<Imported> {
        vec![Imported { path: path.to_vec(), named }]
    }

    fn of_tree(tree: &UseTree, prefix: &[Name]) -> Vec<Imported> {
        match tree {
            UseTree::Path(step) => Self::through(step, prefix),
            UseTree::Name(leaf) => Self::alone(prefix, Some(Name::from(&leaf.ident))),
            UseTree::Rename(leaf) => Self::alone(prefix, Some(Name::from(&leaf.ident))),
            UseTree::Glob(_) => Self::alone(prefix, None),
            UseTree::Group(group) => group.items.iter().flat_map(|item| Self::of_tree(item, prefix)).collect(),
        }
    }

    fn through(step: &UsePath, prefix: &[Name]) -> Vec<Imported> {
        let longer: Vec<Name> = prefix.iter().cloned().chain(Some(Name::from(&step.ident))).collect();
        Self::of_tree(&step.tree, &longer)
    }
}

pub(crate) struct Imports<'file> {
    krate: &'file CrateName,
    module: ModulePath,
    found: Vec<Import<'file>>,
}

impl<'file> Imports<'file> {
    pub(crate) fn in_file(file: &'file SourceFile) -> Vec<Import<'file>> {
        let krate = file.placement().package().crate_name();
        let mut scan = Imports { krate, module: file.placement().module().clone(), found: Vec::new() };
        scan.visit_file(file.syntax());
        scan.found
    }

    fn reached(&self, path: &[Name]) -> ModulePath {
        path.iter().map(Step::from).fold(self.module.clone(), |from, step| step.applied(from))
    }
}

impl<'ast> Visit<'ast> for Imports<'_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        let inside = self.module.child(Name::from(&module.ident));
        let outside = std::mem::replace(&mut self.module, inside);
        visit::visit_item_mod(self, module);
        self.module = outside;
    }

    fn visit_item_use(&mut self, found: &'ast ItemUse) {
        let imports: Vec<Import<'_>> = Imported::of_tree(&found.tree, &[])
            .into_iter()
            .map(|imported| Import::new(self.krate, self.module.clone(), self.reached(&imported.path), imported.named))
            .collect();
        self.found.extend(imports);
    }
}
