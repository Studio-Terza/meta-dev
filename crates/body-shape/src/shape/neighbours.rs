use super::mark::Mark;
use super::position::Position;
use super::raw_token::RawToken;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Neighbours<'tokens> {
    earlier: Option<&'tokens RawToken>,
    before: Option<&'tokens RawToken>,
    after: Option<&'tokens RawToken>,
    then: Option<&'tokens RawToken>,
}

impl<'tokens> Neighbours<'tokens> {
    const CALLED: [RawToken; 2] = [RawToken::Mark(Mark::OPEN_PAREN), RawToken::Mark(Mark::BANG)];

    pub(crate) fn all(tokens: &'tokens [RawToken]) -> impl Iterator<Item = Neighbours<'tokens>> {
        (0..tokens.len()).map(|index| {
            let before = index.checked_sub(1);
            Neighbours {
                earlier: before.and_then(|before| before.checked_sub(1)).and_then(|earlier| tokens.get(earlier)),
                before: before.and_then(|before| tokens.get(before)),
                after: tokens.get(index + 1),
                then: tokens.get(index + 1 + 1),
            }
        })
    }

    pub(crate) fn position(self) -> Position {
        self.named()
            .or_else(|| self.after.filter(|after| **after == RawToken::Literal).map(|_| Position::BeforeLiteral))
            .unwrap_or(Position::Plain)
    }

    fn named(self) -> Option<Position> {
        self.before
            .filter(|before| **before == RawToken::Mark(Mark::DOT))
            .or_else(|| Self::paired(self.earlier, self.before))
            .or_else(|| self.after.filter(|after| Self::CALLED.contains(after)))
            .or_else(|| Self::paired(self.after, self.then))
            .map(|_| Position::Named)
    }

    fn paired(first: Option<&'tokens RawToken>, second: Option<&'tokens RawToken>) -> Option<&'tokens RawToken> {
        let colon = RawToken::Mark(Mark::COLON);
        first.filter(|first| **first == colon).and(second).filter(|second| **second == colon)
    }
}
