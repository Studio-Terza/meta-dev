use lint_layout::{Name, Word};
use lint_placement::CrateName;

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
    origin: Option<CrateName>,
}

impl Endpoint {
    const UNIT: Word<'static> = Word::new("()");
    const OPAQUE: Word<'static> = Word::new("_");

    pub(crate) fn new(nature: Nature, name: Name) -> Endpoint {
        Endpoint { nature, name, origin: None }
    }

    pub(crate) fn originating(self, origin: Option<CrateName>) -> Endpoint {
        match self.nature {
            Nature::Named => Endpoint { origin, ..self },
            Nature::Interface | Nature::Primitive | Nature::Boxed => self,
        }
    }

    pub(crate) fn tupled(elements: impl Iterator<Item = Endpoint>) -> Endpoint {
        elements
            .into_iter()
            .find(|element| element.nature == Nature::Primitive)
            .unwrap_or_else(|| Endpoint::from(Unnamed::Opaque))
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
    fn from(unnamed: Unnamed) -> Endpoint {
        let name = match unnamed {
            Unnamed::Unit => Self::UNIT,
            Unnamed::Opaque => Self::OPAQUE,
        };
        Endpoint { nature: Nature::Boxed, name: Name::from(name), origin: None }
    }
}

impl AsRef<Name> for Endpoint {
    fn as_ref(&self) -> &Name {
        &self.name
    }
}
