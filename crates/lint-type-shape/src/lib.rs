use lint_core::{Headline, Lint};
use lint_file::FileLint;
use proc_macro2::Span;
use std::path::Path;
use syn::spanned::Spanned;
use syn::visit::Visit;

use syn::{
    Field, Fields, GenericArgument, Ident, ItemEnum, ItemImpl, ItemMod, ItemStruct, ItemType, Macro, Signature, Type,
    TypePath, TypeTuple,
};

use lint_core::Fix;
use lint_core::Macros;
use lint_core::Message;
use lint_core::Problem;
use lint_core::Report;
use lint_core::Site;
use lint_core::SourceFile;
use lint_core::Syntax;
use lint_core::Violation;
use lint_core::Walk;
use lint_layout::LineNumber;

#[derive(Debug)]
pub struct TypeShape;

impl Lint for TypeShape {
    fn report(files: &[SourceFile]) -> Report<'_> {
        Report::new(
            Headline::new("types carry their meaning: no bool fields, no tuples in signatures, no String errors"),
            files.iter().flat_map(Self::violations).collect(),
        )
    }
}

impl TypeShape {
    fn findings(ty: &Type) -> Vec<Message> {
        let mut findings = TypeFindings::default();
        findings.visit_type(ty);
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

impl FileLint for TypeShape {
    fn violations(file: &SourceFile) -> Vec<Violation<'_>> {
        let mut shapes = TypeShapes { path: file.path(), violations: Vec::new() };
        shapes.visit_file(file.syntax());
        shapes.violations
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

impl<'ast> Visit<'ast> for TypeShapes<'_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        match Walk::outside_tests(&module.attrs) {
            Walk::Entered => syn::visit::visit_item_mod(self, module),
            Walk::Skipped => {}
        }
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        match Walk::inherent_impls(item) {
            Walk::Entered => syn::visit::visit_item_impl(self, item),
            Walk::Skipped => {}
        }
    }

    fn visit_item_struct(&mut self, item: &'ast ItemStruct) {
        self.check_fields(&item.fields);
    }

    fn visit_item_enum(&mut self, item: &'ast ItemEnum) {
        item.variants.iter().for_each(|variant| self.check_fields(&variant.fields));
    }

    fn visit_item_type(&mut self, item: &'ast ItemType) {
        self.check_type(&item.ty);
    }

    fn visit_macro(&mut self, invocation: &'ast Macro) {
        Macros::walk(invocation, |statement| self.visit_stmt(statement));
    }

    fn visit_signature(&mut self, signature: &'ast Signature) {
        signature
            .inputs
            .iter()
            .filter_map(Syntax::typed)
            .map(|typed| &*typed.ty)
            .chain(Syntax::returned(&signature.output))
            .for_each(|ty| self.check_type(ty));
    }
}

#[derive(Default)]
struct TypeFindings(Vec<Message>);

impl<'ast> Visit<'ast> for TypeFindings {
    fn visit_type_tuple(&mut self, tuple: &'ast TypeTuple) {
        self.0.extend(tuple.elems.first().map(|_| {
            Message::new(
                Problem::new("tuple in a signature or field"),
                Fix::new("values that travel together are a struct"),
            )
        }));
        syn::visit::visit_type_tuple(self, tuple);
    }

    fn visit_type_path(&mut self, path: &'ast TypePath) {
        self.0.extend(TypeShape::string_error(path));
        syn::visit::visit_type_path(self, path);
    }
}
