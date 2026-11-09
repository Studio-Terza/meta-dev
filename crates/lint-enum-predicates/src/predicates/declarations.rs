use syn::visit::Visit;
use syn::{Ident, ItemEnum, ItemMod, ItemStruct, ItemUnion, Macro};

use lint_core::{Macros, SourceFile};

use super::{Declaration, Kind};

pub struct Declarations<'file> {
    file: &'file SourceFile,
    open: Vec<Declaration<'file>>,
    closed: Vec<Declaration<'file>>,
}

impl<'file> Declarations<'file> {
    pub fn in_file(file: &'file SourceFile) -> Vec<Declaration<'file>> {
        let placement = file.placement();
        let mut scan = Declarations {
            file,
            open: vec![Declaration::new(placement.crate_root(), placement.module())],
            closed: Vec::new(),
        };
        scan.visit_file(file.syntax());
        scan.closed.into_iter().chain(scan.open).collect()
    }

    fn record(&mut self, ident: &Ident, kind: Kind) {
        self.open.last_mut().into_iter().for_each(|declaration| declaration.record(ident, kind));
    }
}

impl<'ast> Visit<'ast> for Declarations<'_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        let inner = self.open.last().map(|outer| outer.nested(&module.ident));
        self.open.extend(inner);
        syn::visit::visit_item_mod(self, module);
        self.closed.extend(self.open.pop());
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
        Macros::walk(self.file, invocation, |statement| self.visit_stmt(statement));
    }
}
