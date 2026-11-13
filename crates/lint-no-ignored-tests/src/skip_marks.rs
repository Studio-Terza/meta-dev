use std::slice;

mod gate;

use proc_macro2::{Ident, TokenTree};
use syn::{Attribute, ItemFn, ItemMod, Macro, MetaList};

use gate::Gate;
use lint_core::{Macros, Walk};
use lint_core::{Site, SourceFile, Violation};
use lint_sweep::{Descend, Finds, Kind, Watch};

use crate::NoIgnoredTests;
use lint_layout::Name;
use lint_layout::TestMarks;
use lint_layout::Whole;
use lint_layout::{LineNumber, Word, words};

#[derive(Debug)]
pub(crate) struct SkipMarks<'file> {
    file: &'file SourceFile,
    lines: Vec<LineNumber>,
    gates: Vec<LineNumber>,
    outer: Vec<Vec<LineNumber>>,
    unrun: Option<Vec<LineNumber>>,
}

impl<'file> SkipMarks<'file> {
    const CONDITIONAL_ATTRIBUTE: &str = "cfg_attr";
    const SKIPPING: [Word<'static>; 2] = words!["ignore", "skip"];

    pub(crate) fn new(file: &'file SourceFile, unrun: Option<Vec<LineNumber>>) -> SkipMarks<'file> {
        SkipMarks { file, lines: Vec::new(), gates: Gate::lines_in(&file.syntax().attrs), outer: Vec::new(), unrun }
    }

    fn skip_named(list: &MetaList) -> Option<Word<'static>> {
        list.tokens
            .clone()
            .into_iter()
            .filter_map(Self::ident)
            .find_map(|ident| Name::from(&ident).matches::<Whole>(&Self::SKIPPING))
    }

    fn ident(tree: TokenTree) -> Option<Ident> {
        match tree {
            TokenTree::Ident(ident) => Some(ident),
            _ => None,
        }
    }

    fn skip(attribute: &Attribute) -> Option<Word<'static>> {
        attribute.meta.require_list().ok().filter(|list| list.path.is_ident(Self::CONDITIONAL_ATTRIBUTE)).map_or_else(
            || {
                attribute
                    .meta
                    .path()
                    .segments
                    .last()
                    .and_then(|last| Name::from(&last.ident).matches::<Whole>(&Self::SKIPPING))
            },
            Self::skip_named,
        )
    }

    fn record_skips(&mut self, attributes: &[Attribute], marked: Marked) {
        self.lines.extend(
            attributes
                .iter()
                .filter_map(|attribute| Self::skip(attribute).map(|_| LineNumber::from(attribute.pound_token.span))),
        );
        self.lines.extend(
            Gate::lines_in(attributes).into_iter().chain(self.gates.iter().copied()).filter(|_| marked == Marked::Test),
        );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Marked {
    Test,
    Other,
}

impl Marked {
    fn among(attributes: &[Attribute], law: Option<&Macro>) -> Marked {
        law.map(|_| Marked::Test)
            .or_else(|| TestMarks::mark_test(attributes).map(|_| Marked::Test))
            .unwrap_or(Marked::Other)
    }
}

impl<'ast> Watch<'ast> for SkipMarks<'_> {
    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemMod, Kind::ItemFn, Kind::Macro, Kind::Attribute]
    }

    fn on_item_mod(&mut self, module: &'ast ItemMod) -> Descend {
        let inner: Vec<LineNumber> = self.gates.iter().copied().chain(Gate::lines_in(&module.attrs)).collect();
        self.outer.push(std::mem::replace(&mut self.gates, inner));
        Descend::Into
    }

    fn off_item_mod(&mut self, _: &'ast ItemMod) {
        self.gates = self.outer.pop().unwrap_or_default();
    }

    fn on_item_fn(&mut self, function: &'ast ItemFn) -> Descend {
        self.record_skips(&function.attrs, Marked::among(&function.attrs, None));
        Descend::Into
    }

    fn on_macro(&mut self, invocation: &'ast Macro) -> Descend {
        let attributes = Macros::attributes(invocation);
        self.record_skips(&attributes, Marked::among(&attributes, TestMarks::law(invocation)));
        Descend::Over
    }

    fn on_attribute(&mut self, attribute: &'ast Attribute) -> Descend {
        let gated = Some(attribute)
            .filter(|candidate| Walk::outside_tests(slice::from_ref(*candidate)) == Walk::Skipped)
            .map(|skipped| LineNumber::from(skipped.pound_token.span));
        self.unrun.iter_mut().for_each(|unrun| unrun.extend(gated));
        Descend::Into
    }
}

impl<'file> Finds<'file> for SkipMarks<'file> {
    fn found(self: Box<Self>) -> Vec<Violation<'file>> {
        let path = self.file.head().path();
        let unrun = self
            .unrun
            .unwrap_or_default()
            .into_iter()
            .map(|line| Violation::new(Site::new(path, line), NoIgnoredTests::UNRUN));
        let target = self
            .file
            .head()
            .switched_off()
            .map(|switched| Violation::new(Site::of_file(switched), NoIgnoredTests::UNTESTED));
        let mut lines = self.lines;
        lines.sort();
        lines.dedup();
        let skipped = lines.into_iter().map(|line| Violation::new(Site::new(path, line), NoIgnoredTests::SKIPPED));
        skipped.chain(target).chain(unrun).collect()
    }
}
