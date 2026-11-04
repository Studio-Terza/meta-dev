#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Erased {
    Locals,
    Names,
    Literals,
}

#[derive(Clone, Copy, Debug)]
pub struct Equivalence {
    erased: &'static [Erased],
}

impl Equivalence {
    #[must_use]
    pub const fn erasing(erased: &'static [Erased]) -> Equivalence {
        Equivalence { erased }
    }

    #[must_use]
    pub fn erases(&self, what: Erased) -> Option<Erased> {
        self.erased.iter().copied().find(|erased| *erased == what)
    }
}
