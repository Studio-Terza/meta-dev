use crate::document::text::Text;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Expansion;

impl Expansion {
    const OPEN: char = '{';
    const CLOSE: char = '}';
    const UNDERSCORE: char = '_';
}

impl<'text> TryFrom<Text<'text>> for Expansion {
    type Error = Text<'text>;

    fn try_from(rest: Text<'text>) -> Result<Expansion, Text<'text>> {
        let spelled = rest.0;
        let end = spelled.match_indices(Self::CLOSE).next().map_or(spelled.len(), |(at, _)| at);
        spelled
            .get(..end)
            .filter(|_| spelled.chars().next().is_none_or(|letter| letter != Self::OPEN))
            .filter(|name| {
                name.chars().filter(|letter| letter.is_alphanumeric()).count() + name.matches(Self::UNDERSCORE).count()
                    < name.chars().count()
            })
            .map(|_| Expansion)
            .ok_or(rest)
    }
}
