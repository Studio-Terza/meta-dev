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
        match last {
            Some(name) if name == Self::ROUNDTRIP && TestMarks::is_law(invocation) => Kind::Roundtrip,
            Some(name) if name == Self::PROPTEST => Kind::Proptest,
            _ if TestMarks::is_law(invocation) => Kind::Law,
            _ => Kind::Other,
        }
    }
}
