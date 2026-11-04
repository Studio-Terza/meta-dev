use lint_core::Site;
use lint_layout::Name;

use crate::method::Method;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Implementation<'file> {
    implemented: Name,
    argument: Option<Name>,
    methods: Vec<Method>,
    site: Site<'file>,
}

impl<'file> Implementation<'file> {
    pub(crate) fn new(
        implemented: Name,
        argument: Option<Name>,
        methods: Vec<Method>,
        site: Site<'file>,
    ) -> Implementation<'file> {
        Implementation { implemented, argument, methods, site }
    }

    #[must_use]
    pub fn argument(&self) -> Option<&Name> {
        self.argument.as_ref()
    }

    #[must_use]
    pub fn methods(&self) -> &[Method] {
        &self.methods
    }
}

impl AsRef<Name> for Implementation<'_> {
    fn as_ref(&self) -> &Name {
        &self.implemented
    }
}

impl<'file> AsRef<Site<'file>> for Implementation<'file> {
    fn as_ref(&self) -> &Site<'file> {
        &self.site
    }
}
