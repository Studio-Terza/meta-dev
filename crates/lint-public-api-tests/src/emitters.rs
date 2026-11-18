use std::collections::BTreeSet;

use algebra::{Monoid, Semigroup};
use syntax::ItemMacro;

use lint_core::Macros;
use lint_layout::Name;
use lint_layout::TestMarks;
use lint_sweep::{Contribute, Contributed, Contributing, Facts, Gathers, Kind, Sweep, Watching};

#[derive(Debug, Default)]
pub struct Emitters {
    names: BTreeSet<Name>,
}

#[derive(Debug, Default)]
pub struct EmitterScan;

impl EmitterScan {
    const DEFINITION: &'static str = "macro_rules";
}

impl Gathers<Emitters> for Contributing<EmitterScan> {
    fn watching(&mut self) -> impl IntoIterator<Item = &mut Watching<'_>> {
        Sweep::alone(self)
    }

    fn gathered(self) -> Emitters {
        self.concluded(|_, (), emitters| emitters)
    }
}

impl FromIterator<Emitters> for Emitters {
    fn from_iter<Parts: IntoIterator<Item = Emitters>>(parts: Parts) -> Emitters {
        Monoid::concat(parts)
    }
}

impl Semigroup for Emitters {
    fn combine(mut self, other: Emitters) -> Emitters {
        self.names.extend(other.names);
        self
    }
}

impl Monoid for Emitters {}

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

    fn on_item_macro(&self, definition: &ItemMacro, _: &Facts<'_, ItemMacro>) -> Contributed<(), Emitters> {
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
