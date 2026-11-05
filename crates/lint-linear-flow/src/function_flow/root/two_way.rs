use std::ops::Not;

use syn::{Block, Expr, ExprBlock, ExprIf, Ident, Path, Stmt};

use lint_core::Syntax;
use lint_layout::Name;
use lint_placement::CrateName;
use type_facts::{Facts, TypeFact};

#[derive(Clone, Copy, Debug)]
pub struct Target<'ast> {
    name: &'ast Ident,
    facts: &'ast Facts<'ast>,
    krate: &'ast CrateName,
}

impl<'ast> Target<'ast> {
    pub fn new(name: &'ast Ident, facts: &'ast Facts<'ast>, krate: &'ast CrateName) -> Target<'ast> {
        Target { name, facts, krate }
    }

    pub fn enum_only(self) -> Option<Target<'ast>> {
        Some(self).filter(|target| target.facts.enums_named(target.krate, &Name::from(target.name)).is_empty().not())
    }

    fn declaring(&self, chosen: &Ident, other: &Ident) -> Option<&'ast TypeFact<'ast>> {
        let wanted = [Name::from(chosen), Name::from(other)];
        self.facts
            .enums_named(self.krate, &Name::from(self.name))
            .into_iter()
            .find(|each| wanted.iter().all(|variant| each.variant(variant).is_some()))
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
        Some(chosen).filter(|chosen| *chosen != other).filter(|chosen| target.declaring(chosen, other).is_some())
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
    }

    fn unit(branch: &Block) -> Option<&Path> {
        TwoWay::sole(branch).and_then(Syntax::tail).and_then(Syntax::expr_path).map(|bare| &bare.path)
    }
}
