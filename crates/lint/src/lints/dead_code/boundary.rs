use std::collections::BTreeSet;
use std::fmt;

use proc_macro2::Ident;

use super::{Export, Graphed};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Name(String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct At(usize);

impl Name {
    pub(crate) fn of(ident: &Ident) -> Name {
        Name(ident.to_string())
    }

    pub(crate) fn spelled(text: String) -> Name {
        Name(text)
    }
}

impl PartialEq<str> for Name {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl fmt::Display for Name {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Graphed {
    pub(super) fn indexed(&self) -> impl Iterator<Item = (At, &Export)> {
        self.exports.iter().enumerate().map(|(at, export)| (At(at), export))
    }

    pub(super) fn at(&self, at: At) -> Option<&Export> {
        self.exports.get(at.0)
    }

    pub(super) fn dead(&self, alive: &BTreeSet<At>) -> impl Iterator<Item = &Export> {
        self.indexed().filter(|(at, _)| !alive.contains(at)).map(|(_, export)| export)
    }
}
