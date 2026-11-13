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
        let mut uses = NameUses {
            crate_name: Some(CrateName::from(file.head().placement().crate_root())),
            names: BTreeSet::new(),
        };
        uses.visit_file(file.syntax());
        uses
    }

    pub fn mentions(&self, name: &Name) -> Option<&Name> {
        self.names.get(name)
    }

    pub fn sharing_crate(&self, other: &NameUses) -> Option<&NameUses> {
        (self.crate_name == other.crate_name).then_some(self)
    }
}

impl<'ast> Visit<'ast> for NameUses {
    fn visit_ident(&mut self, ident: &'ast Ident) {
        self.names.insert(Name::from(ident));
    }
}
