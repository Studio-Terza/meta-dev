use std::collections::BTreeMap;

use syn::punctuated::Punctuated;
use syn::{Expr, ExprPath, Pat, PatType, Token};

use crate::law_path::Owner;
use crate::reader::scope::Scope;
use lint_layout::Name;

#[derive(Debug)]
struct Parameter {
    name: Name,
    typed: Option<Name>,
}

impl Parameter {
    fn written(scope: &Scope, pattern: &Pat) -> Option<Self> {
        match pattern {
            Pat::Ident(named) => Some(Self { name: Name::from(&named.ident), typed: None }),
            Pat::Type(typed) => Self::annotated(scope, typed),
            _ => None,
        }
    }

    fn annotated(scope: &Scope, typed: &PatType) -> Option<Self> {
        match &*typed.pat {
            Pat::Ident(named) => Some(Self { name: Name::from(&named.ident), typed: scope.type_named(&typed.ty) }),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub(crate) struct Parameters<'operand> {
    by_name: BTreeMap<Name, Option<Name>>,
    operand: Option<&'operand Owner>,
}

impl<'operand> Parameters<'operand> {
    pub(crate) fn declared(
        scope: &Scope,
        inputs: &Punctuated<Pat, Token![,]>,
        operand: Option<&'operand Owner>,
    ) -> Self {
        let by_name = inputs
            .iter()
            .filter_map(|pattern| Parameter::written(scope, pattern))
            .map(|parameter| (parameter.name, parameter.typed))
            .collect();
        Parameters { by_name, operand }
    }

    pub(crate) fn typed(&self, receiver: &Expr) -> Option<Owner> {
        match receiver {
            Expr::Path(found) => self.named(found),
            Expr::Paren(inner) => self.typed(&inner.expr),
            Expr::Reference(reference) => self.typed(&reference.expr),
            Expr::Unary(unary) => self.typed(&unary.expr),
            _ => None,
        }
    }

    fn named(&self, found: &ExprPath) -> Option<Owner> {
        found
            .path
            .get_ident()
            .and_then(|ident| self.by_name.get(&Name::from(ident)))
            .and_then(|typed| self.or_operand(typed.as_ref()))
    }

    fn or_operand(&self, typed: Option<&Name>) -> Option<Owner> {
        typed.cloned().map(Owner::Named).or_else(|| self.operand.cloned())
    }
}
