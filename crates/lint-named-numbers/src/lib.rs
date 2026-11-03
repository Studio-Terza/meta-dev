use lint_core::{Headline, Lint};
use std::path::PathBuf;

use syn::visit::Visit;
use syn::{ExprConst, ImplItemConst, ItemConst, ItemImpl, ItemMod, LitFloat, LitInt, Macro, TraitItemConst};

use lint_core::Fix;
use lint_core::Gate;
use lint_core::Macros;
use lint_core::Message;
use lint_core::Problem;
use lint_core::Report;
use lint_core::Site;
use lint_core::SourceFile;
use lint_core::SupportModules;
use lint_core::Violation;
use lint_core::Walk;
use lint_layout::{LineNumber, Word, words};

#[derive(Debug)]
pub struct NamedNumbers;

impl Lint for NamedNumbers {
    fn report(files: &[SourceFile]) -> Report<'_> {
        let support: Vec<PathBuf> = files.iter().flat_map(SupportModules::paths).collect();
        Report::new(
            Headline::new(
                "a number has a name: outside tests and boundary modules, a literal other than 0 and 1 sits in a const item",
            ),
            files
                .iter()
                .filter_map(SourceFile::shipped)
                .filter(|file| Gate::BOUNDARY.admits(file).is_none() && !support.contains(&file.path().to_owned()))
                .flat_map(|file| {
                    Self::flagged::<Numbers>(
                        file,
                        Message::new(
                            Problem::new("numeric literal outside a const item"),
                            Fix::new("name it as a const so it can be found and tuned"),
                        ),
                    )
                })
                .collect(),
        )
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

    pub fn flagged<Found: Literals>(file: &SourceFile, message: Message) -> impl Iterator<Item = Violation<'_>> {
        let mut found = Found::default();
        found.visit_file(file.syntax());
        found.lines().into_iter().map(move |line| Violation::new(Site::new(file.path(), line), message))
    }
}

pub trait Literals: for<'ast> Visit<'ast> + Default {
    fn lines(self) -> Vec<LineNumber>;
}

impl Literals for Numbers {
    fn lines(self) -> Vec<LineNumber> {
        self.0
    }
}

#[derive(Default)]
struct Numbers(Vec<LineNumber>);

impl<'ast> Visit<'ast> for Numbers {
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
        Macros::walk(invocation, |statement| self.visit_stmt(statement));
    }

    fn visit_lit_int(&mut self, literal: &'ast LitInt) {
        self.0.extend(NamedNumbers::unnamed(Word::new(literal.base10_digits()), LineNumber::from(literal.span())));
    }

    fn visit_lit_float(&mut self, literal: &'ast LitFloat) {
        self.0.extend(NamedNumbers::unnamed(Word::new(literal.base10_digits()), LineNumber::from(literal.span())));
    }
}
