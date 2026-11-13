mod declared;

use std::borrow::Cow;

use syn::Ident;

use lint_layout::Name;
use lint_placement::{CrateRoot, ModulePath};

use super::Kind;
use super::known::Known;
use declared::Declared;

#[derive(Debug)]
pub struct Declaration<'file> {
    crate_root: &'file CrateRoot,
    module: Cow<'file, ModulePath>,
    types: Vec<Declared>,
}

impl<'file> Declaration<'file> {
    pub fn new(crate_root: &'file CrateRoot, module: &'file ModulePath) -> Declaration<'file> {
        Declaration { crate_root, module: Cow::Borrowed(module), types: Vec::new() }
    }

    pub fn nested(&self, ident: &Ident) -> Declaration<'file> {
        Declaration {
            crate_root: self.crate_root,
            module: Cow::Owned(self.module.child(Name::from(ident))),
            types: Vec::new(),
        }
    }

    pub fn record(&mut self, ident: &Ident, kind: Kind) {
        self.types.push(Declared::new(Name::from(ident), kind));
    }

    pub fn filed(self, known: Known<'file>) -> Known<'file> {
        self.types.iter().fold(known, |filing, declared| {
            filing.recorded(self.crate_root, &self.module, declared.as_ref(), Kind::from(declared))
        })
    }
}
