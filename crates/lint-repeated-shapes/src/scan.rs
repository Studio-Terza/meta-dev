use lint_core::SourceFile;
use syntax::{
    Attribute, Block, Expr, ExprClosure, ExprMatch, ImplItemFn, ItemFn, ItemImpl, ItemMod, Span, Spanned, TraitItemFn,
};

use algebra::{Monoid, Semigroup};
use body_shape::{Bounds, Reading, Shaper};
use equivalence::Equivalence;
use std::path::Path;

use lint_core::{Bound, Parsed, Site, Walk};
use lint_layout::{LineNumber, TestMarks};
use lint_sweep::{
    Contribute, Contributed, Contributing, Contribution, Descend, Facts, Gathers, Kind as Node, Scope, Sweep,
};

use crate::Occurrences;
use crate::extent::Extent;
use crate::kind::Kind;
use crate::occurrence::Occurrence;

#[derive(Debug)]
pub struct Scan<'file> {
    path: &'file Path,
    kind: Kind,
    shapes: Contributing<ShapeScan<'file>>,
    tested: Vec<Occurrence<'file>>,
}

#[derive(Debug)]
pub struct ShapeScan<'file> {
    path: &'file Path,
    shaper: Shaper,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum Placed {
    #[default]
    Loose,
    Item,
    Test,
}

impl Semigroup for Placed {
    fn combine(self, other: Placed) -> Placed {
        self.max(other)
    }
}

impl Monoid for Placed {}

impl Placed {
    fn marked(attributes: &[Attribute]) -> Placed {
        Descend::from(Walk::outside_tests(attributes))
            .entered()
            .and_then(|_| TestMarks::mark_test(attributes).map_or(Some(Placed::Item), |_| None))
            .unwrap_or(Placed::Test)
    }

    fn kind(self, file: Kind) -> Kind {
        match self {
            Placed::Loose => file,
            Placed::Item => Some(file).filter(|kind| *kind == Kind::Shipped).unwrap_or(Kind::Tests),
            Placed::Test => Kind::Tests,
        }
    }
}

#[derive(Debug)]
pub struct Sighted<'file> {
    reading: Reading,
    site: Site<'file>,
    extent: Extent,
    placed: Placed,
}

impl<'file> Sighted<'file> {
    fn occurrence(self, file: Kind) -> Occurrence<'file> {
        Occurrence::new(self.reading, self.site, self.extent, self.placed.kind(file))
    }
}

impl<'file> Scan<'file> {
    pub(crate) fn of_file(parsed: Parsed<'_, 'file>, equivalence: Equivalence) -> Vec<Occurrence<'file>> {
        Sweep::gathered(<&SourceFile>::from(parsed), Self::scanning(parsed, equivalence)).found
    }

    pub(crate) fn scanning(parsed: Parsed<'_, 'file>, equivalence: Equivalence) -> Scan<'file> {
        let tests = parsed.test_modules().into_iter().flat_map(|module| {
            Sweep::gathered(<&SourceFile>::from(module), Self::new(module, Kind::Tests, Vec::new(), equivalence)).found
        });
        Self::new(parsed, Kind::of_role(parsed.head().placement().role()), tests.collect(), equivalence)
    }

    fn new(
        parsed: Parsed<'_, 'file>,
        kind: Kind,
        tested: Vec<Occurrence<'file>>,
        equivalence: Equivalence,
    ) -> Scan<'file> {
        let shaper = Shaper::reading(<&SourceFile>::from(parsed).tokens(), equivalence);
        let path = AsRef::<Path>::as_ref(parsed.head());
        Scan { path, kind, shapes: Contributing::new(ShapeScan { path, shaper }), tested }
    }
}

impl<'file> ShapeScan<'file> {
    const FLOOR: Bound = Bound::Least(40);

    fn considered(&self, edges: Edges) -> Contributed<(), Vec<Sighted<'file>>> {
        let extent = Extent::spanning(edges.first, edges.last);
        let bounds = Bounds::from(extent);
        let large = Self::FLOOR.allows(self.shaper.sized(bounds));
        let site = Site::new(self.path, LineNumber::from(edges.first));
        large.map_or_else(
            || Contributed::descending(Descend::Over),
            |_| {
                let reading = self.shaper.erasure().read(bounds);
                Contributed::parted(vec![Sighted { reading, site, extent, placed: Placed::Loose }])
            },
        )
    }

    fn closed(attributes: &[Attribute], inner: Contribution<(), Vec<Sighted<'file>>>) -> Vec<Sighted<'file>> {
        let placed = Placed::marked(attributes);
        inner.concluded(|(), sighted| {
            sighted.into_iter().map(|found| Sighted { placed: found.placed.combine(placed), ..found }).collect()
        })
    }
}

impl<'file> Contribute for ShapeScan<'file> {
    type Open = ();
    type Part = Vec<Sighted<'file>>;

    fn kinds(&self) -> &'static [Node] {
        &[Node::Block, Node::ExprMatch, Node::ExprClosure]
    }

    fn scopes(&self) -> &'static [Scope] {
        const {
            &[
                Scope::new(Node::ItemMod),
                Scope::new(Node::ItemImpl),
                Scope::new(Node::ItemFn),
                Scope::new(Node::ImplItemFn),
                Scope::new(Node::TraitItemFn),
            ]
        }
    }

    fn off_item_mod(
        &self,
        module: &ItemMod,
        _: &Facts<'_, ItemMod>,
        inner: Contribution<(), Vec<Sighted<'file>>>,
    ) -> Vec<Sighted<'file>> {
        Self::closed(&module.attrs, inner)
    }

    fn off_item_impl(
        &self,
        item: &ItemImpl,
        _: &Facts<'_, ItemImpl>,
        inner: Contribution<(), Vec<Sighted<'file>>>,
    ) -> Vec<Sighted<'file>> {
        Self::closed(&item.attrs, inner)
    }

    fn off_item_fn(
        &self,
        function: &ItemFn,
        _: &Facts<'_, ItemFn>,
        inner: Contribution<(), Vec<Sighted<'file>>>,
    ) -> Vec<Sighted<'file>> {
        Self::closed(&function.attrs, inner)
    }

    fn off_impl_item_fn(
        &self,
        function: &ImplItemFn,
        _: &Facts<'_, ImplItemFn>,
        inner: Contribution<(), Vec<Sighted<'file>>>,
    ) -> Vec<Sighted<'file>> {
        Self::closed(&function.attrs, inner)
    }

    fn off_trait_item_fn(
        &self,
        function: &TraitItemFn,
        _: &Facts<'_, TraitItemFn>,
        inner: Contribution<(), Vec<Sighted<'file>>>,
    ) -> Vec<Sighted<'file>> {
        Self::closed(&function.attrs, inner)
    }

    fn on_block(
        &self,
        block: &Block,
        _: &Facts<'_, Block>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), Vec<Sighted<'file>>> {
        self.considered(Edges::of_block(block))
    }

    fn on_expr_match(
        &self,
        found: &ExprMatch,
        _: &Facts<'_, ExprMatch>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), Vec<Sighted<'file>>> {
        self.considered(Edges::of_match(found))
    }

    fn on_expr_closure(
        &self,
        found: &ExprClosure,
        _: &Facts<'_, ExprClosure>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), Vec<Sighted<'file>>> {
        self.considered(Edges::of_closure(found))
    }
}

impl<'file> Gathers<Occurrences<'file>> for Scan<'file> {
    fn gathered(self, file: &SourceFile) -> Occurrences<'file> {
        let kind = self.kind;
        let sighted = self.shapes.concluded(file, |_, (), sighted| sighted);
        let found = sighted.into_iter().map(|found| found.occurrence(kind)).chain(self.tested).collect();
        Occurrences { path: self.path, found }
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
