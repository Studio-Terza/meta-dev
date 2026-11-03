use std::collections::BTreeSet;

use syn::visit::{self, Visit};
use syn::{ItemMod, ItemTrait};

use lint_core::{Gate, SourceFile, Walk};
use lint_layout::Name;
use lint_paths::Ground;
use lint_placement::CrateName;

#[derive(Debug, Default)]
pub struct PortTraits(BTreeSet<PortTrait>);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct PortTrait {
    crate_name: CrateName,
    trait_name: Name,
}

impl PortTraits {
    pub fn declared(files: &[SourceFile]) -> PortTraits {
        files.iter().filter_map(SourceFile::shipped).fold(PortTraits::default(), |found, file| {
            let ground = Gate::PORTS.admits(file).map_or(Ground::Open, |_| Ground::Ports);
            let mut reader = Reader { crate_name: CrateName::from(file.placement().crate_root()), ground, found };
            reader.visit_file(file.syntax());
            reader.found
        })
    }

    pub fn declares(&self, crate_name: &Name, trait_name: &Name) -> Option<&Name> {
        self.0
            .get(&PortTrait { crate_name: CrateName::from(crate_name), trait_name: trait_name.clone() })
            .map(|port| &port.trait_name)
    }
}

struct Reader {
    crate_name: CrateName,
    ground: Ground,
    found: PortTraits,
}

impl<'ast> Visit<'ast> for Reader {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        let outer = self.ground;
        self.ground = Gate::PORTS.names(module).map_or(outer, |_| Ground::Ports);
        Walk::enter(&module.attrs, || visit::visit_item_mod(self, module));
        self.ground = outer;
    }

    fn visit_item_trait(&mut self, item: &'ast ItemTrait) {
        let declared = PortTrait { crate_name: self.crate_name.clone(), trait_name: Name::from(&item.ident) };
        self.found.0.extend((self.ground == Ground::Ports).then_some(declared));
    }
}
