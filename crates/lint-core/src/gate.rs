mod suffix;

use syn::ItemMod;

use lint_layout::{Name, Word};
use lint_placement::Placement;
use newtype::Wrapping;

use crate::source_file::FileHead;
use crate::walk::Walk;
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
    pub fn admits<'head>(&self, file: &'head FileHead) -> Option<&'head FileHead> {
        let placement = file.placement();
        self.package_suffix
            .and_then(|suffix| placement.crate_root().package()?.matches::<Suffix>(&[suffix]))
            .map(|_| file)
            .or_else(|| {
                placement.module().segments().iter().find(|segment| **segment == Name::from(self.module)).map(|_| file)
            })
    }

    #[must_use]
    pub fn owns<'placement>(&self, placement: &'placement Placement) -> Option<&'placement Placement> {
        self.package_suffix
            .and_then(|suffix| placement.package().root().package()?.matches::<Suffix>(&[suffix]))
            .map(|_| placement)
    }

    pub fn enter_outside(&self, module: &ItemMod, visit: impl FnOnce()) {
        Walk::enter(&module.attrs, || self.names(module).map_or_else(visit, |_| ()));
    }

    #[must_use]
    pub fn names<'module>(&self, module: &'module ItemMod) -> Option<&'module ItemMod> {
        (module.ident == self.module.into_inner()).then_some(module)
    }
}
