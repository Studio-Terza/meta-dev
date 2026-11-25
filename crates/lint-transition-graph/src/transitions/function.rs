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

#[derive(Debug)]
pub(crate) struct Outlined<'file> {
    site: Site<'file>,
    name: Name,
    takes: Vec<Resolved>,
    gives: Resolved,
}

#[derive(Debug)]
pub(crate) struct Bodied<'file> {
    outline: Outlined<'file>,
    mutation: Mutation,
    handle: Handle,
    body: Body,
}

#[derive(Debug)]
pub(crate) struct Peeled<'file> {
    bodied: Bodied<'file>,
    wrapping: Wrapping,
    takings: Vec<Wrapping>,
}

impl<'file> Outlined<'file> {
    pub(crate) fn new(site: Site<'file>, name: Name, takes: Vec<Resolved>, gives: Resolved) -> Outlined<'file> {
        Outlined { site, name, takes, gives }
    }

    pub(crate) fn shaped(self, mutation: Mutation, body: Body, handle: Handle) -> Bodied<'file> {
        Bodied { outline: self, mutation, handle, body }
    }
}

impl<'file> Bodied<'file> {
    pub(crate) fn wrapped(self, wrapping: Wrapping, takings: Vec<Wrapping>) -> Peeled<'file> {
        Peeled { bodied: self, wrapping, takings }
    }
}

impl<'file> Peeled<'file> {
    pub(crate) fn called(self, calls: &'file [Invocation]) -> Function<'file> {
        let Peeled { bodied: Bodied { outline, mutation, handle, body }, wrapping, takings } = self;
        let Outlined { site, name, takes, gives } = outline;
        Function { site, name, takes, gives, mutation, handle, body, wrapping, takings, calls }
    }
}

impl<'file> Function<'file> {
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
