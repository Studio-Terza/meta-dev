use syntax::{Attribute, Block, Ident, ImplItemFn, ItemFn, ItemMod, TraitItemFn};

use graph_reader::{Branches, Propagation};
use lint_core::{Site, SourceFile, Walk};
use lint_layout::{LineNumber, Name, TestMarks};
use lint_sweep::{Attaches, Attachment, Descend, Facts, Gathers, Kind, Scoped, Sweep, Watch, Watching};

use super::function::Function;

#[derive(Debug)]
pub(crate) struct Functions<'file> {
    file: &'file SourceFile,
    opening: Vec<Attachment<Scoring<'file>>>,
    found: Vec<Scoring<'file>>,
}

#[derive(Debug)]
pub(crate) struct Scoring<'file> {
    name: Name,
    line: LineNumber,
    site: Site<'file>,
    branches: Branches,
}

impl<'file> Gathers<Scoring<'file>> for Scoring<'file> {
    fn watching(&mut self) -> impl IntoIterator<Item = &mut Watching<'_>> {
        std::iter::once::<&mut Watching<'_>>(&mut self.branches)
    }

    fn gathered(self) -> Scoring<'file> {
        self
    }
}

impl<'file> Functions<'file> {
    pub(crate) fn of_file(file: &'file SourceFile) -> Vec<Function<'file>> {
        let scan =
            Sweep::gathered(file, Scanning(Scoped::new(Functions { file, opening: Vec::new(), found: Vec::new() })));
        let mut found = scan.found;
        found.sort_by_key(|scoring| scoring.line);
        found.into_iter().map(|scoring| Function::new(scoring.name, scoring.site, scoring.branches.score())).collect()
    }

    fn recorded(&mut self, attributes: &[Attribute], name: &Ident, body: Option<&Block>) -> Descend {
        let line = LineNumber::from(name.span());
        let site = Site::new(self.file.head().path(), line);
        let opened = body
            .filter(|_| Walk::outside_tests(attributes) == Walk::Entered)
            .filter(|_| TestMarks::mark_test(attributes).is_none())
            .map(|block| {
                Attachment::reaching(
                    block,
                    Scoring { name: Name::from(name), line, site, branches: Branches::new(Propagation::Branch) },
                )
            });
        self.opening.extend(opened);
        Descend::Into
    }
}

#[derive(Debug)]
struct Scanning<'file>(Scoped<Functions<'file>, Scoring<'file>>);

impl<'file> Gathers<Functions<'file>> for Scanning<'file> {
    fn watching(&mut self) -> impl IntoIterator<Item = &mut Watching<'_>> {
        std::iter::once::<&mut Watching<'_>>(&mut self.0)
    }

    fn gathered(self) -> Functions<'file> {
        self.0.into_parent()
    }
}

impl<'file> Attaches<Scoring<'file>> for Functions<'file> {
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

    fn attaching(&mut self) -> Vec<Attachment<Scoring<'file>>> {
        std::mem::take(&mut self.opening)
    }

    fn detached(&mut self, scoring: Scoring<'file>) {
        self.found.push(scoring);
    }
}

impl<'ast> Watch<'ast> for Functions<'_> {
    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemMod, Kind::ItemFn, Kind::ImplItemFn, Kind::TraitItemFn]
    }

    fn on_item_mod(&mut self, _: &'ast ItemMod, facts: &Facts<'_, ItemMod>) -> Descend {
        Descend::from(facts.gate())
    }

    fn on_item_fn(&mut self, function: &'ast ItemFn, _: &Facts<'_, ItemFn>) -> Descend {
        self.recorded(&function.attrs, &function.sig.ident, Some(&function.block))
    }

    fn on_impl_item_fn(&mut self, function: &'ast ImplItemFn, _: &Facts<'_, ImplItemFn>) -> Descend {
        self.recorded(&function.attrs, &function.sig.ident, Some(&function.block))
    }

    fn on_trait_item_fn(&mut self, function: &'ast TraitItemFn, _: &Facts<'_, TraitItemFn>) -> Descend {
        self.recorded(&function.attrs, &function.sig.ident, function.default.as_ref())
    }
}
