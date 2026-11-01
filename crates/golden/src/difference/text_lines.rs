use newtype::Newtype;

#[derive(Newtype, Debug)]
pub struct TextLines<'text>(&'text str);

impl TextLines<'_> {
    const BREAK: char = '\n';
}

impl<'text> Iterator for TextLines<'text> {
    type Item = &'text str;

    fn next(&mut self) -> Option<Self::Item> {
        let end = self.0.match_indices(Self::BREAK).next().map_or(self.0.len(), |(at, _)| at + Self::BREAK.len_utf8());
        let (line, rest) = self.0.split_at(end);
        self.0 = rest;
        (!line.is_empty()).then_some(line)
    }
}
