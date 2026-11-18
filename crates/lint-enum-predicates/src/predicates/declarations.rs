use std::borrow::Cow;

use syntax::{ItemEnum, ItemMod, ItemStruct, ItemUnion, Macro};

use lint_core::Parsed;
use lint_layout::Name;
use lint_placement::{CrateRoot, ModulePath};
use lint_sweep::{
    Contribute, Contributed, Contributing, Contribution, Descend, Facts, Gathers, Kind as Node, Scope, Sweep, Watching,
};

use super::Kind;
use super::declaration::{Declaration, Declared};

#[derive(Debug)]
pub struct Declarations<'head> {
    crate_root: &'head CrateRoot,
    module: &'head ModulePath,
    scan: Contributing<DeclarationScan>,
}

#[derive(Debug)]
pub struct DeclarationScan;

#[derive(Debug)]
pub struct Nested {
    path: Vec<Name>,
    types: Vec<Declared>,
}

impl Nested {
    fn under(self, name: Name) -> Nested {
        Nested { path: [name].into_iter().chain(self.path).collect(), types: self.types }
    }

    fn declaration<'head>(self, crate_root: &'head CrateRoot, module: &'head ModulePath) -> Declaration<'head> {
        let placed = self.path.into_iter().fold(Cow::Borrowed(module), |within, name| Cow::Owned(within.child(name)));
        Declaration::placed(crate_root, placed, self.types)
    }
}

impl<'head> Declarations<'head> {
    pub fn in_file(parsed: Parsed<'_, 'head>) -> Declarations<'head> {
        let placement = parsed.head().placement();
        Declarations {
            crate_root: placement.crate_root(),
            module: placement.module(),
            scan: Contributing::new(DeclarationScan),
        }
    }
}

impl DeclarationScan {
    fn record(name: &syntax::Ident, kind: Kind) -> Contributed<Vec<Declared>, Vec<Nested>> {
        Contributed::new(vec![Declared::new(Name::from(name), kind)], Vec::new(), Descend::Over)
    }
}

impl Contribute for DeclarationScan {
    type Open = Vec<Declared>;
    type Part = Vec<Nested>;

    fn kinds(&self) -> &'static [Node] {
        &[Node::ItemEnum, Node::ItemStruct, Node::ItemUnion, Node::Macro]
    }

    fn scopes(&self) -> &'static [Scope] {
        const { &[Scope::new(Node::ItemMod)] }
    }

    fn off_item_mod(
        &self,
        module: &ItemMod,
        _: &Facts<'_, ItemMod>,
        inner: Contribution<Vec<Declared>, Vec<Nested>>,
    ) -> Vec<Nested> {
        inner.concluded(|types, nested| {
            [Nested { path: Vec::new(), types }]
                .into_iter()
                .chain(nested)
                .map(|within| within.under(Name::from(&module.ident)))
                .collect()
        })
    }

    fn on_item_enum(
        &self,
        item: &ItemEnum,
        _: &Facts<'_, ItemEnum>,
        _around: lint_sweep::Around<'_, Self::Open, Self::Part>,
    ) -> Contributed<Vec<Declared>, Vec<Nested>> {
        Self::record(&item.ident, Kind::Enum)
    }

    fn on_item_struct(
        &self,
        item: &ItemStruct,
        _: &Facts<'_, ItemStruct>,
        _around: lint_sweep::Around<'_, Self::Open, Self::Part>,
    ) -> Contributed<Vec<Declared>, Vec<Nested>> {
        Self::record(&item.ident, Kind::Other)
    }

    fn on_item_union(
        &self,
        item: &ItemUnion,
        _: &Facts<'_, ItemUnion>,
        _around: lint_sweep::Around<'_, Self::Open, Self::Part>,
    ) -> Contributed<Vec<Declared>, Vec<Nested>> {
        Self::record(&item.ident, Kind::Other)
    }

    fn on_macro(
        &self,
        _: &Macro,
        _: &Facts<'_, Macro>,
        _around: lint_sweep::Around<'_, Self::Open, Self::Part>,
    ) -> Contributed<Vec<Declared>, Vec<Nested>> {
        Contributed::descending(Descend::Expand)
    }
}

impl<'head> Gathers<Vec<Declaration<'head>>> for Declarations<'head> {
    fn watching(&mut self) -> impl IntoIterator<Item = &mut Watching<'_>> {
        Sweep::alone(&mut self.scan)
    }

    fn gathered(self) -> Vec<Declaration<'head>> {
        let Declarations { crate_root, module, scan } = self;
        scan.concluded(|_, types, nested| {
            [Nested { path: Vec::new(), types }]
                .into_iter()
                .chain(nested)
                .map(|within| within.declaration(crate_root, module))
                .collect()
        })
    }
}
