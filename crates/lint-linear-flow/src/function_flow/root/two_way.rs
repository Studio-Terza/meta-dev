mod boundary;

use syn::{Block, Expr, ExprBlock, ExprIf, Ident, Path, Stmt};

use boundary::Casing;
use lint_core::Syntax;

#[derive(Debug)]
pub struct TwoWay;

impl TwoWay {
    pub fn sole(body: &Block) -> Option<&Stmt> {
        body.stmts.split_first().and_then(|(only, rest)| rest.first().map_or(Some(only), |_| None))
    }

    pub fn within<'body>(body: &'body Block, input: &Ident, target: &Ident) -> Option<&'body ExprIf> {
        Self::sole(body)
            .and_then(Self::branching)
            .filter(|routed| Syntax::expr_path(&routed.cond).is_some_and(|cond| cond.path.is_ident(input)))
            .filter(|routed| Variants::between(routed, target).is_some())
    }

    fn branching(statement: &Stmt) -> Option<&ExprIf> {
        match statement {
            Stmt::Expr(Expr::If(routed), None) => Some(routed),
            _ => None,
        }
    }

    pub fn block(otherwise: &Expr) -> Option<&ExprBlock> {
        match otherwise {
            Expr::Block(block) => Some(block),
            _ => None,
        }
    }
}

#[derive(Debug)]
struct Variants;

impl Variants {
    const SELF: &'static str = "Self";

    pub fn between<'routed>(routed: &'routed ExprIf, target: &Ident) -> Option<&'routed Ident> {
        let (_, otherwise) = routed.else_branch.as_ref()?;
        let chosen = Self::variant_of(Self::unit(&routed.then_branch)?, target)?;
        let other = Self::variant_of(Self::unit(&TwoWay::block(otherwise)?.block)?, target)?;
        Some(chosen).filter(|chosen| *chosen != other)
    }

    fn variant_of<'path>(path: &'path Path, target: &Ident) -> Option<&'path Ident> {
        let own = Ident::new(Self::SELF, target.span());
        let accepted = [target, &own];
        path.segments
            .iter()
            .nth_back(1)
            .filter(|qualifier| accepted.iter().any(|name| qualifier.ident == **name))
            .and(path.segments.last())
            .map(|segment| &segment.ident)
            .filter(|variant| Casing::from(*variant) == Casing::Variant)
    }

    fn unit(branch: &Block) -> Option<&Path> {
        TwoWay::sole(branch).and_then(Self::tail).and_then(Syntax::expr_path).map(|bare| &bare.path)
    }

    fn tail(statement: &Stmt) -> Option<&Expr> {
        match statement {
            Stmt::Expr(expr, None) => Some(expr),
            _ => None,
        }
    }
}
