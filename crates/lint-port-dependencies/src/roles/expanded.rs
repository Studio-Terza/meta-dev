use std::path::Path;

use syn::visit::{self, Visit};
use syn::{Attribute, ItemMod, Macro};

use lint_core::{AttributeMacros, MacroCalls, Parsed, Walk};
use lint_layout::{LineNumber, Name, UsePath};
use lint_paths::{Imports, Module};

#[derive(Debug)]
struct Called {
    candidates: Vec<UsePath>,
    line: LineNumber,
}

#[derive(Debug)]
pub struct FileCalls<'head> {
    path: &'head Path,
    package: Option<&'head Name>,
    called: Vec<Called>,
}

impl<'head> FileCalls<'head> {
    pub(super) fn read(parsed: Parsed<'_, 'head>, imports: &Imports<'_>) -> FileCalls<'head> {
        let head = parsed.head();
        let mut expanded = Expanded { module: Module::of_head(head), imports, called: Vec::new() };
        expanded.visit_file(parsed.file().syntax());
        FileCalls { path: head.path(), package: head.placement().package().root().package(), called: expanded.called }
    }

    pub(super) fn path(&self) -> &'head Path {
        self.path
    }

    pub(super) fn admitted<'names>(&self, admitted: &'names std::collections::BTreeSet<Name>) -> Option<&'names Name> {
        self.package.and_then(|name| admitted.get(name))
    }

    pub(super) fn lines(&self, wanted: &[UsePath]) -> Vec<LineNumber> {
        self.called
            .iter()
            .filter(|called| called.candidates.iter().any(|candidate| wanted.contains(candidate)))
            .map(|called| called.line)
            .collect()
    }
}

struct Expanded<'module, 'run> {
    module: Module<'module>,
    imports: &'run Imports<'run>,
    called: Vec<Called>,
}

impl Expanded<'_, '_> {
    fn met(&mut self, path: &syn::Path) {
        let line = path.segments.first().map(|segment| LineNumber::from(segment.ident.span()));
        let candidates = self.imports.candidates(&self.module, UsePath::from(path));
        self.called.extend(line.map(|found| Called { candidates, line: found }));
    }

    fn invoked(&mut self, invocation: &Macro) {
        self.met(&invocation.path);
        MacroCalls::within(invocation).iter().fold(self, |expanded, nested| {
            expanded.invoked(nested);
            expanded
        });
    }
}

impl<'ast> Visit<'ast> for Expanded<'_, '_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        Walk::enter(&module.attrs, || {
            self.module.enter(Name::from(&module.ident));
            visit::visit_item_mod(self, module);
            self.module.leave();
        });
    }

    fn visit_macro(&mut self, invocation: &'ast Macro) {
        self.invoked(invocation);
    }

    fn visit_attribute(&mut self, attribute: &'ast Attribute) {
        AttributeMacros::expanded(&attribute.meta).iter().fold(self, |expanded, path| {
            expanded.met(path);
            expanded
        });
    }
}
