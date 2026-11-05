use syn::parse::{Parse, ParseStream};
use syn::{Expr, Ident, Macro, Pat, Token};

use lint_layout::{Casing, Name, Whole, Word, words};

use super::super::super::trace::{Extent, Operations, Trace};
use crate::boundary::{Spelling, Truth};
use crate::claim::{Accepted, Claim, Union};

pub(crate) struct Matching {
    subject: Expr,
    pattern: Pat,
    guarded: Option<Expr>,
}

impl Parse for Matching {
    fn parse(input: ParseStream) -> syn::Result<Matching> {
        let subject = input.parse()?;
        input.parse::<Token![,]>()?;
        let pattern = Pat::parse_multi_with_leading_vert(input)?;
        let guarded = input.parse::<Option<Token![if]>>()?.map(|_| input.parse::<Expr>()).transpose()?;
        input.parse::<Option<Token![,]>>()?;
        Ok(Matching { subject, pattern, guarded })
    }
}

impl Matching {
    const MATCHES: [Word<'static>; 1] = words!["matches"];

    pub(crate) fn matched(trace: &Trace<'_, '_>, invocation: &Macro, holds: Truth) -> Option<Vec<Claim>> {
        let named = invocation.path.segments.last().map(|last| Name::from(&last.ident));
        let shape = named
            .filter(|name| name.matches::<Whole>(&Self::MATCHES).is_some())
            .filter(|_| holds == Truth::Holds)
            .and_then(|_| invocation.parse_body_with(Matching::parse).ok())?;
        let accepted = shape.guarded.as_ref().map_or_else(
            || Self::accepted(trace, &shape.pattern),
            |_| Accepted::Written(Spelling::written(invocation)),
        );
        Some(trace.stated(&shape.subject, accepted))
    }

    fn bare(ident: &Ident) -> Accepted {
        match Name::from(ident).casing() {
            Casing::Camel => Accepted::shaped(Name::from(ident), Vec::new()),
            Casing::Shouting => Accepted::Written(Spelling::written(ident)),
            Casing::Lower | Casing::Other => Accepted::Any,
        }
    }

    fn accepted(trace: &Trace<'_, '_>, pattern: &Pat) -> Accepted {
        match pattern {
            Pat::Lit(literal) => Accepted::equal(Operations::literal(&literal.lit)),
            Pat::Range(range) => Extent::ranged(trace, range)
                .map_or_else(|| Accepted::Written(Spelling::written(range)), Accepted::Within),
            Pat::Wild(_) | Pat::Rest(_) => Accepted::Any,
            Pat::Ident(named) => named
                .subpat
                .as_ref()
                .map_or_else(|| Self::bare(&named.ident), |(_, inner)| Self::accepted(trace, inner)),
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
