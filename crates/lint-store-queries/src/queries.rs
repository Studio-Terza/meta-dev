mod boundary;
mod unformatted;

use syn::visit::{self, Visit};
use syn::{ItemMod, Macro};

use lint_core::{MacroCalls, Site, SourceFile, Violation};
use lint_layout::{Name, UsePath};
use lint_paths::{Imports, Module};

use boundary::Sql;
use unformatted::Unformatted;

pub(crate) struct Queries<'file, 'run> {
    file: &'file SourceFile,
    module: Module<'file>,
    imports: &'run Imports<'run>,
    found: Vec<Violation<'file>>,
}

impl<'file, 'run> Queries<'file, 'run> {
    pub(crate) fn found(file: &'file SourceFile, imports: &'run Imports<'run>) -> Vec<Violation<'file>> {
        let mut queries = Queries { file, module: Module::of_file(file), imports, found: Vec::new() };
        queries.visit_file(file.syntax());
        queries.found
    }

    fn met(&mut self, invocation: &Macro) {
        let candidates = self.imports.candidates(&self.module, UsePath::from(&invocation.path));
        self.judged(invocation, Sql::queried(invocation, &candidates));
    }

    fn judged(&mut self, invocation: &Macro, sql: Option<Sql>) {
        match sql {
            Some(found) => self.checked(&found),
            None => MacroCalls::within(invocation).iter().for_each(|inner| self.met(inner)),
        }
    }

    fn checked(&mut self, sql: &Sql) {
        let path = self.file.head().path();
        self.found.extend(sql.unformatted().map(|line| Violation::new(Site::new(path, line), Unformatted)));
    }
}

impl<'file> Visit<'file> for Queries<'file, '_> {
    fn visit_item_mod(&mut self, module: &'file ItemMod) {
        self.module.enter(Name::from(&module.ident));
        visit::visit_item_mod(self, module);
        self.module.leave();
    }

    fn visit_macro(&mut self, invocation: &'file Macro) {
        self.met(invocation);
    }
}
