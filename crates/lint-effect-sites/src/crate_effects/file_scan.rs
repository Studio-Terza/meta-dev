use std::collections::BTreeSet;

use proc_macro2::Span;
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Attribute, ImplItemFn, ItemFn, ItemImpl, ItemMod, Macro};

use lint_core::{Gate, Site, SourceFile, Walk};
use lint_layout::{LineNumber, Name, Whole};
use lint_paths::Ground;

use super::called::Called;
use super::effect::Effect;
use super::effect_site::EffectSite;
use super::vocabulary::Vocabulary;

pub(super) struct FileScan<'file> {
    file: &'file SourceFile,
    ground: Ground,
    printed: Vec<EffectSite<'file>>,
    tested: BTreeSet<Site<'file>>,
}

impl<'file> FileScan<'file> {
    pub(super) fn scanned(file: &'file SourceFile) -> FileScan<'file> {
        let ground = Gate::PORTS.admits(file).map_or(Ground::Open, |_| Ground::Ports);
        let mut scan = FileScan { file, ground, printed: Vec::new(), tested: BTreeSet::new() };
        scan.visit_file(file.syntax());
        scan
    }

    pub(super) fn printed(&self) -> &[EffectSite<'file>] {
        &self.printed
    }

    pub(super) fn tested(&self) -> &BTreeSet<Site<'file>> {
        &self.tested
    }

    fn skipping(&mut self, attributes: &[Attribute], span: Span) {
        let lines = span.start().line..=span.end().line;
        let tested = lines.map(|line| Site::new(self.file.path(), LineNumber::from(line)));
        self.tested.extend(tested.filter(|_| Walk::outside_tests(attributes) == Walk::Skipped));
    }
}

impl<'ast> Visit<'ast> for FileScan<'_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        let outer = self.ground;
        self.ground = Gate::PORTS.names(module).map_or(outer, |_| Ground::Ports);
        self.skipping(&module.attrs, module.span());
        Walk::enter(&module.attrs, || visit::visit_item_mod(self, module));
        self.ground = outer;
    }

    fn visit_item_fn(&mut self, function: &'ast ItemFn) {
        self.skipping(&function.attrs, function.span());
        Walk::enter(&function.attrs, || visit::visit_item_fn(self, function));
    }

    fn visit_item_impl(&mut self, block: &'ast ItemImpl) {
        self.skipping(&block.attrs, block.span());
        Walk::enter(&block.attrs, || visit::visit_item_impl(self, block));
    }

    fn visit_impl_item_fn(&mut self, function: &'ast ImplItemFn) {
        self.skipping(&function.attrs, function.span());
        Walk::enter(&function.attrs, || visit::visit_impl_item_fn(self, function));
    }

    fn visit_macro(&mut self, invocation: &'ast Macro) {
        let printed = invocation
            .path
            .segments
            .last()
            .map(|last| (Name::from(&last.ident), LineNumber::from(last.ident.span())))
            .filter(|(name, _)| name.matches::<Whole>(&Vocabulary::MACROS).is_some())
            .map(|(name, line)| {
                EffectSite::new(Site::new(self.file.path(), line), Effect::Stdio, Called::Macro(name), self.ground)
            });
        self.printed.extend(printed);
        visit::visit_macro(self, invocation);
    }
}
