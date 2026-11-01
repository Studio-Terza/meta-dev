mod suffix;

use syn::ItemMod;

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
    pub fn admits(&self, file: &SourceFile) -> bool {
        let placement = file.placement();
        self.package_suffix.is_some_and(|suffix| {
            placement.crate_root().package().is_some_and(|package| package.matches::<Suffix>(&[suffix]))
        }) || placement.module().segments().iter().any(|segment| *segment == self.module)
    }

    #[must_use]
    pub fn names(&self, module: &ItemMod) -> bool {
        module.ident == self.module
    }
}
