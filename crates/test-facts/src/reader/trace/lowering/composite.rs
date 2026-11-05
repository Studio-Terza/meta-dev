use std::iter;

use syn::{Expr, ExprBinary, ExprCall, ExprMethodCall, ExprUnary, Macro, UnOp};

use lint_core::{Macros, Syntax};
use lint_layout::{Name, Whole, Word, words};

use super::{Lowering, Operations};
use crate::boundary::{Quantity, Spelling};
use crate::claim::{Callee, Operator, Term};

pub(crate) struct Composite;

impl Composite {
    const TRANSPARENT: [Word<'static>; 3] = words!["clone", "to_owned", "borrow"];
    const SEQUENCES: [Word<'static>; 1] = words!["vec"];

    pub(crate) fn unary(lowering: &mut Lowering<'_, '_, '_>, unary: &ExprUnary) -> Term {
        match unary.op {
            UnOp::Neg(_) => Callee::Operator(Operator::Subtract)
                .folded(vec![Term::Number(Quantity::ZERO), lowering.term(&unary.expr)]),
            UnOp::Not(_) => Callee::Operator(Operator::Not).folded(vec![lowering.term(&unary.expr)]),
            _ => lowering.term(&unary.expr),
        }
    }

    pub(crate) fn binary(lowering: &mut Lowering<'_, '_, '_>, binary: &ExprBinary) -> Term {
        Operations::operator(binary.op).map_or_else(
            || Term::Opaque(Spelling::written(binary)),
            |operator| {
                Callee::Operator(operator).folded(vec![lowering.term(&binary.left), lowering.term(&binary.right)])
            },
        )
    }

    pub(crate) fn call(lowering: &mut Lowering<'_, '_, '_>, call: &ExprCall) -> Term {
        let arguments: Vec<Term> = call.args.iter().map(|argument| lowering.term(argument)).collect();
        let trace = lowering.trace;
        Syntax::expr_path(&call.func).map_or_else(
            || Term::Opaque(Spelling::written(call)),
            |path| {
                let callee = Operations::constructed(&path.path).map_or_else(
                    || Callee::Function(trace.scope().called(&path.path, trace.place())),
                    Callee::Constructor,
                );
                callee.folded(arguments)
            },
        )
    }

    pub(crate) fn method(lowering: &mut Lowering<'_, '_, '_>, call: &ExprMethodCall) -> Term {
        let name = Name::from(&call.method);
        let receiver = lowering.term(&call.receiver);
        let arguments: Vec<Term> =
            iter::once(receiver.clone()).chain(call.args.iter().map(|argument| lowering.term(argument))).collect();
        name.matches::<Whole>(&Self::TRANSPARENT)
            .map_or_else(|| Callee::Method(name.clone()).folded(arguments), |_| receiver)
    }

    pub(crate) fn invoked(lowering: &mut Lowering<'_, '_, '_>, invocation: &Macro, expression: &Expr) -> Term {
        let name = invocation.path.segments.last().map(|last| Name::from(&last.ident)).unwrap_or_default();
        let elements: Vec<Term> =
            Macros::arguments(invocation).iter().map(|argument| lowering.term(argument)).collect();
        name.matches::<Whole>(&Self::SEQUENCES)
            .map_or_else(|| Term::Opaque(Spelling::written(expression)), |_| Callee::Sequence.folded(elements))
    }
}
