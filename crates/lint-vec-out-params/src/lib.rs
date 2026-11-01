mod finding;

use finding::Finding;
use lint_core::Lint;
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{FnArg, GenericArgument, ItemImpl, ItemMod, PathArguments, PathSegment, Signature, Type, TypePath};

use lint_core::{Headline, Macros, Report, Site, SourceFile, Violation, Walk};
use lint_layout::{LineNumber, Name};

#[derive(Debug)]
pub struct VecOutParams<'file> {
    file: &'file SourceFile,
    violations: Vec<Violation>,
}

impl Lint for VecOutParams<'_> {
    fn report(files: &[SourceFile]) -> Report {
        Report::new(
            Headline::new(
                "a &mut Vec parameter lets any caller push without the checks; give the collection a type that owns its rules",
            ),
            files
                .iter()
                .flat_map(|file| {
                    let mut scan = VecOutParams { file, violations: Vec::new() };
                    scan.visit_file(file.syntax());
                    scan.violations
                })
                .collect(),
        )
    }
}

impl VecOutParams<'_> {
    const VECTOR: &'static str = "Vec";
    const OPTION: &'static str = "Option";

    fn is_vec_out(parameter: &Type) -> bool {
        match parameter {
            Type::Reference(reference) => {
                reference.mutability.is_some()
                    && matches!(reference.elem.as_ref(), Type::Path(path) if Self::last_segment(path, Self::VECTOR).is_some())
            }
            Type::Path(path) => Self::last_segment(path, Self::OPTION)
                .and_then(Self::first_argument)
                .is_some_and(|argument| matches!(argument, GenericArgument::Type(inner) if Self::is_vec_out(inner))),
            _ => false,
        }
    }

    fn first_argument(segment: &PathSegment) -> Option<&GenericArgument> {
        match &segment.arguments {
            PathArguments::AngleBracketed(arguments) => arguments.args.first(),
            _ => None,
        }
    }

    fn last_segment<'path>(path: &'path TypePath, name: &str) -> Option<&'path PathSegment> {
        path.path.segments.last().filter(|segment| segment.ident == name)
    }
}

impl<'ast> Visit<'ast> for VecOutParams<'_> {
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

    fn visit_macro(&mut self, invocation: &'ast syn::Macro) {
        Macros::walk(invocation, |statement| self.visit_stmt(statement));
    }

    fn visit_signature(&mut self, signature: &'ast Signature) {
        let file = self.file;
        self.violations.extend(
            signature
                .inputs
                .iter()
                .filter_map(|input| match input {
                    FnArg::Typed(typed) => Some(typed),
                    FnArg::Receiver(_) => None,
                })
                .filter(|typed| Self::is_vec_out(&typed.ty))
                .map(|typed| {
                    Violation::new(
                        Site::new(file.path(), LineNumber::from(typed.pat.span())),
                        Finding::VecOut { name: Name::from(&signature.ident) },
                    )
                }),
        );
    }
}
