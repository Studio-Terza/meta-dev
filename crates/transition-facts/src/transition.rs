use std::fmt;

use body_lifts::{Body, Branching};
use lint_core::Site;
use lint_layout::Name;
use lint_placement::CrateName;

use crate::endpoint::{Endpoint, Unnamed};
use crate::filler::Filler;
use crate::ground::Ground;
use crate::mutation::Mutation;

#[derive(Clone, Debug)]
pub struct Transition<'file> {
    krate: CrateName,
    site: Site<'file>,
    owner: Option<Name>,
    name: Name,
    takes: Vec<Endpoint>,
    gives: Endpoint,
    mutation: Mutation,
    body: Body,
    branching: Branching,
    ground: Ground,
    filler: Filler,
}

impl<'file> Transition<'file> {
    pub(crate) fn new(krate: CrateName, site: Site<'file>, owner: Option<Name>, name: Name) -> Transition<'file> {
        Transition {
            krate,
            site,
            owner,
            name,
            takes: Vec::new(),
            gives: Endpoint::from(Unnamed::Unit),
            mutation: Mutation::Absent,
            body: Body::Written,
            branching: Branching::new(1),
            ground: Ground::Core,
            filler: Filler::Free,
        }
    }

    pub(crate) fn taking(mut self, takes: Vec<Endpoint>, gives: Endpoint, mutation: Mutation) -> Transition<'file> {
        self.takes = takes;
        self.gives = gives;
        self.mutation = mutation;
        self
    }

    pub(crate) fn placed(mut self, ground: Ground, filler: Filler, body: Body) -> Transition<'file> {
        self.ground = ground;
        self.filler = filler;
        self.body = body;
        self
    }

    pub(crate) fn branching(mut self, branching: Branching) -> Transition<'file> {
        self.branching = branching;
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
    pub fn branches(&self) -> Branching {
        self.branching
    }

    #[must_use]
    pub fn filler(&self) -> &Filler {
        &self.filler
    }
}

impl<'file> AsRef<Ground> for Transition<'file> {
    fn as_ref(&self) -> &Ground {
        &self.ground
    }
}

impl<'file> AsRef<CrateName> for Transition<'file> {
    fn as_ref(&self) -> &CrateName {
        &self.krate
    }
}

impl<'file> AsRef<Name> for Transition<'file> {
    fn as_ref(&self) -> &Name {
        &self.name
    }
}

impl<'file> AsRef<Site<'file>> for Transition<'file> {
    fn as_ref(&self) -> &Site<'file> {
        &self.site
    }
}

impl<'file> fmt::Display for Transition<'file> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.owner {
            Some(owner) => write!(formatter, "{}::{owner}::{}", self.krate, self.name),
            None => write!(formatter, "{}::{}", self.krate, self.name),
        }
    }
}
