mod finding;

use std::path::Path;

use finding::Finding;
use lint_file::{Reads, SweptLint};
use lint_sweep::{Concludes, Contribute, Contributed, Contributing, Descend, Facts, Kind};
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

    fn watcher(file: &SourceFile) -> Contributing<impl Concludes<'_>> {
        Contributing::new(OutParams { path: AsRef::<Path>::as_ref(file.head()) })
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

type Found<'file> = Contributed<(), Vec<Violation<'file>>>;

impl<'file> Contribute for OutParams<'file> {
    type Open = ();
    type Part = Vec<Violation<'file>>;

    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemMod, Kind::ItemImpl, Kind::Macro, Kind::Signature]
    }

    fn on_item_mod(
        &self,
        _: &ItemMod,
        facts: &Facts<'_, ItemMod>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Found<'file> {
        Contributed::descending(Descend::from(facts.gate()))
    }

    fn on_item_impl(
        &self,
        _: &ItemImpl,
        facts: &Facts<'_, ItemImpl>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Found<'file> {
        Contributed::descending(facts.inherent().map_or(Descend::Over, |_| Descend::Into))
    }

    fn on_macro(&self, _: &Macro, _: &Facts<'_, Macro>, _around: &lint_sweep::Around<'_, Self::Open>) -> Found<'file> {
        Contributed::descending(Descend::Expand)
    }

    fn on_signature(
        &self,
        signature: &Signature,
        facts: &Facts<'_, Signature>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Found<'file> {
        let path = self.path;
        let found = signature
            .inputs
            .iter()
            .filter_map(Syntax::typed)
            .filter(|typed| VecOutParams::vec_out(&typed.ty).is_some())
            .filter_map(|typed| {
                let name = facts.name()?.clone();
                Some(Violation::new(Site::new(path, LineNumber::from(typed.pat.span())), Finding::VecOut { name }))
            })
            .collect();
        Contributed::new((), found, Descend::Over)
    }
}

impl<'file> Concludes<'file> for OutParams<'file> {
    fn violations(self, (): (), found: Vec<Violation<'file>>) -> Vec<Violation<'file>> {
        found
    }
}
