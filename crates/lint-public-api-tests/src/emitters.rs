use std::collections::BTreeSet;

use syn::ItemMacro;
use syn::visit::{self, Visit};

use lint_core::Macros;
use lint_layout::Name;
use lint_layout::TestMarks;

#[derive(Debug, Default)]
pub(crate) struct Emitters {
    names: BTreeSet<Name>,
}

impl Emitters {
    const DEFINITION: &'static str = "macro_rules";

    pub(crate) fn defined_in(files: &[&syn::File]) -> BTreeSet<Name> {
        files
            .iter()
            .fold(Emitters::default(), |mut emitters, file| {
                emitters.visit_file(file);
                emitters
            })
            .names
    }
}

impl<'ast> Visit<'ast> for Emitters {
    fn visit_item_macro(&mut self, definition: &'ast ItemMacro) {
        self.names.extend(
            definition
                .ident
                .iter()
                .filter(|_| {
                    definition.mac.path.is_ident(Self::DEFINITION)
                        && Macros::attributes(&definition.mac).iter().any(TestMarks::is_test)
                })
                .map(Name::from),
        );
        visit::visit_item_macro(self, definition);
    }
}
