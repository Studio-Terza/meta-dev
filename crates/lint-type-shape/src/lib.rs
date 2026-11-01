use lint_core::Lint;
use proc_macro2::Span;
use std::path::Path;
use syn::spanned::Spanned;
use syn::visit::Visit;

use syn::{
    Field, Fields, FnArg, GenericArgument, Ident, ItemEnum, ItemImpl, ItemMod, ItemStruct, ItemType, Macro,
    PathArguments, ReturnType, Signature, Type, TypePath, TypeTuple,
};

use lint_core::LineNumber;
use lint_core::Macros;
use lint_core::Report;
use lint_core::Site;
use lint_core::SourceFile;
use lint_core::Violation;
use lint_core::Walk;

#[derive(Debug)]
pub struct TypeShape;

impl Lint for TypeShape {
    fn report(files: &[SourceFile]) -> Report {
        Report::new(
            "types carry their meaning: no bool fields, no tuples in signatures, no String errors",
            files.iter().flat_map(Self::violations).collect(),
        )
    }
}

impl TypeShape {
    fn violations(file: &SourceFile) -> Vec<Violation> {
        let mut shapes = TypeShapes { path: file.path(), violations: Vec::new() };
        shapes.visit_file(file.syntax());
        shapes.violations
    }

    fn findings(ty: &Type) -> Vec<&'static str> {
        let mut findings = TypeFindings::default();
        findings.visit_type(ty);
        findings.0
    }

    fn string_error(path: &TypePath) -> Option<&'static str> {
        path.path
            .segments
            .last()
            .filter(|segment| segment.ident == "Result")
            .and_then(|segment| match &segment.arguments {
                PathArguments::AngleBracketed(arguments) => arguments.args.iter().nth(1),
                _ => None,
            })
            .filter(|argument| {
                matches!(
                    argument,
                    GenericArgument::Type(Type::Path(error))
                        if error.path.segments.last().is_some_and(|segment| segment.ident == "String")
                )
            })
            .map(|_| "Result with a String error; errors are an enum with thiserror")
    }
}

struct TypeShapes<'scan> {
    path: &'scan Path,
    violations: Vec<Violation>,
}

impl TypeShapes<'_> {
    fn violation(&self, span: Span, message: &str) -> Violation {
        Violation::new(Site::new(self.path, LineNumber::from(span)), message)
    }

    fn check_type(&mut self, ty: &Type) {
        let violations: Vec<Violation> =
            TypeShape::findings(ty).into_iter().map(|finding| self.violation(ty.span(), finding)).collect();
        self.violations.extend(violations);
    }

    fn check_fields(&mut self, fields: &Fields) {
        fields.iter().for_each(|field| {
            self.violations.extend(self.bool_field(field));
            self.check_type(&field.ty);
        });
    }

    fn bool_field(&self, field: &Field) -> Option<Violation> {
        matches!(
            &field.ty,
            Type::Path(path) if path.path.segments.last().is_some_and(|segment| segment.ident == "bool")
        )
        .then(|| {
            self.violation(
                field.ident.as_ref().map_or_else(|| field.ty.span(), Ident::span),
                "bool field; a stored flag is a stored mode, make it an enum or a type parameter",
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
            .filter_map(|argument| match argument {
                FnArg::Typed(typed) => Some(&*typed.ty),
                FnArg::Receiver(_) => None,
            })
            .for_each(|ty| self.check_type(ty));
        match &signature.output {
            ReturnType::Type(_, ty) => self.check_type(ty),
            ReturnType::Default => {}
        }
    }
}

#[derive(Default)]
struct TypeFindings(Vec<&'static str>);

impl<'ast> Visit<'ast> for TypeFindings {
    fn visit_type_tuple(&mut self, tuple: &'ast TypeTuple) {
        self.0.extend(
            (!tuple.elems.is_empty())
                .then_some("tuple in a signature or field; values that travel together are a struct"),
        );
        syn::visit::visit_type_tuple(self, tuple);
    }

    fn visit_type_path(&mut self, path: &'ast TypePath) {
        self.0.extend(TypeShape::string_error(path));
        syn::visit::visit_type_path(self, path);
    }
}
