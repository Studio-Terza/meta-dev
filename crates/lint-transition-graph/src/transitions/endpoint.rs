use lint_layout::{Name, Word};

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
    origin: Option<Name>,
}

impl Unnamed {
    const UNIT: Word<'static> = Word::new("()");
    const OPAQUE: Word<'static> = Word::new("_");

    fn word(self) -> Word<'static> {
        match self {
            Unnamed::Unit => Self::UNIT,
            Unnamed::Opaque => Self::OPAQUE,
        }
    }
}

impl Endpoint {
    pub(crate) fn new(nature: Nature, name: Name) -> Endpoint {
        Endpoint { nature, name, origin: None }
    }

    pub(crate) fn originating(self, origin: Option<Name>) -> Endpoint {
        Endpoint { origin, ..self }
    }

    #[must_use]
    pub fn nature(&self) -> Nature {
        self.nature
    }

    #[must_use]
    pub fn origin(&self) -> Option<&Name> {
        self.origin.as_ref()
    }
}

impl From<Unnamed> for Endpoint {
    fn from(unnamed: Unnamed) -> Endpoint {
        Endpoint { nature: Nature::Boxed, name: Name::from(unnamed.word()), origin: None }
    }
}

impl AsRef<Name> for Endpoint {
    fn as_ref(&self) -> &Name {
        &self.name
    }
}
