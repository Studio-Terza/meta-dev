use lint_layout::{Name, Word};
use lint_placement::CrateName;

use crate::spelling::Spelling;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Nature {
    Named,
    Interface,
    Primitive,
    Boxed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Unnamed {
    Unit,
    Opaque,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Endpoint {
    nature: Nature,
    name: Name,
    spelling: Option<Spelling>,
    origin: Option<CrateName>,
}

impl Unnamed {
    const UNIT: Word<'static> = Word::new("()");
    const OPAQUE: Word<'static> = Word::new("_");

    fn word(self) -> Word<'static> {
        match self {
            Self::Unit => Self::UNIT,
            Self::Opaque => Self::OPAQUE,
        }
    }
}

impl Endpoint {
    pub(crate) fn new(nature: Nature, name: Name) -> Self {
        Self { nature, name, spelling: None, origin: None }
    }

    pub(crate) fn spelled(self, spelling: Option<Spelling>) -> Self {
        Self { spelling, ..self }
    }

    pub(crate) fn spelling(&self) -> Option<&Spelling> {
        self.spelling.as_ref()
    }

    pub(crate) fn originating(self, origin: Option<CrateName>) -> Self {
        match self.nature {
            Nature::Named => Self { origin, ..self },
            Nature::Interface | Nature::Primitive | Nature::Boxed => self,
        }
    }

    #[must_use]
    pub fn nature(&self) -> Nature {
        self.nature
    }

    #[must_use]
    pub fn origin(&self) -> Option<&CrateName> {
        self.origin.as_ref()
    }
}

impl From<Unnamed> for Endpoint {
    fn from(unnamed: Unnamed) -> Self {
        Self { nature: Nature::Boxed, name: Name::from(unnamed.word()), spelling: None, origin: None }
    }
}

impl AsRef<Name> for Endpoint {
    fn as_ref(&self) -> &Name {
        &self.name
    }
}
