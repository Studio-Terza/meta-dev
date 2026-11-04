use syn::Macro;

use lint_layout::Name;
use lint_layout::TestMarks;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    Roundtrip,
    Law,
    Proptest,
    Other,
}

impl Kind {
    const ROUNDTRIP: &'static str = "roundtrip";
    const PROPTEST: &'static str = "proptest";

    pub(crate) fn of_macro(invocation: &Macro) -> Kind {
        let last = invocation.path.segments.last().map(|segment| Name::from(&segment.ident));
        let law = TestMarks::law(invocation);
        let roundtrip = last.as_ref().filter(|name| **name == Self::ROUNDTRIP).and(law).map(|_| Kind::Roundtrip);
        let proptest = last.as_ref().filter(|name| **name == Self::PROPTEST).map(|_| Kind::Proptest);
        roundtrip.or(proptest).or(law.map(|_| Kind::Law)).unwrap_or(Kind::Other)
    }
}
