use syn::{Attribute, Meta};

use lint_layout::LineNumber;

#[derive(Debug)]
pub(super) struct Gate;

impl Gate {
    const CONDITION: &'static str = "cfg";
    const TEST_CONDITION: &'static str = "test";

    pub(super) fn lines_in(attributes: &[Attribute]) -> Vec<LineNumber> {
        attributes
            .iter()
            .filter(|attribute| {
                attribute.path().is_ident(Self::CONDITION)
                    && !attribute
                        .parse_args::<Meta>()
                        .is_ok_and(|condition| condition.path().is_ident(Self::TEST_CONDITION))
            })
            .map(|attribute| LineNumber::from(attribute.pound_token.span))
            .collect()
    }
}
