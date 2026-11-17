mod finding;

use std::path::Path;

use finding::Finding;
use lint_file::{Reads, SweptLint};
use lint_sweep::{Descend, Facts, Finds, Fold, Folding, Kind, Visit};
use syntax::Spanned;
use syntax::{ItemImpl, ItemMod, Macro, PathSegment, Signature, Type, TypePath, TypeReference};

use lint_core::{Headline, Site, SourceFile, Syntax, Violation};
use lint_layout::{LineNumber, Name, Word};

#[derive(Debug)]
pub struct VecOutParams;

impl SweptLint for VecOutParams {
    const HEADLINE: Headline = Headline::new(
        "a &mut Vec parameter lets any caller push without the checks; give the collection a type that owns its rules",
    );
    const READS: Reads = Reads::Every;

    fn watcher(file: &SourceFile) -> Box<dyn Finds<'_> + '_> {
        Box::new(Folding::from(OutParams { path: AsRef::<Path>::as_ref(file.head()) }))
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
}

impl<'file> Fold for OutParams<'file> {
    type Found = Vec<Violation<'file>>;

    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemMod, Kind::ItemImpl, Kind::Macro, Kind::Signature]
    }

    fn on_item_mod(&self, found: Self::Found, _: &ItemMod, facts: &Facts<'_, ItemMod>) -> Visit<Self::Found> {
        Visit::new(found, Descend::from(facts.gate()))
    }

    fn on_item_impl(&self, found: Self::Found, _: &ItemImpl, facts: &Facts<'_, ItemImpl>) -> Visit<Self::Found> {
        Visit::new(found, facts.inherent().map_or(Descend::Over, |_| Descend::Into))
    }

    fn on_macro(&self, found: Self::Found, _: &Macro, _: &Facts<'_, Macro>) -> Visit<Self::Found> {
        Visit::new(found, Descend::Expand)
    }

    fn on_signature(
        &self,
        found: Self::Found,
        signature: &Signature,
        facts: &Facts<'_, Signature>,
    ) -> Visit<Self::Found> {
        let path = self.path;
        let noted: Self::Found = found
            .into_iter()
            .chain(
                signature
                    .inputs
                    .iter()
                    .filter_map(Syntax::typed)
                    .filter(|typed| VecOutParams::vec_out(&typed.ty).is_some())
                    .filter_map(|typed| {
                        let name = facts.name()?.clone();
                        Some(Violation::new(
                            Site::new(path, LineNumber::from(typed.pat.span())),
                            Finding::VecOut { name },
                        ))
                    }),
            )
            .collect();
        Visit::new(noted, Descend::Over)
    }
}
