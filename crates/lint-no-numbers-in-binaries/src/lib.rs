use lint_core::{Headline, Lint};
use syn::visit::Visit;
use syn::{ItemMod, LitFloat, LitInt};

use lint_core::BinaryFiles;
use lint_core::Report;
use lint_core::Site;
use lint_core::SourceFile;
use lint_core::Violation;
use lint_core::Walk;
use lint_layout::{LineNumber, Word};
use lint_named_numbers::NamedNumbers;

#[derive(Debug)]
pub struct NoNumbersInBinaries;

impl Lint for NoNumbersInBinaries {
    fn report(files: &[SourceFile]) -> Report {
        let binaries = BinaryFiles::collect(files);
        Report::new(
            Headline::new("a binary is wiring: a number it needs is a setting in its config file"),
            files
                .iter()
                .filter(|file| binaries.contains(file.path()))
                .flat_map(|file| {
                    let mut numbers = Numbers::default();
                    numbers.visit_file(file.syntax());
                    numbers.0.into_iter().map(move |line| {
                        Violation::new(
                            Site::new(file.path(), line),
                            "numeric literal in a binary; read it from the settings file or its embedded defaults",
                        )
                    })
                })
                .collect(),
        )
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

    fn visit_lit_int(&mut self, literal: &'ast LitInt) {
        self.0.extend(
            (!NamedNumbers::is_unit(Word::new(literal.base10_digits()))).then(|| LineNumber::from(literal.span())),
        );
    }

    fn visit_lit_float(&mut self, literal: &'ast LitFloat) {
        self.0.extend(
            (!NamedNumbers::is_unit(Word::new(literal.base10_digits()))).then(|| LineNumber::from(literal.span())),
        );
    }
}
