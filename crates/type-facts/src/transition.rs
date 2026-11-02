use std::fmt;

use lint_core::Site;
use lint_layout::Name;
use lint_placement::CrateName;

use crate::endpoint::Endpoint;

#[derive(Clone, Debug)]
pub struct Transition {
    krate: CrateName,
    site: Site,
    owner: Option<Name>,
    name: Name,
    takes: Vec<Endpoint>,
    gives: Endpoint,
}

impl Transition {
    pub(crate) fn new(krate: CrateName, site: Site, owner: Option<Name>, name: Name) -> Transition {
        Transition { krate, site, owner, name, takes: Vec::new(), gives: Endpoint::Unit }
    }

    pub(crate) fn taking(mut self, takes: Vec<Endpoint>, gives: Endpoint) -> Transition {
        self.takes = takes;
        self.gives = gives;
        self
    }

    #[must_use]
    pub fn takes(&self) -> &[Endpoint] {
        &self.takes
    }

    #[must_use]
    pub fn gives(&self) -> &Endpoint {
        &self.gives
    }
}

impl AsRef<CrateName> for Transition {
    fn as_ref(&self) -> &CrateName {
        &self.krate
    }
}

impl AsRef<Site> for Transition {
    fn as_ref(&self) -> &Site {
        &self.site
    }
}

impl fmt::Display for Transition {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.owner {
            Some(owner) => write!(formatter, "{}::{owner}::{}", self.krate, self.name),
            None => write!(formatter, "{}::{}", self.krate, self.name),
        }
    }
}
