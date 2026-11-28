use std::fmt;

use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{
    Field, FnArg, GenericArgument, ItemEnum, ItemImpl, ItemMod, ItemStruct, ItemType, Macro, PathArguments, ReturnType,
    Signature, Type, TypePath, TypeTuple,
};

use crate::finding::{Found, Rule};
use crate::gate::Gate;
use crate::source::Source;
use crate::style::{Reads, Style};
use crate::terms::{Fix, Line, Word};
use crate::walk::{Descent, Expanded, Hooks};

#[derive(Debug, Default)]
pub(crate) struct TypeShape {
    found: Vec<Found>,
}

#[derive(Debug, Default)]
struct Shapes(Vec<Shape>);

#[derive(Clone, Copy, Debug)]
enum Shape {
    Tuple,
    StringError,
}

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

    fn rooted<'ty>(&mut self, roots: impl Iterator<Item = &'ty Type>) -> Descent {
        roots.for_each(|root| {
            let mut shapes = Shapes::default();
            shapes.visit_type(root);
            let line = Line::of(root.span());
            self.found.extend(shapes.0.into_iter().map(|shape| Found::at(line, shape, shape.fix())));
        });
        Descent::Into
    }

    fn fielded<'field>(&mut self, given: impl Iterator<Item = &'field Field>) -> Descent {
        let fields: Vec<&Field> = given.collect();
        let flags =
            fields.iter().filter(|field| TypeShape::last_named(&field.ty, Word("bool")).is_some()).map(|field| {
                let span = field.ident.as_ref().map_or_else(|| field.ty.span(), syn::Ident::span);
                Found::spanned(span, "bool field", TypeShape::BOOL_FIX)
            });
        self.found.extend(flags);
        self.rooted(fields.into_iter().map(|field| &field.ty))
    }

    fn last_named(ty: &Type, name: Word) -> Option<()> {
        match ty {
            Type::Path(TypePath { path, .. }) => {
                path.segments.last().filter(|segment| segment.ident == name).map(|_| ())
            }
            _ => None,
        }
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
                GenericArgument::Type(error) => TypeShape::last_named(error, Word("String")),
                _ => None,
            });
        self.0.extend(error.map(|()| Shape::StringError));
        visit::visit_type_path(self, path);
    }
}

impl Hooks for TypeShape {
    fn item_mod(&mut self, module: &ItemMod) -> Descent {
        Descent::from(Gate::of(&module.attrs).entered())
    }

    fn item_impl(&mut self, item: &ItemImpl) -> Descent {
        Descent::from(Gate::inherent(item).entered())
    }

    fn macro_called(&mut self, _: &Macro) -> Expanded {
        Expanded::Statements
    }

    fn item_struct(&mut self, item: &ItemStruct) -> Descent {
        self.fielded(item.fields.iter())
    }

    fn item_enum(&mut self, item: &ItemEnum) -> Descent {
        self.fielded(item.variants.iter().flat_map(|variant| variant.fields.iter()))
    }

    fn item_type(&mut self, item: &ItemType) -> Descent {
        self.rooted([&*item.ty].into_iter())
    }

    fn signature(&mut self, signature: &Signature) -> Descent {
        let inputs = signature.inputs.iter().filter_map(|input| match input {
            FnArg::Typed(typed) => Some(&*typed.ty),
            FnArg::Receiver(_) => None,
        });
        let output = match &signature.output {
            ReturnType::Type(_, returned) => Some(&**returned),
            ReturnType::Default => None,
        };
        self.rooted(inputs.chain(output))
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
