use std::collections::BTreeSet;

use lint_core::Joins;
use syntax::ItemMacro;

use lint_core::Macros;
use lint_core::SourceFile;
use lint_layout::Name;
use lint_layout::TestMarks;
use lint_sweep::{Contribute, Contributed, Contributing, Facts, Gathers, Kind};

#[derive(Debug, Default)]
pub struct Emitters {
    names: BTreeSet<Name>,
}

#[derive(Debug)]
pub struct EmitterScan;

impl EmitterScan {
    const DEFINITION: &'static str = "macro_rules";
}

impl Gathers<Emitters> for Contributing<EmitterScan> {
    fn gathered(self, file: &SourceFile) -> Emitters {
        self.concluded(file, |_, (), emitters| emitters)
    }
}

impl FromIterator<Emitters> for Emitters {
    fn from_iter<Parts: IntoIterator<Item = Emitters>>(parts: Parts) -> Emitters {
        Joins::concat(parts)
    }
}

impl Joins for Emitters {
    fn combine(mut self, other: Emitters) -> Emitters {
        self.names.extend(other.names);
        self
    }
}

impl AsRef<BTreeSet<Name>> for Emitters {
    fn as_ref(&self) -> &BTreeSet<Name> {
        &self.names
    }
}

impl Contribute for EmitterScan {
    type Open = ();
    type Part = Emitters;

    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemMacro]
    }

    fn on_item_macro(
        &self,
        definition: &ItemMacro,
        _: &Facts<'_, ItemMacro>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), Emitters> {
        let names = definition
            .ident
            .iter()
            .filter(|_| definition.mac.path.is_ident(Self::DEFINITION))
            .filter(|_| Macros::attributes(&definition.mac).iter().find_map(TestMarks::test).is_some())
            .map(Name::from)
            .collect();
        Contributed::parted(Emitters { names })
    }
}
