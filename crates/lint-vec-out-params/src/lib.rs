mod finding;

use std::path::Path;

use finding::Finding;
use lint_file::{Reads, SweptLint};
use lint_sweep::{Descend, Finds, Kind, Watch};
use syn::spanned::Spanned;
use syn::{ItemImpl, ItemMod, Macro, PathSegment, Signature, Type, TypePath, TypeReference};

use lint_core::{Headline, Site, SourceFile, Syntax, Violation, Walk};
use lint_layout::{LineNumber, Name, Word};

#[derive(Debug)]
pub struct VecOutParams;

impl SweptLint for VecOutParams {
    const HEADLINE: Headline = Headline::new(
        "a &mut Vec parameter lets any caller push without the checks; give the collection a type that owns its rules",
    );
    const READS: Reads = Reads::Every;

    fn watcher(file: &SourceFile) -> Box<dyn Finds<'_> + '_> {
        Box::new(OutParams { path: file.head().path(), violations: Vec::new() })
    }
}

impl VecOutParams {
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
        path.path.segments.last().filter(|segment| segment.ident == Name::from(name))
    }
}

#[derive(Debug)]
struct OutParams<'file> {
    path: &'file Path,
    violations: Vec<Violation<'file>>,
}

impl<'ast> Watch<'ast> for OutParams<'_> {
    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemMod, Kind::ItemImpl, Kind::Macro, Kind::Signature]
    }

    fn on_item_mod(&mut self, module: &'ast ItemMod) -> Descend {
        Descend::from(Walk::outside_tests(&module.attrs))
    }

    fn on_item_impl(&mut self, item: &'ast ItemImpl) -> Descend {
        Descend::from(Walk::inherent_impls(item))
    }

    fn on_macro(&mut self, _: &'ast Macro) -> Descend {
        Descend::Expand
    }

    fn on_signature(&mut self, signature: &'ast Signature) -> Descend {
        let path = self.path;
        self.violations.extend(
            signature
                .inputs
                .iter()
                .filter_map(Syntax::typed)
                .filter(|typed| VecOutParams::vec_out(&typed.ty).is_some())
                .map(|typed| {
                    Violation::new(
                        Site::new(path, LineNumber::from(typed.pat.span())),
                        Finding::VecOut { name: Name::from(&signature.ident) },
                    )
                }),
        );
        Descend::Over
    }
}

impl<'file> Finds<'file> for OutParams<'file> {
    fn found(self: Box<Self>) -> Vec<Violation<'file>> {
        self.violations
    }
}
