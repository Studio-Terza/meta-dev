use syntax::{BinOp, Expr, ExprReference, ExprTree, Fold, LitText, Member, UnOp};

use graph::{Argument, Infix, LiteralValue, Operator, Prefix};
use lint_core::Syntax;
use lint_layout::Name;

#[derive(Clone, Debug)]
pub(super) enum Shaped<Leaf> {
    Leaf(Leaf),
    Literal(LiteralValue),
    Applied { operator: Operator, operands: Vec<Shaped<Leaf>> },
    Opaque,
}

impl<Leaf> Shaped<Leaf> {
    pub(super) fn folded(expr: &Expr, leaf: &impl Fn(&Expr) -> Option<Leaf>) -> Shaped<Leaf> {
        let fold = Fold::over(ExprTree::from(expr), |done, node| Shaping { done: Done(done), leaf }.shaped(node));
        Done(&fold).child(expr)
    }

    pub(super) fn leaf(&self) -> Option<&Leaf> {
        match self {
            Shaped::Leaf(found) => Some(found),
            Shaped::Literal(_) | Shaped::Applied { .. } | Shaped::Opaque => None,
        }
    }

    pub(super) fn mapped<Next>(self, leaf: &impl Fn(Leaf) -> Shaped<Next>) -> Shaped<Next> {
        match self {
            Shaped::Leaf(found) => leaf(found),
            Shaped::Literal(value) => Shaped::Literal(value),
            Shaped::Applied { operator, operands } => Shaped::Applied {
                operator,
                operands: operands.into_iter().map(|operand| operand.mapped(leaf)).collect(),
            },
            Shaped::Opaque => Shaped::Opaque,
        }
    }

    pub(super) fn argument(self, leaf: &impl Fn(Leaf) -> Argument) -> Argument {
        match self {
            Shaped::Leaf(found) => leaf(found),
            Shaped::Literal(value) => Argument::Literal(value),
            Shaped::Applied { operator, operands } => {
                Argument::applied(operator, operands.into_iter().map(|operand| operand.argument(leaf)).collect())
            }
            Shaped::Opaque => Argument::Opaque,
        }
    }
}

struct Done<'fold, 'syn, Leaf>(&'fold Fold<'syn, Expr, Shaped<Leaf>>);

impl<Leaf> Done<'_, '_, Leaf> {
    fn child(&self, expr: &Expr) -> Shaped<Leaf> {
        self.0.take(expr).unwrap_or(Shaped::Opaque)
    }
}

struct Shaping<'fold, 'syn, Leaf, Reads> {
    done: Done<'fold, 'syn, Leaf>,
    leaf: &'fold Reads,
}

impl<Leaf, Reads: Fn(&Expr) -> Option<Leaf>> Shaping<'_, '_, Leaf, Reads> {
    fn shaped(&self, expr: &Expr) -> Shaped<Leaf> {
        let leaf = (self.leaf)(expr).map(Shaped::Leaf);
        leaf.or_else(|| self.node(expr)).unwrap_or(Shaped::Opaque)
    }

    fn node(&self, expr: &Expr) -> Option<Shaped<Leaf>> {
        match expr {
            Expr::Paren(inner) => Some(self.done.child(&inner.expr)),
            Expr::Lit(literal) => Some(Shaped::Literal(LiteralValue::from(LitText::written(&literal.lit).as_str()))),
            Expr::Binary(binary) => self.operated(Operators::infix(&binary.op), [&*binary.left, &*binary.right]),
            Expr::Unary(unary) => self.operated(Operators::prefix(unary.op), [&*unary.expr]),
            Expr::Tuple(tuple) => self.operated(Some(Operator::Tuple), tuple.elems.iter()),
            Expr::Array(array) => self.operated(Some(Operator::Array), array.elems.iter()),
            Expr::Index(index) => self.operated(Some(Operator::Index), [&*index.expr, &*index.index]),
            Expr::Reference(reference) => self.operated(Operators::borrow(reference), [&*reference.expr]),
            Expr::Call(call) => self.operated(Some(Operator::Call), [&*call.func].into_iter().chain(&call.args)),
            Expr::MethodCall(call) => self.operated(
                Some(Operator::Method(Name::from(&call.method))),
                [&*call.receiver].into_iter().chain(&call.args),
            ),
            Expr::Field(field) => self.operated(Operators::field(&field.member), [&*field.base]),
            _ => None,
        }
    }

    fn operated<'expr>(
        &self,
        operator: Option<Operator>,
        operands: impl IntoIterator<Item = &'expr Expr>,
    ) -> Option<Shaped<Leaf>> {
        operator.map(|found| Shaped::Applied {
            operator: found,
            operands: operands.into_iter().map(|operand| self.done.child(operand)).collect(),
        })
    }
}

struct Operators;

impl Operators {
    fn infix(op: &BinOp) -> Option<Operator> {
        match op {
            BinOp::Add(_) => Some(Operator::Infix(Infix::Add)),
            BinOp::Sub(_) => Some(Operator::Infix(Infix::Subtract)),
            BinOp::Mul(_) => Some(Operator::Infix(Infix::Multiply)),
            BinOp::Div(_) => Some(Operator::Infix(Infix::Divide)),
            BinOp::Rem(_) => Some(Operator::Infix(Infix::Remainder)),
            BinOp::And(_) => Some(Operator::Infix(Infix::And)),
            BinOp::Or(_) => Some(Operator::Infix(Infix::Or)),
            BinOp::BitXor(_) => Some(Operator::Infix(Infix::BitXor)),
            BinOp::BitAnd(_) => Some(Operator::Infix(Infix::BitAnd)),
            BinOp::BitOr(_) => Some(Operator::Infix(Infix::BitOr)),
            BinOp::Shl(_) => Some(Operator::Infix(Infix::ShiftLeft)),
            BinOp::Shr(_) => Some(Operator::Infix(Infix::ShiftRight)),
            BinOp::Eq(_) => Some(Operator::Infix(Infix::Equal)),
            BinOp::Lt(_) => Some(Operator::Infix(Infix::Less)),
            BinOp::Le(_) => Some(Operator::Infix(Infix::LessOrEqual)),
            BinOp::Ne(_) => Some(Operator::Infix(Infix::NotEqual)),
            BinOp::Ge(_) => Some(Operator::Infix(Infix::GreaterOrEqual)),
            BinOp::Gt(_) => Some(Operator::Infix(Infix::Greater)),
            _ => None,
        }
    }

    fn prefix(op: UnOp) -> Option<Operator> {
        match op {
            UnOp::Deref(_) => Some(Operator::Prefix(Prefix::Deref)),
            UnOp::Not(_) => Some(Operator::Prefix(Prefix::Not)),
            UnOp::Neg(_) => Some(Operator::Prefix(Prefix::Negate)),
            _ => None,
        }
    }

    fn borrow(reference: &ExprReference) -> Option<Operator> {
        reference.mutability.map_or(Some(Operator::Borrow), |_| None)
    }

    fn field(member: &Member) -> Option<Operator> {
        Syntax::named_member(member).map(|named| Operator::Field(Name::from(named)))
    }
}
