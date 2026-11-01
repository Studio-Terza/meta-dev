use lint_core::Lint;
use std::path::PathBuf;

use syn::visit::Visit;
use syn::{ExprConst, ImplItemConst, ItemConst, ItemImpl, ItemMod, LitFloat, LitInt, Macro, TraitItemConst};

use lint_core::LineNumber;
use lint_core::Macros;
use lint_core::Report;
use lint_core::Site;
use lint_core::SourceFile;
use lint_core::SupportModules;
use lint_core::Violation;
use lint_core::Walk;

#[derive(Debug)]
pub struct NamedNumbers;

impl Lint for NamedNumbers {
    fn report(files: &[SourceFile]) -> Report {
        let support: Vec<PathBuf> = files.iter().flat_map(SupportModules::paths).collect();
        Report::new(
            "a number has a name: outside tests, a literal other than 0 and 1 sits in a const item",
            files
                .iter()
                .filter(|file| file.is_shipped() && !support.contains(&file.path().to_owned()))
                .flat_map(|file| {
                    let mut numbers = Numbers::default();
                    numbers.visit_file(file.syntax());
                    numbers.0.into_iter().map(move |line| {
                        Violation::new(
                            Site::new(file.path(), line),
                            "numeric literal outside a const item; name it as a const so it can be found and tuned",
                        )
                    })
                })
                .collect(),
        )
    }
}

impl NamedNumbers {
    const UNITS: &[&str] = &["0", "1", "0.0", "1.0"];

    #[must_use]
    pub fn is_unit(digits: &str) -> bool {
        Self::UNITS.contains(&digits)
    }

    fn unnamed(digits: &str, line: LineNumber) -> Option<LineNumber> {
        (!Self::is_unit(digits)).then_some(line)
    }
}

#[derive(Default)]
struct Numbers(Vec<LineNumber>);

impl<'ast> Visit<'ast> for Numbers {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        match Walk::outside_tests(&module.attrs) {
            Walk::Entered => syn::visit::visit_item_mod(self, module),
            Walk::Skipped => {}
        }
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
        self.0.extend(NamedNumbers::unnamed(literal.base10_digits(), LineNumber::from(literal.span())));
    }

    fn visit_lit_float(&mut self, literal: &'ast LitFloat) {
        self.0.extend(NamedNumbers::unnamed(literal.base10_digits(), LineNumber::from(literal.span())));
    }
}
