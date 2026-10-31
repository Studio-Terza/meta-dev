mod method_shape;
mod repeated_shape;

use std::collections::BTreeMap;
use std::path::PathBuf;

use syn::visit::Visit;
use syn::{ImplItem, ImplItemFn, ItemImpl, ItemMod, Type, Visibility};

use lint_core::{LineNumber, Most, Name, Site, SourceFile, Violation, Walk};

use method_shape::MethodShape;
use repeated_shape::RepeatedShape;

#[derive(Debug, Default)]
pub struct ShapeIndex {
    shapes: BTreeMap<MethodShape, BTreeMap<Name, Site>>,
    file: PathBuf,
}

impl ShapeIndex {
    #[must_use]
    pub fn of_files(files: &[&SourceFile]) -> ShapeIndex {
        files.iter().fold(ShapeIndex::default(), |index, file| index.reading(file))
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
        let mut index = ShapeIndex { file: file.path().to_path_buf(), ..self };
        index.visit_file(file.syntax());
        index
    }

    fn self_type(self_type: &Type) -> Option<Name> {
        match self_type {
            Type::Path(path) => path.path.segments.last().map(|segment| Name::from(&segment.ident)),
            _ => None,
        }
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
        let self_type = Self::self_type(&item.self_ty)
            .filter(|_| Walk::inherent_impls(item) == Walk::Entered)
            .filter(|_| Walk::outside_tests(&item.attrs) == Walk::Entered);
        let public: Vec<&ImplItemFn> = item
            .items
            .iter()
            .filter_map(|member| match member {
                ImplItem::Fn(function) if !matches!(function.vis, Visibility::Inherited) => Some(function),
                _ => None,
            })
            .collect();
        let file = &self.file;
        self_type.iter().flat_map(|self_type| public.iter().map(move |function| (self_type, function))).fold(
            &mut self.shapes,
            |shapes, (self_type, function)| {
                let site = Site::new(file, LineNumber::from(function.sig.ident.span()));
                shapes
                    .entry(MethodShape::described(&function.sig, self_type))
                    .or_default()
                    .entry(self_type.clone())
                    .or_insert(site);
                shapes
            },
        );
    }
}
