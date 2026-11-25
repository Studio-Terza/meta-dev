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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Agreement {
    Same,
    Undeclared,
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
    pub fn agree<Law: PartialEq>(self, ours: &[Law], theirs: &[Law]) -> Option<Agreement> {
        match self {
            Laws::Matched => (ours == theirs).then_some(Agreement::Same),
            Laws::WhereDeclared => ours
                .first()
                .zip(theirs.first())
                .map_or(Some(Agreement::Undeclared), |_| (ours == theirs).then_some(Agreement::Same)),
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
    pub const fn erasing(erased: &'static [Erased]) -> Equivalence {
        Equivalence { erased, laws: None }
    }

    #[must_use]
    pub const fn comparing(erased: &'static [Erased], laws: Laws) -> Equivalence {
        Equivalence { erased, laws: Some(laws) }
    }

    #[must_use]
    pub fn unerased<Value>(&self, what: Erased, value: Value) -> Option<Value> {
        self.erased.iter().find(|erased| **erased == what).map_or(Some(value), |_| None)
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
