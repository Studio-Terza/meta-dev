use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Attribute, Expr, ExprBlock, Ident, Macro, Token, bracketed};

use law_facts::Kind;
use lint_core::{MacroAliases, Macros};
use newtype::Counter;

use super::super::body::Body;
use super::super::scope::Scope;
use super::super::trace::{Leaves, Locals, Trace};
use crate::check::{Cases, Check, Domain, Relation, Rows, Values};

#[derive(Debug)]
pub(crate) struct Table;

impl Table {
    pub(crate) fn checks(invocation: &Macro, scope: &Scope<'_>) -> Vec<Check> {
        let rows: Vec<Expr> = invocation.parse_body_with(Listed::parse).map(|listed| listed.wanted).unwrap_or_default();
        let locals = Locals::default();
        let trace = Trace::new(scope, &locals);
        let leaves = Values::total(rows.iter().map(|row| Leaves::counted(&trace, row)));
        vec![Check::new(Relation::Equal, leaves, Domain::listed(Rows::new(Cases::new(rows.len()), leaves)))]
    }

    pub(crate) fn law(invocation: &Macro, kind: Kind, scope: &Scope<'_>, tables: &MacroAliases) -> Vec<Check> {
        Macros::arguments(invocation)
            .iter()
            .filter_map(Self::body)
            .flat_map(|body| Body::checks(scope, tables, &body.block, Domain::GENERATED))
            .chain([Check::new(Relation::Law(kind), Values::zero(), Domain::GENERATED)])
            .collect()
    }

    fn body(argument: &Expr) -> Option<&ExprBlock> {
        match argument {
            Expr::Block(body) => Some(body),
            _ => None,
        }
    }
}

struct Row {
    wanted: Expr,
}

impl Parse for Row {
    fn parse(input: ParseStream) -> syn::Result<Row> {
        input.parse::<Expr>()?;
        input.parse::<Token![=>]>()?;
        Ok(Row { wanted: input.parse()? })
    }
}

struct Listed {
    wanted: Vec<Expr>,
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
        let wanted = Punctuated::<Row, Token![,]>::parse_terminated(&rows)?.into_iter().map(|row| row.wanted).collect();
        input.parse::<Option<Token![,]>>()?;
        Ok(Listed { wanted })
    }
}
