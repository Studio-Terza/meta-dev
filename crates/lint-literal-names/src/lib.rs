mod finding;

use finding::Finding;
use lint_core::{Headline, Lint};
use lint_file::FileLint;
use std::path::Path;

use syn::visit::Visit;
use syn::{Ident, ImplItemFn, ItemFn, ItemImpl, ItemMod, Macro};

use lint_core::Bound;
use lint_core::Macros;
use lint_core::Report;
use lint_core::Site;
use lint_core::SourceFile;
use lint_core::Violation;
use lint_core::Walk;
use lint_layout::LineNumber;
use lint_layout::{Name, Whole, Word, words};

#[derive(Debug)]
pub struct LiteralNames;

impl Lint for LiteralNames {
    fn report(files: &[SourceFile]) -> Report<'_> {
        Report::new(Headline::new("function names say what the function does"), Self::in_shipped(files))
    }
}

impl LiteralNames {
    const SHORTEST: Bound = Bound::Least(3);
    const VAGUE: [Word<'static>; 24] = words![
        "with", "from", "into", "for", "get", "set", "make", "check", "build", "create", "init", "process", "handle",
        "helper", "util", "data", "value", "item", "thing", "execute", "perform", "update", "compute", "manage",
    ];

    fn vague(name: &Name) -> Option<Finding<'_>> {
        Self::SHORTEST
            .allows(name.length())
            .is_none()
            .then_some(Finding::VagueName { name })
            .or_else(|| name.matches::<Whole>(&Self::VAGUE).map(|_| Finding::VagueName { name }))
    }
}

impl FileLint for LiteralNames {
    fn violations(file: &SourceFile) -> Vec<Violation<'_>> {
        let mut names = VagueNames { file, path: file.path(), found: Vec::new() };
        names.visit_file(file.syntax());
        names.found
    }
}

struct VagueNames<'file> {
    file: &'file SourceFile,
    path: &'file Path,
    found: Vec<Violation<'file>>,
}

impl VagueNames<'_> {
    fn inherent(&mut self, item: &ItemImpl) {
        match Walk::inherent_impls(item) {
            Walk::Entered => syn::visit::visit_item_impl(self, item),
            Walk::Skipped => {}
        }
    }

    fn note(&mut self, ident: &Ident) {
        let site = Site::new(self.path, LineNumber::from(ident.span()));
        self.found.extend(LiteralNames::vague(&Name::from(ident)).map(|finding| Violation::new(site, finding)));
    }
}

impl<'ast> Visit<'ast> for VagueNames<'_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        Walk::enter(&module.attrs, || syn::visit::visit_item_mod(self, module));
    }

    fn visit_macro(&mut self, invocation: &'ast Macro) {
        Macros::walk(self.file, invocation, |statement| self.visit_stmt(statement));
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        Walk::enter(&item.attrs, || self.inherent(item));
    }

    fn visit_item_fn(&mut self, function: &'ast ItemFn) {
        Walk::enter(&function.attrs, || {
            self.note(&function.sig.ident);
            syn::visit::visit_item_fn(self, function);
        });
    }

    fn visit_impl_item_fn(&mut self, function: &'ast ImplItemFn) {
        Walk::enter(&function.attrs, || {
            self.note(&function.sig.ident);
            syn::visit::visit_impl_item_fn(self, function);
        });
    }
}
