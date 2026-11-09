use syn::punctuated::Punctuated;
use syn::visit::{self, Visit};
use syn::{Attribute, ItemMod, Macro, Path, Token};

use lint_core::{MacroCalls, SourceFile, Walk};
use lint_layout::{LineNumber, Name, UsePath};
use lint_paths::{Imports, Module};

pub(super) struct Expanded<'file, 'run> {
    module: Module<'file>,
    imports: &'run Imports<'run>,
    wanted: &'run [UsePath],
    found: Vec<LineNumber>,
}

impl<'file, 'run> Expanded<'file, 'run> {
    pub(super) fn lines(
        file: &'file SourceFile,
        imports: &'run Imports<'run>,
        wanted: &'run [UsePath],
    ) -> Vec<LineNumber> {
        let mut expanded = Expanded { module: Module::of_file(file), imports, wanted, found: Vec::new() };
        expanded.visit_file(file.syntax());
        expanded.found
    }

    fn met(&self, path: &Path) -> Option<LineNumber> {
        let line = path.segments.first().map(|segment| LineNumber::from(segment.ident.span()));
        let candidates = self.imports.candidates(&self.module, UsePath::from(path));
        candidates.iter().find(|candidate| self.wanted.contains(candidate)).and(line)
    }

    fn called(&self, invocation: &Macro) -> Vec<LineNumber> {
        let inner = MacroCalls::within(invocation);
        self.met(&invocation.path).into_iter().chain(inner.iter().flat_map(|inner| self.called(inner))).collect()
    }

    fn derived(attribute: &Attribute) -> Vec<Path> {
        Some(attribute)
            .filter(|attribute| attribute.path().is_ident("derive"))
            .and_then(|attribute| attribute.parse_args_with(Punctuated::<Path, Token![,]>::parse_terminated).ok())
            .map(|paths| paths.into_iter().collect())
            .unwrap_or_default()
    }
}

impl<'file> Visit<'file> for Expanded<'file, '_> {
    fn visit_item_mod(&mut self, module: &'file ItemMod) {
        Walk::enter(&module.attrs, || {
            self.module.enter(Name::from(&module.ident));
            visit::visit_item_mod(self, module);
            self.module.leave();
        });
    }

    fn visit_macro(&mut self, invocation: &'file Macro) {
        let found = self.called(invocation);
        self.found.extend(found);
    }

    fn visit_attribute(&mut self, attribute: &'file Attribute) {
        let found: Vec<LineNumber> = Self::derived(attribute).iter().filter_map(|path| self.met(path)).collect();
        self.found.extend(found);
    }
}
