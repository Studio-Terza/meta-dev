use std::collections::BTreeSet;

use syn::visit::{self, Visit};
use syn::{FnArg, ItemMod, ItemTrait, Path as TraitPath, TraitItem, TraitItemFn, Type};

use lint_core::{Gate, SourceFile, Walk};
use lint_layout::{Name, UsePath};
use lint_paths::{Ground, Imports, Module};
use lint_placement::CrateName;

use crate::doubles::Kind;

#[derive(Debug, Default)]
pub struct PortTraits {
    declared: BTreeSet<PortTrait>,
    paced: BTreeSet<Name>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct PortTrait {
    crate_name: CrateName,
    trait_name: Name,
}

impl PortTraits {
    const PACING: &str = "Pacing";

    #[must_use]
    pub fn declared(files: &[SourceFile]) -> PortTraits {
        files.iter().filter_map(SourceFile::shipped).fold(PortTraits::default(), |found, file| {
            let ground = Gate::PORTS.admits(file).map_or(Ground::Open, |_| Ground::Ports);
            let mut reader = Reader { crate_name: CrateName::from(file.placement().crate_root()), ground, found };
            reader.visit_file(file.syntax());
            reader.found
        })
    }

    #[must_use]
    pub fn declares(&self, crate_name: &Name, trait_name: &Name) -> Option<&Name> {
        self.declared
            .get(&PortTrait { crate_name: CrateName::from(crate_name), trait_name: trait_name.clone() })
            .map(|port| &port.trait_name)
    }

    #[must_use]
    pub fn judged(&self, imports: &Imports, module: &Module, path: &TraitPath) -> Kind {
        let written = UsePath::from(path);
        Kind::judged(&written, &imports.candidates(module, &written), self)
    }

    #[must_use]
    pub fn paces(&self, method: &Name) -> Option<&Name> {
        self.paced.get(method)
    }

    fn paced_by(method: &TraitItemFn) -> Option<Name> {
        method
            .sig
            .inputs
            .iter()
            .filter_map(|input| match input {
                FnArg::Typed(typed) => Some(typed.ty.as_ref()),
                FnArg::Receiver(_) => None,
            })
            .any(|ty| matches!(ty, Type::Path(path) if path.path.segments.last().is_some_and(|last| last.ident == Self::PACING)))
            .then(|| Name::from(&method.sig.ident))
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
        let ported = (self.ground == Ground::Ports).then_some(declared);
        let paced = ported.iter().flat_map(|_| &item.items).filter_map(|member| match member {
            TraitItem::Fn(method) => PortTraits::paced_by(method),
            _ => None,
        });
        self.found.paced.extend(paced);
        self.found.declared.extend(ported);
    }
}
