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
        match block.stmts.as_slice() {
            [Stmt::Expr(only, None)] => Self::shaped(only),
            _ => Body::Written,
        }
    }

    fn shaped(found: &Expr) -> Body {
        match found {
            _ if Focus::reached(found).is_some() => Body::Focused,
            Expr::Match(routed) if routed.arms.iter().all(|arm| Route::taken(arm).is_some()) => Body::Routed,
            _ if Chain::unwound(found).lifted().is_some() => Body::Mapped,
            _ => Body::Written,
        }
    }
}
