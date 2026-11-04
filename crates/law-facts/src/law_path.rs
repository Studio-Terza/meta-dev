use std::fmt;

use lint_core::Site;
use lint_layout::Name;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct LawPath {
    owner: Name,
    member: Name,
}

impl LawPath {
    pub(crate) fn new(owner: Name, member: Name) -> LawPath {
        LawPath { owner, member }
    }

    #[must_use]
    pub fn reaching(&self, owner: &Name, member: &Name) -> Option<&LawPath> {
        Some(self).filter(|path| path.owner == *owner).filter(|path| path.member == *member)
    }
}

#[derive(Debug)]
pub(crate) enum Route {
    Path(LawPath),
    Closure,
}

impl Route {
    pub(crate) fn reaching(&self, owner: &Name, member: &Name) -> Option<&LawPath> {
        match self {
            Route::Path(path) => path.reaching(owner, member),
            Route::Closure => None,
        }
    }

    pub(crate) fn closure(&self) -> Option<&Route> {
        match self {
            Route::Path(_) => None,
            Route::Closure => Some(self),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Unresolved<'file> {
    site: Site<'file>,
    name: Name,
}

impl<'file> Unresolved<'file> {
    pub(crate) fn new(site: Site<'file>, name: Name) -> Unresolved<'file> {
        Unresolved { site, name }
    }
}

impl fmt::Display for Unresolved<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "law {}! at {} checks a closure the lint cannot resolve to an operation; pass the operation path",
            self.name, self.site
        )
    }
}
