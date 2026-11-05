use syn::visit::{self, Visit};
use syn::{Attribute, ItemEnum, ItemMod};

use lint_core::{Site, SourceFile, Walk};
use lint_layout::{LineNumber, Name};

use crate::shape::Shape;
use crate::type_fact::TypeFact;

pub(crate) struct Tested<'file> {
    file: &'file SourceFile,
    enclosing: Vec<Attribute>,
    found: Vec<TypeFact<'file>>,
}

impl<'file> Tested<'file> {
    pub(crate) fn enums_in(file: &'file SourceFile) -> Vec<TypeFact<'file>> {
        let mut scan = Tested { file, enclosing: Vec::new(), found: Vec::new() };
        scan.visit_file(file.syntax());
        scan.found
    }

    fn fact_of(&self, found: &ItemEnum) -> TypeFact<'file> {
        let variants = found.variants.iter().map(|variant| Name::from(&variant.ident)).collect();
        let site = Site::new(self.file.path(), LineNumber::from(found.ident.span()));
        let krate = self.file.placement().package().crate_name();
        TypeFact::declared(krate, Name::from(&found.ident), site, Vec::new())
            .shaped(Shape::Other)
            .with_variants(variants)
    }
}

impl<'ast> Visit<'ast> for Tested<'_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        let depth = self.enclosing.len();
        self.enclosing.extend(module.attrs.iter().cloned());
        visit::visit_item_mod(self, module);
        self.enclosing.truncate(depth);
    }

    fn visit_item_enum(&mut self, found: &'ast ItemEnum) {
        let attributes: Vec<Attribute> = self.enclosing.iter().chain(&found.attrs).cloned().collect();
        let fact = self.fact_of(found);
        (Walk::outside_tests(&attributes) == Walk::Skipped)
            .then_some(fact)
            .into_iter()
            .for_each(|each| self.found.push(each));
    }
}
