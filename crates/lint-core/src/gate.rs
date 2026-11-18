mod suffix;

use syntax::ItemMod;

use algebra::Semigroup;

use lint_layout::{Name, Word};
use newtype::Wrapping;

use crate::source_file::FileHead;
use crate::walk::Walk;
use crate::watch::Descend;
use suffix::Suffix;

#[derive(Clone, Copy, Debug)]
pub struct Admitted<'item, Item>(&'item Item);

impl<Item> AsRef<Item> for Admitted<'_, Item> {
    fn as_ref(&self) -> &Item {
        self.0
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
    pub fn admits<'head>(&self, file: &'head FileHead) -> Option<Admitted<'head, FileHead>> {
        let placement = file.placement();
        self.package_suffix
            .and_then(|suffix| placement.crate_root().package()?.matches::<Suffix>(&[suffix]))
            .map(|_| Admitted(file))
            .or_else(|| {
                AsRef::<[Name]>::as_ref(placement.module())
                    .iter()
                    .find(|segment| **segment == Name::from(self.module))
                    .map(|_| Admitted(file))
            })
    }

    #[must_use]
    pub fn names_crate<'name>(&self, krate: &'name Name) -> Option<Admitted<'name, Name>> {
        self.package_suffix.and_then(|suffix| krate.matches::<Suffix>(&[suffix])).map(|_| Admitted(krate))
    }

    #[must_use]
    pub fn outside(&self, module: &ItemMod) -> Descend {
        Descend::from(Walk::outside_tests(&module.attrs))
            .combine(self.names(module).map_or(Descend::Into, |_| Descend::Over))
    }

    #[must_use]
    pub fn names<'module>(&self, module: &'module ItemMod) -> Option<Admitted<'module, ItemMod>> {
        Some(module).filter(|named| named.ident == self.module.into_inner()).map(Admitted)
    }
}
