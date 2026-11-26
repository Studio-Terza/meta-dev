mod whole;

use syntax::Punctuated;
use syntax::{Attribute, PathSegment, Token};

use crate::matching::Fitted;
use crate::name::Name;
use crate::word::Word;
use crate::words;
pub use whole::Whole;

#[derive(Debug)]
pub struct TestMarks;

impl TestMarks {
    const ATTRIBUTE: [Word<'static>; 3] = words!["test", "rstest", "test_case"];

    #[must_use]
    pub fn test(attribute: &Attribute) -> Option<&Attribute> {
        Self::ends_in(&attribute.path().segments, &Self::ATTRIBUTE).map(|_| attribute)
    }

    #[must_use]
    pub fn mark_test(attributes: &[Attribute]) -> Option<&Attribute> {
        attributes.iter().find_map(Self::test)
    }

    fn ends_in<'word>(segments: &Punctuated<PathSegment, Token![::]>, words: &[Word<'word>]) -> Option<Fitted<'word>> {
        segments.last().and_then(|segment| Name::from(&segment.ident).matches::<Whole>(words))
    }
}
