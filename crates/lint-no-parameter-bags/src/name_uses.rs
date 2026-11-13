use std::collections::BTreeSet;
use std::path::Path;

use syn::Ident;
use syn::visit::Visit;

use lint_core::Parsed;
use lint_layout::Name;
use lint_placement::CrateName;

#[derive(Debug)]
pub struct NameUses<'head> {
    path: &'head Path,
    crate_name: CrateName,
    names: BTreeSet<Name>,
}

impl<'head> From<Parsed<'_, 'head>> for NameUses<'head> {
    fn from(parsed: Parsed<'_, 'head>) -> NameUses<'head> {
        let head = parsed.head();
        let mut uses = NameUses {
            path: head.path(),
            crate_name: CrateName::from(head.placement().crate_root()),
            names: BTreeSet::new(),
        };
        uses.visit_file(parsed.file().syntax());
        uses
    }
}

impl NameUses<'_> {
    pub fn mentions(&self, name: &Name) -> Option<&Name> {
        self.names.get(name)
    }

    pub fn consumes<'uses>(
        &'uses self,
        path: &Path,
        crate_name: &CrateName,
        name: &Name,
    ) -> Option<&'uses NameUses<'uses>> {
        Some(self)
            .filter(|other| other.path != path)
            .filter(|other| other.crate_name == *crate_name)
            .filter(|other| other.mentions(name).is_some())
    }
}

impl<'ast> Visit<'ast> for NameUses<'_> {
    fn visit_ident(&mut self, ident: &'ast Ident) {
        self.names.insert(Name::from(ident));
    }
}
