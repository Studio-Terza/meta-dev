use lint_core::SourceFile;

use syntax::{
    Attribute, Block, Expr, ExprClosure, ExprMatch, ImplItemFn, ItemFn, ItemImpl, ItemMod, Span, Spanned, TraitItemFn,
};

use body_shape::{Bounds, Shaper};
use equivalence::Equivalence;
use std::path::Path;

use lint_core::{Bound, Parsed, Site, Walk};
use lint_layout::{LineNumber, TestMarks};
use lint_sweep::{Descend, Facts, Fold, Folding, Gathers, Kind as Node, Sweep, Visit, Watching};

use crate::Occurrences;
use crate::extent::Extent;
use crate::kind::Kind;
use crate::occurrence::Occurrence;

#[derive(Debug)]
pub struct Scan<'file> {
    path: &'file Path,
    kind: Kind,
    shaper: Shaper,
}

#[derive(Debug)]
pub struct Scanning<'file> {
    path: &'file Path,
    folding: Folding<Scan<'file>>,
    tested: Vec<Occurrence<'file>>,
}

#[derive(Debug, Default)]
pub struct Scanned<'file> {
    kind: Option<Kind>,
    outer: Vec<Kind>,
    occurrences: Vec<Occurrence<'file>>,
}

impl<'file> Scan<'file> {
    const FLOOR: Bound = Bound::Least(40);

    pub(crate) fn of_file(parsed: Parsed<'_, 'file>, equivalence: Equivalence) -> Vec<Occurrence<'file>> {
        Sweep::gathered(<&SourceFile>::from(parsed), Self::scanning(parsed, equivalence)).found
    }

    pub(crate) fn scanning(parsed: Parsed<'_, 'file>, equivalence: Equivalence) -> Scanning<'file> {
        let tests = parsed.test_modules().into_iter().flat_map(|module| {
            Sweep::gathered(<&SourceFile>::from(module), Self::begun(module, Kind::Tests, Vec::new(), equivalence))
                .found
        });
        Self::begun(parsed, Kind::of_role(parsed.head().placement().role()), tests.collect(), equivalence)
    }

    fn begun(
        parsed: Parsed<'_, 'file>,
        kind: Kind,
        tested: Vec<Occurrence<'file>>,
        equivalence: Equivalence,
    ) -> Scanning<'file> {
        let shaper = Shaper::reading(<&SourceFile>::from(parsed).tokens(), equivalence);
        let path = AsRef::<Path>::as_ref(parsed.head());
        Scanning { path, folding: Folding::from(Scan { path, kind, shaper }), tested }
    }

    fn kind(&self, found: &Scanned<'file>) -> Kind {
        found.kind.unwrap_or(self.kind)
    }

    fn considered(&self, found: Scanned<'file>, edges: Edges) -> Visit<Scanned<'file>> {
        let extent = Extent::spanning(edges.first, edges.last);
        let bounds = Bounds::from(extent);
        let large = Self::FLOOR.allows(self.shaper.sized(bounds)).map(|_| Self::FLOOR);
        let site = Site::new(self.path, LineNumber::from(edges.first));
        let kind = self.kind(&found);
        let occurrence = large.map(|_| Occurrence::new(self.shaper.erasure().read(bounds), site, extent, kind));
        let descend = large.map_or(Descend::Over, |_| Descend::Into);
        Visit::new(Scanned { occurrences: found.occurrences.into_iter().chain(occurrence).collect(), ..found }, descend)
    }

    fn within(&self, found: Scanned<'file>, attributes: &[Attribute]) -> Visit<Scanned<'file>> {
        let current = self.kind(&found);
        let inner = Some(current)
            .filter(|kind| *kind == Kind::Shipped)
            .filter(|_| Walk::outside_tests(attributes) == Walk::Entered)
            .filter(|_| TestMarks::mark_test(attributes).is_none())
            .unwrap_or(Kind::Tests);
        Visit::descending(Scanned {
            kind: Some(inner),
            outer: found.outer.into_iter().chain([current]).collect(),
            ..found
        })
    }
}

impl Scanned<'_> {
    fn without(self) -> Self {
        let mut outer = self.outer;
        let kind = outer.pop().or(self.kind);
        Scanned { kind, outer, ..self }
    }
}

impl<'file> Fold for Scan<'file> {
    type Found = Scanned<'file>;

    fn kinds(&self) -> &'static [Node] {
        &[
            Node::ItemMod,
            Node::ItemImpl,
            Node::ItemFn,
            Node::ImplItemFn,
            Node::TraitItemFn,
            Node::Block,
            Node::ExprMatch,
            Node::ExprClosure,
        ]
    }

    fn on_item_mod(&self, found: Scanned<'file>, module: &ItemMod, _: &Facts<'_, ItemMod>) -> Visit<Scanned<'file>> {
        self.within(found, &module.attrs)
    }

    fn off_item_mod(&self, found: Scanned<'file>, _: &ItemMod, _: &Facts<'_, ItemMod>) -> Scanned<'file> {
        found.without()
    }

    fn on_item_impl(&self, found: Scanned<'file>, item: &ItemImpl, _: &Facts<'_, ItemImpl>) -> Visit<Scanned<'file>> {
        self.within(found, &item.attrs)
    }

    fn off_item_impl(&self, found: Scanned<'file>, _: &ItemImpl, _: &Facts<'_, ItemImpl>) -> Scanned<'file> {
        found.without()
    }

    fn on_item_fn(&self, found: Scanned<'file>, function: &ItemFn, _: &Facts<'_, ItemFn>) -> Visit<Scanned<'file>> {
        self.within(found, &function.attrs)
    }

    fn off_item_fn(&self, found: Scanned<'file>, _: &ItemFn, _: &Facts<'_, ItemFn>) -> Scanned<'file> {
        found.without()
    }

    fn on_impl_item_fn(
        &self,
        found: Scanned<'file>,
        function: &ImplItemFn,
        _: &Facts<'_, ImplItemFn>,
    ) -> Visit<Scanned<'file>> {
        self.within(found, &function.attrs)
    }

    fn off_impl_item_fn(&self, found: Scanned<'file>, _: &ImplItemFn, _: &Facts<'_, ImplItemFn>) -> Scanned<'file> {
        found.without()
    }

    fn on_trait_item_fn(
        &self,
        found: Scanned<'file>,
        function: &TraitItemFn,
        _: &Facts<'_, TraitItemFn>,
    ) -> Visit<Scanned<'file>> {
        self.within(found, &function.attrs)
    }

    fn off_trait_item_fn(&self, found: Scanned<'file>, _: &TraitItemFn, _: &Facts<'_, TraitItemFn>) -> Scanned<'file> {
        found.without()
    }

    fn on_block(&self, found: Scanned<'file>, block: &Block, _: &Facts<'_, Block>) -> Visit<Scanned<'file>> {
        self.considered(found, Edges::of_block(block))
    }

    fn on_expr_match(
        &self,
        found: Scanned<'file>,
        matched: &ExprMatch,
        _: &Facts<'_, ExprMatch>,
    ) -> Visit<Scanned<'file>> {
        self.considered(found, Edges::of_match(matched))
    }

    fn on_expr_closure(
        &self,
        found: Scanned<'file>,
        closure: &ExprClosure,
        _: &Facts<'_, ExprClosure>,
    ) -> Visit<Scanned<'file>> {
        self.considered(found, Edges::of_closure(closure))
    }
}

impl<'file> Gathers<Occurrences<'file>> for Scanning<'file> {
    fn watching(&mut self) -> impl IntoIterator<Item = &mut Watching<'_>> {
        Sweep::alone(&mut self.folding)
    }

    fn gathered(self) -> Occurrences<'file> {
        let scanned: Scanned<'file> = self.folding.concluded();
        Occurrences { path: self.path, found: scanned.occurrences.into_iter().chain(self.tested).collect() }
    }
}

#[derive(Clone, Copy, Debug)]
struct Edges {
    first: Span,
    last: Span,
}

impl Edges {
    fn of_block(block: &Block) -> Edges {
        Edges { first: block.brace_token.span.open(), last: block.brace_token.span.close() }
    }

    fn of_match(found: &ExprMatch) -> Edges {
        let attributed = found.attrs.first().map(|attribute| attribute.pound_token.span);
        Edges { first: attributed.unwrap_or(found.match_token.span), last: found.brace_token.span.close() }
    }

    fn of_closure(found: &ExprClosure) -> Edges {
        let opened = found
            .attrs
            .first()
            .map(|attribute| attribute.pound_token.span)
            .or_else(|| found.lifetimes.as_ref().map(|bound| bound.for_token.span))
            .or_else(|| found.constness.map(|token| token.span))
            .or_else(|| found.movability.map(|token| token.span))
            .or_else(|| found.asyncness.map(|token| token.span))
            .or_else(|| found.capture.map(|token| token.span))
            .unwrap_or(found.or1_token.spans[0]);
        Edges { first: opened, last: Self::ending(&found.body) }
    }

    fn ending(body: &Expr) -> Span {
        match body {
            Expr::Block(block) => block.block.brace_token.span.close(),
            _ => body.span(),
        }
    }
}
