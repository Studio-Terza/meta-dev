use lint_core::{Headline, Lint};
use syn::visit::Visit;
use syn::{ItemMod, LitFloat, LitInt};

use lint_core::BinaryFiles;
use lint_core::Fix;
use lint_core::Message;
use lint_core::Problem;
use lint_core::Report;
use lint_core::SourceFile;
use lint_core::Walk;
use lint_layout::{LineNumber, Word};
use lint_named_numbers::{Literals, NamedNumbers};

#[derive(Debug)]
pub struct NoNumbersInBinaries;

impl Lint for NoNumbersInBinaries {
    fn report(files: &[SourceFile]) -> Report<'_> {
        let binaries = BinaryFiles::collect(files);
        Report::new(
            Headline::new("a binary is wiring: a number it needs is a setting in its config file"),
            files
                .iter()
                .filter(|file| binaries.holding(file.path()).is_some())
                .flat_map(|file| {
                    NamedNumbers::flagged(
                        file,
                        Numbers::default(),
                        Message::new(
                            Problem::new("numeric literal in a binary"),
                            Fix::new("read it from the settings file or its embedded defaults"),
                        ),
                    )
                })
                .collect(),
        )
    }
}

#[derive(Default)]
struct Numbers(Vec<LineNumber>);

impl<'ast> Visit<'ast> for Numbers {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        Walk::enter(&module.attrs, || syn::visit::visit_item_mod(self, module));
    }

    fn visit_lit_int(&mut self, literal: &'ast LitInt) {
        self.0.extend(NamedNumbers::unnamed(Word::new(literal.base10_digits()), LineNumber::from(literal.span())));
    }

    fn visit_lit_float(&mut self, literal: &'ast LitFloat) {
        self.0.extend(NamedNumbers::unnamed(Word::new(literal.base10_digits()), LineNumber::from(literal.span())));
    }
}

impl Literals for Numbers {
    fn lines(self) -> Vec<LineNumber> {
        self.0
    }
}
