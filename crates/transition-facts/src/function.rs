use body_lifts::{Body, Branching};
use lint_core::Site;
use lint_layout::Name;

use crate::mutation::Mutation;
use crate::resolved::Resolved;

#[derive(Debug)]
pub struct Function<'file> {
    site: Site<'file>,
    name: Name,
    takes: Vec<Resolved>,
    gives: Resolved,
    mutation: Mutation,
    body: Body,
    branching: Branching,
}

impl<'file> Function<'file> {
    pub(crate) fn new(site: Site<'file>, name: Name, takes: Vec<Resolved>, gives: Resolved) -> Function<'file> {
        Function {
            site,
            name,
            takes,
            gives,
            mutation: Mutation::Absent,
            body: Body::Written,
            branching: Branching::new(1),
        }
    }

    pub(crate) fn shaped(mut self, mutation: Mutation, body: Body, branching: Branching) -> Function<'file> {
        self.mutation = mutation;
        self.body = body;
        self.branching = branching;
        self
    }

    pub(crate) fn takes(&self) -> &[Resolved] {
        &self.takes
    }

    pub(crate) fn gives(&self) -> &Resolved {
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
}

impl AsRef<Name> for Function<'_> {
    fn as_ref(&self) -> &Name {
        &self.name
    }
}

impl<'file> AsRef<Site<'file>> for Function<'file> {
    fn as_ref(&self) -> &Site<'file> {
        &self.site
    }
}
