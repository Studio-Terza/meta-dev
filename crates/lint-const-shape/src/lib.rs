use lint_core::Headline;
use lint_file::{Reads, SweptLint};
use lint_sweep::{Address, Descend, Finds, Kind, Watch};
use proc_macro2::Span;
use std::path::Path;

use syn::{
    Expr, ExprCast, ExprConst, ExprForLoop, ExprLoop, ExprRepeat, ExprWhile, ImplItemConst, ImplItemFn, ItemConst,
    ItemFn, ItemMod, ItemStatic, Macro, TraitItemConst, TraitItemFn, TypeArray, Variant,
};

use lint_core::Fix;
use lint_core::Gate;
use lint_core::Message;
use lint_core::Problem;
use lint_core::Site;
use lint_core::SourceFile;
use lint_core::Violation;
use lint_core::Walk;
use lint_layout::LineNumber;

#[derive(Debug)]
pub struct ConstShape;

impl ConstShape {}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Context {
    Surface,
    Const,
}

impl SweptLint for ConstShape {
    const HEADLINE: Headline =
        Headline::new("casts live in const fns or boundary modules, and const loops walk slices");
    const READS: Reads = Reads::Every;

    fn watcher(file: &SourceFile) -> Box<dyn Finds<'_> + '_> {
        Box::new(ConstContexts {
            path: file.head().path(),
            context: Contexts { current: Context::Surface, outer: Vec::new() },
            zone: Zone::of_file(file),
            zones: Vec::new(),
            pending: Vec::new(),
            violations: Vec::new(),
        })
    }
}

impl Context {
    fn of_fn(constness: Option<&syn::token::Const>) -> Context {
        constness.map_or(Context::Surface, |_| Context::Const)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Looping {
    Counted,
    Walking,
}

impl From<&Expr> for Looping {
    fn from(condition: &Expr) -> Looping {
        match condition {
            Expr::Let(_) => Looping::Walking,
            _ => Looping::Counted,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Zone {
    Boundary,
    Elsewhere,
}

impl Zone {
    fn of_file(file: &SourceFile) -> Zone {
        Gate::BOUNDARY.admits(file.head()).map_or(Zone::Elsewhere, |_| Zone::Boundary)
    }

    fn entering(self, module: &ItemMod) -> Zone {
        Gate::BOUNDARY.names(module).map_or(self, |_| Zone::Boundary)
    }
}

struct ConstContexts<'scan> {
    path: &'scan Path,
    context: Contexts,
    zone: Zone,
    zones: Vec<Zone>,
    pending: Vec<Address>,
    violations: Vec<Violation<'scan>>,
}

impl<'scan> ConstContexts<'scan> {
    fn violation(&self, span: Span, message: Message) -> Violation<'scan> {
        Violation::new(Site::new(self.path, LineNumber::from(span)), message)
    }

    fn cast_finding(&self, cast: &ExprCast) -> Option<Violation<'scan>> {
        Some(self)
            .filter(|scan| scan.context.current == Context::Surface)
            .filter(|scan| scan.zone == Zone::Elsewhere)
            .map(|scan| {
                scan.violation(
                    cast.as_token.span,
                    Message::new(
                        Problem::new("as cast outside a const fn"),
                        Fix::new("use From, TryFrom, or an EnumMap"),
                    ),
                )
            })
    }

    fn loop_finding(&self, span: Span, message: Message) -> Option<Violation<'scan>> {
        (self.context.current == Context::Const).then(|| self.violation(span, message))
    }
}

struct Contexts {
    current: Context,
    outer: Vec<Context>,
}

impl Contexts {
    fn within(&mut self, context: Context) -> Descend {
        self.outer.push(std::mem::replace(&mut self.current, context));
        Descend::Into
    }

    fn restored(&mut self) {
        self.current = self.outer.pop().unwrap_or(Context::Surface);
    }
}

impl<'ast> Watch<'ast> for ConstContexts<'_> {
    fn kinds(&self) -> &'static [Kind] {
        &[
            Kind::ItemMod,
            Kind::ItemFn,
            Kind::ImplItemFn,
            Kind::TraitItemFn,
            Kind::ItemConst,
            Kind::ItemStatic,
            Kind::ImplItemConst,
            Kind::TraitItemConst,
            Kind::ExprConst,
            Kind::TypeArray,
            Kind::ExprRepeat,
            Kind::Variant,
            Kind::Macro,
            Kind::ExprCast,
            Kind::ExprWhile,
            Kind::ExprLoop,
            Kind::ExprForLoop,
            Kind::Expr,
        ]
    }

    fn on_item_mod(&mut self, module: &'ast ItemMod) -> Descend {
        self.zones.push(self.zone);
        self.zone = self.zone.entering(module);
        Descend::from(Walk::outside_tests(&module.attrs))
    }

    fn off_item_mod(&mut self, _: &'ast ItemMod) {
        self.zone = self.zones.pop().unwrap_or(self.zone);
    }

    fn on_item_fn(&mut self, function: &'ast ItemFn) -> Descend {
        self.context.within(Context::of_fn(function.sig.constness.as_ref()))
    }

    fn off_item_fn(&mut self, _: &'ast ItemFn) {
        self.context.restored();
    }

    fn on_impl_item_fn(&mut self, function: &'ast ImplItemFn) -> Descend {
        self.context.within(Context::of_fn(function.sig.constness.as_ref()))
    }

    fn off_impl_item_fn(&mut self, _: &'ast ImplItemFn) {
        self.context.restored();
    }

    fn on_trait_item_fn(&mut self, _: &'ast TraitItemFn) -> Descend {
        self.context.within(Context::Surface)
    }

    fn off_trait_item_fn(&mut self, _: &'ast TraitItemFn) {
        self.context.restored();
    }

    fn on_item_const(&mut self, _: &'ast ItemConst) -> Descend {
        self.context.within(Context::Const)
    }

    fn off_item_const(&mut self, _: &'ast ItemConst) {
        self.context.restored();
    }

    fn on_item_static(&mut self, _: &'ast ItemStatic) -> Descend {
        self.context.within(Context::Const)
    }

    fn off_item_static(&mut self, _: &'ast ItemStatic) {
        self.context.restored();
    }

    fn on_impl_item_const(&mut self, _: &'ast ImplItemConst) -> Descend {
        self.context.within(Context::Const)
    }

    fn off_impl_item_const(&mut self, _: &'ast ImplItemConst) {
        self.context.restored();
    }

    fn on_trait_item_const(&mut self, _: &'ast TraitItemConst) -> Descend {
        self.context.within(Context::Const)
    }

    fn off_trait_item_const(&mut self, _: &'ast TraitItemConst) {
        self.context.restored();
    }

    fn on_expr_const(&mut self, _: &'ast ExprConst) -> Descend {
        self.context.within(Context::Const)
    }

    fn off_expr_const(&mut self, _: &'ast ExprConst) {
        self.context.restored();
    }

    fn on_type_array(&mut self, array: &'ast TypeArray) -> Descend {
        self.pending.push(Address::from(&array.len));
        Descend::Into
    }

    fn on_expr_repeat(&mut self, repeat: &'ast ExprRepeat) -> Descend {
        self.pending.push(Address::from(&*repeat.len));
        Descend::Into
    }

    fn on_variant(&mut self, variant: &'ast Variant) -> Descend {
        self.pending.extend(variant.discriminant.iter().map(|(_, discriminant)| Address::from(discriminant)));
        Descend::Into
    }

    fn on_expr(&mut self, expr: &'ast Expr) -> Descend {
        let at = Address::from(expr);
        let pending = self.pending.iter().rposition(|waiting| *waiting == at);
        pending.map_or(Descend::Into, |_| self.context.within(Context::Const))
    }

    fn off_expr(&mut self, expr: &'ast Expr) {
        let at = Address::from(expr);
        self.pending.iter().rposition(|waiting| *waiting == at).into_iter().for_each(|index| {
            self.pending.remove(index);
            self.context.restored();
        });
    }

    fn on_macro(&mut self, _: &'ast Macro) -> Descend {
        Descend::Expand
    }

    fn on_expr_cast(&mut self, cast: &'ast ExprCast) -> Descend {
        let finding = self.cast_finding(cast);
        self.violations.extend(finding);
        Descend::Into
    }

    fn on_expr_while(&mut self, expr: &'ast ExprWhile) -> Descend {
        let finding = Some(Looping::from(&*expr.cond)).filter(|looping| *looping == Looping::Counted).map(|_| {
            self.loop_finding(
                expr.while_token.span,
                Message::new(
                    Problem::new("counter loop in const context"),
                    Fix::new("walk a slice with while let [head, rest @ ..]"),
                ),
            )
        });
        self.violations.extend(finding.flatten());
        Descend::Into
    }

    fn on_expr_loop(&mut self, expr: &'ast ExprLoop) -> Descend {
        let finding = self.loop_finding(
            expr.loop_token.span,
            Message::new(
                Problem::new("open loop in const context"),
                Fix::new("walk a slice with while let or recurse"),
            ),
        );
        self.violations.extend(finding);
        Descend::Into
    }

    fn on_expr_for_loop(&mut self, expr: &'ast ExprForLoop) -> Descend {
        let finding = self.loop_finding(
            expr.for_token.span,
            Message::new(
                Problem::new("for loop in const context"),
                Fix::new("walk a slice with while let [head, rest @ ..]"),
            ),
        );
        self.violations.extend(finding);
        Descend::Into
    }
}

impl<'file> Finds<'file> for ConstContexts<'file> {
    fn found(self: Box<Self>) -> Vec<Violation<'file>> {
        self.violations
    }
}
