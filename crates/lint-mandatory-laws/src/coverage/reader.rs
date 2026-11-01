use syn::Macro;
use syn::visit::{self, Visit};

mod mentions;

use super::kind::Kind;
use super::law::Law;
use crate::package::Package;
use lint_core::SourceFile;
use mentions::Mentions;

pub(crate) struct Reader {
    package: Package,
    laws: Vec<Law>,
}

impl Reader {
    pub(crate) fn read(file: &SourceFile) -> Vec<Law> {
        let mut reader = Reader { package: Package::containing(file), laws: Vec::new() };
        reader.visit_file(file.syntax());
        reader.laws
    }
}

impl<'ast> Visit<'ast> for Reader {
    fn visit_macro(&mut self, invocation: &'ast Macro) {
        let kind = Kind::of_macro(invocation);
        let mentions = Mentions::beyond_title(invocation.tokens.clone(), kind);
        let law = Law::new(self.package.clone(), kind, mentions);
        self.laws.extend((kind != Kind::Other).then_some(law));
        visit::visit_macro(self, invocation);
    }
}
