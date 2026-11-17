use std::path::Path;
use std::slice;

mod gate;

use syntax::{Attribute, ItemFn, ItemMod, Macro, MetaList};
use syntax::{Ident, TokenTree};

use gate::Gate;
use lint_core::{Macros, Walk};
use lint_core::{Site, SourceFile, Violation};
use lint_sweep::{Descend, Facts, Fold, Kind, Visit};

use crate::NoIgnoredTests;
use lint_layout::Name;
use lint_layout::TestMarks;
use lint_layout::Whole;
use lint_layout::{LineNumber, Word, words};

#[derive(Debug)]
pub(crate) struct SkipMarks<'file> {
    file: &'file SourceFile,
    gates: Vec<LineNumber>,
    unrun: Option<()>,
}

#[derive(Debug, Default)]
pub(crate) struct Marks<'file> {
    file: Option<&'file SourceFile>,
    lines: Vec<LineNumber>,
    gates: Option<Vec<LineNumber>>,
    outer: Vec<Option<Vec<LineNumber>>>,
    unrun: Vec<LineNumber>,
}

#[derive(Debug)]
struct Skipping;

impl<'file> SkipMarks<'file> {
    pub(crate) fn new(file: &'file SourceFile, unrun: Option<()>) -> SkipMarks<'file> {
        SkipMarks { file, gates: Gate::lines_in(&file.syntax().attrs), unrun }
    }

    fn gates(&self, found: &Marks<'file>) -> Vec<LineNumber> {
        found.gates.clone().unwrap_or_else(|| self.gates.clone())
    }

    fn record_skips(&self, found: Marks<'file>, attributes: &[Attribute], marked: Marked) -> Marks<'file> {
        let skipped = attributes
            .iter()
            .filter_map(|attribute| Skipping::skip(attribute).map(|_| LineNumber::from(attribute.pound_token.span)));
        let gated = Gate::lines_in(attributes).into_iter().chain(self.gates(&found)).filter(|_| marked == Marked::Test);
        Marks { lines: found.lines.into_iter().chain(skipped).chain(gated).collect(), ..found }
    }
}

impl Skipping {
    const CONDITIONAL_ATTRIBUTE: &str = "cfg_attr";
    const SKIPPING: [Word<'static>; 2] = words!["ignore", "skip"];

    fn skip_named(list: &MetaList) -> Option<Word<'static>> {
        list.tokens
            .clone()
            .into_iter()
            .filter_map(Skipping::ident)
            .find_map(|ident| Name::from(&ident).matches::<Whole>(&Skipping::SKIPPING))
    }

    fn ident(tree: TokenTree) -> Option<Ident> {
        match tree {
            TokenTree::Ident(ident) => Some(ident),
            _ => None,
        }
    }

    fn skip(attribute: &Attribute) -> Option<Word<'static>> {
        attribute
            .meta
            .require_list()
            .ok()
            .filter(|list| list.path.is_ident(Skipping::CONDITIONAL_ATTRIBUTE))
            .map_or_else(
                || {
                    attribute
                        .meta
                        .path()
                        .segments
                        .last()
                        .and_then(|last| Name::from(&last.ident).matches::<Whole>(&Skipping::SKIPPING))
                },
                Skipping::skip_named,
            )
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

impl<'file> Fold for SkipMarks<'file> {
    type Found = Marks<'file>;

    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemMod, Kind::ItemFn, Kind::Macro, Kind::Attribute]
    }

    fn finished(&self, found: Marks<'file>) -> Marks<'file> {
        Marks { file: Some(self.file), ..found }
    }

    fn on_item_mod(&self, found: Marks<'file>, module: &ItemMod, _: &Facts<'_, ItemMod>) -> Visit<Marks<'file>> {
        let inner: Vec<LineNumber> = self.gates(&found).into_iter().chain(Gate::lines_in(&module.attrs)).collect();
        let outer = found.outer.into_iter().chain([found.gates]).collect();
        Visit::descending(Marks { gates: Some(inner), outer, ..found })
    }

    fn off_item_mod(&self, found: Marks<'file>, _: &ItemMod, _: &Facts<'_, ItemMod>) -> Marks<'file> {
        let mut outer = found.outer;
        let gates = outer.pop().flatten();
        Marks { gates, outer, ..found }
    }

    fn on_item_fn(&self, found: Marks<'file>, function: &ItemFn, _: &Facts<'_, ItemFn>) -> Visit<Marks<'file>> {
        Visit::descending(self.record_skips(found, &function.attrs, Marked::among(&function.attrs, None)))
    }

    fn on_macro(&self, found: Marks<'file>, invocation: &Macro, facts: &Facts<'_, Macro>) -> Visit<Marks<'file>> {
        let attributes = Macros::attributes_in_file(self.file, invocation);
        Visit::new(self.record_skips(found, &attributes, Marked::among(&attributes, facts.law())), Descend::Over)
    }

    fn on_attribute(
        &self,
        found: Marks<'file>,
        attribute: &Attribute,
        _: &Facts<'_, Attribute>,
    ) -> Visit<Marks<'file>> {
        let gated = Some(attribute)
            .filter(|candidate| Walk::outside_tests(slice::from_ref(*candidate)) == Walk::Skipped)
            .and(self.unrun)
            .map(|()| LineNumber::from(attribute.pound_token.span));
        Visit::descending(Marks { unrun: found.unrun.into_iter().chain(gated).collect(), ..found })
    }
}

impl<'file> From<Marks<'file>> for Vec<Violation<'file>> {
    fn from(marks: Marks<'file>) -> Vec<Violation<'file>> {
        marks.file.map_or_default(|file| {
            let path = AsRef::<Path>::as_ref(file.head());
            let unrun =
                marks.unrun.into_iter().map(|line| Violation::new(Site::new(path, line), NoIgnoredTests::UNRUN));
            let target = file
                .head()
                .switched_off()
                .map(|switched| Violation::new(Site::of_file(switched), NoIgnoredTests::UNTESTED));
            let mut lines = marks.lines;
            lines.sort();
            lines.dedup();
            let skipped = lines.into_iter().map(|line| Violation::new(Site::new(path, line), NoIgnoredTests::SKIPPED));
            skipped.chain(target).chain(unrun).collect()
        })
    }
}
