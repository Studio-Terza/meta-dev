#[derive(Debug)]
pub struct Lines<'text>(&'text str);

impl<'text> From<&'text str> for Lines<'text> {
    fn from(text: &'text str) -> Lines<'text> {
        Lines(text)
    }
}

impl Lines<'_> {
    const BREAK: char = '\n';
}

impl<'text> Iterator for Lines<'text> {
    type Item = &'text str;

    fn next(&mut self) -> Option<Self::Item> {
        (!self.0.is_empty()).then(|| {
            let end = self.0.match_indices(Self::BREAK).next().map_or(self.0.len(), |(at, _)| at);
            let (line, rest) = self.0.split_at(end);
            self.0 = rest.get(Self::BREAK.len_utf8()..).unwrap_or_default();
            line
        })
    }
}
