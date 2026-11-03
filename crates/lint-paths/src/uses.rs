use crate::cover::Cover;
use crate::ground::Ground;
use lint_core::Site;
use lint_layout::{UsePath, Word};

#[derive(Clone, Debug)]
pub struct Use {
    site: Site,
    candidates: Vec<UsePath>,
    ground: Ground,
    cover: Cover,
}

impl Use {
    pub(crate) fn new(site: Site, candidates: Vec<UsePath>, ground: Ground, cover: Cover) -> Use {
        Use { site, candidates, ground, cover }
    }

    #[must_use]
    pub fn place(&self) -> &Site {
        &self.site
    }

    #[must_use]
    pub fn spelling<const LENGTH: usize>(&self, tail: &[Word<'_>; LENGTH]) -> Option<&UsePath> {
        self.candidates.iter().find(|candidate| candidate.ends_with(tail))
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
