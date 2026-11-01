use syn::visit::Visit;
use syn::{Ident, ItemEnum, ItemMod, ItemStruct, ItemUnion, Macro};

use lint_core::{Macros, SourceFile};
use lint_layout::Name;
use lint_placement::ModulePath;

use super::{Declaration, Kind};

pub struct Declarations<'scan> {
    file: &'scan SourceFile,
    module: ModulePath,
    found: Vec<Declaration>,
}

impl Declarations<'_> {
    pub fn in_file(file: &SourceFile) -> Vec<Declaration> {
        let mut scan = Declarations { file, module: file.placement().module().clone(), found: Vec::new() };
        scan.visit_file(file.syntax());
        scan.found
    }

    fn record(&mut self, ident: &Ident, kind: Kind) {
        self.found.push(Declaration::new(
            self.file.placement().crate_root(),
            self.module.clone(),
            Name::from(ident),
            kind,
        ));
    }
}

impl<'ast> Visit<'ast> for Declarations<'_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        let inner = self.module.child(&Name::from(&module.ident));
        let outer = std::mem::replace(&mut self.module, inner);
        syn::visit::visit_item_mod(self, module);
        self.module = outer;
    }

    fn visit_item_enum(&mut self, item: &'ast ItemEnum) {
        self.record(&item.ident, Kind::Enum);
    }

    fn visit_item_struct(&mut self, item: &'ast ItemStruct) {
        self.record(&item.ident, Kind::Other);
    }

    fn visit_item_union(&mut self, item: &'ast ItemUnion) {
        self.record(&item.ident, Kind::Other);
    }

    fn visit_macro(&mut self, invocation: &'ast Macro) {
        Macros::walk(invocation, |statement| self.visit_stmt(statement));
    }
}
