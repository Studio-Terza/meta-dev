#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Erased {
    Locals,
    Names,
    Literals,
    TraitLabels,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Laws {
    WhereDeclared,
    Matched,
}

impl Laws {
    #[must_use]
    pub fn kept<Law>(self, laws: &[Law]) -> &[Law] {
        match self {
            Laws::WhereDeclared => &[],
            Laws::Matched => laws,
        }
    }

    #[must_use]
    pub fn agree<Law: PartialEq>(self, ours: &[Law], theirs: &[Law]) -> Option<Laws> {
        ours.first().zip(theirs.first()).map_or(Some(self), |_| (ours == theirs).then_some(self))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Equivalence {
    erased: &'static [Erased],
    laws: Laws,
}

impl Equivalence {
    #[must_use]
    pub const fn erasing(erased: &'static [Erased], laws: Laws) -> Equivalence {
        Equivalence { erased, laws }
    }

    #[must_use]
    pub fn erases(&self, what: Erased) -> Option<Erased> {
        self.erased.iter().copied().find(|erased| *erased == what)
    }

    #[must_use]
    pub fn laws(&self) -> Laws {
        self.laws
    }
}
