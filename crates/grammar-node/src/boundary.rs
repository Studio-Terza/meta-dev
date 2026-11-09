use grammar::logos::{Lexer, Logos};

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
#[logos(crate = grammar::logos)]
enum Spelling {
    #[regex("n0|n[1-9][0-9]*", Spelling::counted)]
    #[regex("n0[0-9]+", |_padded| Spelled::Misnumbered)]
    Id(Spelled),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Spelled {
    Numbered(i64),
    Misnumbered,
    Unspelled,
}

#[derive(Debug)]
pub struct Ids;

impl Ids {
    #[must_use]
    pub fn spelled(text: &str) -> Spelled {
        let spelled: Vec<Result<Spelling, ()>> = Spelling::lexer(text).collect();
        <[Result<Spelling, ()>; 1]>::try_from(spelled)
            .ok()
            .and_then(|[only]| only.ok())
            .map_or(Spelled::Unspelled, Spelling::spelled)
    }
}

impl Spelling {
    fn counted(lexer: &Lexer<'_, Spelling>) -> Spelled {
        lexer
            .slice()
            .get(1..)
            .and_then(|digits| digits.parse::<i64>().ok())
            .map_or(Spelled::Misnumbered, Spelled::Numbered)
    }

    fn spelled(self) -> Spelled {
        match self {
            Spelling::Id(spelled) => spelled,
        }
    }
}
