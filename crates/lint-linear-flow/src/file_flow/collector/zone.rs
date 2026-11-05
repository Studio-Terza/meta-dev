use syn::ItemMod;

use lint_core::{Gate, SourceFile};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Zone {
    Boundary,
    Elsewhere,
}

impl Zone {
    pub fn of_file(file: &SourceFile) -> Zone {
        match Gate::BOUNDARY.admits(file).or_else(|| Gate::PORTS.admits(file)) {
            Some(_) => Zone::Boundary,
            None => Zone::Elsewhere,
        }
    }

    pub fn entering(self, module: &ItemMod) -> Zone {
        match Gate::BOUNDARY.names(module).or_else(|| Gate::PORTS.names(module)) {
            Some(_) => Zone::Boundary,
            None => self,
        }
    }
}
