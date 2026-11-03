mod impl_block;
mod referenced;
mod spelled;

use syn::visit::{self, Visit};
use syn::{
    Attribute, Fields, Ident, ImplItem, ItemEnum, ItemImpl, ItemMod, ItemStruct, ItemTrait, Meta, ReturnType,
    Signature, punctuated::Punctuated,
};

use lint_core::{Site, SourceFile, Walk};
use lint_layout::{LineNumber, Name};
use lint_placement::CrateName;

use crate::arity::Arity;
use crate::dependency::Dependency;
use crate::facts::Facts;
use crate::implementation::Implementation;
use crate::method::Method;
use crate::shape::Shape;
use crate::trait_fact::TraitFact;
use crate::type_fact::TypeFact;
use crate::via::Via;
pub(crate) use impl_block::ImplBlock;
use referenced::Referenced;
use spelled::Spelled;

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

    fn site(&self, name: &Ident) -> Site<'file> {
        Site::new(self.file.path(), LineNumber::from(name.span()))
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

    fn method(signature: &Signature) -> Method {
        let returns = match &signature.output {
            ReturnType::Type(_, returned) => Spelled::last_of(returned),
            ReturnType::Default => None,
        };
        Method::new(Name::from(&signature.ident), Arity::new(signature.inputs.len()), returns)
    }

    fn declared(&mut self, attributes: &[Attribute], name: &Ident, shape: Shape, dependencies: Vec<Dependency<'file>>) {
        let found = (Walk::outside_tests(attributes) == Walk::Entered).then(|| {
            TypeFact::declared(self.krate, Name::from(name), self.site(name), Self::derives(attributes))
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
        let shape = match &found.fields {
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => Shape::OneField,
            _ => Shape::Other,
        };
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
        let fact = (Walk::outside_tests(&found.attrs) == Walk::Entered).then(|| {
            TraitFact::new(
                self.krate,
                Name::from(&found.ident),
                self.site(&found.ident),
                Referenced::started(self.file.path(), &found.ident, &found.generics).of_trait(found),
            )
        });
        self.facts.add_trait(fact);
    }

    fn visit_item_impl(&mut self, found: &'ast ItemImpl) {
        let methods: Vec<Method> = found
            .items
            .iter()
            .filter_map(|item| match item {
                ImplItem::Fn(function) => Some(Self::method(&function.sig)),
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
        let block = Spelled::last_of(&found.self_ty)
            .map(|target| ImplBlock::new(self.krate, target, implementation, inherent))
            .filter(|_| walk == Walk::Entered);
        self.facts.add_block(block);
    }
}
