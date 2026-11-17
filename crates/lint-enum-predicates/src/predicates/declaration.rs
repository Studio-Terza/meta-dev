mod declared;

use std::borrow::Cow;

use lint_placement::{CrateRoot, ModulePath};

use super::Kind;
use super::known::Known;
pub use declared::Declared;

#[derive(Debug)]
pub struct Declaration<'file> {
    crate_root: &'file CrateRoot,
    module: Cow<'file, ModulePath>,
    types: Vec<Declared>,
}

impl<'file> Declaration<'file> {
    pub fn placed(
        crate_root: &'file CrateRoot,
        module: Cow<'file, ModulePath>,
        types: Vec<Declared>,
    ) -> Declaration<'file> {
        Declaration { crate_root, module, types }
    }

    pub fn filed(self, known: Known<'file>) -> Known<'file> {
        self.types.iter().fold(known, |filing, declared| {
            filing.recorded(self.crate_root, &self.module, declared.as_ref(), Kind::from(declared))
        })
    }
}
