use syn::Ident;

use super::callee::Callee;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Forwarder<'file> {
    name: &'file Ident,
    callee: Callee<'file>,
}

impl<'file> Forwarder<'file> {
    pub const fn new(name: &'file Ident, callee: Callee<'file>) -> Self {
        Forwarder { name, callee }
    }

    pub const fn name(&self) -> &'file Ident {
        self.name
    }

    pub const fn callee(&self) -> Callee<'file> {
        self.callee
    }
}
