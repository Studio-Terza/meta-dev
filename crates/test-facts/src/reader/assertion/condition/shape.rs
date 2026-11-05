use syn::parse::{Parse, ParseStream};
use syn::{Expr, Macro, Pat, Token};

use lint_layout::{Name, Whole, Word, words};

use super::super::super::trace::{Extent, Operations, Trace};
use crate::boundary::{Spelling, Truth};
use crate::claim::{Accepted, Claim, Union};

pub(crate) struct Shape {
    subject: Expr,
    pattern: Pat,
    guarded: Option<Expr>,
}

impl Parse for Shape {
    fn parse(input: ParseStream) -> syn::Result<Shape> {
        let subject = input.parse()?;
        input.parse::<Token![,]>()?;
        let pattern = Pat::parse_multi_with_leading_vert(input)?;
        let guarded = input.parse::<Option<Token![if]>>()?.map(|_| input.parse::<Expr>()).transpose()?;
        input.parse::<Option<Token![,]>>()?;
        Ok(Shape { subject, pattern, guarded })
    }
}

impl Shape {
    const MATCHES: [Word<'static>; 1] = words!["matches"];

    pub(crate) fn matched(trace: &Trace<'_, '_>, invocation: &Macro, holds: Truth) -> Option<Vec<Claim>> {
        let named = invocation.path.segments.last().map(|last| Name::from(&last.ident));
        let shape = named
            .filter(|name| name.matches::<Whole>(&Self::MATCHES).is_some())
            .filter(|_| holds == Truth::Holds)
            .and_then(|_| invocation.parse_body_with(Shape::parse).ok())?;
        let accepted = shape.guarded.as_ref().map_or_else(
            || Self::accepted(trace, &shape.pattern),
            |_| Accepted::Written(Spelling::written(invocation)),
        );
        Some(trace.stated(&shape.subject, accepted))
    }

    fn accepted(trace: &Trace<'_, '_>, pattern: &Pat) -> Accepted {
        match pattern {
            Pat::Lit(literal) => Accepted::equal(Operations::literal(&literal.lit)),
            Pat::Range(range) => Extent::ranged(trace, range)
                .map_or_else(|| Accepted::Written(Spelling::written(range)), Accepted::Within),
            Pat::Wild(_) | Pat::Rest(_) => Accepted::Any,
            Pat::Ident(named) => named.subpat.as_ref().map_or(Accepted::Any, |(_, inner)| Self::accepted(trace, inner)),
            Pat::TupleStruct(tuple) => Accepted::shaped(
                tuple.path.segments.last().map(|last| Name::from(&last.ident)).unwrap_or_default(),
                tuple.elems.iter().map(|inner| Self::accepted(trace, inner)).collect(),
            ),
            Pat::Path(path) => Accepted::shaped(
                path.path.segments.last().map(|last| Name::from(&last.ident)).unwrap_or_default(),
                Vec::new(),
            ),
            Pat::Tuple(tuple) => Accepted::shaped(
                Name::default(),
                tuple.elems.iter().map(|inner| Self::accepted(trace, inner)).collect(),
            ),
            Pat::Or(cases) => {
                Accepted::Joined(cases.cases.iter().map(|inner| Self::accepted(trace, inner)).collect::<Union>())
            }
            Pat::Reference(reference) => Self::accepted(trace, &reference.pat),
            Pat::Paren(inner) => Self::accepted(trace, &inner.pat),
            _ => Accepted::Written(Spelling::written(pattern)),
        }
    }
}
