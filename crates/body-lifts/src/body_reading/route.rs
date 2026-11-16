use syntax::{Application, Arm, Expr, ExprMatch, Pat, PatEdge, PatReached, PatTree};

use super::BodyReading;

#[derive(Debug)]
pub(super) struct Route;

impl Route {
    pub(super) fn taken(arm: &Arm) -> Option<&Expr> {
        Self::binding(&arm.pat).filter(|_| arm.guard.is_none())?;
        Self::body(&arm.body).and_then(Application::root)
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
        let opens = |pat: &Pat, _: PatEdge| {
            matches!(
                pat,
                Pat::TupleStruct(_) | Pat::Tuple(_) | Pat::Struct(_) | Pat::Reference(_) | Pat::Paren(_) | Pat::Or(_)
            )
        };
        PatTree::reached(pattern, &opens).into_iter().map(PatReached::pat).find(|pat| matches!(pat, Pat::Ident(_)))
    }
}
