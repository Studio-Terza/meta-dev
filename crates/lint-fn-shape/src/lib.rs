mod finding;
mod nesting_depth;

use algebra::{Monoid, Semigroup};
use finding::Finding;
use lint_core::Headline;
use lint_file::{Reads, SweptLint};
use lint_sweep::{Concludes, Contribute, Contributed, Contributing, Contribution, Descend, Facts, Finds, Kind, Scope};
use std::path::Path;
use syntax::Span;
use syntax::Spanned;
use syntax::{Past, Spine};

use syntax::{
    Arm, BinOp, Expr, ExprClosure, ExprForLoop, ExprIf, ExprLoop, ExprMethodCall, ExprWhile, FnArg, ImplItemFn, ItemFn,
    ItemImpl, ItemMod, ItemTrait, Lit, Local, Macro, Pat, Signature, UnOp,
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
        Box::new(Contributing::new(Shapes { path: AsRef::<Path>::as_ref(file.head()) }))
    }
}

#[derive(Clone, Copy, Debug)]
struct Nest {
    span: Span,
    depth: NestingDepth,
}

impl Nest {
    fn within(self, bodies: &[Span]) -> Nest {
        let start = self.span.byte_range().start;
        bodies
            .iter()
            .find(|body| body.byte_range().contains(&start))
            .map_or(self, |_| Nest { span: self.span, depth: self.depth.combine(NestingDepth::LEVEL) })
    }
}

#[derive(Debug, Default)]
struct Shaped<'scan> {
    found: Vec<Violation<'scan>>,
    nests: Vec<Nest>,
}

impl Semigroup for Shaped<'_> {
    fn combine(self, other: Self) -> Self {
        Shaped { found: self.found.combine(other.found), nests: self.nests.combine(other.nests) }
    }
}

impl Monoid for Shaped<'_> {}

impl<'scan> Shaped<'scan> {
    fn nested(self, bodies: impl IntoIterator<Item = Option<Span>>, own: Span) -> Shaped<'scan> {
        let ranges: Vec<Span> = bodies.into_iter().flatten().collect();
        let past = NestingDepth::beyond(FnShape::NESTING);
        let deeper = self.nests.into_iter().map(|nest| nest.within(&ranges)).filter(|nest| nest.depth <= past);
        let nests = deeper.chain([Nest { span: own, depth: NestingDepth::LEVEL }]).collect();
        Shaped { found: self.found, nests }
    }
}

#[derive(Debug)]
struct Shapes<'scan> {
    path: &'scan Path,
}

impl<'scan> Shapes<'scan> {
    fn found(path: &'scan Path, span: Span, finding: Finding) -> Violation<'scan> {
        Violation::new(Site::new(path, LineNumber::from(span)), finding)
    }

    fn findings(found: impl IntoIterator<Item = Violation<'scan>>) -> Contributed<(), Shaped<'scan>> {
        Contributed::parted(Shaped { found: found.into_iter().collect(), nests: Vec::new() })
    }

    fn unchained(branch: &Expr) -> Option<Span> {
        match branch {
            Expr::If(_) => None,
            _ => Some(branch.span()),
        }
    }
}

impl<'scan> Contribute for Shapes<'scan> {
    type Open = ();
    type Part = Shaped<'scan>;

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
        ]
    }

    fn scopes(&self) -> &'static [Scope] {
        const {
            &[
                Scope::new(Kind::ExprIf),
                Scope::new(Kind::Arm),
                Scope::new(Kind::ExprForLoop),
                Scope::new(Kind::ExprWhile),
                Scope::new(Kind::ExprLoop),
                Scope::new(Kind::ExprClosure),
            ]
        }
    }

    fn on_item_mod(
        &self,
        _: &ItemMod,
        facts: &Facts<'_, ItemMod>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), Shaped<'scan>> {
        Contributed::descending(Descend::from(facts.gate()))
    }

    fn on_item_impl(
        &self,
        _: &ItemImpl,
        facts: &Facts<'_, ItemImpl>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), Shaped<'scan>> {
        Contributed::descending(Descend::from(facts.gate()))
    }

    fn on_item_trait(
        &self,
        _: &ItemTrait,
        facts: &Facts<'_, ItemTrait>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), Shaped<'scan>> {
        Contributed::descending(Descend::from(facts.gate()))
    }

    fn on_macro(
        &self,
        _: &Macro,
        _: &Facts<'_, Macro>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), Shaped<'scan>> {
        Contributed::descending(Descend::Expand)
    }

    fn on_item_fn(
        &self,
        _: &ItemFn,
        facts: &Facts<'_, ItemFn>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), Shaped<'scan>> {
        Contributed::descending(Descend::from(facts.gate()))
    }

    fn on_impl_item_fn(
        &self,
        _: &ImplItemFn,
        facts: &Facts<'_, ImplItemFn>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), Shaped<'scan>> {
        Contributed::descending(Descend::from(facts.gate()))
    }

    fn on_signature(
        &self,
        signature: &Signature,
        _: &Facts<'_, Signature>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), Shaped<'scan>> {
        let typed = signature.inputs.iter().filter(|argument| matches!(argument, FnArg::Typed(_))).count();
        let wide = FnShape::PARAMETERS.allows(Length::new(typed)).map_or(Some(signature.ident.span()), |_| None);
        Self::findings(
            wide.map(|span| Self::found(self.path, span, Finding::WideSignature { most: FnShape::PARAMETERS })),
        )
    }

    fn on_local(
        &self,
        local: &Local,
        _: &Facts<'_, Local>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), Shaped<'scan>> {
        let boolean =
            FnShape::binds_bool(local).map(|binding| Self::found(self.path, binding.span(), Finding::BooleanLocal));
        let tuple = local
            .init
            .as_ref()
            .filter(|init| matches!(*init.expr, Expr::Tuple(_)))
            .map(|_| Self::found(self.path, local.span(), Finding::TupleLocal));
        Self::findings(boolean.into_iter().chain(tuple))
    }

    fn on_expr_method_call(
        &self,
        call: &ExprMethodCall,
        _: &Facts<'_, ExprMethodCall>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), Shaped<'scan>> {
        Self::findings(
            call.args
                .first()
                .filter(|seed| matches!(seed, Expr::Tuple(_)))
                .filter(|_| FnShape::FOLDS.iter().any(|fold| call.method == fold))
                .map(|_| Self::found(self.path, call.method.span(), Finding::TupleAccumulator)),
        )
    }

    fn off_expr_if(
        &self,
        expr: &ExprIf,
        _: &Facts<'_, ExprIf>,
        inner: Contribution<(), Shaped<'scan>>,
    ) -> Shaped<'scan> {
        let otherwise = expr.else_branch.as_ref().and_then(|(_, branch)| Self::unchained(branch));
        inner.concluded(|(), shaped| shaped.nested([Some(expr.then_branch.span()), otherwise], expr.if_token.span))
    }

    fn off_arm(&self, arm: &Arm, _: &Facts<'_, Arm>, inner: Contribution<(), Shaped<'scan>>) -> Shaped<'scan> {
        inner.concluded(|(), shaped| shaped.nested([Some(arm.body.span())], arm.body.span()))
    }

    fn off_expr_for_loop(
        &self,
        expr: &ExprForLoop,
        _: &Facts<'_, ExprForLoop>,
        inner: Contribution<(), Shaped<'scan>>,
    ) -> Shaped<'scan> {
        inner.concluded(|(), shaped| shaped.nested([Some(expr.body.span())], expr.for_token.span))
    }

    fn off_expr_while(
        &self,
        expr: &ExprWhile,
        _: &Facts<'_, ExprWhile>,
        inner: Contribution<(), Shaped<'scan>>,
    ) -> Shaped<'scan> {
        inner.concluded(|(), shaped| shaped.nested([Some(expr.body.span())], expr.while_token.span))
    }

    fn off_expr_loop(
        &self,
        expr: &ExprLoop,
        _: &Facts<'_, ExprLoop>,
        inner: Contribution<(), Shaped<'scan>>,
    ) -> Shaped<'scan> {
        inner.concluded(|(), shaped| shaped.nested([Some(expr.body.span())], expr.loop_token.span))
    }

    fn off_expr_closure(
        &self,
        closure: &ExprClosure,
        _: &Facts<'_, ExprClosure>,
        inner: Contribution<(), Shaped<'scan>>,
    ) -> Shaped<'scan> {
        inner.concluded(|(), shaped| shaped.nested([Some(closure.body.span())], closure.or1_token.span))
    }
}

impl<'scan> Concludes<'scan> for Shapes<'scan> {
    fn violations(self, (): (), shaped: Shaped<'scan>) -> Vec<Violation<'scan>> {
        let past = NestingDepth::beyond(FnShape::NESTING);
        let deep = shaped
            .nests
            .into_iter()
            .filter(|nest| nest.depth == past)
            .map(|nest| Self::found(self.path, nest.span, Finding::DeepNesting { most: FnShape::NESTING }));
        let mut found: Vec<Violation<'scan>> = shaped.found.into_iter().chain(deep).collect();
        found.sort_by_key(|violation| *violation.site());
        found
    }
}
