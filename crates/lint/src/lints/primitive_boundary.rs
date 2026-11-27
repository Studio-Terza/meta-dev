use std::fmt;

use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{
    ImplItemType, ItemEnum, ItemImpl, ItemMod, ItemStruct, ItemType, ItemUnion, Macro, Signature, TraitItemType,
    TypePath,
};

use crate::finding::{Found, Rule};
use crate::source::Source;
use crate::style::{Reads, Style};
use crate::walk::{Descent, Expanded, Hooks};

#[derive(Debug)]
pub(crate) struct PrimitiveBoundary {
    found: Vec<Found>,
    bounded: Option<()>,
    modules: Vec<Option<()>>,
}

#[derive(Clone, Copy, Debug)]
enum Holder {
    Signature,
    Field,
    Variant,
    Parameter,
    Alias,
    Implemented,
    Associated,
}

#[derive(Debug, Default)]
struct Primitives(Vec<TypePath>);

#[derive(Debug)]
struct Named<'path>(&'path TypePath);

impl PrimitiveBoundary {
    const BOUNDARY: &'static str = "boundary";
    const FIX: &'static str =
        "take and return a newtype or a type that carries meaning, or move the code into a module named boundary";

    fn inside(&self) -> Option<()> {
        self.bounded.or(self.modules.iter().flatten().next().copied())
    }

    fn read(&mut self, holder: Holder, walk: impl FnOnce(&mut Primitives)) {
        let mut primitives = Primitives::default();
        walk(&mut primitives);
        let unbounded = self.inside().xor(Some(())).map_or_default(|()| primitives.0);
        self.found.extend(unbounded.iter().map(|primitive| {
            Found::spanned(primitive.span(), format_args!("{} in {holder}", Named(primitive)), PrimitiveBoundary::FIX)
        }));
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
            Holder::Implemented => "an implemented trait or type",
            Holder::Associated => "an associated type",
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

impl Hooks for PrimitiveBoundary {
    fn macro_called(&mut self, _: &Macro) -> Expanded {
        Expanded::Statements
    }

    fn item_mod(&mut self, module: &ItemMod) -> Descent {
        self.modules.push((module.ident == PrimitiveBoundary::BOUNDARY).then_some(()));
        Descent::Into
    }

    fn left_item_mod(&mut self, _: &ItemMod) {
        self.modules.pop();
    }

    fn item_impl(&mut self, item: &ItemImpl) -> Descent {
        self.read(Holder::Implemented, |primitives| {
            primitives.visit_generics(&item.generics);
            primitives.visit_type(&item.self_ty);
            item.trait_.iter().for_each(|(_, path, _)| primitives.visit_path(path));
        });
        Descent::Into
    }

    fn impl_item_type(&mut self, item: &ImplItemType) -> Descent {
        self.read(Holder::Associated, |primitives| primitives.visit_type(&item.ty));
        Descent::Over
    }

    fn trait_item_type(&mut self, item: &TraitItemType) -> Descent {
        self.read(Holder::Associated, |primitives| {
            item.default.iter().for_each(|(_, ty)| primitives.visit_type(ty));
            item.bounds.iter().for_each(|bound| primitives.visit_type_param_bound(bound));
        });
        Descent::Over
    }

    fn signature(&mut self, signature: &Signature) -> Descent {
        self.read(Holder::Signature, |primitives| primitives.visit_signature(signature));
        Descent::Over
    }

    fn item_struct(&mut self, item: &ItemStruct) -> Descent {
        self.read(Holder::Field, |primitives| primitives.visit_fields(&item.fields));
        self.read(Holder::Parameter, |primitives| primitives.visit_generics(&item.generics));
        Descent::Over
    }

    fn item_enum(&mut self, item: &ItemEnum) -> Descent {
        self.read(Holder::Variant, |primitives| {
            item.variants.iter().for_each(|variant| primitives.visit_fields(&variant.fields));
        });
        self.read(Holder::Parameter, |primitives| primitives.visit_generics(&item.generics));
        Descent::Over
    }

    fn item_union(&mut self, item: &ItemUnion) -> Descent {
        self.read(Holder::Field, |primitives| primitives.visit_fields_named(&item.fields));
        Descent::Over
    }

    fn item_type(&mut self, item: &ItemType) -> Descent {
        self.read(Holder::Alias, |primitives| primitives.visit_type(&item.ty));
        Descent::Over
    }
}

impl<'file> Style<'file> for PrimitiveBoundary {
    const RULE: Rule = Rule::PrimitiveBoundary;
    const READS: Reads = Reads::Every;

    fn new(source: &'file Source) -> PrimitiveBoundary {
        PrimitiveBoundary { found: Vec::new(), bounded: source.bounded(), modules: Vec::new() }
    }

    fn found(self) -> Vec<Found> {
        self.found
    }
}
