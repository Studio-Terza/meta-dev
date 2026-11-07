#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Erased {
    Locals,
    Names,
    Literals,
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
            Self::WhereDeclared => &[],
            Self::Matched => laws,
        }
    }

    #[must_use]
    pub fn agree<Law: PartialEq>(self, ours: &[Law], theirs: &[Law]) -> Option<Self> {
        match self {
            Self::Matched => (ours == theirs).then_some(self),
            Self::WhereDeclared => {
                ours.first().zip(theirs.first()).map_or(Some(self), |_| (ours == theirs).then_some(self))
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Equivalence {
    erased: &'static [Erased],
    laws: Option<Laws>,
}

impl Equivalence {
    #[must_use]
    pub const fn erasing(erased: &'static [Erased]) -> Self {
        Self { erased, laws: None }
    }

    #[must_use]
    pub const fn comparing(self, laws: Laws) -> Self {
        Self { laws: Some(laws), ..self }
    }

    #[must_use]
    pub fn erases(&self, what: Erased) -> Option<Erased> {
        self.erased.iter().copied().find(|erased| *erased == what)
    }

    #[must_use]
    pub fn kept<'laws, Law>(&self, laws: &'laws [Law]) -> &'laws [Law] {
        self.laws.map_or(&[], |compared| compared.kept(laws))
    }

    #[must_use]
    pub fn laws(&self) -> Option<Laws> {
        self.laws
    }
}
