mod impl_block;

use proc_macro2::Span;
use syn::visit::{self, Visit};
use syn::{
    Attribute, Fields, Ident, ImplItem, ItemEnum, ItemImpl, ItemMod, ItemStruct, ItemTrait, Meta, ReturnType,
    Signature, TraitItem, punctuated::Punctuated,
};

use lint_core::{Site, SourceFile, Walk};
use lint_layout::{LineNumber, Name};
use lint_placement::CrateName;

use crate::arity::Arity;
use crate::facts::Facts;
use crate::generic_impl::GenericImpl;
use crate::implementation::Implementation;
use crate::method::Method;
use crate::method_shape::MethodShape;
use crate::shape::Shape;
use crate::spelled::Spelled;
use crate::trait_fact::TraitFact;
use crate::type_fact::TypeFact;
pub(crate) use impl_block::ImplBlock;

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

    fn method(signature: &Signature, selfish: Option<&Name>) -> Method {
        let returns = match &signature.output {
            ReturnType::Type(_, returned) => Spelled::last_of(returned),
            ReturnType::Default => None,
        };
        let shape = MethodShape::of_signature(signature, selfish);
        Method::new(Name::from(&signature.ident), Arity::new(signature.inputs.len()), returns, shape)
    }

    fn declared(&mut self, attributes: &[Attribute], name: &Ident, shape: Shape) {
        let found = (Walk::outside_tests(attributes) == Walk::Entered).then(|| {
            TypeFact::declared(self.krate, Name::from(name), self.site_at(name.span()), Self::derives(attributes))
                .shaped(shape)
        });
        self.facts.add_type(found);
    }
}

impl<'ast> Visit<'ast> for Collector<'_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        Walk::enter(&module.attrs, || visit::visit_item_mod(self, module));
    }

    fn visit_item_struct(&mut self, found: &'ast ItemStruct) {
        let shape = match &found.fields {
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => Shape::OneField,
            _ => Shape::Other,
        };
        self.declared(&found.attrs, &found.ident, shape);
    }

    fn visit_item_enum(&mut self, found: &'ast ItemEnum) {
        self.declared(&found.attrs, &found.ident, Shape::Other);
    }

    fn visit_item_trait(&mut self, found: &'ast ItemTrait) {
        let methods = found
            .items
            .iter()
            .filter_map(|item| match item {
                TraitItem::Fn(function) if Walk::outside_tests(&function.attrs) == Walk::Entered => {
                    Some(Self::method(&function.sig, None))
                }
                _ => None,
            })
            .collect();
        let fact = (Walk::outside_tests(&found.attrs) == Walk::Entered)
            .then(|| TraitFact::new(self.krate, Name::from(&found.ident), self.site_at(found.ident.span()), methods));
        self.facts.add_trait(fact);
    }

    fn visit_item_impl(&mut self, found: &'ast ItemImpl) {
        let target = Spelled::last_of(&found.self_ty);
        let methods: Vec<Method> = found
            .items
            .iter()
            .filter_map(|item| match item {
                ImplItem::Fn(function) => Some(Self::method(&function.sig, target.as_ref())),
                _ => None,
            })
            .collect();
        let implemented = found.trait_.as_ref().and_then(|(_, path, _)| path.segments.last());
        let (implementation, inherent) = match implemented {
            Some(segment) => (
                Some(Implementation::new(Name::from(&segment.ident), Spelled::first_argument_of(segment), methods)),
                Vec::new(),
            ),
            None => (None, methods),
        };
        let walk = Walk::outside_tests(&found.attrs);
        let generic =
            found.generics.type_params().next().and(target.clone()).filter(|_| walk == Walk::Entered).map(|spelled| {
                let site = self.site_at(found.impl_token.span);
                let implemented = implementation.as_ref().map(|each| AsRef::<Name>::as_ref(each).clone());
                let methods = implementation.as_ref().map_or(inherent.as_slice(), Implementation::methods).to_vec();
                GenericImpl::new(self.krate, site, spelled).filled(implemented, methods)
            });
        self.facts.add_generic(generic);
        let block = target
            .map(|target| ImplBlock::new(self.krate, target, implementation, inherent))
            .filter(|_| walk == Walk::Entered);
        self.facts.add_block(block);
    }
}
