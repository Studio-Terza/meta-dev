mod boundary;

use boundary::Numeral;
use lint_core::{Headline, Parsed};
use lint_file::{ContextLint, Reads, WatchingLint};
use lint_sweep::Gathers;
use lint_sweep::{Concludes, Contribute, Contributed, Contributing, Descend, Facts, Finds, Kind, Watchers};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use syntax::{ExprConst, ImplItemConst, ItemConst, ItemImpl, ItemMod, LitFloat, LitInt, Macro, TraitItemConst};

use lint_core::Fix;
use lint_core::Gate;
use lint_core::Message;
use lint_core::Problem;
use lint_core::Site;
use lint_core::SourceFile;
use lint_core::SupportScan;
use lint_core::Violation;
use lint_layout::{LineNumber, Word, words};

#[derive(Debug)]
pub struct NamedNumbers;

#[derive(Debug, Default)]
pub struct SupportPaths(BTreeSet<PathBuf>);

#[derive(Debug)]
pub struct SupportGathering<'head> {
    file: &'head std::path::Path,
    modules: Contributing<SupportScan>,
}

impl Gathers<SupportPaths> for SupportGathering<'_> {
    fn gathered(self, source: &SourceFile) -> SupportPaths {
        let file = self.file;
        SupportPaths(self.modules.concluded(source, |_, (), modules| modules).paths(file).into_iter().collect())
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
        SupportGathering { file: AsRef::<Path>::as_ref(parsed.head()), modules: Contributing::new(SupportScan) }
    }
}

impl WatchingLint for NamedNumbers {
    type Found<'file> = Vec<Violation<'file>>;
    type Watched<'file> = Watchers<'file>;

    fn watched(file: &SourceFile) -> Watchers<'_> {
        let counted = Gate::BOUNDARY.admits(file.head(), None).unwrap_or(Some(file));
        let numbers = counted.map(|found| Contributing::new(Numbers { path: AsRef::<Path>::as_ref(found.head()) }));
        numbers.into_iter().map(|watched| -> Box<dyn Finds<'_> + '_> { Box::new(watched) }).collect()
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

type Lines = Contributed<(), Vec<LineNumber>>;

impl Contribute for Numbers<'_> {
    type Open = ();
    type Part = Vec<LineNumber>;

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

    fn on_item_mod(
        &self,
        module: &ItemMod,
        _: &Facts<'_, ItemMod>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Lines {
        Contributed::descending(Gate::BOUNDARY.outside(module))
    }

    fn on_item_impl(
        &self,
        _: &ItemImpl,
        facts: &Facts<'_, ItemImpl>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Lines {
        Contributed::descending(Descend::from(facts.gate()))
    }

    fn on_item_const(
        &self,
        _: &ItemConst,
        _: &Facts<'_, ItemConst>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Lines {
        Contributed::descending(Descend::Over)
    }

    fn on_impl_item_const(
        &self,
        _: &ImplItemConst,
        _: &Facts<'_, ImplItemConst>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Lines {
        Contributed::descending(Descend::Over)
    }

    fn on_trait_item_const(
        &self,
        _: &TraitItemConst,
        _: &Facts<'_, TraitItemConst>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Lines {
        Contributed::descending(Descend::Over)
    }

    fn on_expr_const(
        &self,
        _: &ExprConst,
        _: &Facts<'_, ExprConst>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Lines {
        Contributed::descending(Descend::Over)
    }

    fn on_macro(&self, _: &Macro, _: &Facts<'_, Macro>, _around: &lint_sweep::Around<'_, Self::Open>) -> Lines {
        Contributed::descending(Descend::Expand)
    }

    fn on_lit_int(&self, _: &LitInt, facts: &Facts<'_, LitInt>, _around: &lint_sweep::Around<'_, Self::Open>) -> Lines {
        Contributed::parted(facts.unnamed().into_iter().collect())
    }

    fn on_lit_float(
        &self,
        _: &LitFloat,
        facts: &Facts<'_, LitFloat>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Lines {
        Contributed::parted(facts.unnamed().into_iter().collect())
    }
}

impl<'file> Concludes<'file> for Numbers<'file> {
    fn violations(self, (): (), lines: Vec<LineNumber>) -> Vec<Violation<'file>> {
        lines.into_iter().map(|line| Violation::new(Site::new(self.path, line), NamedNumbers::OUTSIDE_CONSTS)).collect()
    }
}
