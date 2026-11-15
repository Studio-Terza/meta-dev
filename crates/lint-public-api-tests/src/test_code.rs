use syn::{Attribute, ImplItemFn, ItemFn, ItemImpl, ItemMod, Macro};

use std::collections::BTreeSet;

use super::PublicApiTests;
use lint_core::Macros;
use lint_core::Walk;
use lint_core::{Site, SourceFile, Violation};
use lint_layout::LineNumber;
use lint_layout::Name;
use lint_layout::TestMarks;
use lint_sweep::{Descend, Facts, Finds, Kind, Watch};

#[derive(Debug)]
pub(crate) struct TestCode<'run, 'file> {
    file: &'file SourceFile,
    emitters: &'run BTreeSet<Name>,
    lines: Vec<LineNumber>,
}

impl<'run, 'file> TestCode<'run, 'file> {
    const DEFINITION: &'static str = "macro_rules";

    pub(crate) fn new(file: &'file SourceFile, emitters: &'run BTreeSet<Name>) -> TestCode<'run, 'file> {
        TestCode { file, emitters, lines: Vec::new() }
    }

    fn gated(&mut self, attributes: &[Attribute], line: LineNumber) -> Descend {
        let walk = Walk::outside_tests(attributes);
        self.lines.extend((walk == Walk::Skipped).then_some(line));
        Descend::from(walk)
    }
}

impl<'ast> Watch<'ast> for TestCode<'_, '_> {
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
        let emitted =
            invocation.path.get_ident().filter(|name| self.emitters.contains(&Name::from(*name))).map(|_| line);
        self.lines.extend(law.or(marked).or(emitted));
        Descend::Over
    }
}

impl<'file> Finds<'file> for TestCode<'_, 'file> {
    fn found(self: Box<Self>) -> Vec<Violation<'file>> {
        let path = self.file.head().path();
        self.lines.into_iter().map(|line| Violation::new(Site::new(path, line), PublicApiTests::IN_SOURCE)).collect()
    }
}
