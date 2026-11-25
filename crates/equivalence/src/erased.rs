#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Erased {
    Locals,
    Names,
    Literals,
}

pub trait Erasable: Default {
    const KIND: Erased;
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Locals;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Names;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Literals;

impl Erasable for Locals {
    const KIND: Erased = Erased::Locals;
}

impl Erasable for Names {
    const KIND: Erased = Erased::Names;
}

impl Erasable for Literals {
    const KIND: Erased = Erased::Literals;
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
    pub fn erases<Kind: Erasable>(&self) -> Option<Kind> {
        self.erased.iter().find(|erased| **erased == Kind::KIND).map(|_| Kind::default())
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
