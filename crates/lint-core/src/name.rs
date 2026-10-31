use std::fmt;

use syn::Ident;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Name(String);

impl Name {
    #[must_use]
    pub fn length(&self) -> usize {
        self.0.len()
    }
}

impl From<&Ident> for Name {
    fn from(ident: &Ident) -> Name {
        Name(ident.to_string())
    }
}

impl fmt::Display for Name {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
