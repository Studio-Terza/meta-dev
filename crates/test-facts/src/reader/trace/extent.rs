mod integer;

use syn::token::Minus;
use syn::{BinOp, Expr, ExprCall, ExprRange, RangeLimits, Type};

use lint_core::Syntax;
use lint_layout::{Name, Whole, Word, words};

use super::{Evaluation, Trace};
use crate::boundary::Quantity;
use crate::claim::Ranges;
use integer::Integer;

pub(crate) struct Extent;

impl Extent {
    const ANY: [Word<'static>; 1] = words!["any"];

    pub(crate) fn ranged(trace: &Trace<'_, '_>, range: &ExprRange) -> Option<Ranges> {
        let low = range.start.as_deref().map_or(Some(Quantity::ZERO), |start| Evaluation::quantity_of(trace, start))?;
        let end = range.end.as_deref().and_then(|end| Evaluation::quantity_of(trace, end))?;
        Some(Ranges::between(low, Self::last(&range.limits, end)?))
    }

    pub(crate) fn strategy(trace: &Trace<'_, '_>, strategy: &Expr) -> Option<Ranges> {
        match strategy {
            Expr::Range(range) => Self::ranged(trace, range),
            Expr::Paren(inner) => Self::strategy(trace, &inner.expr),
            Expr::Call(call) => Self::arbitrary(call),
            _ => None,
        }
    }

    pub(crate) fn typed(declared: &Type) -> Option<Ranges> {
        Name::of_type(declared).and_then(|name| Integer::named(&name)).map(Integer::ranges)
    }

    fn last(limits: &RangeLimits, end: Quantity) -> Option<Quantity> {
        match limits {
            RangeLimits::HalfOpen(_) => end.applied(&BinOp::Sub(Minus::default()), Quantity::UNIT),
            RangeLimits::Closed(_) => Some(end),
        }
    }

    fn arbitrary(call: &ExprCall) -> Option<Ranges> {
        Syntax::expr_path(&call.func)
            .and_then(|path| path.path.segments.last())
            .filter(|last| Name::from(&last.ident).matches::<Whole>(&Self::ANY).is_some())
            .and_then(|last| Syntax::angled(&last.arguments))
            .and_then(|angled| angled.args.iter().find_map(Syntax::type_argument))
            .and_then(Self::typed)
    }
}
