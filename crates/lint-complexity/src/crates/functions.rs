use lint_sweep::{Drawn, Fold, Folding, Visit};
use std::path::Path;
use syntax::{Attribute, Block, Ident, ImplItemFn, ItemFn, ItemMod, TraitItemFn};

use body_lifts::Branching;
use graph_reader::{Branches, Propagation};
use lint_core::Extended;
use lint_core::{Site, SourceFile, Walk};
use lint_layout::{LineNumber, Name, TestMarks};
use lint_sweep::{Attaching, Attachment, Descend, Facts, Gathers, Grouped, Kind, Scoped, Sweep, Watching};

use super::function::Function;

#[derive(Debug)]
pub(crate) struct Functions<'file> {
    file: &'file SourceFile,
}

#[derive(Debug)]
struct Scoring<'file> {
    name: Name,
    line: LineNumber,
    site: Site<'file>,
}

#[derive(Debug)]
struct Scored<'file> {
    scoring: Scoring<'file>,
    branching: Branching,
}

#[derive(Debug, Default)]
pub(crate) struct Listed<'file> {
    opening: Vec<Attachment<Branches>>,
    open: Vec<Scoring<'file>>,
    found: Vec<Scored<'file>>,
}

impl<'file> Functions<'file> {
    pub(crate) fn of_file(file: &'file SourceFile) -> Vec<Function<'file>> {
        let listed = Sweep::gathered(file, Scanning(Scoped::new(Folding::from(Functions { file }))));
        let mut found = listed.found;
        found.sort_by_key(|scored| scored.scoring.line);
        found
            .into_iter()
            .map(|scored| Function::new(scored.scoring.name, scored.scoring.site, scored.branching))
            .collect()
    }

    fn recorded(
        &self,
        found: Listed<'file>,
        attributes: &[Attribute],
        name: &Ident,
        body: Option<&Block>,
    ) -> Visit<Listed<'file>> {
        let line = LineNumber::from(name.span());
        let site = Site::new(AsRef::<Path>::as_ref(self.file.head()), line);
        let opened = body
            .filter(|_| Walk::outside_tests(attributes) == Walk::Entered)
            .filter(|_| TestMarks::mark_test(attributes).is_none());
        let attached = opened.map(|block| Attachment::reaching(block, Branches::new(Propagation::Branch)));
        let scoring = opened.map(|_| Scoring { name: Name::from(name), line, site });
        Visit::descending(Listed {
            opening: found.opening.extended(attached),
            open: found.open.extended(scoring),
            ..found
        })
    }
}

#[derive(Debug)]
struct Scanning<'file>(Scoped<Folding<Functions<'file>>, Grouped<Branches>>);

impl<'file> Gathers<Listed<'file>> for Scanning<'file> {
    fn watching(&mut self) -> impl IntoIterator<Item = &mut Watching<'_>> {
        std::iter::once::<&mut Watching<'_>>(&mut self.0)
    }

    fn gathered(self) -> Listed<'file> {
        self.0.into_parent().concluded()
    }
}

impl<'file> Attaching<Branches> for Functions<'file> {
    const ATTACHED: &'static [Kind] = &[
        Kind::Block,
        Kind::Item,
        Kind::ExprIf,
        Kind::ExprWhile,
        Kind::ExprForLoop,
        Kind::ExprTry,
        Kind::ExprMatch,
        Kind::ExprBinary,
    ];

    fn attaching(&self, found: Listed<'file>) -> Drawn<Listed<'file>, Vec<Attachment<Branches>>> {
        Drawn::new(Listed { opening: Vec::new(), ..found }, found.opening)
    }

    fn detached(&self, found: Listed<'file>, branched: graph_reader::Branched) -> Listed<'file> {
        let mut open = found.open;
        let scored = open.pop().map(|scoring| Scored { scoring, branching: Branching::from(branched) });
        Listed { open, found: found.found.extended(scored), ..found }
    }
}

impl<'file> Fold for Functions<'file> {
    type Found = Listed<'file>;

    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemMod, Kind::ItemFn, Kind::ImplItemFn, Kind::TraitItemFn]
    }

    fn on_item_mod(&self, found: Listed<'file>, _: &ItemMod, facts: &Facts<'_, ItemMod>) -> Visit<Listed<'file>> {
        Visit::new(found, Descend::from(facts.gate()))
    }

    fn on_item_fn(&self, found: Listed<'file>, function: &ItemFn, _: &Facts<'_, ItemFn>) -> Visit<Listed<'file>> {
        self.recorded(found, &function.attrs, &function.sig.ident, Some(&function.block))
    }

    fn on_impl_item_fn(
        &self,
        found: Listed<'file>,
        function: &ImplItemFn,
        _: &Facts<'_, ImplItemFn>,
    ) -> Visit<Listed<'file>> {
        self.recorded(found, &function.attrs, &function.sig.ident, Some(&function.block))
    }

    fn on_trait_item_fn(
        &self,
        found: Listed<'file>,
        function: &TraitItemFn,
        _: &Facts<'_, TraitItemFn>,
    ) -> Visit<Listed<'file>> {
        self.recorded(found, &function.attrs, &function.sig.ident, function.default.as_ref())
    }
}
