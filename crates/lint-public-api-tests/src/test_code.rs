use syntax::{Attribute, ImplItemFn, ItemFn, ItemImpl, ItemMod, Macro};

use std::collections::BTreeSet;
use std::path::Path;

use super::PublicApiTests;
use lint_core::Joins;
use lint_core::Macros;
use lint_core::Walk;
use lint_core::{Site, SourceFile, Violation};
use lint_layout::LineNumber;
use lint_layout::Name;
use lint_layout::TestMarks;
use lint_sweep::{Contribute, Contributed, Descend, Facts, Kind};

#[derive(Debug)]
pub struct TestCode<'file> {
    file: &'file SourceFile,
}

#[derive(Debug, Default)]
pub struct InSource {
    lines: Vec<LineNumber>,
    invoked: Vec<Invoked>,
}

impl Joins for InSource {
    fn combine(self, other: InSource) -> InSource {
        InSource { lines: self.lines.combine(other.lines), invoked: self.invoked.combine(other.invoked) }
    }
}

impl InSource {
    fn lines(lines: impl IntoIterator<Item = LineNumber>) -> InSource {
        InSource { lines: lines.into_iter().collect(), invoked: Vec::new() }
    }
}

#[derive(Debug)]
struct Invoked {
    name: Name,
    line: LineNumber,
}

#[derive(Debug)]
pub struct Tested<'file> {
    path: &'file Path,
    found: Vec<Violation<'file>>,
    invoked: Vec<Invoked>,
}

impl<'file> Tested<'file> {
    pub(crate) fn reached(found: Vec<Violation<'file>>, path: &'file Path) -> Tested<'file> {
        Tested { path, found, invoked: Vec::new() }
    }

    pub(crate) fn emitted(self, emitters: &BTreeSet<Name>) -> Vec<Violation<'file>> {
        let path = self.path;
        let emitting = self.invoked.into_iter().filter(|invoked| emitters.contains(&invoked.name));
        let lines = emitting.map(|invoked| Violation::new(Site::new(path, invoked.line), PublicApiTests::IN_SOURCE));
        self.found.into_iter().chain(lines).collect()
    }
}

impl<'file> TestCode<'file> {
    const DEFINITION: &'static str = "macro_rules";

    pub(crate) fn tested(self, in_source: InSource) -> Tested<'file> {
        let path = AsRef::<Path>::as_ref(self.file.head());
        let found =
            in_source.lines.into_iter().map(|line| Violation::new(Site::new(path, line), PublicApiTests::IN_SOURCE));
        Tested { path, found: found.collect(), invoked: in_source.invoked }
    }

    fn gated(attributes: &[Attribute], line: LineNumber) -> Contributed<(), InSource> {
        let walk = Walk::outside_tests(attributes);
        Contributed::new((), InSource::lines((walk == Walk::Skipped).then_some(line)), Descend::from(walk))
    }
}

impl Contribute for TestCode<'_> {
    type Open = ();
    type Part = InSource;

    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemMod, Kind::ItemImpl, Kind::ImplItemFn, Kind::ItemFn, Kind::Macro]
    }

    fn on_item_mod(
        &self,
        module: &ItemMod,
        facts: &Facts<'_, ItemMod>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), InSource> {
        let walk = facts.gate();
        let lines = InSource::lines((walk == Walk::Skipped).then(|| LineNumber::from(module.ident.span())));
        Contributed::new((), lines, Descend::from(walk))
    }

    fn on_item_impl(
        &self,
        item: &ItemImpl,
        _: &Facts<'_, ItemImpl>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), InSource> {
        Self::gated(&item.attrs, LineNumber::from(item.impl_token.span))
    }

    fn on_impl_item_fn(
        &self,
        method: &ImplItemFn,
        _: &Facts<'_, ImplItemFn>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), InSource> {
        Self::gated(&method.attrs, LineNumber::from(method.sig.ident.span()))
    }

    fn on_item_fn(
        &self,
        function: &ItemFn,
        facts: &Facts<'_, ItemFn>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), InSource> {
        let line = LineNumber::from(function.sig.ident.span());
        let walk = Walk::outside_tests(&function.attrs);
        let skipped = (walk == Walk::Skipped).then_some(line);
        let marked = facts.test_mark().filter(|_| walk == Walk::Entered).map(|_| line);
        Contributed::new((), InSource::lines(skipped.into_iter().chain(marked)), Descend::from(walk))
    }

    fn on_macro(
        &self,
        invocation: &Macro,
        _: &Facts<'_, Macro>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), InSource> {
        let line = LineNumber::from(invocation.bang_token.span);
        let called =
            Some(invocation).filter(|candidate| candidate.path.get_ident().is_none_or(|name| name != Self::DEFINITION));
        let settled = called.and_then(|call| {
            Macros::attributes_in_file(self.file, call).iter().find_map(TestMarks::test).map(|_| line)
        });
        let named = invocation.path.get_ident().map(|name| Invoked { name: Name::from(name), line });
        let invoked = settled.map_or(named, |_| None).into_iter().collect();
        Contributed::new((), InSource { lines: settled.into_iter().collect(), invoked }, Descend::Over)
    }
}

impl<'file> From<&'file SourceFile> for TestCode<'file> {
    fn from(file: &'file SourceFile) -> TestCode<'file> {
        TestCode { file }
    }
}
