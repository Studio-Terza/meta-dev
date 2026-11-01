use syn::visit::{self, Visit};
use syn::{ItemMod, TraitBound};

use lint_core::{SourceFile, Walk};
use lint_layout::Name;

pub(crate) struct Bounds {
    names: Vec<Name>,
}

impl Bounds {
    pub(crate) fn of_file(file: &SourceFile) -> Vec<Name> {
        let mut scan = Bounds { names: Vec::new() };
        scan.visit_file(file.syntax());
        scan.names
    }
}

impl<'ast> Visit<'ast> for Bounds {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        Walk::enter(&module.attrs, || visit::visit_item_mod(self, module));
    }

    fn visit_trait_bound(&mut self, bound: &'ast TraitBound) {
        self.names.extend(bound.path.segments.last().map(|segment| Name::from(&segment.ident)));
        visit::visit_trait_bound(self, bound);
    }
}
