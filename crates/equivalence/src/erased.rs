#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Erased {
    Locals,
    Names,
    Literals,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Laws {
    Ignored,
    Matched,
}

impl Laws {
    #[must_use]
    pub fn kept<Law>(self, laws: &[Law]) -> &[Law] {
        match self {
            Laws::Ignored => &[],
            Laws::Matched => laws,
        }
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
