use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Attribute, Expr, ExprBlock, Ident, Macro, Token, bracketed};

use law_facts::Kind;
use lint_core::{Macros, Syntax};

use super::super::scope::{Place, Scope};
use super::super::trace::{Extent, Frame, Lowering, Quantifiers, Trace};
use crate::boundary::Truth;
use crate::claim::{Accepted, Binder, Callee, Cases, Claim, Domain, Draw, Guard, Term};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Argument {
    Operation,
    Body,
    Strategy,
}

impl Argument {
    fn classified(argument: &Expr) -> Argument {
        match argument {
            Expr::Path(_) => Argument::Operation,
            Expr::Block(_) => Argument::Body,
            _ => Argument::Strategy,
        }
    }
}

#[derive(Debug)]
pub(crate) struct Proof {
    claims: Vec<Claim>,
    quantifiers: Quantifiers,
    bodies: Vec<ExprBlock>,
}

impl Proof {
    pub(crate) fn claims(&self) -> &[Claim] {
        &self.claims
    }

    pub(crate) fn quantifiers(&self) -> &Quantifiers {
        &self.quantifiers
    }

    pub(crate) fn bodies(&self) -> &[ExprBlock] {
        &self.bodies
    }
}

impl From<Vec<Claim>> for Proof {
    fn from(claims: Vec<Claim>) -> Proof {
        Proof { claims, quantifiers: Quantifiers::default(), bodies: Vec::new() }
    }
}

#[derive(Debug)]
pub(crate) struct Table;

impl Table {
    pub(crate) fn claims(invocation: &Macro, scope: &Scope<'_>, place: &Place) -> Vec<Claim> {
        let listed = invocation.parse_body_with(Listed::parse).unwrap_or_default();
        let quantifiers = Quantifiers::default();
        let guard = Guard::default();
        let frame = Frame::default();
        let trace = Trace::new(scope, place, &frame);
        let function = listed.function.as_ref().map(|function| Self::operation(&trace, function));
        function
            .iter()
            .flat_map(|function| {
                listed.rows.iter().flat_map(|row| {
                    let subject = function.invoked(vec![Lowering::term_of(&trace, &row.given)]);
                    Claim::stated(
                        quantifiers.binders(),
                        &guard,
                        subject,
                        Accepted::equal(Lowering::term_of(&trace, &row.wanted)),
                    )
                })
            })
            .collect()
    }

    fn operation(trace: &Trace<'_, '_>, expression: &Expr) -> Term {
        Syntax::expr_path(expression).map_or_else(
            || Lowering::term_of(trace, expression),
            |path| Callee::Function(trace.scope().called(&path.path, trace.place())).folded(Vec::new()),
        )
    }

    pub(crate) fn law(invocation: &Macro, kind: Kind, scope: &Scope<'_>, place: &Place) -> Proof {
        let arguments = Macros::arguments(invocation);
        let guard = Guard::default();
        let frame = Frame::default();
        let trace = Trace::new(scope, place, &frame);
        let operations = arguments
            .iter()
            .skip(1)
            .filter(|argument| Argument::classified(argument) == Argument::Operation)
            .map(|argument| Self::operation(&trace, argument));
        let quantifiers = arguments
            .iter()
            .skip(1)
            .filter(|argument| Argument::classified(argument) == Argument::Strategy)
            .map(|strategy| {
                let domain = Extent::strategy(&trace, strategy)
                    .map_or_else(|| Domain::Drawn(Lowering::term_of(&trace, strategy)), Domain::Ranged);
                Binder::new(domain, Draw::Sampled(Cases::DEFAULT))
            })
            .fold(Quantifiers::default(), |quantifiers, binder| quantifiers.drawn(binder));
        let subject = Callee::Law(kind).folded(operations.chain(quantifiers.bound()).collect());
        Proof {
            claims: Claim::stated(quantifiers.binders(), &guard, subject, Accepted::equal(Term::Truth(Truth::Holds))),
            bodies: arguments.iter().filter_map(Self::body).cloned().collect(),
            quantifiers,
        }
    }

    fn body(argument: &Expr) -> Option<&ExprBlock> {
        match argument {
            Expr::Block(body) => Some(body),
            _ => None,
        }
    }
}

#[derive(PartialEq)]
struct Row {
    given: Expr,
    wanted: Expr,
}

impl Parse for Row {
    fn parse(input: ParseStream) -> syn::Result<Row> {
        let given = input.parse()?;
        input.parse::<Token![=>]>()?;
        Ok(Row { given, wanted: input.parse()? })
    }
}

#[derive(Default)]
struct Listed {
    function: Option<Expr>,
    rows: Vec<Row>,
}

impl Parse for Listed {
    fn parse(input: ParseStream) -> syn::Result<Listed> {
        input.call(Attribute::parse_outer)?;
        input.parse::<Ident>()?;
        input.parse::<Token![,]>()?;
        let function = input.parse::<Expr>()?;
        input.parse::<Token![,]>()?;
        let rows;
        bracketed!(rows in input);
        let rows = Punctuated::<Row, Token![,]>::parse_terminated(&rows)?.into_iter().collect();
        input.parse::<Option<Token![,]>>()?;
        Ok(Listed { function: Some(function), rows })
    }
}
