mod finding;

use finding::Finding;
use lint_core::{Headline, Lint};
use syn::visit::Visit;
use syn::{Ident, ImplItemFn, ItemFn, ItemImpl, ItemMod, Macro};

use lint_core::Least;
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
    fn report(files: &[SourceFile]) -> Report {
        Report::new(
            Headline::new("function names say what the function does"),
            files
                .iter()
                .filter_map(SourceFile::shipped)
                .flat_map(|file| Self::fn_names(file.syntax()).into_iter().map(move |name| (file, name)))
                .filter_map(|(file, name)| {
                    Self::vague(Name::from(&name))
                        .map(|finding| Violation::new(Site::new(file.path(), LineNumber::from(name.span())), finding))
                })
                .collect(),
        )
    }
}

impl LiteralNames {
    const SHORTEST: Least = Least::new(3);
    const VAGUE: [Word<'static>; 24] = words![
        "with", "from", "into", "for", "get", "set", "make", "check", "build", "create", "init", "process", "handle",
        "helper", "util", "data", "value", "item", "thing", "execute", "perform", "update", "compute", "manage",
    ];

    fn vague(name: Name) -> Option<Finding> {
        Some(Finding::VagueName { name }).filter(|Finding::VagueName { name }| {
            Self::SHORTEST.allows(name.length()).is_none() || name.matches::<Whole>(&Self::VAGUE).is_some()
        })
    }

    fn fn_names(file: &syn::File) -> Vec<Ident> {
        let mut names = FnNames { found: Vec::new() };
        names.visit_file(file);
        names.found
    }
}

struct FnNames {
    found: Vec<Ident>,
}

impl FnNames {
    fn note(&mut self, name: &Ident) {
        self.found.push(name.clone());
    }
}

impl<'ast> Visit<'ast> for FnNames {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        Walk::enter(&module.attrs, || syn::visit::visit_item_mod(self, module));
    }

    fn visit_macro(&mut self, invocation: &'ast Macro) {
        Macros::walk(invocation, |statement| self.visit_stmt(statement));
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        Walk::enter(&item.attrs, || match Walk::inherent_impls(item) {
            Walk::Entered => syn::visit::visit_item_impl(self, item),
            Walk::Skipped => {}
        });
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
