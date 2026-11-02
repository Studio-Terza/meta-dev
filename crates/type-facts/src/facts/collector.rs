mod impl_block;
mod signature_reading;
mod spelled;

use syn::visit::{self, Visit};
use syn::{
    Attribute, Fields, Generics, Ident, ImplItem, ItemEnum, ItemFn, ItemImpl, ItemMod, ItemStruct, ItemTrait, Meta,
    ReturnType, Signature, punctuated::Punctuated,
};

use lint_core::{Site, SourceFile, Walk};
use lint_layout::{LineNumber, Name};
use lint_placement::CrateName;

use crate::arity::Arity;
use crate::implementation::Implementation;
use crate::method::Method;
use crate::shape::Shape;
use crate::trait_fact::TraitFact;
use crate::transition::Transition;
use crate::type_fact::TypeFact;
use impl_block::ImplBlock;
use signature_reading::SignatureReading;
use spelled::Spelled;

pub(crate) struct Collector<'file> {
    file: &'file SourceFile,
    krate: CrateName,
    types: Vec<TypeFact>,
    traits: Vec<TraitFact>,
    blocks: Vec<ImplBlock>,
    transitions: Vec<Transition>,
}

impl<'file> Collector<'file> {
    pub(crate) fn of_file(file: &'file SourceFile) -> Collector<'file> {
        let mut scan = Collector {
            file,
            krate: CrateName::from(file.placement().package().root()),
            types: Vec::new(),
            traits: Vec::new(),
            blocks: Vec::new(),
            transitions: Vec::new(),
        };
        scan.visit_file(file.syntax());
        scan
    }

    pub(super) fn types(&self) -> &[TypeFact] {
        &self.types
    }

    pub(super) fn traits(&self) -> &[TraitFact] {
        &self.traits
    }

    pub(super) fn blocks(&self) -> &[ImplBlock] {
        &self.blocks
    }

    pub(super) fn transitions(&self) -> &[Transition] {
        &self.transitions
    }

    fn site(&self, name: &Ident) -> Site {
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

    fn declared(&mut self, attributes: &[Attribute], name: &Ident, shape: Shape) {
        let found =
            TypeFact::declared(self.krate.clone(), Name::from(name), self.site(name), Self::derives(attributes))
                .shaped(shape);
        self.types.extend((Walk::outside_tests(attributes) == Walk::Entered).then_some(found));
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
        let fact = TraitFact::new(self.krate.clone(), Name::from(&found.ident), self.site(&found.ident));
        self.traits.extend((Walk::outside_tests(&found.attrs) == Walk::Entered).then_some(fact));
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
        let implementation = found.trait_.as_ref().and_then(|(_, path, _)| {
            path.segments.last().map(|segment| {
                Implementation::new(Name::from(&segment.ident), Spelled::first_argument_of(segment), methods.clone())
            })
        });
        let inherent = implementation.as_ref().map_or(methods, |_| Vec::new());
        let target = Spelled::last_of(&found.self_ty);
        let walk = Walk::outside_tests(&found.attrs);
        let reading = SignatureReading::within(target.as_ref(), &found.generics);
        let transitions: Vec<Transition> = found
            .items
            .iter()
            .filter_map(|item| match item {
                ImplItem::Fn(function) if Walk::outside_tests(&function.attrs) == Walk::Entered => {
                    Some(reading.transition(self.krate.clone(), self.site(&function.sig.ident), &function.sig))
                }
                _ => None,
            })
            .collect();
        self.transitions.extend(transitions.into_iter().filter(|_| walk == Walk::Entered));
        let block = target.map(|target| ImplBlock::new(self.krate.clone(), target, implementation, inherent));
        self.blocks.extend(block.filter(|_| walk == Walk::Entered));
    }

    fn visit_item_fn(&mut self, function: &'ast ItemFn) {
        let found = SignatureReading::within(None, &Generics::default()).transition(
            self.krate.clone(),
            self.site(&function.sig.ident),
            &function.sig,
        );
        self.transitions.extend((Walk::outside_tests(&function.attrs) == Walk::Entered).then_some(found));
    }
}
