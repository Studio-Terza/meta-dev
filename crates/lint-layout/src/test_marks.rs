use syn::punctuated::Punctuated;
use syn::{Attribute, Macro, PathSegment, Token};

use crate::name::Name;
use crate::whole::Whole;
use crate::word::Word;
use crate::words;

#[derive(Debug)]
pub struct TestMarks;

impl TestMarks {
    const ATTRIBUTE: [Word<'static>; 3] = words!["test", "rstest", "test_case"];
    const LAWS: [Word<'static>; 4] = words!["law", "roundtrip", "idempotent", "table"];
    const LAW_CRATE: &'static str = "laws";

    #[must_use]
    pub fn is_test(attribute: &Attribute) -> bool {
        Self::ends_in(&attribute.path().segments, &Self::ATTRIBUTE)
    }

    #[must_use]
    pub fn mark_test(attributes: &[Attribute]) -> bool {
        attributes.iter().any(Self::is_test)
    }

    #[must_use]
    pub fn is_law(invocation: &Macro) -> bool {
        let segments = &invocation.path.segments;
        Self::ends_in(segments, &Self::LAWS)
            && segments.first().is_some_and(|first| segments.len() == 1 || first.ident == Self::LAW_CRATE)
    }

    fn ends_in(segments: &Punctuated<PathSegment, Token![::]>, words: &[Word<'_>]) -> bool {
        segments.last().is_some_and(|segment| Name::from(&segment.ident).matches::<Whole>(words))
    }
}
