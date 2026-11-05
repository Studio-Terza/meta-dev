use lint_placement::{CrateName, ModulePath};

use crate::gating::Gating;

#[derive(Clone, Debug)]
pub struct Vantage<'file> {
    krate: &'file CrateName,
    module: ModulePath,
    gating: Gating,
}

impl<'file> Vantage<'file> {
    #[must_use]
    pub fn new(krate: &'file CrateName, module: ModulePath, gating: Gating) -> Vantage<'file> {
        Vantage { krate, module, gating }
    }

    pub(crate) fn krate(&self) -> &CrateName {
        self.krate
    }

    pub(crate) fn module(&self) -> &ModulePath {
        &self.module
    }

    pub(crate) fn gating(&self) -> Gating {
        self.gating
    }
}
