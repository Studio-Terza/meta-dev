use syntax::{Application, Expr, ExprMethodCall, Spine};

use lint_layout::{Name, Whole, Word, words};

#[derive(Debug)]
pub(super) struct Chain<'body> {
    calls: Vec<&'body ExprMethodCall>,
    root: &'body Expr,
}

impl<'body> Chain<'body> {
    const STEPS: [Word<'static>; 9] =
        words!["iter", "into_iter", "map", "collect", "cloned", "copied", "as_ref", "as_deref", "as_slice"];
    const MAP: [Word<'static>; 1] = words!["map"];

    pub(super) fn unwound(found: &'body Expr) -> Chain<'body> {
        let mut calls: Vec<&ExprMethodCall> = Spine::from(found).map_while(Self::called).collect();
        calls.reverse();
        let root = Spine::from(found).find(|node| Self::called(node).is_none()).unwrap_or(found);
        Chain { calls, root }
    }

    fn called(node: &'body Expr) -> Option<&'body ExprMethodCall> {
        match node {
            Expr::MethodCall(call) => Some(call),
            _ => None,
        }
    }

    pub(super) fn lifted(&self) -> Option<&'body Expr> {
        let maps: Vec<&ExprMethodCall> = self
            .calls
            .iter()
            .copied()
            .filter(|call| Name::from(&call.method).matches::<Whole>(&Self::MAP).is_some())
            .collect();
        let mapped = maps.first();
        self.calls
            .iter()
            .all(|call| Name::from(&call.method).matches::<Whole>(&Self::STEPS).is_some())
            .then_some(&maps)
            .filter(|mapping_calls| {
                mapping_calls.iter().flat_map(|call| call.args.iter()).all(|argument| Self::mapping(argument).is_some())
            })
            .and(mapped)
            .and_then(|_| Application::root(self.root))
    }

    fn mapping(argument: &'body Expr) -> Option<&'body Expr> {
        match argument {
            Expr::Path(_) => Some(argument),
            Expr::Closure(closure) => Application::root(&closure.body).map(|_| argument),
            _ => None,
        }
    }
}
