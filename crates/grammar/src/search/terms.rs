mod boundary;

use super::Term;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Terms {
    first: Term,
    rest: Vec<Term>,
}

impl Terms {
    #[must_use]
    pub fn first(&self) -> &Term {
        &self.first
    }

    #[must_use]
    pub fn rest(&self) -> &[Term] {
        &self.rest
    }
}
