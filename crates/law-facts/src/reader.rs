mod mentions;
mod routes;

use syn::Macro;
use syn::visit::{self, Visit};

use crate::kind::Kind;
use crate::law::Law;
use crate::package::Package;
use lint_core::SourceFile;
use mentions::Mentions;
use routes::Routes;

pub(crate) struct Reader<'file> {
    package: Package<'file>,
    laws: Vec<Law<'file>>,
}

impl<'file> Reader<'file> {
    pub(crate) fn read(file: &'file SourceFile) -> Vec<Law<'file>> {
        let mut reader = Reader { package: Package::containing(file), laws: Vec::new() };
        reader.visit_file(file.syntax());
        reader.laws
    }
}

impl<'ast> Visit<'ast> for Reader<'_> {
    fn visit_macro(&mut self, invocation: &'ast Macro) {
        let kind = Kind::of_macro(invocation);
        let package = self.package;
        let law = (kind != Kind::Other).then(|| {
            Law::new(package, kind, Mentions::beyond_title(invocation.tokens.clone(), kind), Routes::read(invocation))
        });
        self.laws.extend(law);
        visit::visit_macro(self, invocation);
    }
}
