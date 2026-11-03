mod whole;

use syn::punctuated::Punctuated;
use syn::{Attribute, Macro, PathSegment, Token};

use crate::name::Name;
use crate::word::Word;
use crate::words;
pub use whole::Whole;

#[derive(Debug)]
pub struct TestMarks;

impl TestMarks {
    const ATTRIBUTE: [Word<'static>; 3] = words!["test", "rstest", "test_case"];
    const LAWS: [Word<'static>; 4] = words!["law", "roundtrip", "idempotent", "table"];
    const LAW_CRATE: &'static str = "laws";

    #[must_use]
    pub fn test(attribute: &Attribute) -> Option<&Attribute> {
        Self::ends_in(&attribute.path().segments, &Self::ATTRIBUTE).map(|_| attribute)
    }

    #[must_use]
    pub fn mark_test(attributes: &[Attribute]) -> Option<&Attribute> {
        attributes.iter().find_map(Self::test)
    }

    #[must_use]
    pub fn law(invocation: &Macro) -> Option<&Macro> {
        let segments = &invocation.path.segments;
        Self::ends_in(segments, &Self::LAWS)
            .and(segments.first())
            .filter(|first| segments.len() == 1 || first.ident == Self::LAW_CRATE)
            .map(|_| invocation)
    }

    fn ends_in<'word>(segments: &Punctuated<PathSegment, Token![::]>, words: &[Word<'word>]) -> Option<Word<'word>> {
        segments.last().and_then(|segment| Name::from(&segment.ident).matches::<Whole>(words))
    }
}
