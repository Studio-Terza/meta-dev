use syn::Ident;

use super::callee::Callee;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Forwarder {
    name: Ident,
    callee: Callee,
}

impl Forwarder {
    pub fn new(name: Ident, callee: Callee) -> Forwarder {
        Forwarder { name, callee }
    }

    pub fn name(&self) -> &Ident {
        &self.name
    }

    pub fn callee(&self) -> &Callee {
        &self.callee
    }
}
