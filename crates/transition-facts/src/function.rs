use body_lifts::{Body, Branching};
use lint_core::Site;
use lint_layout::Name;

use crate::callee::Callee;
use crate::mutation::Mutation;
use crate::resolved::Resolved;
use crate::wrapping::Wrapping;

#[derive(Debug)]
pub struct Function<'file> {
    site: Site<'file>,
    name: Name,
    takes: Vec<Resolved>,
    gives: Resolved,
    fails: Option<Resolved>,
    mutation: Mutation,
    body: Body,
    branching: Branching,
    wrapping: Wrapping,
    takings: Vec<Wrapping>,
    calls: Vec<Callee>,
}

impl<'file> Function<'file> {
    pub(crate) fn new(site: Site<'file>, name: Name, takes: Vec<Resolved>, gives: Resolved) -> Function<'file> {
        Function {
            site,
            name,
            takes,
            gives,
            fails: None,
            mutation: Mutation::Absent,
            body: Body::Written,
            branching: Branching::new(1),
            wrapping: Wrapping::Plain,
            takings: Vec::new(),
            calls: Vec::new(),
        }
    }

    pub(crate) fn shaped(mut self, mutation: Mutation, body: Body, branching: Branching) -> Function<'file> {
        self.mutation = mutation;
        self.body = body;
        self.branching = branching;
        self
    }

    pub(crate) fn wrapped(mut self, wrapping: Wrapping, takings: Vec<Wrapping>) -> Function<'file> {
        self.wrapping = wrapping;
        self.takings = takings;
        self
    }

    pub(crate) fn failing(mut self, fails: Option<Resolved>) -> Function<'file> {
        self.fails = fails;
        self
    }

    pub(crate) fn called(mut self, calls: Vec<Callee>) -> Function<'file> {
        self.calls = calls;
        self
    }

    pub(crate) fn takes(&self) -> &[Resolved] {
        &self.takes
    }

    pub(crate) fn gives(&self) -> &Resolved {
        &self.gives
    }

    pub(crate) fn fails(&self) -> Option<&Resolved> {
        self.fails.as_ref()
    }

    #[must_use]
    pub fn mutation(&self) -> Mutation {
        self.mutation
    }

    #[must_use]
    pub fn wrapping(&self) -> Wrapping {
        self.wrapping
    }

    #[must_use]
    pub fn takings(&self) -> &[Wrapping] {
        &self.takings
    }

    #[must_use]
    pub fn body(&self) -> Body {
        self.body
    }

    #[must_use]
    pub fn calls(&self) -> &[Callee] {
        &self.calls
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
