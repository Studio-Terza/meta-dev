use syn::visit::{self, Visit};
use syn::{ItemStruct, Lifetime, TypeReference};

#[derive(Debug, Default)]
pub struct Borrows(usize);

impl Borrows {
    pub fn owning(item: &ItemStruct) -> Option<&ItemStruct> {
        let mut borrows = Borrows::default();
        borrows.visit_generics(&item.generics);
        borrows.visit_fields(&item.fields);
        (borrows.0 == 0).then_some(item)
    }
}

impl<'ast> Visit<'ast> for Borrows {
    fn visit_type_reference(&mut self, reference: &'ast TypeReference) {
        self.0 += 1;
        visit::visit_type_reference(self, reference);
    }

    fn visit_lifetime(&mut self, lifetime: &'ast Lifetime) {
        self.0 += 1;
        visit::visit_lifetime(self, lifetime);
    }
}
