mod boundary;

use boundary::Numeral;
use lint_core::{Headline, Parsed};
use lint_file::{ContextLint, Reads};
use lint_sweep::{Descend, Finds, Judgement, Kind, Watch};
use lint_sweep::{Gathers, Sweep, Watching};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use syn::{ExprConst, ImplItemConst, ItemConst, ItemImpl, ItemMod, LitFloat, LitInt, Macro, TraitItemConst};

use lint_core::Fix;
use lint_core::Gate;
use lint_core::Message;
use lint_core::Problem;
use lint_core::Site;
use lint_core::SourceFile;
use lint_core::SupportModules;
use lint_core::Violation;
use lint_core::Walk;
use lint_layout::{LineNumber, Word, words};

#[derive(Debug)]
pub struct NamedNumbers;

#[derive(Debug, Default)]
pub struct SupportPaths(BTreeSet<PathBuf>);

#[derive(Debug)]
pub struct SupportGathering<'head> {
    file: &'head std::path::Path,
    modules: SupportModules,
}

impl Gathers<SupportPaths> for SupportGathering<'_> {
    fn watching(&mut self) -> Vec<&mut Watching<'_>> {
        Sweep::alone(&mut self.modules)
    }

    fn gathered(self) -> SupportPaths {
        SupportPaths(self.modules.paths(self.file).into_iter().collect())
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
        SupportGathering { file: parsed.head().path(), modules: SupportModules::default() }
    }

    fn judgement<'run, 'file: 'run, 'head: 'file>(
        file: &'file SourceFile,
        support: &'run SupportPaths,
    ) -> Judgement<'file, 'run> {
        let counted = Some(file)
            .filter(|found| Gate::BOUNDARY.admits(found.head()).is_none())
            .and_then(|found| support.0.get(found.head().path()).map_or(Some(found), |_| None));
        let numbers = counted.map(|found| Numbers { path: found.head().path(), lines: Vec::new() });
        numbers.map_or_else(|| Judgement::found(Vec::new()), |watched| Judgement::watching(Box::new(watched)))
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
    lines: Vec<LineNumber>,
}

impl<'ast> Watch<'ast> for Numbers<'_> {
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

    fn on_item_mod(&mut self, module: &'ast ItemMod) -> Descend {
        Gate::BOUNDARY.outside(module)
    }

    fn on_item_impl(&mut self, item: &'ast ItemImpl) -> Descend {
        Descend::from(Walk::outside_tests(&item.attrs))
    }

    fn on_item_const(&mut self, _: &'ast ItemConst) -> Descend {
        Descend::Over
    }

    fn on_impl_item_const(&mut self, _: &'ast ImplItemConst) -> Descend {
        Descend::Over
    }

    fn on_trait_item_const(&mut self, _: &'ast TraitItemConst) -> Descend {
        Descend::Over
    }

    fn on_expr_const(&mut self, _: &'ast ExprConst) -> Descend {
        Descend::Over
    }

    fn on_macro(&mut self, _: &'ast Macro) -> Descend {
        Descend::Expand
    }

    fn on_lit_int(&mut self, literal: &'ast LitInt) -> Descend {
        self.lines.extend(NamedNumbers::unnamed(literal));
        Descend::Into
    }

    fn on_lit_float(&mut self, literal: &'ast LitFloat) -> Descend {
        self.lines.extend(NamedNumbers::unnamed(literal));
        Descend::Into
    }
}

impl<'file> Finds<'file> for Numbers<'file> {
    fn found(self: Box<Self>) -> Vec<Violation<'file>> {
        let path = self.path;
        self.lines.into_iter().map(|line| Violation::new(Site::new(path, line), NamedNumbers::OUTSIDE_CONSTS)).collect()
    }
}
