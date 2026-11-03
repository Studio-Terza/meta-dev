mod declared;

use std::borrow::Cow;

use syn::Ident;

use lint_layout::Name;
use lint_placement::{CrateRoot, ModulePath};

use super::Kind;
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

    pub fn kinds<'scan>(&'scan self, crate_root: &CrateRoot, name: &'scan Name) -> impl Iterator<Item = Kind> + 'scan {
        let types = if self.crate_root == crate_root { self.types.as_slice() } else { &[] };
        types.iter().filter_map(move |declared| declared.named(name))
    }

    pub fn in_module(&self, module: &ModulePath) -> Option<&Declaration<'file>> {
        (*self.module == *module).then_some(self)
    }
}
