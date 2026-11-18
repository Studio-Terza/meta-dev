use syntax::{Attribute, Meta};

use crate::skip_marks::ignored::Ignored;
use invariants::Ignoring;
use lint_layout::LineNumber;

#[derive(Debug)]
pub(super) struct Gate;

impl Gate {
    const CONDITION: &'static str = "cfg";
    const TEST_CONDITION: &'static str = "test";

    pub(super) fn lines_in(attributes: &[Attribute]) -> Vec<LineNumber> {
        attributes
            .iter()
            .filter(|attribute| attribute.path().is_ident(Self::CONDITION))
            .filter(|attribute| {
                attribute
                    .parse_args::<Meta>()
                    .ignoring(Ignored::Unparsed)
                    .and_then(|condition| condition.path().get_ident().cloned())
                    .is_none_or(|ident| ident != Self::TEST_CONDITION)
            })
            .map(|attribute| LineNumber::from(attribute.pound_token.span))
            .collect()
    }
}
