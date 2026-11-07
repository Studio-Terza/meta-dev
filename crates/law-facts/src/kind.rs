use syn::Macro;

use lint_layout::Name;
use lint_layout::TestMarks;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Property {
    Associative,
    Commutative,
    Identity,
    Inverse,
    Idempotent,
    Absorbing,
    Distributive,
    Preserves,
}

impl Property {
    const NAMED: [(&'static str, Self); 8] = [
        ("associative", Self::Associative),
        ("commutative", Self::Commutative),
        ("identity", Self::Identity),
        ("inverse", Self::Inverse),
        ("idempotent", Self::Idempotent),
        ("absorbing", Self::Absorbing),
        ("distributive", Self::Distributive),
        ("preserves", Self::Preserves),
    ];

    fn declared_by(name: &Name) -> Option<Self> {
        Self::NAMED.iter().find(|(word, _)| *name == Name::from(*word)).map(|(_, property)| *property)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Roundtrip,
    Law,
    Proptest,
    Property(Property),
    Other,
}

impl Kind {
    const ROUNDTRIP: &'static str = "roundtrip";
    const PROPTEST: &'static str = "proptest";

    #[must_use]
    pub fn of_macro(invocation: &Macro) -> Self {
        let last = invocation.path.segments.last().map(|segment| Name::from(&segment.ident));
        let law = TestMarks::law(invocation);
        let roundtrip =
            last.as_ref().filter(|name| **name == Name::from(Self::ROUNDTRIP)).and(law).map(|_| Self::Roundtrip);
        let proptest = last.as_ref().filter(|name| **name == Name::from(Self::PROPTEST)).map(|_| Self::Proptest);
        let property = law.and(last.as_ref()).and_then(Property::declared_by).map(Kind::Property);
        roundtrip.or(proptest).or(property).or(law.map(|_| Self::Law)).unwrap_or(Self::Other)
    }

    pub(crate) const fn titled(self) -> Option<Self> {
        match self {
            Self::Roundtrip | Self::Law | Self::Property(_) => Some(self),
            Self::Proptest | Self::Other => None,
        }
    }

    #[must_use]
    pub const fn property(self) -> Option<Property> {
        match self {
            Self::Property(property) => Some(property),
            Self::Roundtrip | Self::Law | Self::Proptest | Self::Other => None,
        }
    }
}
