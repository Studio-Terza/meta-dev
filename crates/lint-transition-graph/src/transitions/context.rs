use std::fmt;

use lint_layout::Name;
use lint_placement::CrateName;

use crate::transitions::endpoint::Endpoint;
use crate::transitions::filler::Filler;
use crate::transitions::function::Function;
use crate::transitions::ground::Ground;

#[derive(Debug)]
pub struct Context<'file> {
    krate: CrateName,
    selfish: Endpoint,
    ground: Ground,
    filler: Filler,
    functions: Vec<Function<'file>>,
}

impl<'file> Context<'file> {
    pub(crate) fn new(krate: &Name, selfish: Endpoint, ground: Ground, filler: Filler) -> Context<'file> {
        Context { krate: CrateName::from(krate), selfish, ground, filler, functions: Vec::new() }
    }

    pub(crate) fn with_functions(mut self, functions: impl IntoIterator<Item = Function<'file>>) -> Context<'file> {
        self.functions.extend(functions);
        self
    }

    pub(crate) fn selfish(&self) -> &Endpoint {
        &self.selfish
    }

    pub(crate) fn filler(&self) -> &Filler {
        &self.filler
    }

    pub(crate) fn functions(&self) -> &[Function<'file>] {
        &self.functions
    }

    #[must_use]
    pub fn subject(&self) -> Option<&Endpoint> {
        self.filler.owned().map(|_| &self.selfish)
    }

    #[must_use]
    pub fn trait_name(&self) -> Option<&Name> {
        self.filler.trait_name(self.selfish.as_ref())
    }
}

impl AsRef<Ground> for Context<'_> {
    fn as_ref(&self) -> &Ground {
        &self.ground
    }
}

impl AsRef<CrateName> for Context<'_> {
    fn as_ref(&self) -> &CrateName {
        &self.krate
    }
}

impl fmt::Display for Context<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.filler.owned() {
            Some(_) => write!(formatter, "{}::{}", self.krate, AsRef::<Name>::as_ref(&self.selfish)),
            None => write!(formatter, "{}", self.krate),
        }
    }
}
