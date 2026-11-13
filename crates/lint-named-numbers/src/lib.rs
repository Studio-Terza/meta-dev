use lint_core::{Headline, Parsed};
use lint_file::{ContextLint, Reads};
use lint_sweep::{Gathers, Sweep, Watching};
use std::collections::BTreeSet;
use std::path::PathBuf;

use syn::visit::Visit;
use syn::{ExprConst, ImplItemConst, ItemConst, ItemImpl, ItemMod, LitFloat, LitInt, Macro, TraitItemConst};

use lint_core::Fix;
use lint_core::Gate;
use lint_core::Macros;
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

    fn judged<'file, 'head: 'file>(file: &'file SourceFile, support: &SupportPaths) -> Vec<Violation<'file>> {
        let counted = Some(file)
            .filter(|found| Gate::BOUNDARY.admits(found.head()).is_none())
            .and_then(|found| support.0.get(found.head().path()).map_or(Some(found), |_| None));
        counted.map_or_default(|found: &SourceFile| {
            Self::flagged(
                found,
                Numbers { file: found, lines: Vec::new() },
                Message::new(
                    Problem::new("numeric literal outside a const item"),
                    Fix::new("name it as a const so it can be found and tuned"),
                ),
            )
            .collect()
        })
    }
}

impl NamedNumbers {
    const UNITS: &[Word<'static>] = &words!["0", "1", "0.0", "1.0", "-1", "-1.0"];

    #[must_use]
    pub fn unnamed(digits: Word<'_>, line: LineNumber) -> Option<LineNumber> {
        match Self::UNITS.iter().find(|unit| **unit == digits) {
            Some(_) => None,
            None => Some(line),
        }
    }

    pub fn flagged(
        file: &SourceFile,
        mut found: impl Literals,
        message: Message,
    ) -> impl Iterator<Item = Violation<'_>> {
        found.visit_file(file.syntax());
        found.lines().into_iter().map(move |line| Violation::new(Site::new(file.head().path(), line), message))
    }
}

pub trait Literals: for<'ast> Visit<'ast> {
    fn lines(self) -> Vec<LineNumber>;
}

impl Literals for Numbers<'_> {
    fn lines(self) -> Vec<LineNumber> {
        self.lines
    }
}

struct Numbers<'file> {
    file: &'file SourceFile,
    lines: Vec<LineNumber>,
}

impl<'ast> Visit<'ast> for Numbers<'_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        Gate::BOUNDARY.enter_outside(module, || syn::visit::visit_item_mod(self, module));
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        match Walk::outside_tests(&item.attrs) {
            Walk::Entered => syn::visit::visit_item_impl(self, item),
            Walk::Skipped => {}
        }
    }

    fn visit_item_const(&mut self, _: &'ast ItemConst) {}

    fn visit_impl_item_const(&mut self, _: &'ast ImplItemConst) {}

    fn visit_trait_item_const(&mut self, _: &'ast TraitItemConst) {}

    fn visit_expr_const(&mut self, _: &'ast ExprConst) {}

    fn visit_macro(&mut self, invocation: &'ast Macro) {
        Macros::walk(self.file, invocation, |statement| self.visit_stmt(statement));
    }

    fn visit_lit_int(&mut self, literal: &'ast LitInt) {
        self.lines.extend(NamedNumbers::unnamed(Word::new(literal.base10_digits()), LineNumber::from(literal.span())));
    }

    fn visit_lit_float(&mut self, literal: &'ast LitFloat) {
        self.lines.extend(NamedNumbers::unnamed(Word::new(literal.base10_digits()), LineNumber::from(literal.span())));
    }
}
