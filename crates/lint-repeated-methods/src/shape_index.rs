mod inners;
mod method_shape;
mod placed;
mod repeated_shape;

use std::collections::BTreeMap;
use std::path::Path;

use syn::visit::Visit;
use syn::{ImplItem, ItemImpl, ItemMod, Visibility};

use lint_core::{DeclaredType, Home, Most, Site, SourceFile, TypeHomes, Violation, Walk};
use lint_layout::{Length, LineNumber, Name};

use inners::Inners;
use method_shape::{Erasure, MethodShape};
use placed::Placed;
use repeated_shape::RepeatedShape;

#[derive(Debug)]
pub struct ShapeIndex<'file> {
    owners: BTreeMap<DeclaredType<'file>, BTreeMap<MethodShape, Site<'file>>>,
    inners: Inners<'file>,
    homes: TypeHomes<'file>,
    home: Home<'file>,
    file: &'file Path,
}

impl<'file> ShapeIndex<'file> {
    #[must_use]
    pub fn of_files(files: &[&'file SourceFile]) -> ShapeIndex<'file> {
        let index = ShapeIndex {
            owners: BTreeMap::new(),
            inners: Inners::gathered_from(files),
            homes: TypeHomes::declared_in(files),
            home: Home::default(),
            file: Path::new(""),
        };
        files.iter().fold(index, |index, file| index.reading(file))
    }

    #[must_use]
    pub fn repeated(self, most: Most) -> Vec<Violation<'file>> {
        let shapes = self
            .owners
            .iter()
            .flat_map(|(owner, methods)| methods.iter().map(move |(shape, site)| (shape, owner, *site)))
            .fold(
                BTreeMap::new(),
                |mut shapes: BTreeMap<&MethodShape, BTreeMap<&DeclaredType<'file>, Site<'file>>>,
                 (shape, owner, site)| {
                    shapes.entry(shape).or_default().insert(owner, site);
                    shapes
                },
            );
        shapes
            .into_iter()
            .filter(|(_, types)| most.allows(Length::new(types.len())).is_none())
            .filter_map(|(shape, types)| {
                let site = types.values().min().copied()?;
                Some(Violation::new(site, RepeatedShape::new(shape, types.into_keys().collect())))
            })
            .collect()
    }

    fn reading(self, file: &'file SourceFile) -> ShapeIndex<'file> {
        let mut index = ShapeIndex { file: file.path(), home: Home::containing(file), ..self };
        index.visit_file(file.syntax());
        index
    }

    fn placed(&self, item: &ItemImpl, written: Name) -> Placed<'file> {
        let self_type = self.homes.resolve(written, self.home);
        let erasure = Erasure::around(self_type.as_ref(), self.inners.inner_of(&self_type.home(), self_type.as_ref()));
        let shapes = item
            .items
            .iter()
            .filter_map(|member| match (member, &item.trait_) {
                (ImplItem::Fn(function), None)
                    if !matches!(function.vis, Visibility::Inherited) && function.sig.constness.is_none() =>
                {
                    Some(function)
                }
                _ => None,
            })
            .fold(BTreeMap::new(), |mut shapes, function| {
                let site = Site::new(self.file, LineNumber::from(function.sig.ident.span()));
                shapes.entry(MethodShape::described(&function.sig, &erasure)).or_insert(site);
                shapes
            });
        Placed::new(self_type, shapes)
    }
}

impl<'ast> Visit<'ast> for ShapeIndex<'_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        match Walk::outside_tests(&module.attrs) {
            Walk::Entered => syn::visit::visit_item_mod(self, module),
            Walk::Skipped => {}
        }
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        let self_type = Name::of_type(&item.self_ty).filter(|_| Walk::outside_tests(&item.attrs) == Walk::Entered);
        let placed = self_type.map(|self_type| self.placed(item, self_type));
        placed.into_iter().fold(&mut self.owners, |owners, placed| {
            placed.recorded_in(owners);
            owners
        });
    }
}
