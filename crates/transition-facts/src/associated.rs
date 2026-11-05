use syn::{PathSegment, Type, TypePath};

use lint_layout::{Name, Whole, Word};

#[derive(Debug)]
pub(crate) struct Associated;

impl Associated {
    const OUTPUT: Word<'static> = Word::new("Output");
    const SELF: Word<'static> = Word::new("Self");

    pub(crate) fn of_type(found: &Type) -> Option<&TypePath> {
        match found {
            Type::Path(path) => Self::of_path(path),
            _ => None,
        }
    }

    pub(crate) fn of_path(path: &TypePath) -> Option<&TypePath> {
        let qualified = path.qself.as_ref().map(|_| path);
        let selfish = Self::matching(path.path.segments.first(), Self::SELF).map(|()| path);
        qualified.or(selfish).filter(|found| Self::matching(found.path.segments.last(), Self::OUTPUT).is_some())
    }

    fn matching(segment: Option<&PathSegment>, word: Word<'static>) -> Option<()> {
        segment.and_then(|segment| Name::from(&segment.ident).matches::<Whole>(&[word]).map(|_| ()))
    }
}
