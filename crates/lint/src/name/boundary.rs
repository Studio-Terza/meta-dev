use std::fmt;
use std::ops::Deref;

use proc_macro2::Ident;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Name(String);

impl Name {
    pub(crate) fn of(ident: &Ident) -> Name {
        Name(ident.to_string())
    }

    pub(crate) fn spelled(spelling: impl fmt::Display) -> Name {
        Name(spelling.to_string())
    }
}

impl Deref for Name {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}

impl PartialEq<Ident> for Name {
    fn eq(&self, ident: &Ident) -> bool {
        ident == &self.0
    }
}

impl fmt::Display for Name {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
