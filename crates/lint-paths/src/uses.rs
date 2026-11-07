use crate::cover::Cover;
use crate::ground::Ground;
use lint_core::Site;
use lint_layout::{Name, PathTail, UsePath, Word};

#[derive(Clone, Debug)]
pub struct Use<'file> {
    site: Site<'file>,
    candidates: Vec<UsePath>,
    ground: Ground,
    cover: Cover,
}

impl<'file> Use<'file> {
    pub(crate) const fn new(site: Site<'file>, candidates: Vec<UsePath>, ground: Ground, cover: Cover) -> Self {
        Use { site, candidates, ground, cover }
    }

    #[must_use]
    pub const fn place(&self) -> Site<'file> {
        self.site
    }

    #[must_use]
    pub fn spelling(&self, tail: PathTail<'_>) -> Option<&UsePath> {
        self.candidates.iter().find_map(|candidate| candidate.ends_with(tail))
    }

    #[must_use]
    pub fn under(&self, root: &[Word<'_>]) -> Option<&UsePath> {
        self.candidates.iter().filter(|candidate| candidate.segments().len() > root.len()).find(|candidate| {
            candidate.segments().iter().zip(root).all(|(segment, spelled)| *segment == Name::from(*spelled))
        })
    }

    #[must_use]
    pub const fn ground(&self) -> Ground {
        self.ground
    }

    #[must_use]
    pub const fn cover(&self) -> Cover {
        self.cover
    }
}
