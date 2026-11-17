use lint_core::Extended;
use lint_core::Headline;
use lint_file::{Reads, SweptLint};
use lint_sweep::{Address, Descend, Facts, Finds, Kind};
use lint_sweep::{Fold, Folding, Visit};
use std::path::Path;
use syntax::Span;

use syntax::{
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
use lint_layout::LineNumber;

#[derive(Debug)]
pub struct ConstShape;

impl ConstShape {}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Context {
    #[default]
    Surface,
    Const,
}

impl SweptLint for ConstShape {
    const HEADLINE: Headline =
        Headline::new("casts live in const fns or boundary modules, and const loops walk slices");
    const READS: Reads = Reads::Every;

    fn watcher(file: &SourceFile) -> Box<dyn Finds<'_> + '_> {
        Box::new(Folding::from(ConstContexts { path: AsRef::<Path>::as_ref(file.head()), zone: Zone::of_file(file) }))
    }
}

impl Context {
    fn of_fn(constness: Option<&syntax::token::Const>) -> Context {
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

#[derive(Debug)]
struct ConstContexts<'scan> {
    path: &'scan Path,
    zone: Zone,
}

#[derive(Debug, Default)]
struct Consted<'scan> {
    zone: Option<Zone>,
    zones: Vec<Zone>,
    context: Contexts,
    pending: Vec<Address>,
    violations: Vec<Violation<'scan>>,
}

impl<'scan> ConstContexts<'scan> {
    fn violation(&self, span: Span, message: Message) -> Violation<'scan> {
        Violation::new(Site::new(self.path, LineNumber::from(span)), message)
    }

    fn cast_finding(&self, found: &Consted<'scan>, cast: &ExprCast) -> Option<Violation<'scan>> {
        Some(found)
            .filter(|scan| scan.context.current == Context::Surface)
            .filter(|scan| scan.zone.unwrap_or(self.zone) == Zone::Elsewhere)
            .map(|_| {
                self.violation(
                    cast.as_token.span,
                    Message::new(
                        Problem::new("as cast outside a const fn"),
                        Fix::new("use From, TryFrom, or an EnumMap"),
                    ),
                )
            })
    }

    fn looped(&self, found: Consted<'scan>, span: Span, message: Message) -> Visit<Consted<'scan>> {
        let flagged = (found.context.current == Context::Const).then(|| self.violation(span, message));
        Visit::descending(found.flagged(flagged))
    }
}

impl<'scan> Consted<'scan> {
    fn flagged(self, more: impl IntoIterator<Item = Violation<'scan>>) -> Consted<'scan> {
        Consted { violations: self.violations.extended(more), ..self }
    }

    fn within(self, context: Context) -> Visit<Consted<'scan>> {
        Visit::descending(Consted { context: self.context.within(context), ..self })
    }

    fn restored(self) -> Consted<'scan> {
        Consted { context: self.context.restored(), ..self }
    }

    fn awaiting(self, more: impl IntoIterator<Item = Address>) -> Visit<Consted<'scan>> {
        Visit::descending(Consted { pending: self.pending.extended(more), ..self })
    }
}

#[derive(Clone, Copy, Debug)]
enum Mark {
    Entered,
    Left,
    Unmarked,
}

#[derive(Debug, Default)]
struct Contexts {
    current: Context,
    outer: Vec<Context>,
}

impl Contexts {
    fn within(self, context: Context) -> Contexts {
        Contexts { current: context, outer: self.outer.extended([self.current]) }
    }

    fn restored(self) -> Contexts {
        let mut outer = self.outer;
        let current = outer.pop().unwrap_or(Context::Surface);
        Contexts { current, outer }
    }

    fn marked(self, mark: Mark) -> Contexts {
        match mark {
            Mark::Entered => self.within(Context::Const),
            Mark::Left => self.restored(),
            Mark::Unmarked => self,
        }
    }
}

impl<'scan> Fold for ConstContexts<'scan> {
    type Found = Consted<'scan>;

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

    fn on_item_mod(
        &self,
        found: Consted<'scan>,
        module: &ItemMod,
        facts: &Facts<'_, ItemMod>,
    ) -> Visit<Consted<'scan>> {
        let zone = found.zone.unwrap_or(self.zone);
        let zones = found.zones.extended([zone]);
        Visit::new(Consted { zone: Some(zone.entering(module)), zones, ..found }, Descend::from(facts.gate()))
    }

    fn off_item_mod(&self, found: Consted<'scan>, _: &ItemMod, _: &Facts<'_, ItemMod>) -> Consted<'scan> {
        let mut zones = found.zones;
        let zone = zones.pop().or(found.zone);
        Consted { zone, zones, ..found }
    }

    fn on_item_fn(&self, found: Consted<'scan>, function: &ItemFn, _: &Facts<'_, ItemFn>) -> Visit<Consted<'scan>> {
        found.within(Context::of_fn(function.sig.constness.as_ref()))
    }

    fn off_item_fn(&self, found: Consted<'scan>, _: &ItemFn, _: &Facts<'_, ItemFn>) -> Consted<'scan> {
        found.restored()
    }

    fn on_impl_item_fn(
        &self,
        found: Consted<'scan>,
        function: &ImplItemFn,
        _: &Facts<'_, ImplItemFn>,
    ) -> Visit<Consted<'scan>> {
        found.within(Context::of_fn(function.sig.constness.as_ref()))
    }

    fn off_impl_item_fn(&self, found: Consted<'scan>, _: &ImplItemFn, _: &Facts<'_, ImplItemFn>) -> Consted<'scan> {
        found.restored()
    }

    fn on_trait_item_fn(
        &self,
        found: Consted<'scan>,
        _: &TraitItemFn,
        _: &Facts<'_, TraitItemFn>,
    ) -> Visit<Consted<'scan>> {
        found.within(Context::Surface)
    }

    fn off_trait_item_fn(&self, found: Consted<'scan>, _: &TraitItemFn, _: &Facts<'_, TraitItemFn>) -> Consted<'scan> {
        found.restored()
    }

    fn on_item_const(&self, found: Consted<'scan>, _: &ItemConst, _: &Facts<'_, ItemConst>) -> Visit<Consted<'scan>> {
        found.within(Context::Const)
    }

    fn off_item_const(&self, found: Consted<'scan>, _: &ItemConst, _: &Facts<'_, ItemConst>) -> Consted<'scan> {
        found.restored()
    }

    fn on_item_static(
        &self,
        found: Consted<'scan>,
        _: &ItemStatic,
        _: &Facts<'_, ItemStatic>,
    ) -> Visit<Consted<'scan>> {
        found.within(Context::Const)
    }

    fn off_item_static(&self, found: Consted<'scan>, _: &ItemStatic, _: &Facts<'_, ItemStatic>) -> Consted<'scan> {
        found.restored()
    }

    fn on_impl_item_const(
        &self,
        found: Consted<'scan>,
        _: &ImplItemConst,
        _: &Facts<'_, ImplItemConst>,
    ) -> Visit<Consted<'scan>> {
        found.within(Context::Const)
    }

    fn off_impl_item_const(
        &self,
        found: Consted<'scan>,
        _: &ImplItemConst,
        _: &Facts<'_, ImplItemConst>,
    ) -> Consted<'scan> {
        found.restored()
    }

    fn on_trait_item_const(
        &self,
        found: Consted<'scan>,
        _: &TraitItemConst,
        _: &Facts<'_, TraitItemConst>,
    ) -> Visit<Consted<'scan>> {
        found.within(Context::Const)
    }

    fn off_trait_item_const(
        &self,
        found: Consted<'scan>,
        _: &TraitItemConst,
        _: &Facts<'_, TraitItemConst>,
    ) -> Consted<'scan> {
        found.restored()
    }

    fn on_expr_const(&self, found: Consted<'scan>, _: &ExprConst, _: &Facts<'_, ExprConst>) -> Visit<Consted<'scan>> {
        found.within(Context::Const)
    }

    fn off_expr_const(&self, found: Consted<'scan>, _: &ExprConst, _: &Facts<'_, ExprConst>) -> Consted<'scan> {
        found.restored()
    }

    fn on_type_array(
        &self,
        found: Consted<'scan>,
        array: &TypeArray,
        _: &Facts<'_, TypeArray>,
    ) -> Visit<Consted<'scan>> {
        found.awaiting([Address::from(&array.len)])
    }

    fn on_expr_repeat(
        &self,
        found: Consted<'scan>,
        repeat: &ExprRepeat,
        _: &Facts<'_, ExprRepeat>,
    ) -> Visit<Consted<'scan>> {
        found.awaiting([Address::from(&*repeat.len)])
    }

    fn on_variant(&self, found: Consted<'scan>, variant: &Variant, _: &Facts<'_, Variant>) -> Visit<Consted<'scan>> {
        found.awaiting(variant.discriminant.iter().map(|(_, discriminant)| Address::from(discriminant)))
    }

    fn on_expr(&self, found: Consted<'scan>, _: &Expr, facts: &Facts<'_, Expr>) -> Visit<Consted<'scan>> {
        let at = facts.address();
        let pending = found.pending.iter().rposition(|waiting| *waiting == at);
        Visit::descending(Consted {
            context: found.context.marked(pending.map_or(Mark::Unmarked, |_| Mark::Entered)),
            ..found
        })
    }

    fn off_expr(&self, found: Consted<'scan>, _: &Expr, facts: &Facts<'_, Expr>) -> Consted<'scan> {
        let at = facts.address();
        let index = found.pending.iter().rposition(|waiting| *waiting == at);
        let mut pending = found.pending;
        let waited = index.map(|waiting| pending.remove(waiting));
        Consted { pending, context: found.context.marked(waited.map_or(Mark::Unmarked, |_| Mark::Left)), ..found }
    }

    fn on_macro(&self, found: Consted<'scan>, _: &Macro, _: &Facts<'_, Macro>) -> Visit<Consted<'scan>> {
        Visit::new(found, Descend::Expand)
    }

    fn on_expr_cast(&self, found: Consted<'scan>, cast: &ExprCast, _: &Facts<'_, ExprCast>) -> Visit<Consted<'scan>> {
        let finding = self.cast_finding(&found, cast);
        Visit::descending(found.flagged(finding))
    }

    fn on_expr_while(
        &self,
        found: Consted<'scan>,
        expr: &ExprWhile,
        _: &Facts<'_, ExprWhile>,
    ) -> Visit<Consted<'scan>> {
        match Looping::from(&*expr.cond) {
            Looping::Counted => self.looped(
                found,
                expr.while_token.span,
                Message::new(
                    Problem::new("counter loop in const context"),
                    Fix::new("walk a slice with while let [head, rest @ ..]"),
                ),
            ),
            Looping::Walking => Visit::descending(found),
        }
    }

    fn on_expr_loop(&self, found: Consted<'scan>, expr: &ExprLoop, _: &Facts<'_, ExprLoop>) -> Visit<Consted<'scan>> {
        self.looped(
            found,
            expr.loop_token.span,
            Message::new(
                Problem::new("open loop in const context"),
                Fix::new("walk a slice with while let or recurse"),
            ),
        )
    }

    fn on_expr_for_loop(
        &self,
        found: Consted<'scan>,
        expr: &ExprForLoop,
        _: &Facts<'_, ExprForLoop>,
    ) -> Visit<Consted<'scan>> {
        self.looped(
            found,
            expr.for_token.span,
            Message::new(
                Problem::new("for loop in const context"),
                Fix::new("walk a slice with while let [head, rest @ ..]"),
            ),
        )
    }
}

impl<'scan> From<Consted<'scan>> for Vec<Violation<'scan>> {
    fn from(consted: Consted<'scan>) -> Vec<Violation<'scan>> {
        consted.violations
    }
}
