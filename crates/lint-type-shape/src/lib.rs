use lint_core::Headline;
use lint_file::{Reads, SweptLint};
use lint_sweep::{Descend, Finds, Kind, Sweep, Watch};
use proc_macro2::Span;
use std::path::Path;
use syn::spanned::Spanned;

use syn::{
    Field, Fields, GenericArgument, Ident, ItemEnum, ItemImpl, ItemMod, ItemStruct, ItemType, Macro, Signature, Type,
    TypePath, TypeTuple,
};

use lint_core::Fix;
use lint_core::Message;
use lint_core::Problem;
use lint_core::Site;
use lint_core::SourceFile;
use lint_core::Syntax;
use lint_core::Violation;
use lint_core::Walk;
use lint_layout::LineNumber;

#[derive(Debug)]
pub struct TypeShape;

impl TypeShape {
    fn findings(ty: &Type) -> Vec<Message> {
        let mut findings = TypeFindings::default();
        Sweep::through_type(ty, &mut findings);
        findings.0
    }

    fn string_error(path: &TypePath) -> Option<Message> {
        path.path
            .segments
            .last()
            .filter(|segment| segment.ident == "Result")
            .and_then(|segment| Syntax::angled(&segment.arguments))
            .and_then(|arguments| arguments.args.iter().nth(1))
            .filter(|argument| {
                matches!(
                    argument,
                    GenericArgument::Type(Type::Path(error))
                        if error.path.segments.last().is_some_and(|segment| segment.ident == "String")
                )
            })
            .map(|_| {
                Message::new(Problem::new("Result with a String error"), Fix::new("errors are an enum with thiserror"))
            })
    }
}

impl SweptLint for TypeShape {
    const HEADLINE: Headline =
        Headline::new("types carry their meaning: no bool fields, no tuples in signatures, no String errors");
    const READS: Reads = Reads::Every;

    fn watcher(file: &SourceFile) -> Box<dyn Finds<'_> + '_> {
        Box::new(TypeShapes { path: file.head().path(), violations: Vec::new() })
    }
}

struct TypeShapes<'scan> {
    path: &'scan Path,
    violations: Vec<Violation<'scan>>,
}

impl<'scan> TypeShapes<'scan> {
    fn violation(&self, span: Span, message: Message) -> Violation<'scan> {
        Violation::new(Site::new(self.path, LineNumber::from(span)), message)
    }

    fn check_type(&mut self, ty: &Type) {
        let violations: Vec<Violation<'scan>> =
            TypeShape::findings(ty).into_iter().map(|finding| self.violation(ty.span(), finding)).collect();
        self.violations.extend(violations);
    }

    fn check_fields(&mut self, fields: &Fields) {
        fields.iter().for_each(|field| {
            self.violations.extend(self.bool_field(field));
            self.check_type(&field.ty);
        });
    }

    fn bool_field(&self, field: &Field) -> Option<Violation<'scan>> {
        matches!(
            &field.ty,
            Type::Path(path) if path.path.segments.last().is_some_and(|segment| segment.ident == "bool")
        )
        .then(|| {
            self.violation(
                field.ident.as_ref().map_or_else(|| field.ty.span(), Ident::span),
                Message::new(
                    Problem::new("bool field"),
                    Fix::new("a stored flag is a stored mode, make it an enum or a type parameter"),
                ),
            )
        })
    }
}

impl<'ast> Watch<'ast> for TypeShapes<'_> {
    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemMod, Kind::ItemImpl, Kind::ItemStruct, Kind::ItemEnum, Kind::ItemType, Kind::Macro, Kind::Signature]
    }

    fn on_item_mod(&mut self, module: &'ast ItemMod) -> Descend {
        Descend::from(Walk::outside_tests(&module.attrs))
    }

    fn on_item_impl(&mut self, item: &'ast ItemImpl) -> Descend {
        Descend::from(Walk::inherent_impls(item))
    }

    fn on_item_struct(&mut self, item: &'ast ItemStruct) -> Descend {
        self.check_fields(&item.fields);
        Descend::Over
    }

    fn on_item_enum(&mut self, item: &'ast ItemEnum) -> Descend {
        item.variants.iter().for_each(|variant| self.check_fields(&variant.fields));
        Descend::Over
    }

    fn on_item_type(&mut self, item: &'ast ItemType) -> Descend {
        self.check_type(&item.ty);
        Descend::Over
    }

    fn on_macro(&mut self, _: &'ast Macro) -> Descend {
        Descend::Expand
    }

    fn on_signature(&mut self, signature: &'ast Signature) -> Descend {
        signature
            .inputs
            .iter()
            .filter_map(Syntax::typed)
            .map(|typed| &*typed.ty)
            .chain(Syntax::returned(&signature.output))
            .for_each(|ty| self.check_type(ty));
        Descend::Over
    }
}

impl<'file> Finds<'file> for TypeShapes<'file> {
    fn found(self: Box<Self>) -> Vec<Violation<'file>> {
        self.violations
    }
}

#[derive(Default)]
struct TypeFindings(Vec<Message>);

impl<'ast> Watch<'ast> for TypeFindings {
    fn kinds(&self) -> &'static [Kind] {
        &[Kind::TypeTuple, Kind::TypePath]
    }

    fn on_type_tuple(&mut self, tuple: &'ast TypeTuple) -> Descend {
        self.0.extend(tuple.elems.first().map(|_| {
            Message::new(
                Problem::new("tuple in a signature or field"),
                Fix::new("values that travel together are a struct"),
            )
        }));
        Descend::Into
    }

    fn on_type_path(&mut self, path: &'ast TypePath) -> Descend {
        self.0.extend(TypeShape::string_error(path));
        Descend::Into
    }
}
