mod suffix;

use syntax::ItemMod;

use crate::joins::Joins;

use lint_layout::{Name, Word};
use newtype::Wrapping;

use crate::source_file::FileHead;
use crate::walk::Walk;
use crate::watch::Descend;
use suffix::Suffix;

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
    pub fn admits<Verdict: Copy>(&self, file: &FileHead, verdict: Verdict) -> Option<Verdict> {
        let placement = file.placement();
        let module =
            AsRef::<[Name]>::as_ref(placement.module()).iter().find(|segment| **segment == Name::from(self.module));
        self.package_suffix
            .and_then(|suffix| placement.crate_root().package()?.matches::<Suffix>(&[suffix]))
            .map(|_| verdict)
            .or_else(|| module.map(|_| verdict))
    }

    #[must_use]
    pub fn names_crate<Verdict>(&self, krate: &Name, verdict: Verdict) -> Option<Verdict> {
        self.package_suffix.and_then(|suffix| krate.matches::<Suffix>(&[suffix])).map(|_| verdict)
    }

    #[must_use]
    pub fn outside(&self, module: &ItemMod) -> Descend {
        Descend::from(Walk::outside_tests(&module.attrs))
            .combine(self.names(module, Descend::Over).unwrap_or(Descend::Into))
    }

    #[must_use]
    pub fn names<Verdict>(&self, module: &ItemMod, verdict: Verdict) -> Option<Verdict> {
        (module.ident == self.module.into_inner()).then_some(verdict)
    }
}
