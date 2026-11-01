mod boilerplate;
mod inners;
mod method_shape;
mod placed;
mod repeated_shape;

use std::collections::BTreeMap;
use std::path::PathBuf;

use syn::visit::Visit;
use syn::{ImplItem, ItemImpl, ItemMod, Type, Visibility};

use lint_core::{DeclaredType, Home, Most, Site, SourceFile, TypeHomes, Violation, Walk};
use lint_layout::{LineNumber, Name};

use boilerplate::Boilerplate;
use inners::Inners;
use method_shape::{Erasure, MethodShape};
use placed::Placed;
use repeated_shape::RepeatedShape;

#[derive(Debug, Default)]
pub struct ShapeIndex {
    shapes: BTreeMap<MethodShape, BTreeMap<DeclaredType, Site>>,
    inners: Inners,
    homes: TypeHomes,
    home: Home,
    file: PathBuf,
}

impl ShapeIndex {
    #[must_use]
    pub fn of_files(files: &[&SourceFile]) -> ShapeIndex {
        let index = ShapeIndex {
            inners: Inners::gathered_from(files),
            homes: TypeHomes::declared_in(files),
            ..ShapeIndex::default()
        };
        files.iter().fold(index, |index, file| index.reading(file))
    }

    #[must_use]
    pub fn repeated(self, most: Most) -> Vec<Violation> {
        self.shapes
            .into_iter()
            .filter(|(_, types)| !most.allows(types.len()))
            .filter_map(|(shape, types)| {
                let site = types.values().min().cloned()?;
                let repeated = RepeatedShape::new(shape, types.into_keys().collect());
                Some(Violation::new(site, repeated))
            })
            .collect()
    }

    fn reading(self, file: &SourceFile) -> ShapeIndex {
        let mut index = ShapeIndex { file: file.path().to_path_buf(), home: Home::containing(file), ..self };
        index.visit_file(file.syntax());
        index
    }

    fn self_type(self_type: &Type) -> Option<Name> {
        match self_type {
            Type::Path(path) => path.path.segments.last().map(|segment| Name::from(&segment.ident)),
            _ => None,
        }
    }

    fn placed(&self, item: &ItemImpl, written: &Name) -> Vec<Placed> {
        let self_type = self.homes.resolve(written, &self.home);
        let erasure = Erasure::around(written, self.inners.inner_of(&self_type));
        item.items
            .iter()
            .filter_map(|member| match (member, &item.trait_) {
                (ImplItem::Fn(function), None) if !matches!(function.vis, Visibility::Inherited) => {
                    Some((function, MethodShape::described(&function.sig, &erasure)))
                }
                (ImplItem::Fn(function), Some((_, path, _)))
                    if self.inners.inner_of(&self_type).is_some() && Boilerplate::names(path) =>
                {
                    Some((
                        function,
                        MethodShape::described(&function.sig, &erasure).implementing(path, &function.block, &erasure),
                    ))
                }
                _ => None,
            })
            .map(|(function, shape)| {
                let site = Site::new(&self.file, LineNumber::from(function.sig.ident.span()));
                Placed::new(self_type.clone(), shape, site)
            })
            .collect()
    }
}

impl<'ast> Visit<'ast> for ShapeIndex {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        match Walk::outside_tests(&module.attrs) {
            Walk::Entered => syn::visit::visit_item_mod(self, module),
            Walk::Skipped => {}
        }
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        let self_type = Self::self_type(&item.self_ty).filter(|_| Walk::outside_tests(&item.attrs) == Walk::Entered);
        let placed: Vec<Placed> = self_type.iter().flat_map(|self_type| self.placed(item, self_type)).collect();
        placed.into_iter().fold(&mut self.shapes, |shapes, placed| {
            placed.recorded_in(shapes);
            shapes
        });
    }
}
