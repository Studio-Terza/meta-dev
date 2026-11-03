use std::fmt;

use lint_core::Site;
use lint_layout::Name;
use lint_placement::CrateName;

use crate::context::Context;
use crate::endpoint::Endpoint;
use crate::function::Function;
use crate::ground::Ground;

#[derive(Clone, Copy, Debug)]
pub struct Transition<'found, 'file> {
    context: &'found Context<'file>,
    function: &'found Function<'file>,
}

impl<'found, 'file> Transition<'found, 'file> {
    pub(crate) fn new(context: &'found Context<'file>, function: &'found Function<'file>) -> Transition<'found, 'file> {
        Transition { context, function }
    }

    pub fn takes(&self) -> impl Iterator<Item = &'found Endpoint> {
        let selfish = self.context.selfish();
        self.function.takes().iter().map(move |taken| taken.resolved_with(selfish))
    }

    #[must_use]
    pub fn gives(&self) -> &'found Endpoint {
        self.function.gives().resolved_with(self.context.selfish())
    }

    #[must_use]
    pub fn implementer(&self) -> Option<&'found Endpoint> {
        self.context.filler().implementer(self.context.selfish())
    }

    #[must_use]
    pub fn context(&self) -> &'found Context<'file> {
        self.context
    }

    #[must_use]
    pub fn function(&self) -> &'found Function<'file> {
        self.function
    }
}

impl AsRef<Ground> for Transition<'_, '_> {
    fn as_ref(&self) -> &Ground {
        self.context.as_ref()
    }
}

impl AsRef<CrateName> for Transition<'_, '_> {
    fn as_ref(&self) -> &CrateName {
        self.context.as_ref()
    }
}

impl<'file> AsRef<Site<'file>> for Transition<'_, 'file> {
    fn as_ref(&self) -> &Site<'file> {
        self.function.as_ref()
    }
}

impl fmt::Display for Transition<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}::{}", self.context, AsRef::<Name>::as_ref(self.function))
    }
}
