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
use lint_sweep::{Descend, Facts, Kind, Watch};

#[derive(Debug)]
pub struct TestCode<'file> {
    file: &'file SourceFile,
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

impl<'file> TestCode<'file> {
    const DEFINITION: &'static str = "macro_rules";

    pub(crate) fn tested(self) -> Tested<'file> {
        let path = self.file.head().path();
        let found = self.lines.into_iter().map(|line| Violation::new(Site::new(path, line), PublicApiTests::IN_SOURCE));
        Tested { path, found: found.collect(), invoked: self.invoked }
    }

    fn gated(&mut self, attributes: &[Attribute], line: LineNumber) -> Descend {
        let walk = Walk::outside_tests(attributes);
        self.lines.extend((walk == Walk::Skipped).then_some(line));
        Descend::from(walk)
    }
}

impl<'ast> Watch<'ast> for TestCode<'_> {
    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemMod, Kind::ItemImpl, Kind::ImplItemFn, Kind::ItemFn, Kind::Macro]
    }

    fn on_item_mod(&mut self, module: &'ast ItemMod, facts: &Facts<'_, ItemMod>) -> Descend {
        let walk = facts.gate();
        self.lines.extend((walk == Walk::Skipped).then(|| LineNumber::from(module.ident.span())));
        Descend::from(walk)
    }

    fn on_item_impl(&mut self, item: &'ast ItemImpl, _: &Facts<'_, ItemImpl>) -> Descend {
        self.gated(&item.attrs, LineNumber::from(item.impl_token.span))
    }

    fn on_impl_item_fn(&mut self, method: &'ast ImplItemFn, _: &Facts<'_, ImplItemFn>) -> Descend {
        self.gated(&method.attrs, LineNumber::from(method.sig.ident.span()))
    }

    fn on_item_fn(&mut self, function: &'ast ItemFn, facts: &Facts<'_, ItemFn>) -> Descend {
        let line = LineNumber::from(function.sig.ident.span());
        let descend = self.gated(&function.attrs, line);
        self.lines.extend(facts.test_mark().filter(|_| descend == Descend::Into).map(|_| line));
        descend
    }

    fn on_macro(&mut self, invocation: &'ast Macro, facts: &Facts<'_, Macro>) -> Descend {
        let line = LineNumber::from(invocation.bang_token.span);
        let law = facts.law().map(|_| line);
        let called =
            Some(invocation).filter(|candidate| candidate.path.get_ident().is_none_or(|name| name != Self::DEFINITION));
        let marked = called.and_then(|call| {
            Macros::attributes_in_file(self.file, call).iter().find_map(TestMarks::test).map(|_| line)
        });
        let settled = law.or(marked);
        self.lines.extend(settled);
        let named = invocation.path.get_ident().map(|name| Invoked { name: Name::from(name), line });
        self.invoked.extend(settled.map_or(named, |_| None));
        Descend::Over
    }
}

impl<'file> From<&'file SourceFile> for TestCode<'file> {
    fn from(file: &'file SourceFile) -> TestCode<'file> {
        TestCode { file, lines: Vec::new(), invoked: Vec::new() }
    }
}
