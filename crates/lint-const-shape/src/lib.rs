use algebra::{Monoid, Semigroup};
use lint_core::Headline;
use lint_file::{Reads, SweptLint};
use lint_sweep::{Concludes, Contribute, Contributed, Contributing, Contribution, Descend, Facts, Finds, Kind, Scope};
use std::path::Path;
use syntax::Span;
use syntax::Spanned;

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Context {
    Surface,
    Const,
}

impl SweptLint for ConstShape {
    const HEADLINE: Headline =
        Headline::new("casts live in const fns or boundary modules, and const loops walk slices");
    const READS: Reads = Reads::Every;

    fn watcher(file: &SourceFile) -> Box<dyn Finds<'_> + '_> {
        Box::new(Contributing::new(ConstContexts {
            path: AsRef::<Path>::as_ref(file.head()),
            zone: Zone::of_file(file),
        }))
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
        Gate::BOUNDARY.admits(file.head()).chosen(Zone::Boundary, Zone::Elsewhere)
    }

    fn of_module(module: &ItemMod) -> Zone {
        Gate::BOUNDARY.names(module).chosen(Zone::Boundary, Zone::Elsewhere)
    }
}

#[derive(Debug)]
struct ConstContexts<'scan> {
    path: &'scan Path,
    zone: Zone,
}

#[derive(Clone, Copy, Debug)]
struct Looped {
    span: Span,
    message: Message,
}

#[derive(Debug, Default)]
struct Candidates<'scan> {
    found: Vec<Violation<'scan>>,
    casts: Vec<Span>,
    surface_casts: Vec<Span>,
    loops: Vec<Looped>,
}

impl Semigroup for Candidates<'_> {
    fn combine(self, other: Self) -> Self {
        Candidates {
            found: self.found.combine(other.found),
            casts: self.casts.combine(other.casts),
            surface_casts: self.surface_casts.combine(other.surface_casts),
            loops: self.loops.combine(other.loops),
        }
    }
}

impl Monoid for Candidates<'_> {}

impl<'scan> Candidates<'scan> {
    const CAST: Message =
        Message::new(Problem::new("as cast outside a const fn"), Fix::new("use From, TryFrom, or an EnumMap"));

    fn within(self, context: Context, scan: &ConstContexts<'scan>) -> Candidates<'scan> {
        match context {
            Context::Const => Candidates {
                found: self
                    .found
                    .into_iter()
                    .chain(self.loops.into_iter().map(|looped| scan.violation(looped.span, looped.message)))
                    .collect(),
                casts: Vec::new(),
                surface_casts: self.surface_casts,
                loops: Vec::new(),
            },
            Context::Surface => Candidates {
                found: self.found,
                casts: Vec::new(),
                surface_casts: self.surface_casts.combine(self.casts),
                loops: Vec::new(),
            },
        }
    }

    fn zoned(self, zone: Zone) -> Candidates<'scan> {
        match zone {
            Zone::Boundary => Candidates { found: self.found, ..Candidates::default() },
            Zone::Elsewhere => self,
        }
    }

    fn split(self, length: &impl Spanned) -> Split<'scan> {
        let range = length.span().byte_range();
        let inside = |span: &Span| range.contains(&span.byte_range().start);
        let (_, outside_casts): (Vec<Span>, Vec<Span>) = self.casts.into_iter().partition(inside);
        let (loops, outside_loops): (Vec<Looped>, Vec<Looped>) =
            self.loops.into_iter().partition(|looped| inside(&looped.span));
        Split {
            length: Candidates { loops, ..Candidates::default() },
            rest: Candidates {
                found: self.found,
                casts: outside_casts,
                surface_casts: self.surface_casts,
                loops: outside_loops,
            },
        }
    }
}

#[derive(Debug)]
struct Split<'scan> {
    length: Candidates<'scan>,
    rest: Candidates<'scan>,
}

impl<'scan> Split<'scan> {
    fn constant(self, scan: &ConstContexts<'scan>) -> Candidates<'scan> {
        self.rest.combine(self.length.within(Context::Const, scan))
    }
}

impl<'scan> ConstContexts<'scan> {
    fn violation(&self, span: Span, message: Message) -> Violation<'scan> {
        Violation::new(Site::new(self.path, LineNumber::from(span)), message)
    }

    fn closed(&self, context: Context, inner: Contribution<(), Candidates<'scan>>) -> Candidates<'scan> {
        inner.concluded(|(), candidates| candidates.within(context, self))
    }

    fn lengths<'length>(
        &self,
        lengths: impl IntoIterator<Item = &'length Expr>,
        inner: Contribution<(), Candidates<'scan>>,
    ) -> Candidates<'scan> {
        let candidates = inner.concluded(|(), candidates| candidates);
        lengths.into_iter().fold(candidates, |remaining, length| remaining.split(length).constant(self))
    }

    fn looped(span: Span, message: Message) -> Contributed<(), Candidates<'scan>> {
        Contributed::parted(Candidates { loops: vec![Looped { span, message }], ..Candidates::default() })
    }
}

impl<'scan> Contribute for ConstContexts<'scan> {
    type Open = ();
    type Part = Candidates<'scan>;

    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemMod, Kind::Macro, Kind::ExprCast, Kind::ExprWhile, Kind::ExprLoop, Kind::ExprForLoop]
    }

    fn scopes(&self) -> &'static [Scope] {
        const {
            &[
                Scope::new(Kind::ItemMod),
                Scope::new(Kind::ItemFn),
                Scope::new(Kind::ImplItemFn),
                Scope::new(Kind::TraitItemFn),
                Scope::new(Kind::ItemConst),
                Scope::new(Kind::ItemStatic),
                Scope::new(Kind::ImplItemConst),
                Scope::new(Kind::TraitItemConst),
                Scope::new(Kind::ExprConst),
                Scope::new(Kind::TypeArray),
                Scope::new(Kind::ExprRepeat),
                Scope::new(Kind::Variant),
            ]
        }
    }

    fn on_item_mod(
        &self,
        _: &ItemMod,
        facts: &Facts<'_, ItemMod>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), Candidates<'scan>> {
        Contributed::descending(Descend::from(facts.gate()))
    }

    fn off_item_mod(
        &self,
        module: &ItemMod,
        _: &Facts<'_, ItemMod>,
        inner: Contribution<(), Candidates<'scan>>,
    ) -> Candidates<'scan> {
        let zone = Zone::of_module(module);
        inner.concluded(|(), candidates| candidates.zoned(zone))
    }

    fn off_item_fn(
        &self,
        function: &ItemFn,
        _: &Facts<'_, ItemFn>,
        inner: Contribution<(), Candidates<'scan>>,
    ) -> Candidates<'scan> {
        self.closed(Context::of_fn(function.sig.constness.as_ref()), inner)
    }

    fn off_impl_item_fn(
        &self,
        function: &ImplItemFn,
        _: &Facts<'_, ImplItemFn>,
        inner: Contribution<(), Candidates<'scan>>,
    ) -> Candidates<'scan> {
        self.closed(Context::of_fn(function.sig.constness.as_ref()), inner)
    }

    fn off_trait_item_fn(
        &self,
        _: &TraitItemFn,
        _: &Facts<'_, TraitItemFn>,
        inner: Contribution<(), Candidates<'scan>>,
    ) -> Candidates<'scan> {
        self.closed(Context::Surface, inner)
    }

    fn off_item_const(
        &self,
        _: &ItemConst,
        _: &Facts<'_, ItemConst>,
        inner: Contribution<(), Candidates<'scan>>,
    ) -> Candidates<'scan> {
        self.closed(Context::Const, inner)
    }

    fn off_item_static(
        &self,
        _: &ItemStatic,
        _: &Facts<'_, ItemStatic>,
        inner: Contribution<(), Candidates<'scan>>,
    ) -> Candidates<'scan> {
        self.closed(Context::Const, inner)
    }

    fn off_impl_item_const(
        &self,
        _: &ImplItemConst,
        _: &Facts<'_, ImplItemConst>,
        inner: Contribution<(), Candidates<'scan>>,
    ) -> Candidates<'scan> {
        self.closed(Context::Const, inner)
    }

    fn off_trait_item_const(
        &self,
        _: &TraitItemConst,
        _: &Facts<'_, TraitItemConst>,
        inner: Contribution<(), Candidates<'scan>>,
    ) -> Candidates<'scan> {
        self.closed(Context::Const, inner)
    }

    fn off_expr_const(
        &self,
        _: &ExprConst,
        _: &Facts<'_, ExprConst>,
        inner: Contribution<(), Candidates<'scan>>,
    ) -> Candidates<'scan> {
        self.closed(Context::Const, inner)
    }

    fn off_type_array(
        &self,
        array: &TypeArray,
        _: &Facts<'_, TypeArray>,
        inner: Contribution<(), Candidates<'scan>>,
    ) -> Candidates<'scan> {
        self.lengths([&array.len], inner)
    }

    fn off_expr_repeat(
        &self,
        repeat: &ExprRepeat,
        _: &Facts<'_, ExprRepeat>,
        inner: Contribution<(), Candidates<'scan>>,
    ) -> Candidates<'scan> {
        self.lengths([&*repeat.len], inner)
    }

    fn off_variant(
        &self,
        variant: &Variant,
        _: &Facts<'_, Variant>,
        inner: Contribution<(), Candidates<'scan>>,
    ) -> Candidates<'scan> {
        self.lengths(variant.discriminant.iter().map(|(_, discriminant)| discriminant), inner)
    }

    fn on_macro(
        &self,
        _: &Macro,
        _: &Facts<'_, Macro>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), Candidates<'scan>> {
        Contributed::descending(Descend::Expand)
    }

    fn on_expr_cast(
        &self,
        cast: &ExprCast,
        _: &Facts<'_, ExprCast>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), Candidates<'scan>> {
        Contributed::parted(Candidates { casts: vec![cast.as_token.span], ..Candidates::default() })
    }

    fn on_expr_while(
        &self,
        expr: &ExprWhile,
        _: &Facts<'_, ExprWhile>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), Candidates<'scan>> {
        let counted = Some(Looping::from(&*expr.cond)).filter(|looping| *looping == Looping::Counted);
        Contributed::parted(Candidates {
            loops: counted
                .map(|_| Looped {
                    span: expr.while_token.span,
                    message: Message::new(
                        Problem::new("counter loop in const context"),
                        Fix::new("walk a slice with while let [head, rest @ ..]"),
                    ),
                })
                .into_iter()
                .collect(),
            ..Candidates::default()
        })
    }

    fn on_expr_loop(
        &self,
        expr: &ExprLoop,
        _: &Facts<'_, ExprLoop>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), Candidates<'scan>> {
        Self::looped(
            expr.loop_token.span,
            Message::new(
                Problem::new("open loop in const context"),
                Fix::new("walk a slice with while let or recurse"),
            ),
        )
    }

    fn on_expr_for_loop(
        &self,
        expr: &ExprForLoop,
        _: &Facts<'_, ExprForLoop>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), Candidates<'scan>> {
        Self::looped(
            expr.for_token.span,
            Message::new(
                Problem::new("for loop in const context"),
                Fix::new("walk a slice with while let [head, rest @ ..]"),
            ),
        )
    }
}

impl<'scan> Concludes<'scan> for ConstContexts<'scan> {
    fn violations(self, (): (), candidates: Candidates<'scan>) -> Vec<Violation<'scan>> {
        let Candidates { found, casts, surface_casts, .. } = candidates.zoned(self.zone);
        let cast = casts.into_iter().chain(surface_casts).map(|span| self.violation(span, Candidates::CAST));
        let mut violations: Vec<Violation<'scan>> = found.into_iter().chain(cast).collect();
        violations.sort_by_key(|violation| *violation.site());
        violations
    }
}
