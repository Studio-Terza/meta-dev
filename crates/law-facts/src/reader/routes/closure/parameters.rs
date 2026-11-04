use std::collections::BTreeMap;

use syn::punctuated::Punctuated;
use syn::{Expr, ExprPath, Pat, PatType, Token};

use crate::reader::scope::Scope;
use lint_layout::Name;

#[derive(Debug)]
struct Parameter {
    name: Name,
    typed: Option<Name>,
}

impl Parameter {
    fn written(scope: &Scope, pattern: &Pat) -> Option<Parameter> {
        match pattern {
            Pat::Ident(named) => Some(Parameter { name: Name::from(&named.ident), typed: None }),
            Pat::Type(typed) => Self::annotated(scope, typed),
            _ => None,
        }
    }

    fn annotated(scope: &Scope, typed: &PatType) -> Option<Parameter> {
        match &*typed.pat {
            Pat::Ident(named) => Some(Parameter { name: Name::from(&named.ident), typed: scope.type_named(&typed.ty) }),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub(crate) struct Parameters<'operand> {
    by_name: BTreeMap<Name, Option<Name>>,
    operand: Option<&'operand Name>,
}

impl<'operand> Parameters<'operand> {
    pub(crate) fn declared(
        scope: &Scope,
        inputs: &Punctuated<Pat, Token![,]>,
        operand: Option<&'operand Name>,
    ) -> Parameters<'operand> {
        let by_name = inputs
            .iter()
            .filter_map(|pattern| Parameter::written(scope, pattern))
            .map(|parameter| (parameter.name, parameter.typed))
            .collect();
        Parameters { by_name, operand }
    }

    pub(crate) fn typed(&self, receiver: &Expr) -> Option<Name> {
        match receiver {
            Expr::Path(found) => self.named(found),
            Expr::Paren(inner) => self.typed(&inner.expr),
            Expr::Reference(reference) => self.typed(&reference.expr),
            Expr::Unary(unary) => self.typed(&unary.expr),
            _ => None,
        }
    }

    fn named(&self, found: &ExprPath) -> Option<Name> {
        found
            .path
            .get_ident()
            .and_then(|ident| self.by_name.get(&Name::from(ident)))
            .and_then(|typed| self.or_operand(typed.as_ref()))
    }

    fn or_operand(&self, typed: Option<&Name>) -> Option<Name> {
        typed.or(self.operand).cloned()
    }
}
