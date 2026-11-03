use std::collections::{BTreeMap, BTreeSet};

use syn::visit::{self, Visit};
use syn::{ItemMod, ItemTrait, Path as TraitPath, TraitItemFn};

use lint_core::{Gate, SourceFile, Syntax, Walk};
use lint_layout::{Name, UsePath};
use lint_paths::{Ground, Imports, Module};
use lint_placement::CrateName;

use crate::kind::Kind;

#[derive(Debug, Default)]
pub struct PortTraits {
    declared: BTreeMap<CrateName, BTreeSet<Name>>,
    paced: BTreeSet<Name>,
}

impl PortTraits {
    const PACING: &str = "Pacing";

    #[must_use]
    pub fn declared(files: &[SourceFile]) -> PortTraits {
        files.iter().filter_map(SourceFile::shipped).fold(PortTraits::default(), |found, file| {
            let ground = Gate::PORTS.admits(file).map_or(Ground::Open, |_| Ground::Ports);
            let mut reader = Reader { ground, found, traits: BTreeSet::new() };
            reader.visit_file(file.syntax());
            let mut found = reader.found;
            found.declared.entry(CrateName::from(file.placement().crate_root())).or_default().extend(reader.traits);
            found
        })
    }

    #[must_use]
    pub fn declares(&self, crate_name: &Name, trait_name: &Name) -> Option<&Name> {
        self.declared.get(crate_name)?.get(trait_name)
    }

    #[must_use]
    pub fn judged(&self, imports: &Imports<'_>, module: &Module<'_>, path: &TraitPath) -> Kind {
        let written = UsePath::from(path);
        Kind::written(&written).judged(&imports.candidates(module, written), self)
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
            .filter_map(Syntax::typed)
            .filter_map(|typed| Syntax::type_path(&typed.ty))
            .any(|path| path.path.segments.last().is_some_and(|last| last.ident == Self::PACING))
            .then(|| Name::from(&method.sig.ident))
    }
}

struct Reader {
    ground: Ground,
    found: PortTraits,
    traits: BTreeSet<Name>,
}

impl<'ast> Visit<'ast> for Reader {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        let outer = self.ground;
        self.ground = Gate::PORTS.names(module).map_or(outer, |_| Ground::Ports);
        Walk::enter(&module.attrs, || visit::visit_item_mod(self, module));
        self.ground = outer;
    }

    fn visit_item_trait(&mut self, item: &'ast ItemTrait) {
        let ported = (self.ground == Ground::Ports).then(|| Name::from(&item.ident));
        let paced =
            ported.iter().flat_map(|_| &item.items).filter_map(Syntax::trait_fn).filter_map(PortTraits::paced_by);
        self.found.paced.extend(paced);
        self.traits.extend(ported);
    }
}
