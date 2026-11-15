use syn::{Attribute, Block, Ident, ImplItemFn, ItemFn, ItemMod, TraitItemFn};

use graph_reader::{Branches, Propagation};
use lint_core::{Site, SourceFile, Walk};
use lint_layout::{LineNumber, Name, TestMarks};
use lint_sweep::{Descend, Kind, Sweep, Watch};

use super::function::Function;

#[derive(Debug)]
pub(crate) struct Functions<'file> {
    file: &'file SourceFile,
    found: Vec<Function<'file>>,
}

impl<'file> Functions<'file> {
    pub(crate) fn of_file(file: &'file SourceFile) -> Vec<Function<'file>> {
        let mut scan = Functions { file, found: Vec::new() };
        Sweep::through_file(file, &mut scan);
        scan.found
    }

    fn recorded(&mut self, attributes: &[Attribute], name: &Ident, body: Option<&Block>) -> Descend {
        let function = |block: &Block| {
            Function::new(
                Name::from(name),
                Site::new(self.file.head().path(), LineNumber::from(name.span())),
                Branches::scored(block, Propagation::Branch),
            )
        };
        let found = body
            .filter(|_| Walk::outside_tests(attributes) == Walk::Entered)
            .filter(|_| TestMarks::mark_test(attributes).is_none())
            .map(function);
        self.found.extend(found);
        Descend::Into
    }
}

impl<'ast> Watch<'ast> for Functions<'_> {
    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemMod, Kind::ItemFn, Kind::ImplItemFn, Kind::TraitItemFn]
    }

    fn on_item_mod(&mut self, module: &'ast ItemMod) -> Descend {
        Descend::from(Walk::outside_tests(&module.attrs))
    }

    fn on_item_fn(&mut self, function: &'ast ItemFn) -> Descend {
        self.recorded(&function.attrs, &function.sig.ident, Some(&function.block))
    }

    fn on_impl_item_fn(&mut self, function: &'ast ImplItemFn) -> Descend {
        self.recorded(&function.attrs, &function.sig.ident, Some(&function.block))
    }

    fn on_trait_item_fn(&mut self, function: &'ast TraitItemFn) -> Descend {
        self.recorded(&function.attrs, &function.sig.ident, function.default.as_ref())
    }
}
