use lint_core::Site;
use lint_layout::Name;

use super::score::Score;

#[derive(Clone, Debug)]
pub(crate) struct Function {
    name: Name,
    site: Site,
    score: Score,
}

impl Function {
    pub(crate) fn new(name: Name, site: Site, score: Score) -> Function {
        Function { name, site, score }
    }

    pub(crate) fn score(&self) -> Score {
        self.score
    }
}

impl AsRef<Name> for Function {
    fn as_ref(&self) -> &Name {
        &self.name
    }
}

impl AsRef<Site> for Function {
    fn as_ref(&self) -> &Site {
        &self.site
    }
}
