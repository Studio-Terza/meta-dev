use syn::ExprMethodCall;

use lint_layout::{Name, Word};

use super::super::super::trace::Trace;
use crate::boundary::Truth;
use crate::claim::{Accepted, Claim};

pub(crate) struct Predicates;

impl Predicates {
    const SHAPES: [(Word<'static>, Word<'static>, usize); 4] = [
        (Word::new("is_some"), Word::new("Some"), 1),
        (Word::new("is_ok"), Word::new("Ok"), 1),
        (Word::new("is_err"), Word::new("Err"), 1),
        (Word::new("is_none"), Word::new("None"), 0),
    ];

    pub(crate) fn claims(trace: &Trace<'_, '_>, call: &ExprMethodCall, holds: Truth) -> Option<Vec<Claim>> {
        let name = Name::from(&call.method);
        Self::SHAPES.iter().find(|(predicate, _, _)| name == *predicate).filter(|_| holds == Truth::Holds).map(
            |(_, constructor, fields)| {
                trace.stated(&call.receiver, Accepted::shaped(Name::from(*constructor), vec![Accepted::Any; *fields]))
            },
        )
    }
}
