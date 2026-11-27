use std::fmt;

use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{
    Field, FnArg, GenericArgument, ItemEnum, ItemImpl, ItemMod, ItemStruct, ItemType, Macro, PathArguments, ReturnType,
    Signature, Type, TypePath, TypeTuple,
};

use crate::expansion::Expansion;
use crate::finding::{Fix, Found, Rule};
use crate::gate::Gate;
use crate::source::Source;
use crate::style::{Reads, Style};
use crate::word::Word;

#[derive(Debug, Default)]
pub(crate) struct TypeShape {
    found: Vec<Found>,
}

#[derive(Clone, Copy, Debug)]
enum Shape {
    Tuple,
    StringError,
}

#[derive(Debug, Default)]
struct Shapes(Vec<Shape>);

impl fmt::Display for Shape {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Shape::Tuple => "tuple in a signature or field",
            Shape::StringError => "Result with a String error",
        })
    }
}

impl Shape {
    fn fix(self) -> Fix {
        match self {
            Shape::Tuple => Fix("values that travel together are a struct"),
            Shape::StringError => Fix("errors are an enum with thiserror"),
        }
    }
}

impl TypeShape {
    const BOOL_FIX: Fix = Fix("a stored flag is a stored mode, make it an enum or a type parameter");
    const BOOL: Word = Word("bool");
    const STRING: Word = Word("String");

    fn rooted(&mut self, ty: &Type) {
        let mut shapes = Shapes::default();
        shapes.visit_type(ty);
        let span = ty.span();
        self.found.extend(shapes.0.into_iter().map(|shape| Found::spanned(span, shape, shape.fix())));
    }

    fn fields<'field>(&mut self, fields: impl Iterator<Item = &'field Field>) {
        fields.for_each(|field| {
            let flag = TypeShape::BOOL.last(&field.ty);
            let span = field.ident.as_ref().map_or_else(|| field.ty.span(), syn::Ident::span);
            self.found.extend(flag.map(|_| Found::spanned(span, "bool field", TypeShape::BOOL_FIX)));
            self.rooted(&field.ty);
        });
    }
}

impl<'ast> Visit<'ast> for TypeShape {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        Gate::of(&module.attrs).entered().map_or_default(|()| visit::visit_item_mod(self, module));
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        Gate::inherent(item).entered().map_or_default(|()| visit::visit_item_impl(self, item));
    }

    fn visit_macro(&mut self, invocation: &'ast Macro) {
        Expansion::of(invocation).walk(self);
    }

    fn visit_item_struct(&mut self, item: &'ast ItemStruct) {
        self.fields(item.fields.iter());
    }

    fn visit_item_enum(&mut self, item: &'ast ItemEnum) {
        self.fields(item.variants.iter().flat_map(|variant| variant.fields.iter()));
    }

    fn visit_item_type(&mut self, item: &'ast ItemType) {
        self.rooted(&item.ty);
    }

    fn visit_signature(&mut self, signature: &'ast Signature) {
        let inputs = signature.inputs.iter().filter_map(|input| match input {
            FnArg::Typed(typed) => Some(&*typed.ty),
            FnArg::Receiver(_) => None,
        });
        let output = match &signature.output {
            ReturnType::Type(_, returned) => Some(&**returned),
            ReturnType::Default => None,
        };
        inputs.chain(output).for_each(|ty| self.rooted(ty));
    }
}

impl<'ast> Visit<'ast> for Shapes {
    fn visit_type_tuple(&mut self, tuple: &'ast TypeTuple) {
        self.0.extend(tuple.elems.first().map(|_| Shape::Tuple));
        visit::visit_type_tuple(self, tuple);
    }

    fn visit_type_path(&mut self, path: &'ast TypePath) {
        let result = path.path.segments.last().filter(|segment| segment.ident == "Result");
        let error = result
            .and_then(|segment| match &segment.arguments {
                PathArguments::AngleBracketed(arguments) => arguments.args.iter().nth(1),
                PathArguments::None | PathArguments::Parenthesized(_) => None,
            })
            .and_then(|argument| match argument {
                GenericArgument::Type(error) => TypeShape::STRING.last(error),
                _ => None,
            });
        self.0.extend(error.map(|_| Shape::StringError));
        visit::visit_type_path(self, path);
    }
}

impl<'file> Style<'file> for TypeShape {
    const RULE: Rule = Rule::TypeShape;
    const READS: Reads = Reads::Every;

    fn new(_: &'file Source) -> TypeShape {
        TypeShape::default()
    }

    fn found(self) -> Vec<Found> {
        self.found
    }
}
