use std::fmt;

use syn::Ident;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Callee<'file> {
    owner: Option<&'file Ident>,
    method: &'file Ident,
}

impl<'file> Callee<'file> {
    pub fn method(method: &'file Ident) -> Self {
        Callee { owner: None, method }
    }

    pub fn associated(owner: &'file Ident, method: &'file Ident) -> Self {
        Callee { owner: Some(owner), method }
    }
}

impl fmt::Display for Callee<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.owner {
            Some(owner) => write!(formatter, "{owner}::{}", self.method),
            None => write!(formatter, "{}", self.method),
        }
    }
}
