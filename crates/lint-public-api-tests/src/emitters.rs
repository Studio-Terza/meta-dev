use std::collections::BTreeSet;

use syntax::ItemMacro;

use lint_core::Extended;
use lint_core::Macros;
use lint_layout::Name;
use lint_layout::TestMarks;
use lint_sweep::{Facts, Fold, Kind, Visit};

#[derive(Debug, Default)]
pub struct Emitters {
    names: BTreeSet<Name>,
}

impl Emitters {
    const DEFINITION: &'static str = "macro_rules";
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

#[derive(Debug)]
pub struct Emitting;

impl Fold for Emitting {
    type Found = Emitters;

    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemMacro]
    }

    fn on_item_macro(&self, found: Emitters, definition: &ItemMacro, _: &Facts<'_, ItemMacro>) -> Visit<Emitters> {
        let names = definition
            .ident
            .iter()
            .filter(|_| definition.mac.path.is_ident(Emitters::DEFINITION))
            .filter(|_| Macros::attributes(&definition.mac).iter().find_map(TestMarks::test).is_some())
            .map(Name::from);
        Visit::descending(Emitters { names: found.names.extended(names) })
    }
}
