use std::collections::BTreeMap;

use body_shape::{Reading, Shape};
use equivalence::Equivalence;
use lint_core::{SourceFile, Syntax, Walk};
use lint_layout::Name;
use lint_placement::CrateName;
use quote::ToTokens;
use syn::visit::{self, Visit};
use syn::{ItemImpl, ItemMod};

use crate::end::Node;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Filled<'file> {
    krate: &'file CrateName,
    filler: Name,
    interface: Name,
}

#[derive(Debug, Default)]
pub(super) struct Fillers<'file> {
    bodies: BTreeMap<Filled<'file>, BTreeMap<Name, Shape>>,
}

impl<'file> Fillers<'file> {
    pub(super) fn read(files: &'file [SourceFile], equivalence: Equivalence) -> Self {
        files.iter().filter_map(SourceFile::shipped).fold(Fillers::default(), |fillers, file| {
            let mut scan = Scan { krate: file.placement().package().crate_name(), equivalence, fillers };
            scan.visit_file(file.syntax());
            scan.fillers
        })
    }

    pub(super) fn alike<'found>(&self, first: Node<'found>, other: Node<'found>) -> Option<Node<'found>> {
        let ours = self.filled(&first);
        let theirs = self.filled(&other);
        ours.iter()
            .find_map(|(interface, methods)| theirs.get(interface).filter(|their| **their != *methods))
            .map_or(Some(other), |_| None)
    }

    fn filled(&self, node: &Node<'_>) -> BTreeMap<&Name, &BTreeMap<Name, Shape>> {
        self.bodies
            .iter()
            .filter(|(filled, _)| filled.krate == AsRef::<CrateName>::as_ref(node))
            .filter(|(filled, _)| filled.filler == *AsRef::<Name>::as_ref(node))
            .map(|(filled, methods)| (&filled.interface, methods))
            .collect()
    }
}

struct Scan<'file> {
    krate: &'file CrateName,
    equivalence: Equivalence,
    fillers: Fillers<'file>,
}

impl<'ast> Visit<'ast> for Scan<'_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        Walk::enter(&module.attrs, || visit::visit_item_mod(self, module));
    }

    fn visit_item_impl(&mut self, found: &'ast ItemImpl) {
        let interface =
            found.trait_.as_ref().and_then(|(_, path, _)| path.segments.last()).map(|last| Name::from(&last.ident));
        let filled = interface.zip(Name::of_type(&found.self_ty)).map(|(interface, filler)| Filled {
            krate: self.krate,
            filler,
            interface,
        });
        let methods = found
            .items
            .iter()
            .filter_map(Syntax::impl_fn)
            .map(|method| {
                (
                    Name::from(&method.sig.ident),
                    Reading::erasing(method.block.to_token_stream(), self.equivalence).shape(),
                )
            })
            .collect();
        self.fillers.bodies.extend(
            filled.filter(|_| Walk::outside_tests(&found.attrs) == Walk::Entered).map(|filled| (filled, methods)),
        );
    }
}
