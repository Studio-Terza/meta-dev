use std::ops::Not;

use syn::{Block, Expr, ExprBlock, ExprIf, Ident, Path, Stmt};

use lint_core::Syntax;
use lint_layout::Name;
use type_facts::{Facts, TypeFact, Vantage};

#[derive(Clone, Copy, Debug)]
pub struct Target<'ast> {
    name: &'ast Ident,
    facts: &'ast Facts<'ast>,
    vantage: &'ast Vantage<'ast>,
}

impl<'ast> Target<'ast> {
    pub fn new(name: &'ast Ident, facts: &'ast Facts<'ast>, vantage: &'ast Vantage<'ast>) -> Target<'ast> {
        Target { name, facts, vantage }
    }

    pub fn enum_only(self) -> Option<Target<'ast>> {
        Some(self).filter(|target| target.enums().is_empty().not())
    }

    fn enums(&self) -> Vec<&'ast TypeFact<'ast>> {
        self.facts.enums_named(self.vantage, &Name::from(self.name))
    }

    fn declared(&self, variant: &Name) -> Option<&'ast Name> {
        let found: Option<Vec<&'ast Name>> = self.enums().into_iter().map(|each| each.variant(variant)).collect();
        found.and_then(|all| all.first().copied())
    }
}

#[derive(Debug)]
pub struct TwoWay;

impl TwoWay {
    pub fn sole(body: &Block) -> Option<&Stmt> {
        body.stmts.split_first().and_then(|(only, rest)| rest.first().map_or(Some(only), |_| None))
    }

    pub fn within<'body>(body: &'body Block, input: &Ident, target: Target<'_>) -> Option<&'body ExprIf> {
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

    pub fn between<'routed>(routed: &'routed ExprIf, target: Target<'_>) -> Option<&'routed Ident> {
        let (_, otherwise) = routed.else_branch.as_ref()?;
        let chosen = Self::variant_of(Self::unit(&routed.then_branch)?, target)?;
        let other = Self::variant_of(Self::unit(&TwoWay::block(otherwise)?.block)?, target)?;
        Some(chosen).filter(|chosen| *chosen != other)
    }

    fn variant_of<'path>(path: &'path Path, target: Target<'_>) -> Option<&'path Ident> {
        let own = Ident::new(Self::SELF, target.name.span());
        let accepted = [target.name, &own];
        path.segments
            .iter()
            .nth_back(1)
            .filter(|qualifier| accepted.iter().any(|name| qualifier.ident == **name))
            .and(path.segments.last())
            .map(|segment| &segment.ident)
            .filter(|variant| target.declared(&Name::from(*variant)).is_some())
    }

    fn unit(branch: &Block) -> Option<&Path> {
        TwoWay::sole(branch).and_then(Syntax::tail).and_then(Syntax::expr_path).map(|bare| &bare.path)
    }
}
