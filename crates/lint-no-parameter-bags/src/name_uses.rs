use std::collections::BTreeSet;

use syn::Ident;
use syn::visit::Visit;

use lint_core::SourceFile;
use lint_layout::Name;

#[derive(Default)]
pub struct NameUses(BTreeSet<Name>);

impl NameUses {
    pub fn in_file(file: &SourceFile) -> NameUses {
        let mut uses = NameUses::default();
        uses.visit_file(file.syntax());
        uses
    }

    pub fn mentions(&self, name: &Name) -> bool {
        self.0.contains(name)
    }
}

impl<'ast> Visit<'ast> for NameUses {
    fn visit_ident(&mut self, ident: &'ast Ident) {
        self.0.insert(Name::from(ident));
    }
}
