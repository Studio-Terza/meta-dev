mod boundary;
mod unformatted;

use std::path::Path;

use syn::Macro;
use syn::visit::Visit;

use lint_core::{MacroCalls, Site, SourceFile, Violation};

use boundary::Sql;
use unformatted::Unformatted;

#[derive(Debug)]
pub(crate) struct Queries<'file> {
    path: &'file Path,
    found: Vec<Violation<'file>>,
}

impl<'file> From<&'file SourceFile> for Queries<'file> {
    fn from(file: &'file SourceFile) -> Queries<'file> {
        let mut queries = Queries { path: file.path(), found: Vec::new() };
        queries.visit_file(file.syntax());
        queries
    }
}

impl<'file> From<Queries<'file>> for Vec<Violation<'file>> {
    fn from(queries: Queries<'file>) -> Vec<Violation<'file>> {
        queries.found
    }
}

impl<'file> Visit<'file> for Queries<'file> {
    fn visit_macro(&mut self, invocation: &'file Macro) {
        self.met(invocation);
    }
}

impl Queries<'_> {
    fn met(&mut self, invocation: &Macro) {
        match Sql::queried(invocation) {
            Some(sql) => {
                self.found.extend(sql.unformatted().map(|line| Violation::new(Site::new(self.path, line), Unformatted)))
            }
            None => MacroCalls::within(invocation).iter().for_each(|inner| self.met(inner)),
        }
    }
}
