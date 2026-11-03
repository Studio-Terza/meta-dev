use syn::Ident;

use lint_layout::{LineNumber, Name};

pub struct Members {
    first: LineNumber,
    names: Vec<Name>,
}

impl Members {
    pub fn starting(ident: &Ident) -> Members {
        Members { first: LineNumber::from(ident.span()), names: Vec::new() }
    }

    pub fn joined(&mut self, ident: &Ident) {
        self.names.push(Name::from(ident));
    }

    pub fn first(&self) -> LineNumber {
        self.first
    }

    pub fn names(&self) -> &[Name] {
        &self.names
    }
}
