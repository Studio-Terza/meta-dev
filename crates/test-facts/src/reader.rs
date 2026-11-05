mod assertion;
mod properties;
mod run;
mod scope;
mod trace;

use std::path::{Path, PathBuf};

use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{ItemFn, ItemMod, Macro};

use lint_core::{Site, SourceFile};
use lint_layout::{LineNumber, Name, TestMarks};
use lint_placement::{CrateName, ModulePath};

use crate::claim::Claim;
use crate::test::{Test, TestId};
use assertion::Assertion;
use properties::{Properties, Property};
use run::{Attributes, Execution, Run};
use scope::{Place, Scope};
use trace::Quantifiers;

pub(crate) struct Reader<'file> {
    file: &'file SourceFile,
    scope: Scope<'file>,
    module: ModulePath,
    tests: Vec<Test<'file>>,
}

impl<'file> Reader<'file> {
    pub(crate) fn read(file: &'file SourceFile) -> Vec<Test<'file>> {
        let mut reader =
            Reader { file, scope: Scope::from(file), module: file.placement().module().clone(), tests: Vec::new() };
        reader.visit_file(file.syntax());
        reader.tests
    }

    fn tested(&mut self, name: Name, spanned: &impl Spanned, claims: Vec<Claim>) {
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
        self.tests.push(Test::new(identity, site, claims));
    }

    fn marked(&mut self, item: &ItemFn) {
        let claims = Run::claims(
            &self.scope,
            &Place::in_module(&self.module),
            &Execution::new(&item.block, &Quantifiers::default(), Attributes::declared(&item.attrs)),
        );
        self.tested(Name::from(&item.sig.ident), &item.sig.ident, claims);
    }

    fn proven(&mut self, found: &[Property]) {
        let measured: Vec<Vec<Claim>> =
            found.iter().map(|property| property.claims(&self.scope, &Place::in_module(&self.module))).collect();
        found.iter().zip(measured).for_each(|(one, claims)| self.tested(Name::from(one.site()), one.site(), claims));
    }

    fn law(&mut self, invocation: &Macro) {
        let name = Assertion::titled(invocation).unwrap_or_default();
        let place = Place::in_module(&self.module);
        let proof = Assertion::named(invocation, self.scope.tables()).law_claims(invocation, &self.scope, &place);
        let claims = proof
            .bodies()
            .iter()
            .flat_map(|body| {
                Run::claims(
                    &self.scope,
                    &place,
                    &Execution::new(&body.block, proof.quantifiers(), Attributes::declared(&[])),
                )
            })
            .chain(proof.claims().iter().cloned())
            .collect();
        self.tested(name, &invocation.path, claims);
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
