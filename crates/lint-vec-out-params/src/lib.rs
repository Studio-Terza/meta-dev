mod finding;

use finding::Finding;
use lint_core::Lint;
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{ItemImpl, ItemMod, PathSegment, Signature, Type, TypePath, TypeReference};

use lint_core::{Headline, Macros, Report, Site, SourceFile, Syntax, Violation, Walk};
use lint_layout::{LineNumber, Name, Word};

#[derive(Debug)]
pub struct VecOutParams<'file> {
    file: &'file SourceFile,
    violations: Vec<Violation<'file>>,
}

impl Lint for VecOutParams<'_> {
    fn report(files: &[SourceFile]) -> Report<'_> {
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
    const VECTOR: Word<'static> = Word::new("Vec");
    const OPTION: Word<'static> = Word::new("Option");

    fn vec_out(parameter: &Type) -> Option<&Type> {
        match parameter {
            Type::Reference(reference) => Some(reference)
                .filter(|candidate| candidate.mutability.is_some())
                .and_then(Self::vec_reference)
                .map(|_| parameter),
            Type::Path(path) => Self::last_segment(path, Self::OPTION).and_then(Self::vec_argument).map(|_| parameter),
            _ => None,
        }
    }

    fn vec_reference(reference: &TypeReference) -> Option<&PathSegment> {
        Syntax::type_path(&reference.elem).and_then(|path| Self::last_segment(path, Self::VECTOR))
    }

    fn vec_argument(segment: &PathSegment) -> Option<&Type> {
        Syntax::angled(&segment.arguments)
            .and_then(|arguments| arguments.args.first())
            .and_then(Syntax::type_argument)
            .and_then(Self::vec_out)
    }

    fn last_segment<'path>(path: &'path TypePath, name: Word<'_>) -> Option<&'path PathSegment> {
        path.path.segments.last().filter(|segment| Name::from(&segment.ident) == Name::from(name))
    }
}

impl<'ast> Visit<'ast> for VecOutParams<'_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        Walk::enter(&module.attrs, || syn::visit::visit_item_mod(self, module));
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
            signature.inputs.iter().filter_map(Syntax::typed).filter(|typed| Self::vec_out(&typed.ty).is_some()).map(
                |typed| {
                    Violation::new(
                        Site::new(file.path(), LineNumber::from(typed.pat.span())),
                        Finding::VecOut { name: Name::from(&signature.ident) },
                    )
                },
            ),
        );
    }
}
