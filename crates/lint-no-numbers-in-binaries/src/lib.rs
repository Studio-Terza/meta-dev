use std::path::Path;

use lint_core::{Headline, Violation};
use lint_file::{Reads, SweptLint};
use lint_sweep::{Descend, Finds, Kind, Watch};
use syn::{ItemMod, LitFloat, LitInt};

use lint_core::Fix;
use lint_core::Message;
use lint_core::Problem;
use lint_core::Site;
use lint_core::SourceFile;
use lint_core::Walk;
use lint_layout::LineNumber;
use lint_named_numbers::NamedNumbers;

#[derive(Debug)]
pub struct NoNumbersInBinaries;

impl NoNumbersInBinaries {
    const IN_BINARY: Message = Message::new(
        Problem::new("numeric literal in a binary"),
        Fix::new("read it from the settings file or its embedded defaults"),
    );
}

impl SweptLint for NoNumbersInBinaries {
    const HEADLINE: Headline = Headline::new("a binary is wiring: a number it needs is a setting in its config file");
    const READS: Reads = Reads::Every;

    fn watcher(file: &SourceFile) -> Box<dyn Finds<'_> + '_> {
        let kinds = file.head().binary().map_or(&[][..], |_| &[Kind::ItemMod, Kind::LitInt, Kind::LitFloat][..]);
        Box::new(Numbers { path: file.head().path(), kinds, lines: Vec::new() })
    }
}

#[derive(Debug)]
struct Numbers<'file> {
    path: &'file Path,
    kinds: &'static [Kind],
    lines: Vec<LineNumber>,
}

impl<'ast> Watch<'ast> for Numbers<'_> {
    fn kinds(&self) -> &'static [Kind] {
        self.kinds
    }

    fn on_item_mod(&mut self, module: &'ast ItemMod) -> Descend {
        Descend::from(Walk::outside_tests(&module.attrs))
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
        self.lines
            .into_iter()
            .map(|line| Violation::new(Site::new(path, line), NoNumbersInBinaries::IN_BINARY))
            .collect()
    }
}
