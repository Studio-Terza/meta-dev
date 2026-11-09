use grammar::logos::Logos;

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
#[logos(crate = grammar::logos)]
enum Spelling<'text> {
    #[regex("[A-Za-z0-9_][A-Za-z0-9_-]*", |lexer| lexer.slice())]
    Word(&'text str),
    #[token(".")]
    Dot,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cited {
    prefix: String,
    ordinal: usize,
}

impl<'text> Spelling<'text> {
    fn word(self) -> Option<&'text str> {
        match self {
            Spelling::Word(word) => Some(word),
            Spelling::Dot => None,
        }
    }
}

impl Cited {
    #[must_use]
    pub fn read(cited: &str) -> Option<Cited> {
        let spelled: Vec<Spelling<'_>> = Spelling::lexer(cited).collect::<Result<_, ()>>().ok()?;
        let [prefix, dot, digits] = <[Spelling<'_>; 3]>::try_from(spelled).ok()?;
        Some(dot).filter(|spelling| *spelling == Spelling::Dot)?;
        let ordinal = digits.word()?.parse().ok()?;
        prefix.word().map(|word| Cited { prefix: word.to_owned(), ordinal })
    }

    #[must_use]
    pub fn prefix(&self) -> &str {
        &self.prefix
    }

    #[must_use]
    pub fn ordinal(&self) -> usize {
        self.ordinal
    }
}
