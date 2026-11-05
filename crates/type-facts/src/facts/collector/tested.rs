use std::mem;

use syn::visit::{self, Visit};
use syn::{ItemEnum, ItemMod};

use lint_core::{Site, SourceFile};
use lint_layout::{LineNumber, Name};
use lint_placement::ModulePath;

use crate::gating::Gating;
use crate::shape::Shape;
use crate::type_fact::TypeFact;

pub(crate) struct Tested<'file> {
    file: &'file SourceFile,
    module: ModulePath,
    gating: Gating,
    found: Vec<TypeFact<'file>>,
}

impl<'file> Tested<'file> {
    pub(crate) fn enums_in(file: &'file SourceFile, gating: Gating) -> Vec<TypeFact<'file>> {
        let module = file.placement().module().clone();
        let mut scan = Tested { file, module, gating, found: Vec::new() };
        scan.visit_file(file.syntax());
        scan.found
    }

    fn fact_of(&self, found: &ItemEnum) -> TypeFact<'file> {
        let variants = found.variants.iter().map(|variant| Name::from(&variant.ident)).collect();
        let site = Site::new(self.file.path(), LineNumber::from(found.ident.span()));
        let krate = self.file.placement().package().crate_name();
        TypeFact::declared(krate, Name::from(&found.ident), site, Vec::new())
            .housed(self.module.clone())
            .shaped(Shape::Other)
            .with_variants(variants)
    }
}

impl<'ast> Visit<'ast> for Tested<'_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        let inside = self.module.child(Name::from(&module.ident));
        let outside = mem::replace(&mut self.module, inside);
        let entered = self.gating.within(&module.attrs);
        let outer = mem::replace(&mut self.gating, entered);
        visit::visit_item_mod(self, module);
        self.gating = outer;
        self.module = outside;
    }

    fn visit_item_enum(&mut self, found: &'ast ItemEnum) {
        let fact = self.fact_of(found);
        (self.gating.within(&found.attrs) == Gating::Tested)
            .then_some(fact)
            .into_iter()
            .for_each(|each| self.found.push(each));
    }
}
