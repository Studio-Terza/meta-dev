use syn::Ident;

use super::callee::Callee;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Forwarder<'file> {
    name: &'file Ident,
    callee: Callee<'file>,
}

impl<'file> Forwarder<'file> {
    pub fn new(name: &'file Ident, callee: Callee<'file>) -> Self {
        Forwarder { name, callee }
    }

    pub fn name(&self) -> &'file Ident {
        self.name
    }

    pub fn callee(&self) -> Callee<'file> {
        self.callee
    }
}
