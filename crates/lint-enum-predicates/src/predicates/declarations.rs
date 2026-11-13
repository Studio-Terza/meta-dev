use syn::{Ident, ItemEnum, ItemMod, ItemStruct, ItemUnion, Macro};

use lint_core::Parsed;
use lint_sweep::{Descend, Gathers, Kind as Node, Sweep, Watch, Watching};

use super::{Declaration, Kind};

#[derive(Debug)]
pub struct Declarations<'head> {
    open: Vec<Declaration<'head>>,
    closed: Vec<Declaration<'head>>,
}

impl<'head> Declarations<'head> {
    pub fn in_file(parsed: Parsed<'_, 'head>) -> Declarations<'head> {
        let placement = parsed.head().placement();
        Declarations { open: vec![Declaration::new(placement.crate_root(), placement.module())], closed: Vec::new() }
    }

    fn record(&mut self, ident: &Ident, kind: Kind) -> Descend {
        self.open.last_mut().into_iter().for_each(|declaration| declaration.record(ident, kind));
        Descend::Over
    }
}

impl<'ast> Watch<'ast> for Declarations<'_> {
    fn kinds(&self) -> &'static [Node] {
        &[Node::ItemMod, Node::ItemEnum, Node::ItemStruct, Node::ItemUnion, Node::Macro]
    }

    fn on_item_mod(&mut self, module: &'ast ItemMod) -> Descend {
        let inner = self.open.last().map(|outer| outer.nested(&module.ident));
        self.open.extend(inner);
        Descend::Into
    }

    fn off_item_mod(&mut self, _: &'ast ItemMod) {
        self.closed.extend(self.open.pop());
    }

    fn on_item_enum(&mut self, item: &'ast ItemEnum) -> Descend {
        self.record(&item.ident, Kind::Enum)
    }

    fn on_item_struct(&mut self, item: &'ast ItemStruct) -> Descend {
        self.record(&item.ident, Kind::Other)
    }

    fn on_item_union(&mut self, item: &'ast ItemUnion) -> Descend {
        self.record(&item.ident, Kind::Other)
    }

    fn on_macro(&mut self, _: &'ast Macro) -> Descend {
        Descend::Expand
    }
}

impl<'head> Gathers<Vec<Declaration<'head>>> for Declarations<'head> {
    fn watching(&mut self) -> Vec<&mut Watching<'_>> {
        Sweep::alone(self)
    }

    fn gathered(self) -> Vec<Declaration<'head>> {
        self.closed.into_iter().chain(self.open).collect()
    }
}
