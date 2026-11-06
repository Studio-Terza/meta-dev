mod projection;
mod wrapped;

use syn::visit::Visit;
use syn::{ImplItemFn, ItemImpl, ItemStruct, TypePath};

use crate::owner::Owner;
use crate::reach::Reach;
use crate::spelling::Spelled;
use lint_core::SourceFile;
use lint_layout::{Name, Whole, Word, words};
use projection::{Projection, Slot};
pub use wrapped::Wrapped;

#[derive(Debug)]
struct Newtype {
    name: Name,
    wrapped: Wrapped,
    reach: Reach,
}

#[derive(Debug)]
pub struct Declared {
    reach: Reach,
    newtypes: Vec<Newtype>,
    projections: Vec<Projection>,
}

impl Declared {
    const EDGES: [Word<'static>; 3] = words!["new", "try_new", "into_inner"];

    pub fn edge(&self, owner: &Owner, item: &ImplItemFn, reach: Reach) -> Option<Wrapped> {
        let inputs = &item.sig.inputs;
        Name::from(&item.sig.ident)
            .matches::<Whole>(&Self::EDGES)
            .and(inputs.first().xor(inputs.iter().nth(1)))
            .and_then(|_| self.wrapped(owner, reach))
    }

    pub fn in_files(files: &[SourceFile]) -> Declared {
        let empty = Declared { reach: Reach::Blocking, newtypes: Vec::new(), projections: Vec::new() };
        files.iter().fold(empty, |mut declared, file| {
            declared.read(file);
            declared
        })
    }

    pub fn projected(&self, path: &TypePath) -> Option<Spelled> {
        Slot::of_path(path)
            .and_then(|slot| self.projections.iter().find(|projection| *projection.slot() == slot))
            .map(Projection::spelled)
            .cloned()
    }

    fn read(&mut self, file: &SourceFile) {
        self.reach = Reach::of_role(file);
        self.visit_file(file.syntax());
    }

    fn wrapped(&self, owner: &Owner, reach: Reach) -> Option<Wrapped> {
        owner
            .name()
            .and_then(|name| {
                self.newtypes
                    .iter()
                    .filter(|newtype| reach.sees(newtype.reach).is_some())
                    .find(|newtype| newtype.name == *name)
            })
            .map(|newtype| newtype.wrapped)
    }
}

impl<'ast> Visit<'ast> for Declared {
    fn visit_item_struct(&mut self, item: &'ast ItemStruct) {
        self.newtypes.extend(Wrapped::of_fields(&item.fields).map(|wrapped| Newtype {
            name: Name::from(&item.ident),
            wrapped,
            reach: self.reach,
        }));
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        let owner = Owner::of_impl(item);
        let interface = item.trait_.as_ref().and_then(|(_, path, _)| path.segments.last());
        self.projections
            .extend(item.items.iter().filter_map(|member| Projection::of_member(&owner, interface, member)));
        syn::visit::visit_item_impl(self, item);
    }
}
