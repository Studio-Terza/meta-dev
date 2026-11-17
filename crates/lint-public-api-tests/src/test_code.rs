use syntax::{Attribute, ImplItemFn, ItemFn, ItemImpl, ItemMod, Macro};

use std::collections::BTreeSet;
use std::path::Path;

use super::PublicApiTests;
use lint_core::Macros;
use lint_core::Walk;
use lint_core::{Site, SourceFile, Violation};
use lint_layout::LineNumber;
use lint_layout::Name;
use lint_layout::TestMarks;
use lint_sweep::{Descend, Facts, Fold, Kind, Visit};

#[derive(Debug)]
pub struct TestCode<'file> {
    file: &'file SourceFile,
}

#[derive(Debug, Default)]
pub struct Collected {
    lines: Vec<LineNumber>,
    invoked: Vec<Invoked>,
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

impl Collected {
    fn lined(self, line: Option<LineNumber>) -> Collected {
        Collected { lines: self.lines.into_iter().chain(line).collect(), ..self }
    }

    fn named(self, named: Option<Invoked>) -> Collected {
        Collected { invoked: self.invoked.into_iter().chain(named).collect(), ..self }
    }
}

impl<'file> TestCode<'file> {
    const DEFINITION: &'static str = "macro_rules";

    pub(crate) fn tested(&self, collected: Collected) -> Tested<'file> {
        let path = AsRef::<Path>::as_ref(self.file.head());
        let found =
            collected.lines.into_iter().map(|line| Violation::new(Site::new(path, line), PublicApiTests::IN_SOURCE));
        Tested { path, found: found.collect(), invoked: collected.invoked }
    }

    fn gated(found: Collected, attributes: &[Attribute], line: LineNumber) -> Visit<Collected> {
        let walk = Walk::outside_tests(attributes);
        Visit::new(found.lined((walk == Walk::Skipped).then_some(line)), Descend::from(walk))
    }
}

impl Fold for TestCode<'_> {
    type Found = Collected;

    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemMod, Kind::ItemImpl, Kind::ImplItemFn, Kind::ItemFn, Kind::Macro]
    }

    fn on_item_mod(&self, found: Collected, module: &ItemMod, facts: &Facts<'_, ItemMod>) -> Visit<Collected> {
        let walk = facts.gate();
        Visit::new(
            found.lined((walk == Walk::Skipped).then(|| LineNumber::from(module.ident.span()))),
            Descend::from(walk),
        )
    }

    fn on_item_impl(&self, found: Collected, item: &ItemImpl, _: &Facts<'_, ItemImpl>) -> Visit<Collected> {
        Self::gated(found, &item.attrs, LineNumber::from(item.impl_token.span))
    }

    fn on_impl_item_fn(&self, found: Collected, method: &ImplItemFn, _: &Facts<'_, ImplItemFn>) -> Visit<Collected> {
        Self::gated(found, &method.attrs, LineNumber::from(method.sig.ident.span()))
    }

    fn on_item_fn(&self, found: Collected, function: &ItemFn, facts: &Facts<'_, ItemFn>) -> Visit<Collected> {
        let line = LineNumber::from(function.sig.ident.span());
        let walk = Walk::outside_tests(&function.attrs);
        let marked = facts.test_mark().filter(|_| walk == Walk::Entered).map(|_| line);
        Visit::new(found.lined((walk == Walk::Skipped).then_some(line)).lined(marked), Descend::from(walk))
    }

    fn on_macro(&self, found: Collected, invocation: &Macro, facts: &Facts<'_, Macro>) -> Visit<Collected> {
        let line = LineNumber::from(invocation.bang_token.span);
        let law = facts.law().map(|_| line);
        let called =
            Some(invocation).filter(|candidate| candidate.path.get_ident().is_none_or(|name| name != Self::DEFINITION));
        let marked = called.and_then(|call| {
            Macros::attributes_in_file(self.file, call).iter().find_map(TestMarks::test).map(|_| line)
        });
        let settled = law.or(marked);
        let named = invocation.path.get_ident().map(|name| Invoked { name: Name::from(name), line });
        Visit::new(found.lined(settled).named(settled.map_or(named, |_| None)), Descend::Over)
    }
}

impl<'file> From<&'file SourceFile> for TestCode<'file> {
    fn from(file: &'file SourceFile) -> TestCode<'file> {
        TestCode { file }
    }
}
