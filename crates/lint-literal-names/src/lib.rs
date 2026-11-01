mod finding;

use finding::Finding;
use lint_core::{Headline, Lint};
use syn::visit::Visit;
use syn::{Ident, ImplItemFn, ItemFn, ItemImpl, ItemMod, Macro, TraitItemFn};

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
                .filter(|file| file.is_shipped())
                .flat_map(|file| {
                    Self::vague_names(file.syntax()).into_iter().map(move |name| {
                        Violation::new(
                            Site::new(file.path(), LineNumber::from(name.span())),
                            Finding::VagueName { name: Name::from(&name) },
                        )
                    })
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

    fn is_vague(name: &Name) -> bool {
        !Self::SHORTEST.allows(name.length()) || name.matches::<Whole>(&Self::VAGUE)
    }

    fn vague_names(file: &syn::File) -> Vec<Ident> {
        let mut names = VagueNames { found: Vec::new() };
        names.visit_file(file);
        names.found
    }
}

struct VagueNames {
    found: Vec<Ident>,
}

impl VagueNames {
    fn note(&mut self, name: &Ident) {
        self.found.extend(LiteralNames::is_vague(&Name::from(name)).then(|| name.clone()));
    }
}

impl<'ast> Visit<'ast> for VagueNames {
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

    fn visit_trait_item_fn(&mut self, function: &'ast TraitItemFn) {
        self.note(&function.sig.ident);
        syn::visit::visit_trait_item_fn(self, function);
    }
}
