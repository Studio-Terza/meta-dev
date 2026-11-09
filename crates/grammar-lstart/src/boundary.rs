use grammar::logos::Logos;

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
#[logos(crate = grammar::logos)]
#[logos(skip r"[ \t\r\n]+")]
enum Piece<'text> {
    #[regex("[A-Z][a-z][a-z]", |lexer| lexer.slice())]
    Word(&'text str),
    #[regex("[0-9]+", |lexer| lexer.slice().parse::<u16>().ok())]
    Number(u16),
    #[token(":")]
    Colon,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stamp<'text> {
    words: [&'text str; 2],
    numbers: [u16; 5],
}

impl<'text> Piece<'text> {
    fn word(self) -> Option<&'text str> {
        match self {
            Piece::Word(word) => Some(word),
            Piece::Number(_) | Piece::Colon => None,
        }
    }

    fn number(self) -> Option<u16> {
        match self {
            Piece::Number(number) => Some(number),
            Piece::Word(_) | Piece::Colon => None,
        }
    }

    fn colon(self) -> Option<Piece<'text>> {
        match self {
            Piece::Colon => Some(self),
            Piece::Word(_) | Piece::Number(_) => None,
        }
    }
}

impl<'text> Stamp<'text> {
    #[must_use]
    pub fn read(text: &'text str) -> Option<Stamp<'text>> {
        let pieces: Vec<Piece<'text>> = Piece::lexer(text).collect::<Result<Vec<Piece<'text>>, ()>>().ok()?;
        let [weekday, month, day, hour, before, minute, after, second, year] =
            <[Piece<'text>; 9]>::try_from(pieces).ok()?;
        before.colon().zip(after.colon())?;
        Some(Stamp {
            words: [weekday.word()?, month.word()?],
            numbers: [day.number()?, hour.number()?, minute.number()?, second.number()?, year.number()?],
        })
    }

    #[must_use]
    pub fn words(&self) -> [&'text str; 2] {
        self.words
    }

    #[must_use]
    pub fn numbers(&self) -> [u16; 5] {
        self.numbers
    }
}
