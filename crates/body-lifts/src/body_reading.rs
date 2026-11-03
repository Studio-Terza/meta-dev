mod applied;
mod chain;
mod focus;
mod route;

use syn::{Block, Expr, Stmt};

use crate::body::Body;
use chain::Chain;
use focus::Focus;
use route::Route;

#[derive(Debug)]
pub struct BodyReading;

impl BodyReading {
    #[must_use]
    pub fn classified(block: &Block) -> Body {
        Self::only(block).map_or(Body::Written, Self::shaped)
    }

    fn shaped(found: &Expr) -> Body {
        Focus::reached(found)
            .map(|_| Body::Focused)
            .or_else(|| Route::routed(found).map(|_| Body::Routed))
            .or_else(|| Chain::unwound(found).lifted().map(|_| Body::Mapped))
            .unwrap_or(Body::Written)
    }

    fn only(block: &Block) -> Option<&Expr> {
        block.stmts.first().filter(|_| block.stmts.len() == 1).and_then(Self::expression)
    }

    fn expression(statement: &Stmt) -> Option<&Expr> {
        match statement {
            Stmt::Expr(only, None) => Some(only),
            _ => None,
        }
    }
}
