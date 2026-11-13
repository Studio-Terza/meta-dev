mod finding;
mod nesting_depth;

use finding::Finding;
use lint_core::Headline;
use lint_file::{Reads, SweptLint};
use lint_sweep::{Address, Descend, Finds, Kind, Watch};
use proc_macro2::Span;
use std::path::Path;
use syn::spanned::Spanned;

use syn::{
    Arm, BinOp, Block, Expr, ExprClosure, ExprForLoop, ExprIf, ExprLoop, ExprMethodCall, ExprWhile, FnArg, ImplItemFn,
    ItemFn, ItemImpl, ItemMod, ItemTrait, Lit, Local, Macro, Pat, Signature, UnOp,
};

use lint_core::Bound;
use lint_core::Site;
use lint_core::SourceFile;
use lint_core::Syntax;
use lint_core::Violation;
use lint_core::Walk;
use lint_layout::LineNumber;
use lint_layout::{Length, Name, Prefix, Whole, Word, words};

use nesting_depth::NestingDepth;

#[derive(Debug)]
pub struct FnShape;

impl FnShape {
    const BOOLEAN_METHODS: &[Word<'static>] = &words!["contains", "all", "any", "starts_with", "ends_with", "eq", "ne"];
    const BOOLEAN_PREFIXES: [Word<'static>; 2] = words!["is_", "has_"];
    const NESTING: Bound = Bound::Most(2);
    const PARAMETERS: Bound = Bound::Most(4);
    const FOLDS: [&'static str; 2] = ["fold", "try_fold"];

    fn boolean(expr: &Expr) -> Option<&Expr> {
        match expr {
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
            .then_some(expr),
            Expr::Unary(unary) => matches!(unary.op, UnOp::Not(_)).then_some(expr),
            Expr::Lit(literal) => matches!(literal.lit, Lit::Bool(_)).then_some(expr),
            Expr::Paren(paren) => Self::boolean(&paren.expr).map(|_| expr),
            Expr::MethodCall(call) => Self::boolean_method(&Name::from(&call.method)).map(|_| expr),
            _ => None,
        }
    }

    fn boolean_method(name: &Name) -> Option<Word<'static>> {
        name.matches::<Prefix>(&Self::BOOLEAN_PREFIXES).or_else(|| name.matches::<Whole>(Self::BOOLEAN_METHODS))
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
        Box::new(Shapes {
            path: file.head().path(),
            depth: NestingDepth::default(),
            nests: Vec::new(),
            violations: Vec::new(),
        })
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
    depth: NestingDepth,
    nests: Vec<Nest>,
    violations: Vec<Violation<'scan>>,
}

impl<'scan> Shapes<'scan> {
    fn found(path: &'scan Path, span: Span, finding: Finding) -> Violation<'scan> {
        Violation::new(Site::new(path, LineNumber::from(span)), finding)
    }

    fn nesting(&mut self, at: Address) -> Descend {
        self.nests.iter().rev().find(|nest| nest.at == at).copied().into_iter().for_each(|found| {
            self.depth += NestingDepth::LEVEL;
            let first_past = NestingDepth::beyond(FnShape::NESTING);
            let deepest = (self.depth == first_past).then_some(found.span);
            self.violations.extend(
                deepest.map(|nested| Self::found(self.path, nested, Finding::DeepNesting { most: FnShape::NESTING })),
            );
        });
        Descend::Into
    }

    fn unnesting(&mut self, at: Address) {
        self.nests.iter().rposition(|nest| nest.at == at).into_iter().for_each(|index| {
            self.nests.remove(index);
            self.depth -= NestingDepth::LEVEL;
        });
    }
}

impl<'ast> Watch<'ast> for Shapes<'_> {
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

    fn on_item_mod(&mut self, module: &'ast ItemMod) -> Descend {
        Descend::from(Walk::outside_tests(&module.attrs))
    }

    fn on_item_impl(&mut self, item: &'ast ItemImpl) -> Descend {
        Descend::from(Walk::outside_tests(&item.attrs))
    }

    fn on_item_trait(&mut self, item: &'ast ItemTrait) -> Descend {
        Descend::from(Walk::outside_tests(&item.attrs))
    }

    fn on_macro(&mut self, _: &'ast Macro) -> Descend {
        Descend::Expand
    }

    fn on_item_fn(&mut self, function: &'ast ItemFn) -> Descend {
        Descend::from(Walk::outside_tests(&function.attrs))
    }

    fn on_impl_item_fn(&mut self, function: &'ast ImplItemFn) -> Descend {
        Descend::from(Walk::outside_tests(&function.attrs))
    }

    fn on_signature(&mut self, signature: &'ast Signature) -> Descend {
        let typed = signature.inputs.iter().filter(|argument| matches!(argument, FnArg::Typed(_))).count();
        let wide = FnShape::PARAMETERS.allows(Length::new(typed)).map_or(Some(signature.ident.span()), |_| None);
        self.violations.extend(
            wide.map(|span| Self::found(self.path, span, Finding::WideSignature { most: FnShape::PARAMETERS })),
        );
        Descend::Into
    }

    fn on_local(&mut self, local: &'ast Local) -> Descend {
        self.violations.extend(
            FnShape::binds_bool(local).map(|binding| Self::found(self.path, binding.span(), Finding::BooleanLocal)),
        );
        self.violations.extend(
            local
                .init
                .as_ref()
                .filter(|init| matches!(*init.expr, Expr::Tuple(_)))
                .map(|_| Self::found(self.path, local.span(), Finding::TupleLocal)),
        );
        Descend::Into
    }

    fn on_expr_method_call(&mut self, call: &'ast ExprMethodCall) -> Descend {
        self.violations.extend(
            call.args
                .first()
                .filter(|seed| matches!(seed, Expr::Tuple(_)))
                .filter(|_| FnShape::FOLDS.iter().any(|fold| call.method == fold))
                .map(|_| Self::found(self.path, call.method.span(), Finding::TupleAccumulator)),
        );
        Descend::Into
    }

    fn on_expr_if(&mut self, expr: &'ast ExprIf) -> Descend {
        self.nests.push(Nest { at: Address::from(&expr.then_branch), span: expr.if_token.span });
        self.nests.extend(expr.else_branch.as_ref().and_then(|(_, branch)| Nest::unchained(branch)));
        Descend::Into
    }

    fn on_arm(&mut self, arm: &'ast Arm) -> Descend {
        self.nests.push(Nest { at: Address::from(&*arm.body), span: arm.body.span() });
        Descend::Into
    }

    fn on_expr_for_loop(&mut self, expr: &'ast ExprForLoop) -> Descend {
        self.nests.push(Nest { at: Address::from(&expr.body), span: expr.for_token.span });
        Descend::Into
    }

    fn on_expr_while(&mut self, expr: &'ast ExprWhile) -> Descend {
        self.nests.push(Nest { at: Address::from(&expr.body), span: expr.while_token.span });
        Descend::Into
    }

    fn on_expr_loop(&mut self, expr: &'ast ExprLoop) -> Descend {
        self.nests.push(Nest { at: Address::from(&expr.body), span: expr.loop_token.span });
        Descend::Into
    }

    fn on_expr_closure(&mut self, closure: &'ast ExprClosure) -> Descend {
        self.nests.push(Nest { at: Address::from(&*closure.body), span: closure.or1_token.span });
        Descend::Into
    }

    fn on_block(&mut self, block: &'ast Block) -> Descend {
        self.nesting(Address::from(block))
    }

    fn off_block(&mut self, block: &'ast Block) {
        self.unnesting(Address::from(block));
    }

    fn on_expr(&mut self, expr: &'ast Expr) -> Descend {
        self.nesting(Address::from(expr))
    }

    fn off_expr(&mut self, expr: &'ast Expr) {
        self.unnesting(Address::from(expr));
    }
}

impl<'file> Finds<'file> for Shapes<'file> {
    fn found(self: Box<Self>) -> Vec<Violation<'file>> {
        self.violations
    }
}
