use lint_core::Site;
use lint_layout::Name;

use crate::method::Method;

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct TargetPath(Vec<Name>);

impl AsRef<[Name]> for TargetPath {
    fn as_ref(&self) -> &[Name] {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Implementation<'file> {
    implemented: Name,
    argument: Option<Name>,
    written: Vec<Name>,
    target_path: TargetPath,
    methods: Vec<Method>,
    site: Site<'file>,
}

impl<'file> Implementation<'file> {
    pub(crate) const fn new(
        implemented: Name,
        argument: Option<Name>,
        methods: Vec<Method>,
        site: Site<'file>,
    ) -> Self {
        Implementation {
            implemented,
            argument,
            written: Vec::new(),
            target_path: TargetPath(Vec::new()),
            methods,
            site,
        }
    }

    pub(crate) fn qualified(mut self, written: Vec<Name>, target_path: Vec<Name>) -> Self {
        self.written = written;
        self.target_path = TargetPath(target_path);
        self
    }

    #[must_use]
    pub fn written(&self) -> &[Name] {
        &self.written
    }

    #[must_use]
    pub const fn target_path(&self) -> &TargetPath {
        &self.target_path
    }

    #[must_use]
    pub const fn argument(&self) -> Option<&Name> {
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
