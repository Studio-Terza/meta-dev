mod finding;

use finding::Finding;
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{FnArg, ItemMod, Signature, Type};

use lint_core::{LineNumber, Name, Report, Site, SourceFile, Violation, Walk};

#[derive(Debug)]
pub struct VecOutParams<'file> {
    file: &'file SourceFile,
    violations: Vec<Violation>,
}

impl VecOutParams<'_> {
    #[must_use]
    pub fn report(files: &[SourceFile]) -> Report {
        Report::new(
            "a &mut Vec parameter lets any caller push without the checks; give the collection a type that owns its rules",
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

    fn is_vec_out(parameter: &Type) -> bool {
        match parameter {
            Type::Reference(reference) => {
                reference.mutability.is_some()
                    && matches!(
                        reference.elem.as_ref(),
                        Type::Path(path) if path.path.segments.last().is_some_and(|segment| segment.ident == "Vec")
                    )
            }
            _ => false,
        }
    }
}

impl<'ast> Visit<'ast> for VecOutParams<'_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        match Walk::outside_tests(&module.attrs) {
            Walk::Entered => syn::visit::visit_item_mod(self, module),
            Walk::Skipped => {}
        }
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
                        Site::new(file.path(), LineNumber::from(typed.span())),
                        Finding::VecOut { name: Name::from(&signature.ident) },
                    )
                }),
        );
    }
}
