mod suffix;

use syn::ItemMod;

use lint_layout::{Name, Word};
use lint_placement::Placement;

use crate::source_file::SourceFile;
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
    pub fn admits<'file>(&self, file: &'file SourceFile) -> Option<&'file SourceFile> {
        let placement = file.placement();
        self.package_suffix
            .and_then(|suffix| placement.crate_root().package()?.matches::<Suffix>(&[suffix]))
            .map(|_| file)
            .or_else(|| placement.module().segments().iter().find(|segment| **segment == self.module).map(|_| file))
    }

    #[must_use]
    pub fn owns<'placement>(&self, placement: &'placement Placement) -> Option<&'placement Placement> {
        self.package_suffix
            .and_then(|suffix| placement.package().root().package()?.matches::<Suffix>(&[suffix]))
            .map(|_| placement)
    }

    #[must_use]
    pub fn names<'module>(&self, module: &'module ItemMod) -> Option<&'module ItemMod> {
        (Name::from(&module.ident) == self.module).then_some(module)
    }
}
