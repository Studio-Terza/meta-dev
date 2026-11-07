mod described;
mod impl_block;
mod referenced;

use proc_macro2::Span;
use syn::visit::{self, Visit};
use syn::{Attribute, Ident, ItemEnum, ItemImpl, ItemMod, ItemStruct, ItemTrait, Meta, punctuated::Punctuated};

use lint_core::{Site, SourceFile, Walk};
use lint_layout::{LineNumber, Name};
use lint_placement::CrateName;

use crate::dependency::Dependency;
use crate::facts::Facts;
use crate::generic_impl::GenericImpl;
use crate::implementation::Implementation;
use crate::method::Method;
use crate::shape::Shape;
use crate::spelled::Spelled;
use crate::trait_fact::TraitFact;
use crate::type_fact::TypeFact;
use crate::via::Via;
use described::Described;
pub(crate) use impl_block::ImplBlock;
use referenced::Referenced;

pub(crate) struct Collector<'file> {
    file: &'file SourceFile,
    krate: &'file CrateName,
    facts: Facts<'file>,
}

impl<'file> Collector<'file> {
    pub(crate) fn collected(facts: Facts<'file>, file: &'file SourceFile) -> Facts<'file> {
        let mut scan = Collector { file, krate: file.placement().package().crate_name(), facts };
        scan.visit_file(file.syntax());
        scan.facts
    }

    fn site_at(&self, span: Span) -> Site<'file> {
        Site::new(self.file.path(), LineNumber::from(span))
    }

    fn derives(attributes: &[Attribute]) -> Vec<Name> {
        attributes
            .iter()
            .filter(|attribute| attribute.path().is_ident("derive"))
            .filter_map(|attribute| {
                attribute.parse_args_with(Punctuated::<Meta, syn::Token![,]>::parse_terminated).ok()
            })
            .flatten()
            .filter_map(|meta| meta.path().segments.last().map(|segment| Name::from(&segment.ident)))
            .collect()
    }

    fn declared(&mut self, attributes: &[Attribute], name: &Ident, shape: Shape, dependencies: Vec<Dependency<'file>>) {
        let found = (Walk::outside_tests(attributes) == Walk::Entered).then(|| {
            TypeFact::declared(self.krate, Name::from(name), self.site_at(name.span()), Self::derives(attributes))
                .shaped(shape)
                .depending_on(dependencies)
        });
        self.facts.add_type(found);
    }
}

impl<'ast> Visit<'ast> for Collector<'_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        Walk::enter(&module.attrs, || visit::visit_item_mod(self, module));
    }

    fn visit_item_struct(&mut self, found: &'ast ItemStruct) {
        let shape = Shape::from(&found.fields);
        let dependencies = Referenced::started(self.file.path(), &found.ident, &found.generics).of_fields(
            Via::Field,
            found.fields.iter(),
            &found.generics,
        );
        self.declared(&found.attrs, &found.ident, shape, dependencies);
    }

    fn visit_item_enum(&mut self, found: &'ast ItemEnum) {
        let fields = found.variants.iter().flat_map(|variant| variant.fields.iter());
        let dependencies = Referenced::started(self.file.path(), &found.ident, &found.generics).of_fields(
            Via::Variant,
            fields,
            &found.generics,
        );
        self.declared(&found.attrs, &found.ident, Shape::Other, dependencies);
    }

    fn visit_item_trait(&mut self, found: &'ast ItemTrait) {
        let methods = found.items.iter().filter_map(Method::of_trait_item).collect();
        let fact = (Walk::outside_tests(&found.attrs) == Walk::Entered).then(|| {
            TraitFact::new(
                self.krate,
                Name::from(&found.ident),
                self.site_at(found.ident.span()),
                Referenced::started(self.file.path(), &found.ident, &found.generics).of_trait(found),
            )
            .with_methods(methods)
        });
        self.facts.add_trait(fact);
    }

    fn visit_item_impl(&mut self, found: &'ast ItemImpl) {
        let target = Described::name(&found.self_ty);
        let methods = found.items.iter().filter_map(|item| Method::of_impl_item(item, target.as_ref()));
        let trait_path = found.trait_.as_ref().map(|(_, path, _)| path);
        let implemented = trait_path.and_then(|path| path.segments.last());
        let site = self.site_at(found.impl_token.span);
        let (implemented_methods, inherent): (Vec<Method>, Vec<Method>) = methods.partition(|_| implemented.is_some());
        let implementation = implemented.zip(trait_path).map(|(segment, path)| {
            Implementation::new(
                Name::from(&segment.ident),
                Spelled::first_argument_of(segment),
                implemented_methods,
                site,
            )
            .qualified(Spelled::qualifier(path), Spelled::qualifier_of(&found.self_ty))
        });
        let walk = Walk::outside_tests(&found.attrs);
        let generic =
            found.generics.type_params().next().and(target.clone()).filter(|_| walk == Walk::Entered).map(|spelled| {
                let trait_name = implementation.as_ref().map(|each| AsRef::<Name>::as_ref(each).clone());
                let carried = implementation.as_ref().map_or(inherent.as_slice(), Implementation::methods).to_vec();
                GenericImpl::new(self.krate, site, spelled).filled(trait_name, carried)
            });
        self.facts.add_generic(generic);
        let block = target
            .map(|named| ImplBlock::new(self.krate, named, implementation, inherent))
            .filter(|_| walk == Walk::Entered);
        self.facts.add_block(block);
    }
}
