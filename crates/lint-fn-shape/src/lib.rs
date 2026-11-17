mod finding;
mod nesting_depth;

use finding::Finding;
use lint_core::Headline;
use lint_core::{Fold, Folding, Visit};
use lint_file::{Reads, SweptLint};
use lint_sweep::{Address, Descend, Facts, Finds, Kind};
use std::iter;
use std::path::Path;
use syntax::Span;
use syntax::Spanned;
use syntax::{Past, Spine};

use syntax::{
    Arm, BinOp, Block, Expr, ExprClosure, ExprForLoop, ExprIf, ExprLoop, ExprMethodCall, ExprWhile, FnArg, ImplItemFn,
    ItemFn, ItemImpl, ItemMod, ItemTrait, Lit, Local, Macro, Pat, Signature, UnOp,
};

use lint_core::Bound;
use lint_core::Site;
use lint_core::SourceFile;
use lint_core::Syntax;
use lint_core::Violation;
use lint_layout::LineNumber;
use lint_layout::{Length, Name, Prefix, Whole, Word, words};

use nesting_depth::NestingDepth;

#[derive(Debug)]
pub struct FnShape;

#[derive(Debug)]
struct BooleanMethod;

impl BooleanMethod {
    fn named(name: &Name) -> Option<()> {
        name.matches::<Prefix>(&FnShape::BOOLEAN_PREFIXES)
            .or_else(|| name.matches::<Whole>(FnShape::BOOLEAN_METHODS))
            .map(|_| ())
    }
}

impl FnShape {
    const BOOLEAN_METHODS: &[Word<'static>] = &words!["contains", "all", "any", "starts_with", "ends_with", "eq", "ne"];
    const BOOLEAN_PREFIXES: [Word<'static>; 2] = words!["is_", "has_"];
    const NESTING: Bound = Bound::Most(2);
    const PARAMETERS: Bound = Bound::Most(4);
    const FOLDS: [&'static str; 2] = ["fold", "try_fold"];

    fn boolean(expr: &Expr) -> Option<&Expr> {
        let inner = Spine::from(expr).past(|node| matches!(node, Expr::Paren(_)))?;
        Self::bare_boolean(inner).map(|()| expr)
    }

    fn bare_boolean(peeled: Past<'_, Expr>) -> Option<()> {
        match peeled.node() {
            Expr::Binary(binary) => matches!(
                binary.op,
                BinOp::Eq(_)
                    | BinOp::Ne(_)
                    | BinOp::Lt(_)
                    | BinOp::Le(_)
                    | BinOp::Gt(_)
                    | BinOp::Ge(_)
                    | BinOp::And(_)
                    | BinOp::Or(_)
            )
            .then_some(()),
            Expr::Unary(unary) => matches!(unary.op, UnOp::Not(_)).then_some(()),
            Expr::Lit(literal) => matches!(literal.lit, Lit::Bool(_)).then_some(()),
            Expr::MethodCall(call) => BooleanMethod::named(&Name::from(&call.method)),
            _ => None,
        }
    }

    fn binds_bool(local: &Local) -> Option<&Local> {
        Self::typed_bool(&local.pat)
            .map(|_| local)
            .or_else(|| local.init.as_ref().and_then(|init| Self::boolean(&init.expr)).map(|_| local))
    }

    fn typed_bool(pattern: &Pat) -> Option<&Pat> {
        match pattern {
            Pat::Type(typed) => Syntax::type_path(&typed.ty).filter(|path| path.path.is_ident("bool")).map(|_| pattern),
            _ => None,
        }
    }
}

impl SweptLint for FnShape {
    const HEADLINE: Headline =
        Headline::new("hairy control flow; booleans and tuples become types, nesting becomes methods");
    const READS: Reads = Reads::Shipped;

    fn watcher(file: &SourceFile) -> Box<dyn Finds<'_> + '_> {
        Box::new(Folding::from(Shapes { path: AsRef::<Path>::as_ref(file.head()) }))
    }
}

#[derive(Clone, Copy, Debug)]
struct Nest {
    at: Address,
    span: Span,
}

impl Nest {
    fn unchained(branch: &Expr) -> Option<Nest> {
        match branch {
            Expr::If(_) => None,
            _ => Some(Nest { at: Address::from(branch), span: branch.span() }),
        }
    }
}

#[derive(Debug)]
struct Shapes<'scan> {
    path: &'scan Path,
}

#[derive(Debug, Default)]
struct Shaped<'scan> {
    depth: NestingDepth,
    nests: Vec<Nest>,
    violations: Vec<Violation<'scan>>,
}

impl<'scan> Shapes<'scan> {
    fn found(&self, span: Span, finding: Finding) -> Violation<'scan> {
        Violation::new(Site::new(self.path, LineNumber::from(span)), finding)
    }

    fn nesting(&self, found: Shaped<'scan>, at: Address) -> Visit<Shaped<'scan>> {
        let nest = found.nests.iter().rev().find(|nest| nest.at == at).copied();
        let depth = nest.map_or(found.depth, |_| found.depth + NestingDepth::LEVEL);
        let deepest = nest.and_then(|nested| (depth == NestingDepth::beyond(FnShape::NESTING)).then_some(nested.span));
        let flagged = deepest.map(|span| self.found(span, Finding::DeepNesting { most: FnShape::NESTING }));
        Visit::descending(Shaped { depth, ..found }.flagged(flagged))
    }
}

impl<'scan> Shaped<'scan> {
    fn flagged(self, more: impl IntoIterator<Item = Violation<'scan>>) -> Shaped<'scan> {
        Shaped { violations: self.violations.into_iter().chain(more).collect(), ..self }
    }

    fn pushed(self, more: impl IntoIterator<Item = Nest>) -> Visit<Shaped<'scan>> {
        Visit::descending(Shaped { nests: self.nests.into_iter().chain(more).collect(), ..self })
    }

    fn unnesting(self, at: Address) -> Shaped<'scan> {
        let index = self.nests.iter().rposition(|nest| nest.at == at);
        let depth = index.map_or(self.depth, |_| self.depth - NestingDepth::LEVEL);
        let nests = self.nests.into_iter().enumerate().filter(|(kept, _)| Some(*kept) != index).map(|(_, nest)| nest);
        Shaped { depth, nests: nests.collect(), ..self }
    }
}

impl<'scan> From<Shaped<'scan>> for Vec<Violation<'scan>> {
    fn from(shaped: Shaped<'scan>) -> Vec<Violation<'scan>> {
        shaped.violations
    }
}

impl<'scan> Fold for Shapes<'scan> {
    type Found = Shaped<'scan>;

    fn kinds(&self) -> &'static [Kind] {
        &[
            Kind::ItemMod,
            Kind::ItemImpl,
            Kind::ItemTrait,
            Kind::Macro,
            Kind::ItemFn,
            Kind::ImplItemFn,
            Kind::Signature,
            Kind::Local,
            Kind::ExprMethodCall,
            Kind::ExprIf,
            Kind::Arm,
            Kind::ExprForLoop,
            Kind::ExprWhile,
            Kind::ExprLoop,
            Kind::ExprClosure,
            Kind::Block,
            Kind::Expr,
        ]
    }

    fn on_item_mod(&self, found: Shaped<'scan>, _: &ItemMod, facts: &Facts<'_, ItemMod>) -> Visit<Shaped<'scan>> {
        Visit::new(found, Descend::from(facts.gate()))
    }

    fn on_item_impl(&self, found: Shaped<'scan>, _: &ItemImpl, facts: &Facts<'_, ItemImpl>) -> Visit<Shaped<'scan>> {
        Visit::new(found, Descend::from(facts.gate()))
    }

    fn on_item_trait(&self, found: Shaped<'scan>, _: &ItemTrait, facts: &Facts<'_, ItemTrait>) -> Visit<Shaped<'scan>> {
        Visit::new(found, Descend::from(facts.gate()))
    }

    fn on_macro(&self, found: Shaped<'scan>, _: &Macro, _: &Facts<'_, Macro>) -> Visit<Shaped<'scan>> {
        Visit::new(found, Descend::Expand)
    }

    fn on_item_fn(&self, found: Shaped<'scan>, _: &ItemFn, facts: &Facts<'_, ItemFn>) -> Visit<Shaped<'scan>> {
        Visit::new(found, Descend::from(facts.gate()))
    }

    fn on_impl_item_fn(
        &self,
        found: Shaped<'scan>,
        _: &ImplItemFn,
        facts: &Facts<'_, ImplItemFn>,
    ) -> Visit<Shaped<'scan>> {
        Visit::new(found, Descend::from(facts.gate()))
    }

    fn on_signature(
        &self,
        found: Shaped<'scan>,
        signature: &Signature,
        _: &Facts<'_, Signature>,
    ) -> Visit<Shaped<'scan>> {
        let typed = signature.inputs.iter().filter(|argument| matches!(argument, FnArg::Typed(_))).count();
        let wide = FnShape::PARAMETERS.allows(Length::new(typed)).map_or(Some(signature.ident.span()), |_| None);
        let flagged = wide.map(|span| self.found(span, Finding::WideSignature { most: FnShape::PARAMETERS }));
        Visit::descending(found.flagged(flagged))
    }

    fn on_local(&self, found: Shaped<'scan>, local: &Local, _: &Facts<'_, Local>) -> Visit<Shaped<'scan>> {
        let boolean = FnShape::binds_bool(local).map(|binding| self.found(binding.span(), Finding::BooleanLocal));
        let tupled = local
            .init
            .as_ref()
            .filter(|init| matches!(*init.expr, Expr::Tuple(_)))
            .map(|_| self.found(local.span(), Finding::TupleLocal));
        Visit::descending(found.flagged(boolean.into_iter().chain(tupled)))
    }

    fn on_expr_method_call(
        &self,
        found: Shaped<'scan>,
        call: &ExprMethodCall,
        _: &Facts<'_, ExprMethodCall>,
    ) -> Visit<Shaped<'scan>> {
        let flagged = call
            .args
            .first()
            .filter(|seed| matches!(seed, Expr::Tuple(_)))
            .filter(|_| FnShape::FOLDS.iter().any(|fold| call.method == fold))
            .map(|_| self.found(call.method.span(), Finding::TupleAccumulator));
        Visit::descending(found.flagged(flagged))
    }

    fn on_expr_if(&self, found: Shaped<'scan>, expr: &ExprIf, _: &Facts<'_, ExprIf>) -> Visit<Shaped<'scan>> {
        let then = Nest { at: Address::from(&expr.then_branch), span: expr.if_token.span };
        let otherwise = expr.else_branch.as_ref().and_then(|(_, branch)| Nest::unchained(branch));
        found.pushed(iter::once(then).chain(otherwise))
    }

    fn on_arm(&self, found: Shaped<'scan>, arm: &Arm, _: &Facts<'_, Arm>) -> Visit<Shaped<'scan>> {
        found.pushed([Nest { at: Address::from(&*arm.body), span: arm.body.span() }])
    }

    fn on_expr_for_loop(
        &self,
        found: Shaped<'scan>,
        expr: &ExprForLoop,
        _: &Facts<'_, ExprForLoop>,
    ) -> Visit<Shaped<'scan>> {
        found.pushed([Nest { at: Address::from(&expr.body), span: expr.for_token.span }])
    }

    fn on_expr_while(&self, found: Shaped<'scan>, expr: &ExprWhile, _: &Facts<'_, ExprWhile>) -> Visit<Shaped<'scan>> {
        found.pushed([Nest { at: Address::from(&expr.body), span: expr.while_token.span }])
    }

    fn on_expr_loop(&self, found: Shaped<'scan>, expr: &ExprLoop, _: &Facts<'_, ExprLoop>) -> Visit<Shaped<'scan>> {
        found.pushed([Nest { at: Address::from(&expr.body), span: expr.loop_token.span }])
    }

    fn on_expr_closure(
        &self,
        found: Shaped<'scan>,
        closure: &ExprClosure,
        _: &Facts<'_, ExprClosure>,
    ) -> Visit<Shaped<'scan>> {
        found.pushed([Nest { at: Address::from(&*closure.body), span: closure.or1_token.span }])
    }

    fn on_block(&self, found: Shaped<'scan>, _: &Block, facts: &Facts<'_, Block>) -> Visit<Shaped<'scan>> {
        self.nesting(found, facts.address())
    }

    fn off_block(&self, found: Shaped<'scan>, _: &Block, facts: &Facts<'_, Block>) -> Shaped<'scan> {
        found.unnesting(facts.address())
    }

    fn on_expr(&self, found: Shaped<'scan>, _: &Expr, facts: &Facts<'_, Expr>) -> Visit<Shaped<'scan>> {
        self.nesting(found, facts.address())
    }

    fn off_expr(&self, found: Shaped<'scan>, _: &Expr, facts: &Facts<'_, Expr>) -> Shaped<'scan> {
        found.unnesting(facts.address())
    }
}
