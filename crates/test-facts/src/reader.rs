mod assertion;
mod body;
mod properties;
mod scope;
mod trace;

use std::path::{Path, PathBuf};

use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{ItemFn, ItemMod, Macro};

use lint_core::{MacroAliases, Site, SourceFile};
use lint_layout::{LineNumber, Name, TestMarks, Word};
use lint_placement::{CrateName, ModulePath};

use crate::check::{Check, Domain};
use crate::test::{Test, TestId};
use assertion::Assertion;
use body::Body;
use properties::{Properties, Property};
use scope::Scope;

pub(crate) struct Reader<'file> {
    file: &'file SourceFile,
    scope: Scope<'file>,
    tables: MacroAliases,
    module: ModulePath,
    tests: Vec<Test<'file>>,
}

impl<'file> Reader<'file> {
    const TABLE: Word<'static> = Word::new("table");

    pub(crate) fn read(file: &'file SourceFile) -> Vec<Test<'file>> {
        let mut reader = Reader {
            file,
            scope: Scope::of_file(file.syntax()),
            tables: MacroAliases::in_file(file.syntax(), Self::TABLE),
            module: file.placement().module().clone(),
            tests: Vec::new(),
        };
        reader.visit_file(file.syntax());
        reader.tests
    }

    fn tested(&mut self, name: Name, spanned: &impl Spanned, checks: Vec<Check>) {
        let placement = self.file.placement();
        let package = placement.package().root().directory();
        let target = placement.crate_root().directory();
        let identity = TestId::new(
            CrateName::from(placement.package().root()),
            target.strip_prefix(package).map_or_else(|_| PathBuf::new(), Path::to_path_buf),
            self.module.clone(),
            name,
        );
        let site = Site::new(self.file.path(), LineNumber::from(spanned.span()));
        self.tests.push(Test::new(identity, site, checks));
    }

    fn marked(&mut self, item: &ItemFn) {
        let checks = Body::checks(&self.scope, &self.tables, &item.block, Domain::SINGLE);
        self.tested(Name::from(&item.sig.ident), &item.sig.ident, checks);
    }

    fn proven(&mut self, found: &[Property]) {
        let measured: Vec<Vec<Check>> =
            found.iter().map(|property| property.checks(&self.scope, &self.tables)).collect();
        found.iter().zip(measured).for_each(|(one, checks)| self.tested(Name::from(one.site()), one.site(), checks));
    }

    fn law(&mut self, invocation: &Macro) {
        let name = Assertion::titled(invocation).unwrap_or_default();
        let checks = Assertion::named(invocation, &self.tables).law_checks(invocation, &self.scope, &self.tables);
        self.tested(name, &invocation.path, checks);
    }
}

impl<'file> Visit<'file> for Reader<'file> {
    fn visit_item_mod(&mut self, item: &'file ItemMod) {
        let outer = self.module.clone();
        self.module = outer.child(Name::from(&item.ident));
        visit::visit_item_mod(self, item);
        self.module = outer;
    }

    fn visit_item_fn(&mut self, item: &'file ItemFn) {
        match TestMarks::mark_test(&item.attrs) {
            Some(_) => self.marked(item),
            None => visit::visit_item_fn(self, item),
        }
    }

    fn visit_macro(&mut self, invocation: &'file Macro) {
        match TestMarks::law(invocation) {
            Some(law) => self.law(law),
            None => self.proven(&Properties::declared(invocation)),
        }
    }
}
