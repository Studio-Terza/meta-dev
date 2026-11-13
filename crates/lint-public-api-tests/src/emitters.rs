use std::collections::BTreeSet;

use syn::ItemMacro;

use lint_core::Macros;
use lint_layout::Name;
use lint_layout::TestMarks;
use lint_sweep::{Descend, Gathers, Kind, Sweep, Watch, Watching};

#[derive(Debug, Default)]
pub struct Emitters {
    names: BTreeSet<Name>,
}

impl Emitters {
    const DEFINITION: &'static str = "macro_rules";
}

impl Gathers<Emitters> for Emitters {
    fn watching(&mut self) -> Vec<&mut Watching<'_>> {
        Sweep::alone(self)
    }

    fn gathered(self) -> Emitters {
        self
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

impl<'ast> Watch<'ast> for Emitters {
    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemMacro]
    }

    fn on_item_macro(&mut self, definition: &'ast ItemMacro) -> Descend {
        self.names.extend(
            definition
                .ident
                .iter()
                .filter(|_| definition.mac.path.is_ident(Self::DEFINITION))
                .filter(|_| Macros::attributes(&definition.mac).iter().find_map(TestMarks::test).is_some())
                .map(Name::from),
        );
        Descend::Into
    }
}
