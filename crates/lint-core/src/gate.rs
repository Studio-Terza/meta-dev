mod suffix;

use syn::ItemMod;

use lint_layout::Word;

use crate::source_file::SourceFile;
use suffix::Suffix;

#[derive(Debug)]
pub struct Gate {
    module: &'static str,
    package_suffix: Option<&'static str>,
}

impl Gate {
    pub const PORTS: Gate = Gate { module: "ports", package_suffix: Some("ports") };
    pub const BITS: Gate = Gate { module: "bits", package_suffix: None };

    #[must_use]
    pub fn admits<'file>(&self, file: &'file SourceFile) -> Option<&'file SourceFile> {
        let placement = file.placement();
        self.package_suffix
            .and_then(|suffix| placement.crate_root().package()?.matches::<Suffix>(&[Word::new(suffix)]))
            .map(|_| file)
            .or_else(|| placement.module().segments().iter().find(|segment| **segment == self.module).map(|_| file))
    }

    #[must_use]
    pub fn names<'module>(&self, module: &'module ItemMod) -> Option<&'module ItemMod> {
        (module.ident == self.module).then_some(module)
    }
}
