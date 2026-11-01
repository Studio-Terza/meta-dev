use super::position::Position;
use super::raw_token::RawToken;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Neighbours<'tokens> {
    before: Option<&'tokens RawToken>,
    after: Option<&'tokens RawToken>,
    then: Option<&'tokens RawToken>,
}

impl<'tokens> Neighbours<'tokens> {
    pub(crate) fn around(tokens: &'tokens [RawToken], index: usize) -> Neighbours<'tokens> {
        Neighbours {
            before: index.checked_sub(1).and_then(|before| tokens.get(before)),
            after: tokens.get(index + 1),
            then: tokens.get(index + 1 + 1),
        }
    }

    pub(crate) fn position(self) -> Position {
        match (self.before, self.after, self.then) {
            (Some(RawToken::Mark('.' | ':')), _, _)
            | (_, Some(RawToken::Mark('(' | '!')), _)
            | (_, Some(RawToken::Mark(':')), Some(RawToken::Mark(':'))) => Position::Named,
            (_, Some(RawToken::Literal), _) => Position::BeforeLiteral,
            _ => Position::Plain,
        }
    }
}
