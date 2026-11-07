use std::fmt;

use lint_core::Site;
use lint_layout::Name;
use lint_placement::CrateName;

use crate::method::Method;

#[derive(Clone, Debug)]
pub struct GenericImpl<'file> {
    krate: &'file CrateName,
    site: Site<'file>,
    target: Name,
    implemented: Option<Name>,
    methods: Vec<Method>,
}

impl<'file> GenericImpl<'file> {
    pub(crate) const fn new(krate: &'file CrateName, site: Site<'file>, target: Name) -> Self {
        GenericImpl { krate, site, target, implemented: None, methods: Vec::new() }
    }

    pub(crate) fn filled(mut self, implemented: Option<Name>, methods: Vec<Method>) -> Self {
        self.implemented = implemented;
        self.methods = methods;
        self
    }

    #[must_use]
    pub const fn implemented(&self) -> Option<&Name> {
        self.implemented.as_ref()
    }
}

impl AsRef<[Method]> for GenericImpl<'_> {
    fn as_ref(&self) -> &[Method] {
        &self.methods
    }
}

impl AsRef<CrateName> for GenericImpl<'_> {
    fn as_ref(&self) -> &CrateName {
        self.krate
    }
}

impl AsRef<Name> for GenericImpl<'_> {
    fn as_ref(&self) -> &Name {
        &self.target
    }
}

impl<'file> AsRef<Site<'file>> for GenericImpl<'file> {
    fn as_ref(&self) -> &Site<'file> {
        &self.site
    }
}

impl fmt::Display for GenericImpl<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.implemented {
            Some(implemented) => write!(formatter, "{implemented} for {}", self.target),
            None => write!(formatter, "{}", self.target),
        }
    }
}
