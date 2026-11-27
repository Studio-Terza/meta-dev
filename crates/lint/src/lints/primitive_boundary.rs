use std::fmt;

use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Fields, ItemEnum, ItemImpl, ItemMod, ItemStruct, ItemType, ItemUnion, Macro, Signature, TypePath};

use crate::expansion::Expansion;
use crate::finding::{Fix, Found, Rule};
use crate::gate::Gate;
use crate::source::Source;
use crate::style::{Reads, Style};

#[derive(Debug)]
pub(crate) struct PrimitiveBoundary {
    found: Vec<Found>,
    bounded: Option<()>,
}

#[derive(Clone, Copy, Debug)]
enum Holder {
    Signature,
    Field,
    Variant,
    Parameter,
    Alias,
}

#[derive(Debug, Default)]
struct Primitives(Vec<TypePath>);

#[derive(Debug)]
struct Named<'path>(&'path TypePath);

impl PrimitiveBoundary {
    const BOUNDARY: &'static str = "boundary";
    const FIX: Fix =
        Fix("take and return a newtype or a type that carries meaning, or move the code into a module named boundary");

    fn read(&mut self, holder: Holder, walk: impl FnOnce(&mut Primitives)) {
        let mut primitives = Primitives::default();
        walk(&mut primitives);
        let unbounded = self.bounded.xor(Some(())).map_or_default(|()| primitives.0);
        self.found.extend(unbounded.iter().map(|primitive| {
            Found::spanned(primitive.span(), format_args!("{} in {holder}", Named(primitive)), PrimitiveBoundary::FIX)
        }));
    }

    fn wrapped(item: &ItemStruct) -> Option<()> {
        match &item.fields {
            Fields::Unnamed(fields) => (fields.unnamed.len() == 1).then_some(()),
            Fields::Named(_) | Fields::Unit => None,
        }
    }
}

impl fmt::Display for Holder {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Holder::Signature => "a signature",
            Holder::Field => "a field",
            Holder::Variant => "a variant",
            Holder::Parameter => "a type parameter",
            Holder::Alias => "a type alias",
        })
    }
}

impl fmt::Display for Named<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.path.segments.last().map_or(Ok(()), |segment| write!(formatter, "{}", segment.ident))
    }
}

impl Primitives {
    const NAMES: [&'static str; 18] = [
        "str", "String", "bool", "char", "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128",
        "isize", "f32", "f64",
    ];

    fn primitive(path: &TypePath) -> Option<()> {
        let last = path.path.segments.last().filter(|segment| segment.arguments.is_none())?;
        let named = Primitives::NAMES.iter().any(|name| last.ident == name);
        (path.qself.is_none() && named).then_some(())
    }
}

impl<'ast> Visit<'ast> for Primitives {
    fn visit_type_path(&mut self, path: &'ast TypePath) {
        self.0.extend(Primitives::primitive(path).map(|()| path.clone()));
        visit::visit_type_path(self, path);
    }
}

impl<'ast> Visit<'ast> for PrimitiveBoundary {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        let outer = self.bounded;
        self.bounded = outer.or((module.ident == PrimitiveBoundary::BOUNDARY).then_some(()));
        visit::visit_item_mod(self, module);
        self.bounded = outer;
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        Gate::inherent(item).entered().map_or_default(|()| visit::visit_item_impl(self, item));
    }

    fn visit_macro(&mut self, invocation: &'ast Macro) {
        Expansion::of(invocation).walk(self);
    }

    fn visit_signature(&mut self, signature: &'ast Signature) {
        self.read(Holder::Signature, |primitives| primitives.visit_signature(signature));
    }

    fn visit_item_struct(&mut self, item: &'ast ItemStruct) {
        let fields = PrimitiveBoundary::wrapped(item).xor(Some(())).map(|()| &item.fields);
        self.read(Holder::Field, |primitives| fields.into_iter().for_each(|read| primitives.visit_fields(read)));
        self.read(Holder::Parameter, |primitives| primitives.visit_generics(&item.generics));
    }

    fn visit_item_enum(&mut self, item: &'ast ItemEnum) {
        self.read(Holder::Variant, |primitives| {
            item.variants.iter().for_each(|variant| primitives.visit_fields(&variant.fields));
        });
        self.read(Holder::Parameter, |primitives| primitives.visit_generics(&item.generics));
    }

    fn visit_item_union(&mut self, item: &'ast ItemUnion) {
        self.read(Holder::Field, |primitives| primitives.visit_fields_named(&item.fields));
    }

    fn visit_item_type(&mut self, item: &'ast ItemType) {
        self.read(Holder::Alias, |primitives| primitives.visit_type(&item.ty));
    }
}

impl<'file> Style<'file> for PrimitiveBoundary {
    const RULE: Rule = Rule::PrimitiveBoundary;
    const READS: Reads = Reads::Shipped;

    fn new(source: &'file Source) -> PrimitiveBoundary {
        PrimitiveBoundary { found: Vec::new(), bounded: source.bounded() }
    }

    fn found(self) -> Vec<Found> {
        self.found
    }
}
