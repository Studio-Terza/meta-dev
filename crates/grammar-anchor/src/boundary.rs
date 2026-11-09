use grammar::logos::Logos;

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
#[logos(crate = grammar::logos)]
enum Spelling<'text> {
    #[regex("[^#]+", |lexer| lexer.slice())]
    Message(&'text str),
    #[token("#")]
    Hash,
}

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
#[logos(crate = grammar::logos)]
enum Index {
    #[regex("0|[1-9][0-9]*", |lexer| lexer.slice().parse().ok())]
    Ordinal(usize),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Anchoring {
    message: String,
    ordinal: Option<usize>,
}

impl Anchoring {
    #[must_use]
    pub fn read(text: &str) -> Option<Anchoring> {
        let spelled: Vec<Spelling<'_>> = Spelling::lexer(text).collect::<Result<_, ()>>().ok()?;
        let (message, rest) = spelled.split_first()?;
        let ordinal = rest
            .first()
            .map_or(Some(None), |_hash| <&[Spelling<'_>; 2]>::try_from(rest).ok().and_then(Index::hashed).map(Some));
        message.message().zip(ordinal).map(|(said, index)| Anchoring { message: said.to_owned(), ordinal: index })
    }

    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    #[must_use]
    pub fn ordinal(&self) -> Option<usize> {
        self.ordinal
    }
}

impl<'text> Spelling<'text> {
    fn message(self) -> Option<&'text str> {
        match self {
            Spelling::Message(message) => Some(message),
            Spelling::Hash => None,
        }
    }
}

impl Index {
    fn hashed([hash, digits]: &[Spelling<'_>; 2]) -> Option<usize> {
        Some(hash).filter(|spelling| **spelling == Spelling::Hash)?;
        Index::read(digits.message()?)
    }

    fn read(digits: &str) -> Option<usize> {
        let spelled: Vec<Result<Index, ()>> = Index::lexer(digits).collect();
        <[Result<Index, ()>; 1]>::try_from(spelled)
            .ok()
            .and_then(|[only]| only.ok())
            .map(|Index::Ordinal(ordinal)| ordinal)
    }
}
