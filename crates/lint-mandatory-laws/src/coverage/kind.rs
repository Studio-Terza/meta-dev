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
        match (last, TestMarks::law(invocation)) {
            (Some(name), Some(_)) if name == Self::ROUNDTRIP => Kind::Roundtrip,
            (Some(name), _) if name == Self::PROPTEST => Kind::Proptest,
            (_, Some(_)) => Kind::Law,
            _ => Kind::Other,
        }
    }
}
