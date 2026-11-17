mod boundary;

use boundary::Numeral;
use lint_core::{Headline, Parsed};
use lint_file::{ContextLint, Reads, WatchingLint};
use lint_sweep::{Descend, Facts, Fold, Folding, Gathers, Kind, Sweep, Visit, Watchers, Watching};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use syntax::{ExprConst, ImplItemConst, ItemConst, ItemImpl, ItemMod, LitFloat, LitInt, Macro, TraitItemConst};

use lint_core::Fix;
use lint_core::Gate;
use lint_core::Message;
use lint_core::Problem;
use lint_core::Site;
use lint_core::SourceFile;
use lint_core::Violation;
use lint_core::{SupportModules, SupportScan};
use lint_layout::{LineNumber, Word, words};

#[derive(Debug)]
pub struct NamedNumbers;

#[derive(Debug, Default)]
pub struct SupportPaths(BTreeSet<PathBuf>);

#[derive(Debug)]
pub struct SupportGathering<'head> {
    file: &'head std::path::Path,
    modules: Folding<SupportScan>,
}

impl Gathers<SupportPaths> for SupportGathering<'_> {
    fn watching(&mut self) -> impl IntoIterator<Item = &mut Watching<'_>> {
        Sweep::alone(&mut self.modules)
    }

    fn gathered(self) -> SupportPaths {
        SupportPaths(self.modules.concluded::<SupportModules>().paths(self.file).into_iter().collect())
    }
}

impl FromIterator<SupportPaths> for SupportPaths {
    fn from_iter<Parts: IntoIterator<Item = SupportPaths>>(parts: Parts) -> SupportPaths {
        SupportPaths(parts.into_iter().flat_map(|part| part.0).collect())
    }
}

impl ContextLint for NamedNumbers {
    const HEADLINE: Headline = Headline::new(
        "a number has a name: outside tests and boundary modules, a literal other than 0 and 1 sits in a const item",
    );
    const READS: Reads = Reads::Shipped;

    type Part<'head> = SupportPaths;
    type Context<'head> = SupportPaths;

    type Gathering<'head> = SupportGathering<'head>;

    fn gathering<'head>(parsed: Parsed<'_, 'head>) -> SupportGathering<'head> {
        SupportGathering { file: AsRef::<Path>::as_ref(parsed.head()), modules: Folding::from(SupportScan) }
    }
}

impl WatchingLint for NamedNumbers {
    type Found<'file> = Vec<Violation<'file>>;
    type Watched<'file> = Watchers<'file>;

    fn watched(file: &SourceFile) -> Watchers<'_> {
        let counted = Some(file).filter(|found| Gate::BOUNDARY.admits(found.head()).is_none());
        let numbers = counted.map(|found| Numbers { path: AsRef::<Path>::as_ref(found.head()) });
        numbers.into_iter().fold(Watchers::none(), |watchers, watched| watchers.and(Box::new(Folding::from(watched))))
    }

    fn judged<'file, 'head: 'file>(
        file: &'file SourceFile,
        found: Vec<Violation<'file>>,
        support: &SupportPaths,
    ) -> Vec<Violation<'file>> {
        support.0.get(AsRef::<Path>::as_ref(file.head())).map_or(found, |_| Vec::new())
    }
}

impl NamedNumbers {
    const UNITS: &[Word<'static>] = &words!["0", "1", "0.0", "1.0", "-1", "-1.0"];

    #[must_use]
    pub fn unnamed(literal: &impl Numeral) -> Option<LineNumber> {
        let digits = Word::new(literal.digits());
        Self::UNITS.iter().find(|unit| **unit == digits).map_or(Some(LineNumber::from(literal.spot())), |_| None)
    }

    const OUTSIDE_CONSTS: Message = Message::new(
        Problem::new("numeric literal outside a const item"),
        Fix::new("name it as a const so it can be found and tuned"),
    );
}

#[derive(Debug)]
struct Numbers<'file> {
    path: &'file Path,
}

impl<'file> Numbers<'file> {
    fn numbered(&self, found: Vec<Violation<'file>>, unnamed: Option<LineNumber>) -> Visit<Vec<Violation<'file>>> {
        Visit::adding(
            found,
            unnamed.map(|line| Violation::new(Site::new(self.path, line), NamedNumbers::OUTSIDE_CONSTS)),
            Descend::Into,
        )
    }
}

impl<'file> Fold for Numbers<'file> {
    type Found = Vec<Violation<'file>>;

    fn kinds(&self) -> &'static [Kind] {
        &[
            Kind::ItemMod,
            Kind::ItemImpl,
            Kind::ItemConst,
            Kind::ImplItemConst,
            Kind::TraitItemConst,
            Kind::ExprConst,
            Kind::Macro,
            Kind::LitInt,
            Kind::LitFloat,
        ]
    }

    fn on_item_mod(&self, found: Self::Found, module: &ItemMod, _: &Facts<'_, ItemMod>) -> Visit<Self::Found> {
        Visit::new(found, Gate::BOUNDARY.outside(module))
    }

    fn on_item_impl(&self, found: Self::Found, _: &ItemImpl, facts: &Facts<'_, ItemImpl>) -> Visit<Self::Found> {
        Visit::new(found, Descend::from(facts.gate()))
    }

    fn on_item_const(&self, found: Self::Found, _: &ItemConst, _: &Facts<'_, ItemConst>) -> Visit<Self::Found> {
        Visit::new(found, Descend::Over)
    }

    fn on_impl_item_const(
        &self,
        found: Self::Found,
        _: &ImplItemConst,
        _: &Facts<'_, ImplItemConst>,
    ) -> Visit<Self::Found> {
        Visit::new(found, Descend::Over)
    }

    fn on_trait_item_const(
        &self,
        found: Self::Found,
        _: &TraitItemConst,
        _: &Facts<'_, TraitItemConst>,
    ) -> Visit<Self::Found> {
        Visit::new(found, Descend::Over)
    }

    fn on_expr_const(&self, found: Self::Found, _: &ExprConst, _: &Facts<'_, ExprConst>) -> Visit<Self::Found> {
        Visit::new(found, Descend::Over)
    }

    fn on_macro(&self, found: Self::Found, _: &Macro, _: &Facts<'_, Macro>) -> Visit<Self::Found> {
        Visit::new(found, Descend::Expand)
    }

    fn on_lit_int(&self, found: Self::Found, _: &LitInt, facts: &Facts<'_, LitInt>) -> Visit<Self::Found> {
        self.numbered(found, facts.unnamed())
    }

    fn on_lit_float(&self, found: Self::Found, _: &LitFloat, facts: &Facts<'_, LitFloat>) -> Visit<Self::Found> {
        self.numbered(found, facts.unnamed())
    }
}
