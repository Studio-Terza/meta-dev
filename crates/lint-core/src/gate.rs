mod suffix;

use syntax::ItemMod;

use algebra::Semigroup;

use lint_layout::{Name, Word};
use newtype::Wrapping;

use crate::source_file::FileHead;
use crate::walk::Walk;
use crate::watch::Descend;
use suffix::Suffix;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gating {
    Gated,
    Open,
}

impl Gating {
    pub fn chosen<Kind>(self, gated: Kind, open: Kind) -> Kind {
        match self {
            Gating::Gated => gated,
            Gating::Open => open,
        }
    }

    fn descend(self) -> Descend {
        self.chosen(Descend::Over, Descend::Into)
    }
}

#[derive(Debug)]
pub struct Gate {
    module: Word<'static>,
    package_suffix: Option<Word<'static>>,
}

impl Gate {
    pub const PORTS: Gate = Gate { module: Word::new("ports"), package_suffix: Some(Word::new("ports")) };
    pub const BOUNDARY: Gate = Gate { module: Word::new("boundary"), package_suffix: Some(Word::new("boundary")) };
    pub const STORE: Gate = Gate { module: Word::new("store"), package_suffix: Some(Word::new("store")) };

    #[must_use]
    pub fn admits(&self, file: &FileHead) -> Gating {
        let placement = file.placement();
        self.package_suffix
            .and_then(|suffix| placement.crate_root().package()?.matches::<Suffix>(&[suffix]))
            .map(|_| Gating::Gated)
            .or_else(|| {
                AsRef::<[Name]>::as_ref(placement.module())
                    .iter()
                    .find(|segment| **segment == Name::from(self.module))
                    .map(|_| Gating::Gated)
            })
            .unwrap_or(Gating::Open)
    }

    #[must_use]
    pub fn names_crate(&self, krate: &Name) -> Gating {
        self.package_suffix
            .and_then(|suffix| krate.matches::<Suffix>(&[suffix]))
            .map_or(Gating::Open, |_| Gating::Gated)
    }

    #[must_use]
    pub fn outside(&self, module: &ItemMod) -> Descend {
        Descend::from(Walk::outside_tests(&module.attrs)).combine(self.names(module).descend())
    }

    #[must_use]
    pub fn names(&self, module: &ItemMod) -> Gating {
        Some(&module.ident).filter(|ident| **ident == self.module.into_inner()).map_or(Gating::Open, |_| Gating::Gated)
    }
}
