use std::fmt;

use body_lifts::Body;
use lint_core::Site;
use lint_layout::Name;
use lint_placement::CrateName;

use crate::endpoint::{Endpoint, Unnamed};
use crate::filler::Filler;
use crate::ground::Ground;
use crate::mutation::Mutation;

#[derive(Clone, Debug)]
pub struct Transition {
    krate: CrateName,
    site: Site,
    owner: Option<Name>,
    name: Name,
    takes: Vec<Endpoint>,
    gives: Endpoint,
    mutation: Mutation,
    body: Body,
    ground: Ground,
    filler: Filler,
}

impl Transition {
    pub(crate) fn new(krate: CrateName, site: Site, owner: Option<Name>, name: Name) -> Transition {
        Transition {
            krate,
            site,
            owner,
            name,
            takes: Vec::new(),
            gives: Endpoint::from(Unnamed::Unit),
            mutation: Mutation::Absent,
            body: Body::Written,
            ground: Ground::Core,
            filler: Filler::Free,
        }
    }

    pub(crate) fn taking(mut self, takes: Vec<Endpoint>, gives: Endpoint, mutation: Mutation) -> Transition {
        self.takes = takes;
        self.gives = gives;
        self.mutation = mutation;
        self
    }

    pub(crate) fn placed(mut self, ground: Ground, filler: Filler, body: Body) -> Transition {
        self.ground = ground;
        self.filler = filler;
        self.body = body;
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

    #[must_use]
    pub fn mutation(&self) -> Mutation {
        self.mutation
    }

    #[must_use]
    pub fn body(&self) -> Body {
        self.body
    }

    #[must_use]
    pub fn filler(&self) -> &Filler {
        &self.filler
    }
}

impl AsRef<Ground> for Transition {
    fn as_ref(&self) -> &Ground {
        &self.ground
    }
}

impl AsRef<CrateName> for Transition {
    fn as_ref(&self) -> &CrateName {
        &self.krate
    }
}

impl AsRef<Name> for Transition {
    fn as_ref(&self) -> &Name {
        &self.name
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
