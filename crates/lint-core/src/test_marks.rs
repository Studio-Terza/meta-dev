use syn::punctuated::Punctuated;
use syn::{Attribute, Macro, PathSegment, Token};

use crate::name::Name;
use crate::whole::Whole;

#[derive(Debug)]
pub struct TestMarks;

impl TestMarks {
    const ATTRIBUTE: [&'static str; 1] = ["test"];
    const LAWS: [&'static str; 4] = ["law", "roundtrip", "idempotent", "table"];

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
        Self::ends_in(&invocation.path.segments, &Self::LAWS)
    }

    fn ends_in(segments: &Punctuated<PathSegment, Token![::]>, words: &[&str]) -> bool {
        segments.last().is_some_and(|segment| Name::from(&segment.ident).matches::<Whole>(words))
    }
}
