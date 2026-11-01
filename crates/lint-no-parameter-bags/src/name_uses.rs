use std::collections::BTreeSet;

use syn::Ident;
use syn::visit::Visit;

use lint_core::SourceFile;
use lint_layout::Name;
use lint_placement::CrateName;

pub struct NameUses {
    crate_name: Option<CrateName>,
    names: BTreeSet<Name>,
}

impl NameUses {
    pub fn in_file(file: &SourceFile) -> NameUses {
        let mut uses =
            NameUses { crate_name: Some(CrateName::from(file.placement().crate_root())), names: BTreeSet::new() };
        uses.visit_file(file.syntax());
        uses
    }

    pub fn mentions(&self, name: &Name) -> bool {
        self.names.contains(name)
    }

    pub fn shares_crate(&self, other: &NameUses) -> bool {
        self.crate_name == other.crate_name
    }
}

impl<'ast> Visit<'ast> for NameUses {
    fn visit_ident(&mut self, ident: &'ast Ident) {
        self.names.insert(Name::from(ident));
    }
}
