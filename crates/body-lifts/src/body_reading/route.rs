use syn::{Arm, Expr, ExprMatch, Pat};

use super::BodyReading;
use super::applied::Applied;

#[derive(Debug)]
pub(super) struct Route;

impl Route {
    pub(super) fn taken(arm: &Arm) -> Option<&Expr> {
        Self::binding(&arm.pat).filter(|_| arm.guard.is_none())?;
        Self::body(&arm.body).and_then(Applied::traced)
    }

    pub(super) fn routed(found: &Expr) -> Option<&ExprMatch> {
        match found {
            Expr::Match(routed) => Self::all_taken(routed),
            _ => None,
        }
    }

    fn all_taken(routed: &ExprMatch) -> Option<&ExprMatch> {
        Some(routed).filter(|candidate| candidate.arms.iter().all(|arm| Self::taken(arm).is_some()))
    }

    fn body(found: &Expr) -> Option<&Expr> {
        match found {
            Expr::Block(block) => BodyReading::only(&block.block),
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
