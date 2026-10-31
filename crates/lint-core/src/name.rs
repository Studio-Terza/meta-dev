use std::ffi::OsStr;
use std::fmt;

use syn::Ident;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Name(String);

impl Name {
    #[must_use]
    pub fn length(&self) -> usize {
        self.0.chars().count()
    }
}

impl From<&Ident> for Name {
    fn from(ident: &Ident) -> Name {
        Name(ident.to_string())
    }
}

impl From<&OsStr> for Name {
    fn from(segment: &OsStr) -> Name {
        Name(segment.to_string_lossy().into_owned())
    }
}

impl fmt::Display for Name {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
