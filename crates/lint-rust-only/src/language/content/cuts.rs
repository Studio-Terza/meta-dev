use std::iter::once;

use newtype::Wrapping;

use super::text::Text;

pub(super) struct Cuts<'text> {
    text: Text<'text>,
    separator: Text<'static>,
}

impl<'text> Cuts<'text> {
    pub(super) fn new(text: Text<'text>, separator: Text<'static>) -> Cuts<'text> {
        Cuts { text, separator }
    }

    pub(super) fn pieces(self) -> impl Iterator<Item = Text<'text>> {
        let text = self.text.into_inner();
        let separator = self.separator.into_inner();
        let ends = text.match_indices(separator).map(|(at, _)| at).chain(once(text.len()));
        let starts = once(usize::MIN).chain(text.match_indices(separator).map(|(at, _)| at + separator.len()));
        starts.zip(ends).filter_map(move |(from, to)| text.get(from..to)).map(Text::new)
    }
}
