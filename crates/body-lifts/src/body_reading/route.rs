use syn::{Arm, Expr, Pat, Stmt};

use super::applied::Applied;

#[derive(Debug)]
pub(super) struct Route;

impl Route {
    pub(super) fn taken(arm: &Arm) -> Option<&Expr> {
        Self::binding(&arm.pat).filter(|_| arm.guard.is_none())?;
        Self::body(&arm.body).and_then(Applied::traced)
    }

    fn body(found: &Expr) -> Option<&Expr> {
        match found {
            Expr::Block(block) => match block.block.stmts.as_slice() {
                [Stmt::Expr(only, None)] => Some(only),
                _ => None,
            },
            other => Some(other),
        }
    }

    fn binding(pattern: &Pat) -> Option<&Pat> {
        match pattern {
            Pat::Ident(_) => Some(pattern),
            Pat::TupleStruct(found) => found.elems.iter().find_map(Self::binding),
            Pat::Tuple(found) => found.elems.iter().find_map(Self::binding),
            Pat::Struct(found) => found.fields.iter().find_map(|field| Self::binding(&field.pat)),
            Pat::Reference(found) => Self::binding(&found.pat),
            Pat::Paren(found) => Self::binding(&found.pat),
            Pat::Or(found) => found.cases.iter().find_map(Self::binding),
            _ => None,
        }
    }
}
