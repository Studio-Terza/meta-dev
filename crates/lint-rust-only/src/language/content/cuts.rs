use std::iter::once;

pub(super) struct Cuts<'text> {
    text: &'text str,
    separator: &'static str,
}

impl<'text> Cuts<'text> {
    pub(super) fn new(text: &'text str, separator: &'static str) -> Cuts<'text> {
        Cuts { text, separator }
    }

    pub(super) fn pieces(self) -> impl Iterator<Item = &'text str> {
        let text = self.text;
        let ends = text.match_indices(self.separator).map(|(at, _)| at).chain(once(text.len()));
        let starts =
            once(usize::MIN).chain(text.match_indices(self.separator).map(|(at, _)| at + self.separator.len()));
        starts.zip(ends).filter_map(move |(from, to)| text.get(from..to))
    }
}
