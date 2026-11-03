use crate::cover::Cover;
use crate::ground::Ground;
use lint_core::Site;
use lint_layout::{PathTail, UsePath, Word};

#[derive(Clone, Debug)]
pub struct Use<'file> {
    site: Site<'file>,
    candidates: Vec<UsePath>,
    ground: Ground,
    cover: Cover,
}

impl<'file> Use<'file> {
    pub(crate) fn new(site: Site<'file>, candidates: Vec<UsePath>, ground: Ground, cover: Cover) -> Use<'file> {
        Use { site, candidates, ground, cover }
    }

    #[must_use]
    pub fn place(&self) -> Site<'file> {
        self.site
    }

    #[must_use]
    pub fn spelling(&self, tail: PathTail<'_>) -> Option<&UsePath> {
        self.candidates.iter().find_map(|candidate| candidate.ends_with(tail))
    }

    #[must_use]
    pub fn under(&self, root: &[Word<'_>]) -> Option<&UsePath> {
        self.candidates.iter().find(|candidate| {
            let segments = candidate.segments();
            segments.len() > root.len() && segments.iter().zip(root).all(|(segment, spelled)| segment == spelled)
        })
    }

    #[must_use]
    pub fn ground(&self) -> Ground {
        self.ground
    }

    #[must_use]
    pub fn cover(&self) -> Cover {
        self.cover
    }
}
