use syn::Type;

use lint_layout::Name;

pub struct SingleField {
    name: Name,
    ty: Type,
}

impl SingleField {
    pub fn new(name: Name, ty: Type) -> SingleField {
        SingleField { name, ty }
    }

    pub fn name(&self) -> &Name {
        &self.name
    }

    pub fn ty(&self) -> &Type {
        &self.ty
    }
}
