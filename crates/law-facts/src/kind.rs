use std::fmt;

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
    const NAMED: [(&'static str, Property); 8] = [
        ("associative", Property::Associative),
        ("commutative", Property::Commutative),
        ("identity", Property::Identity),
        ("inverse", Property::Inverse),
        ("idempotent", Property::Idempotent),
        ("absorbing", Property::Absorbing),
        ("distributive", Property::Distributive),
        ("preserves", Property::Preserves),
    ];

    fn declared_by(name: &Name) -> Option<Property> {
        Self::NAMED.iter().find(|(word, _)| *name == *word).map(|(_, property)| *property)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
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
    const LAW: &'static str = "declared";
    const OTHER: &'static str = "undeclared";

    #[must_use]
    pub fn of_macro(invocation: &Macro) -> Kind {
        let last = invocation.path.segments.last().map(|segment| Name::from(&segment.ident));
        let law = TestMarks::law(invocation);
        let roundtrip = last.as_ref().filter(|name| **name == Self::ROUNDTRIP).and(law).map(|_| Kind::Roundtrip);
        let proptest = last.as_ref().filter(|name| **name == Self::PROPTEST).map(|_| Kind::Proptest);
        let property = law.and(last.as_ref()).and_then(Property::declared_by).map(Kind::Property);
        roundtrip.or(proptest).or(property).or(law.map(|_| Kind::Law)).unwrap_or(Kind::Other)
    }

    pub(crate) fn titled(self) -> Option<Kind> {
        match self {
            Kind::Roundtrip | Kind::Law | Kind::Property(_) => Some(self),
            Kind::Proptest | Kind::Other => None,
        }
    }

    #[must_use]
    pub fn property(self) -> Option<Property> {
        match self {
            Kind::Property(property) => Some(property),
            Kind::Roundtrip | Kind::Law | Kind::Proptest | Kind::Other => None,
        }
    }
}

impl fmt::Display for Property {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        Self::NAMED.iter().find(|(_, property)| property == self).map_or(Ok(()), |(word, _)| formatter.write_str(word))
    }
}

impl fmt::Display for Kind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Kind::Roundtrip => formatter.write_str(Self::ROUNDTRIP),
            Kind::Law => formatter.write_str(Self::LAW),
            Kind::Proptest => formatter.write_str(Self::PROPTEST),
            Kind::Property(property) => property.fmt(formatter),
            Kind::Other => formatter.write_str(Self::OTHER),
        }
    }
}
