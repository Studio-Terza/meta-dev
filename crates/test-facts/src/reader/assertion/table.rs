use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Attribute, Expr, ExprBlock, Ident, Macro, Token, bracketed};

use law_facts::Kind;
use lint_core::{Macros, Syntax};

use super::super::body::Body;
use super::super::scope::{Place, Scope};
use super::super::trace::{Extent, Locals, Lowering, Quantifiers, Trace};
use crate::boundary::Truth;
use crate::claim::{Accepted, Binder, Callee, Cases, Claim, Domain, Draw, Term};

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
pub(crate) struct Table;

impl Table {
    pub(crate) fn claims(invocation: &Macro, scope: &Scope<'_>, place: &Place) -> Vec<Claim> {
        let listed = invocation.parse_body_with(Listed::parse).unwrap_or_default();
        let locals = Locals::default();
        let quantifiers = Quantifiers::default();
        let trace = Trace::new(scope, &locals, place, &quantifiers);
        let function = listed.function.as_ref().map(|function| Self::operation(&trace, function));
        function
            .iter()
            .flat_map(|function| {
                listed.rows.iter().flat_map(|row| {
                    let subject = function.invoked(vec![Lowering::term_of(&trace, &row.given)]);
                    Claim::stated(
                        quantifiers.binders(),
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

    pub(crate) fn law(invocation: &Macro, kind: Kind, scope: &Scope<'_>, place: &Place) -> Vec<Claim> {
        let arguments = Macros::arguments(invocation);
        let locals = Locals::default();
        let empty = Quantifiers::default();
        let trace = Trace::new(scope, &locals, place, &empty);
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
            .fold(empty.clone(), |quantifiers, binder| quantifiers.drawn(binder));
        let subject = Callee::Law(kind).folded(operations.chain(quantifiers.bound()).collect());
        arguments
            .iter()
            .filter_map(Self::body)
            .flat_map(|body| Body::claims(scope, place, &body.block, &quantifiers))
            .chain(Claim::stated(quantifiers.binders(), subject, Accepted::equal(Term::Truth(Truth::Holds))))
            .collect()
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
