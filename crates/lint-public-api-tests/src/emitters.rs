use std::collections::BTreeSet;

use syn::ItemMacro;
use syn::visit::{self, Visit};

use lint_core::Macros;
use lint_layout::Name;
use lint_layout::TestMarks;

#[derive(Debug, Default)]
pub struct Emitters {
    names: BTreeSet<Name>,
}

impl Emitters {
    const DEFINITION: &'static str = "macro_rules";
}

impl From<&syn::File> for Emitters {
    fn from(file: &syn::File) -> Emitters {
        let mut emitters = Emitters::default();
        emitters.visit_file(file);
        emitters
    }
}

impl FromIterator<Emitters> for Emitters {
    fn from_iter<Parts: IntoIterator<Item = Emitters>>(parts: Parts) -> Emitters {
        Emitters { names: parts.into_iter().flat_map(|part| part.names).collect() }
    }
}

impl AsRef<BTreeSet<Name>> for Emitters {
    fn as_ref(&self) -> &BTreeSet<Name> {
        &self.names
    }
}

impl<'ast> Visit<'ast> for Emitters {
    fn visit_item_macro(&mut self, definition: &'ast ItemMacro) {
        self.names.extend(
            definition
                .ident
                .iter()
                .filter(|_| definition.mac.path.is_ident(Self::DEFINITION))
                .filter(|_| Macros::attributes(&definition.mac).iter().find_map(TestMarks::test).is_some())
                .map(Name::from),
        );
        visit::visit_item_macro(self, definition);
    }
}
