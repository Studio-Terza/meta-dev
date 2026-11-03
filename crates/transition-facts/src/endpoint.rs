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
}

impl Endpoint {
    const UNIT: Word<'static> = Word::new("()");
    const OPAQUE: Word<'static> = Word::new("_");

    pub(crate) fn new(nature: Nature, name: Name) -> Endpoint {
        Endpoint { nature, name }
    }

    #[must_use]
    pub fn nature(&self) -> Nature {
        self.nature
    }
}

impl From<Unnamed> for Endpoint {
    fn from(unnamed: Unnamed) -> Endpoint {
        let name = match unnamed {
            Unnamed::Unit => Self::UNIT,
            Unnamed::Opaque => Self::OPAQUE,
        };
        Endpoint { nature: Nature::Boxed, name: Name::from(name) }
    }
}

impl AsRef<Name> for Endpoint {
    fn as_ref(&self) -> &Name {
        &self.name
    }
}
