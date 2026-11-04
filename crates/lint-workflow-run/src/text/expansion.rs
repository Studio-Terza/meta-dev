use crate::text::Text;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Expansion;

impl Expansion {
    const OPEN: char = '{';
    const CLOSE: char = '}';
}

impl<'text> TryFrom<Text<'text>> for Expansion {
    type Error = Text<'text>;

    fn try_from(rest: Text<'text>) -> Result<Expansion, Text<'text>> {
        let spelled = rest.0;
        let end = spelled.match_indices(Self::CLOSE).next().map_or(spelled.len(), |(at, _)| at);
        spelled
            .get(..end)
            .filter(|_| spelled.chars().next().is_none_or(|letter| letter != Self::OPEN))
            .filter(|name| name.chars().any(|letter| !letter.is_alphanumeric() && letter != '_'))
            .map(|_| Expansion)
            .ok_or(rest)
    }
}
