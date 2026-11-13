use syn::ItemMod;

use crate::walk::Walk;
use lint_layout::Name;

use crate::module::Module;

pub trait Nested<'file>: Sized {
    fn module(&mut self) -> &mut Module<'file>;

    fn nested(&mut self, module: &ItemMod, visit: impl FnOnce(&mut Self)) {
        self.module().enter(Name::from(&module.ident));
        Walk::enter(&module.attrs, || visit(self));
        self.module().leave();
    }
}
