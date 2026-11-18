use body_lifts::Body;
use lint_core::Site;
use lint_layout::Name;

use graph::{Invocation, Mutation};

use super::handle::Handle;
use super::resolved::Resolved;
use super::wrapping::Wrapping;

#[derive(Debug)]
pub struct Function<'file> {
    site: Site<'file>,
    name: Name,
    takes: Vec<Resolved>,
    gives: Resolved,
    mutation: Mutation,
    handle: Handle,
    body: Body,
    wrapping: Wrapping,
    takings: Vec<Wrapping>,
    calls: &'file [Invocation],
}

impl<'file> Function<'file> {
    pub(crate) fn new(site: Site<'file>, name: Name, takes: Vec<Resolved>, gives: Resolved) -> Function<'file> {
        Function {
            site,
            name,
            takes,
            gives,
            mutation: Mutation::Absent,
            handle: Handle::Owned,
            body: Body::Written,
            wrapping: Wrapping::Plain,
            takings: Vec::new(),
            calls: &[],
        }
    }

    pub(crate) fn shaped(mut self, mutation: Mutation, body: Body, handle: Handle) -> Function<'file> {
        self.mutation = mutation;
        self.body = body;
        self.handle = handle;
        self
    }

    pub(crate) fn wrapped(mut self, wrapping: Wrapping, takings: Vec<Wrapping>) -> Function<'file> {
        self.wrapping = wrapping;
        self.takings = takings;
        self
    }

    pub(crate) fn called(mut self, calls: &'file [Invocation]) -> Function<'file> {
        self.calls = calls;
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
    pub fn held(&self) -> Handle {
        self.handle
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
    pub fn calls(&self) -> &'file [Invocation] {
        self.calls
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
