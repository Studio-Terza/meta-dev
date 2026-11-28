use std::ptr;

use syn::spanned::Spanned;
use syn::{
    Field, FnArg, GenericArgument, ItemEnum, ItemImpl, ItemStruct, ItemType, Macro, PathArguments, ReturnType,
    Signature, Type, TypePath, TypeTuple,
};

use crate::finding::Found;
use crate::gate::Gate;
use crate::hooks::{Descent, Expanded, Hooks};
use crate::lint::style;

#[derive(Debug, Default)]
pub(crate) struct TypeShape {
    found: Vec<Found>,
    roots: Vec<*const Type>,
    marks: Vec<usize>,
    open: Option<Root>,
}

#[derive(Clone, Copy, Debug)]
struct Root {
    at: *const Type,
    line: usize,
}

#[derive(Clone, Copy, Debug)]
enum Shape {
    Tuple,
    StringError,
}

impl Shape {
    fn problem(self) -> &'static str {
        match self {
            Shape::Tuple => "tuple in a signature or field",
            Shape::StringError => "Result with a String error",
        }
    }

    fn fix(self) -> &'static str {
        match self {
            Shape::Tuple => "values that travel together are a struct",
            Shape::StringError => "errors are an enum with thiserror",
        }
    }
}

impl TypeShape {
    const BOOL_FIX: &'static str = "a stored flag is a stored mode, make it an enum or a type parameter";

    fn rooted<'ty>(&mut self, roots: impl Iterator<Item = &'ty Type>) -> Descent {
        self.marks.push(self.roots.len());
        self.roots.extend(roots.map(ptr::from_ref));
        Descent::Into
    }

    fn unrooted(&mut self) {
        let mark = self.marks.pop().unwrap_or_default();
        self.roots.truncate(mark);
    }

    fn shaped(&mut self, seen: Option<Shape>) {
        let found = self.open.zip(seen).map(|(root, shape)| Found::at(root.line, shape.problem(), shape.fix()));
        self.found.extend(found);
    }

    fn last_named(ty: &Type, name: &str) -> Option<()> {
        match ty {
            Type::Path(TypePath { path, .. }) => {
                path.segments.last().filter(|segment| segment.ident == name).map(|_| ())
            }
            _ => None,
        }
    }
}

impl Hooks for TypeShape {
    fn item_impl(&mut self, item: &ItemImpl) -> Descent {
        Descent::from(Gate::inherent(item).entered())
    }

    fn macro_called(&mut self, _: &Macro) -> Expanded {
        self.open.map_or(Expanded::Statements, |_| Expanded::Nothing)
    }

    fn item_struct(&mut self, item: &ItemStruct) -> Descent {
        self.rooted(item.fields.iter().map(|field| &field.ty))
    }

    fn left_item_struct(&mut self, _: &ItemStruct) {
        self.unrooted();
    }

    fn item_enum(&mut self, item: &ItemEnum) -> Descent {
        self.rooted(item.variants.iter().flat_map(|variant| variant.fields.iter()).map(|field| &field.ty))
    }

    fn left_item_enum(&mut self, _: &ItemEnum) {
        self.unrooted();
    }

    fn item_type(&mut self, item: &ItemType) -> Descent {
        self.rooted([&*item.ty].into_iter())
    }

    fn left_item_type(&mut self, _: &ItemType) {
        self.unrooted();
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

    fn left_signature(&mut self, _: &Signature) {
        self.unrooted();
    }

    fn field(&mut self, field: &Field) -> Descent {
        let rooted = self.roots.contains(&ptr::from_ref(&field.ty));
        let flag = Some(()).filter(|()| rooted).and_then(|()| TypeShape::last_named(&field.ty, "bool"));
        let span = field.ident.as_ref().map_or_else(|| field.ty.span(), syn::Ident::span);
        self.found.extend(flag.map(|()| Found::spanned(span, "bool field", TypeShape::BOOL_FIX)));
        Descent::Into
    }

    fn ty(&mut self, ty: &Type) -> Descent {
        let at = ptr::from_ref(ty);
        let opened = self.open.xor(Some(Root { at, line: ty.span().start().line }));
        self.open = self.open.or(opened.filter(|_| self.roots.contains(&at)));
        Descent::Into
    }

    fn left_ty(&mut self, ty: &Type) {
        let closing = self.open.filter(|root| ptr::eq(root.at, ty));
        self.open = self.open.xor(closing);
    }

    fn type_tuple(&mut self, tuple: &TypeTuple) -> Descent {
        self.shaped(tuple.elems.first().map(|_| Shape::Tuple));
        Descent::Into
    }

    fn type_path(&mut self, path: &TypePath) -> Descent {
        let result = path.path.segments.last().filter(|segment| segment.ident == "Result");
        let error = result
            .and_then(|segment| match &segment.arguments {
                PathArguments::AngleBracketed(arguments) => arguments.args.iter().nth(1),
                PathArguments::None | PathArguments::Parenthesized(_) => None,
            })
            .and_then(|argument| match argument {
                GenericArgument::Type(error) => TypeShape::last_named(error, "String"),
                _ => None,
            });
        self.shaped(error.map(|()| Shape::StringError));
        Descent::Into
    }
}

style!(TypeShape, Ungated, "types carry their meaning: no bool fields, no tuples in signatures, no String errors");
