use std::path::Path;
use std::slice;

mod gate;
mod ignored;

use syntax::{Attribute, ItemFn, ItemMod, Macro, MetaList};
use syntax::{Ident, TokenTree};

use gate::Gate;
use lint_core::Joins;
use lint_core::{Macros, Walk};
use lint_core::{Site, SourceFile, Violation};
use lint_sweep::{Concludes, Contribute, Contributed, Contribution, Descend, Facts, Kind, Scope};

use crate::NoIgnoredTests;
use crate::skip_marks::ignored::Ignored;
use invariants::Ignoring;
use lint_layout::Name;
use lint_layout::TestMarks;
use lint_layout::Whole;
use lint_layout::{Fitted, LineNumber, Word, words};

#[derive(Debug)]
pub(crate) struct SkipMarks<'file> {
    file: &'file SourceFile,
    gates: Vec<LineNumber>,
    unrun: Option<()>,
}

#[derive(Debug, Default)]
pub(crate) struct Marks {
    lines: Vec<LineNumber>,
    unrun: Vec<LineNumber>,
    tested: Option<()>,
}

impl Joins for Marks {
    fn combine(self, other: Marks) -> Marks {
        Marks {
            lines: self.lines.combine(other.lines),
            unrun: self.unrun.combine(other.unrun),
            tested: self.tested.or(other.tested),
        }
    }
}

impl Marks {
    fn gated(self, gates: Vec<LineNumber>) -> Marks {
        let tested = self.tested.map_or_else(Vec::new, |()| gates);
        Marks { lines: self.lines.combine(tested), ..self }
    }
}

impl<'file> SkipMarks<'file> {
    const CONDITIONAL_ATTRIBUTE: &str = "cfg_attr";
    const SKIPPING: [Word<'static>; 2] = words!["ignore", "skip"];

    pub(crate) fn new(file: &'file SourceFile, unrun: Option<()>) -> SkipMarks<'file> {
        SkipMarks { file, gates: Gate::lines_in(&file.syntax().attrs), unrun }
    }

    fn skip_named(list: &MetaList) -> Option<Fitted<'static>> {
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

    fn skip(attribute: &Attribute) -> Option<Fitted<'static>> {
        attribute
            .meta
            .require_list()
            .ignoring(Ignored::Unparsed)
            .filter(|list| list.path.is_ident(Self::CONDITIONAL_ATTRIBUTE))
            .map_or_else(
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

    fn skips(attributes: &[Attribute], marked: Marked) -> Marks {
        let skipped = attributes
            .iter()
            .filter_map(|attribute| Self::skip(attribute).map(|_| LineNumber::from(attribute.pound_token.span)));
        let gated = Gate::lines_in(attributes).into_iter().filter(|_| marked.test().is_some());
        let tested = marked.test();
        Marks { lines: skipped.chain(gated).collect(), unrun: Vec::new(), tested }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Marked {
    Test,
    Other,
}

impl Marked {
    fn test(self) -> Option<()> {
        match self {
            Marked::Test => Some(()),
            Marked::Other => None,
        }
    }

    fn among(attributes: &[Attribute]) -> Marked {
        TestMarks::mark_test(attributes).map_or(Marked::Other, |_| Marked::Test)
    }
}

impl Contribute for SkipMarks<'_> {
    type Open = Vec<LineNumber>;
    type Part = Marks;

    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemMod, Kind::ItemFn, Kind::Macro, Kind::Attribute]
    }

    fn scopes(&self) -> &'static [Scope] {
        const { &[Scope::new(Kind::ItemMod)] }
    }

    fn on_item_mod(
        &self,
        module: &ItemMod,
        _: &Facts<'_, ItemMod>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<Vec<LineNumber>, Marks> {
        Contributed::new(Gate::lines_in(&module.attrs), Marks::default(), Descend::Into)
    }

    fn off_item_mod(&self, _: &ItemMod, _: &Facts<'_, ItemMod>, inner: Contribution<Vec<LineNumber>, Marks>) -> Marks {
        inner.concluded(|gates, marks| marks.gated(gates))
    }

    fn on_item_fn(
        &self,
        function: &ItemFn,
        _: &Facts<'_, ItemFn>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<Vec<LineNumber>, Marks> {
        Contributed::parted(Self::skips(&function.attrs, Marked::among(&function.attrs)))
    }

    fn on_macro(
        &self,
        invocation: &Macro,
        _: &Facts<'_, Macro>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<Vec<LineNumber>, Marks> {
        let attributes = Macros::attributes_in_file(self.file, invocation);
        let marks = Self::skips(&attributes, Marked::among(&attributes));
        Contributed::new(Vec::new(), marks, Descend::Over)
    }

    fn on_attribute(
        &self,
        attribute: &Attribute,
        _: &Facts<'_, Attribute>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<Vec<LineNumber>, Marks> {
        let unrun = self
            .unrun
            .and(Some(attribute))
            .filter(|candidate| Walk::outside_tests(slice::from_ref(*candidate)) == Walk::Skipped)
            .map(|skipped| LineNumber::from(skipped.pound_token.span));
        Contributed::parted(Marks { unrun: unrun.into_iter().collect(), ..Marks::default() })
    }
}

impl<'file> Concludes<'file> for SkipMarks<'file> {
    fn violations(self, _: Vec<LineNumber>, marks: Marks) -> Vec<Violation<'file>> {
        let path = AsRef::<Path>::as_ref(self.file.head());
        let Marks { mut lines, unrun: never_run, .. } = marks.gated(self.gates);
        let unrun = never_run.into_iter().map(|line| Violation::new(Site::new(path, line), NoIgnoredTests::UNRUN));
        let target = self
            .file
            .head()
            .switched_off()
            .map(|switched| Violation::new(Site::of_file(switched), NoIgnoredTests::UNTESTED));
        lines.sort();
        lines.dedup();
        let skipped = lines.into_iter().map(|line| Violation::new(Site::new(path, line), NoIgnoredTests::SKIPPED));
        skipped.chain(target).chain(unrun).collect()
    }
}
