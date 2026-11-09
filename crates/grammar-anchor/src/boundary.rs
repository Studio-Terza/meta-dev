use grammar::logos::{Lexer, Logos};

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
#[logos(crate = grammar::logos)]
enum Spelling {
    #[regex("[^#]+")]
    Message,
    #[regex("[^#]+#(0|[1-9][0-9]*)", Spelling::hashed)]
    Segment(usize),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Anchoring {
    message: String,
    ordinal: Option<usize>,
}

impl Anchoring {
    #[must_use]
    pub fn read(text: &str) -> Option<Anchoring> {
        let spelled: Vec<Result<Spelling, ()>> = Spelling::lexer(text).collect();
        <[Result<Spelling, ()>; 1]>::try_from(spelled)
            .ok()
            .and_then(|[only]| only.ok())
            .map(|spelling| spelling.read(text))
    }

    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    #[must_use]
    pub fn ordinal(&self) -> Option<usize> {
        self.ordinal
    }

    fn whole(text: &str) -> Anchoring {
        Anchoring { message: text.to_owned(), ordinal: None }
    }

    fn indexed(text: &str, hash: usize) -> Anchoring {
        Anchoring {
            message: text.get(..hash).unwrap_or_default().to_owned(),
            ordinal: text.get(hash.saturating_add(1)..).and_then(|digits| digits.parse().ok()),
        }
    }
}

impl Spelling {
    fn read(self, text: &str) -> Anchoring {
        match self {
            Spelling::Message => Anchoring::whole(text),
            Spelling::Segment(hash) => Anchoring::indexed(text, hash),
        }
    }

    fn hashed(lexer: &Lexer<'_, Spelling>) -> usize {
        lexer.slice().char_indices().find(|(_, character)| *character == '#').map_or(0, |(hash, _)| hash)
    }
}
