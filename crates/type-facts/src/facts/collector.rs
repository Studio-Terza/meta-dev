mod impl_block;
mod spelled;

use syn::visit::{self, Visit};
use syn::{
    Attribute, Ident, ImplItem, ItemEnum, ItemImpl, ItemMod, ItemStruct, ItemTrait, Meta, ReturnType, Signature,
    punctuated::Punctuated,
};

use lint_core::{Site, SourceFile, Walk};
use lint_layout::{LineNumber, Name};
use lint_placement::CrateName;

use crate::arity::Arity;
use crate::implementation::Implementation;
use crate::method::Method;
use crate::trait_fact::TraitFact;
use crate::type_fact::TypeFact;
use impl_block::ImplBlock;
use spelled::Spelled;

pub(crate) struct Collector<'file> {
    file: &'file SourceFile,
    krate: CrateName,
    types: Vec<TypeFact>,
    traits: Vec<TraitFact>,
    blocks: Vec<ImplBlock>,
}

impl<'file> Collector<'file> {
    pub(crate) fn of_file(file: &'file SourceFile) -> Collector<'file> {
        let mut scan = Collector {
            file,
            krate: CrateName::from(file.placement().package().root()),
            types: Vec::new(),
            traits: Vec::new(),
            blocks: Vec::new(),
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

    fn declared(&mut self, attributes: &[Attribute], name: &Ident) {
        let found =
            TypeFact::declared(self.krate.clone(), Name::from(name), self.site(name), Self::derives(attributes));
        self.types.extend((Walk::outside_tests(attributes) == Walk::Entered).then_some(found));
    }
}

impl<'ast> Visit<'ast> for Collector<'_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        Walk::enter(&module.attrs, || visit::visit_item_mod(self, module));
    }

    fn visit_item_struct(&mut self, found: &'ast ItemStruct) {
        self.declared(&found.attrs, &found.ident);
    }

    fn visit_item_enum(&mut self, found: &'ast ItemEnum) {
        self.declared(&found.attrs, &found.ident);
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
        let block = Spelled::last_of(&found.self_ty)
            .map(|target| ImplBlock::new(self.krate.clone(), target, implementation, inherent));
        self.blocks.extend(block.filter(|_| Walk::outside_tests(&found.attrs) == Walk::Entered));
    }
}
