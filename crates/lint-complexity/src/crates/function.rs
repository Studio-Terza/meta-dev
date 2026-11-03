use body_lifts::Branching;
use lint_core::Site;
use lint_layout::Name;

#[derive(Clone, Debug)]
pub(crate) struct Function {
    name: Name,
    site: Site,
    score: Branching,
}

impl Function {
    pub(crate) fn new(name: Name, site: Site, score: Branching) -> Function {
        Function { name, site, score }
    }

    pub(crate) fn score(&self) -> Branching {
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
