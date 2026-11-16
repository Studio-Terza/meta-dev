use std::mem;

use syntax::{
    Attribute, Block, Expr, ExprClosure, ExprMatch, ImplItemFn, ItemFn, ItemImpl, ItemMod, Span, Spanned, TraitItemFn,
};

use body_shape::{Bounds, Shaper};
use equivalence::Equivalence;
use std::path::Path;

use lint_core::{Bound, Parsed, Site, Walk};
use lint_layout::{LineNumber, TestMarks};
use lint_sweep::{Descend, Facts, Gathers, Kind as Node, Sweep, Watch, Watching};

use crate::Occurrences;
use crate::extent::Extent;
use crate::kind::Kind;
use crate::occurrence::Occurrence;

#[derive(Debug)]
pub struct Scan<'file> {
    path: &'file Path,
    kind: Kind,
    outer: Vec<Kind>,
    shaper: Shaper,
    occurrences: Vec<Occurrence<'file>>,
    tested: Vec<Occurrence<'file>>,
}

impl<'file> Scan<'file> {
    const FLOOR: Bound = Bound::Least(40);

    pub(crate) fn of_file(parsed: Parsed<'_, 'file>, equivalence: Equivalence) -> Vec<Occurrence<'file>> {
        Sweep::gathered(parsed.file(), Self::scanning(parsed, equivalence)).found
    }

    pub(crate) fn scanning(parsed: Parsed<'_, 'file>, equivalence: Equivalence) -> Scan<'file> {
        let tests = parsed.test_modules().into_iter().flat_map(|module| {
            Sweep::gathered(module.file(), Self::new(module, Kind::Tests, Vec::new(), equivalence)).found
        });
        Self::new(parsed, Kind::of_role(parsed.head().placement().role()), tests.collect(), equivalence)
    }

    fn new(
        parsed: Parsed<'_, 'file>,
        kind: Kind,
        tested: Vec<Occurrence<'file>>,
        equivalence: Equivalence,
    ) -> Scan<'file> {
        let shaper = Shaper::reading(parsed.file().tokens(), equivalence);
        Scan { path: parsed.head().path(), kind, outer: Vec::new(), shaper, occurrences: Vec::new(), tested }
    }

    fn considered(&mut self, edges: Edges) -> Option<Bound> {
        let extent = Extent::spanning(edges.first, edges.last);
        let bounds = Bounds::from(extent);
        let large = Self::FLOOR.allows(self.shaper.sized(bounds)).map(|_| Self::FLOOR);
        let site = Site::new(self.path, LineNumber::from(edges.first));
        let occurrence = large.map(|_| Occurrence::new(self.shaper.erasure().read(bounds), site, extent, self.kind));
        self.occurrences.extend(occurrence);
        large
    }

    fn within(&mut self, attributes: &[Attribute]) -> Descend {
        let inner = Some(self.kind)
            .filter(|kind| *kind == Kind::Shipped)
            .filter(|_| Walk::outside_tests(attributes) == Walk::Entered)
            .filter(|_| TestMarks::mark_test(attributes).is_none())
            .unwrap_or(Kind::Tests);
        self.outer.push(mem::replace(&mut self.kind, inner));
        Descend::Into
    }

    fn without(&mut self) {
        self.kind = self.outer.pop().unwrap_or(self.kind);
    }
}

impl<'ast> Watch<'ast> for Scan<'_> {
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

    fn on_item_mod(&mut self, module: &'ast ItemMod, _: &Facts<'_, ItemMod>) -> Descend {
        self.within(&module.attrs)
    }

    fn off_item_mod(&mut self, _: &'ast ItemMod, _: &Facts<'_, ItemMod>) {
        self.without();
    }

    fn on_item_impl(&mut self, item: &'ast ItemImpl, _: &Facts<'_, ItemImpl>) -> Descend {
        self.within(&item.attrs)
    }

    fn off_item_impl(&mut self, _: &'ast ItemImpl, _: &Facts<'_, ItemImpl>) {
        self.without();
    }

    fn on_item_fn(&mut self, function: &'ast ItemFn, _: &Facts<'_, ItemFn>) -> Descend {
        self.within(&function.attrs)
    }

    fn off_item_fn(&mut self, _: &'ast ItemFn, _: &Facts<'_, ItemFn>) {
        self.without();
    }

    fn on_impl_item_fn(&mut self, function: &'ast ImplItemFn, _: &Facts<'_, ImplItemFn>) -> Descend {
        self.within(&function.attrs)
    }

    fn off_impl_item_fn(&mut self, _: &'ast ImplItemFn, _: &Facts<'_, ImplItemFn>) {
        self.without();
    }

    fn on_trait_item_fn(&mut self, function: &'ast TraitItemFn, _: &Facts<'_, TraitItemFn>) -> Descend {
        self.within(&function.attrs)
    }

    fn off_trait_item_fn(&mut self, _: &'ast TraitItemFn, _: &Facts<'_, TraitItemFn>) {
        self.without();
    }

    fn on_block(&mut self, block: &'ast Block, _: &Facts<'_, Block>) -> Descend {
        self.considered(Edges::of_block(block)).map_or(Descend::Over, |_| Descend::Into)
    }

    fn on_expr_match(&mut self, found: &'ast ExprMatch, _: &Facts<'_, ExprMatch>) -> Descend {
        self.considered(Edges::of_match(found)).map_or(Descend::Over, |_| Descend::Into)
    }

    fn on_expr_closure(&mut self, found: &'ast ExprClosure, _: &Facts<'_, ExprClosure>) -> Descend {
        self.considered(Edges::of_closure(found)).map_or(Descend::Over, |_| Descend::Into)
    }
}

impl<'file> Gathers<Occurrences<'file>> for Scan<'file> {
    fn watching(&mut self) -> Vec<&mut Watching<'_>> {
        Sweep::alone(self)
    }

    fn gathered(self) -> Occurrences<'file> {
        Occurrences { path: self.path, found: self.occurrences.into_iter().chain(self.tested).collect() }
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
