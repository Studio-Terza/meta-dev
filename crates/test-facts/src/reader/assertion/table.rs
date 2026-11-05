use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Attribute, Expr, ExprBlock, Ident, Macro, Token, bracketed};

use law_facts::Kind;
use lint_core::Macros;
use newtype::Counter;

use super::super::body::Body;
use super::super::scope::{Place, Scope};
use super::super::trace::{Extent, Leaves, Locals, Trace};
use crate::check::{Breadth, Cases, Check, Domain, Relation, Rows, Values};

#[derive(Debug)]
pub(crate) struct Table;

impl Table {
    pub(crate) fn checks(invocation: &Macro, scope: &Scope<'_>, place: &Place) -> Vec<Check> {
        let rows: Vec<Row> = invocation.parse_body_with(Listed::parse).map(|listed| listed.rows).unwrap_or_default();
        let locals = Locals::default();
        let trace = Trace::new(scope, &locals, place);
        let distinct = Trace::distinct(&rows.iter().collect::<Vec<&Row>>());
        let leaves = Values::total(distinct.iter().map(|row| Leaves::counted(&trace, &row.wanted)));
        let domain =
            Domain::new(Breadth::Listed, Rows::listed(Cases::new(rows.len()), Cases::new(distinct.len()), leaves));
        vec![Check::new(Relation::Equal, leaves, domain)]
    }

    pub(crate) fn law(invocation: &Macro, kind: Kind, scope: &Scope<'_>, place: &Place) -> Vec<Check> {
        let arguments = Macros::arguments(invocation);
        let locals = Locals::default();
        let trace = Trace::new(scope, &locals, place);
        let domain = arguments
            .iter()
            .filter_map(|argument| Extent::strategy(&trace, argument))
            .map(|rows| Domain::new(Breadth::Generated, rows))
            .fold(Domain::GENERATED, Domain::max_with);
        arguments
            .iter()
            .filter_map(Self::body)
            .flat_map(|body| Body::checks(scope, place, &body.block, domain.clone()))
            .chain([Check::new(Relation::Law(kind), Values::zero(), domain.clone())])
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

struct Listed {
    rows: Vec<Row>,
}

impl Parse for Listed {
    fn parse(input: ParseStream) -> syn::Result<Listed> {
        input.call(Attribute::parse_outer)?;
        input.parse::<Ident>()?;
        input.parse::<Token![,]>()?;
        input.parse::<Expr>()?;
        input.parse::<Token![,]>()?;
        let rows;
        bracketed!(rows in input);
        let rows = Punctuated::<Row, Token![,]>::parse_terminated(&rows)?.into_iter().collect();
        input.parse::<Option<Token![,]>>()?;
        Ok(Listed { rows })
    }
}
