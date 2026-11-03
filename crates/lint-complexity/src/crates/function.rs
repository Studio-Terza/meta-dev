use body_lifts::Branching;
use lint_core::Site;
use lint_layout::Name;

#[derive(Clone, Debug)]
pub(crate) struct Function<'file> {
    name: Name,
    site: Site<'file>,
    score: Branching,
}

impl<'file> Function<'file> {
    pub(crate) fn new(name: Name, site: Site<'file>, score: Branching) -> Function<'file> {
        Function { name, site, score }
    }

    pub(crate) fn score(&self) -> Branching {
        self.score
    }
}

impl<'file> AsRef<Name> for Function<'file> {
    fn as_ref(&self) -> &Name {
        &self.name
    }
}

impl<'file> AsRef<Site<'file>> for Function<'file> {
    fn as_ref(&self) -> &Site<'file> {
        &self.site
    }
}
