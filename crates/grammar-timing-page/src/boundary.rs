use grammar::logos::Logos;

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
#[logos(crate = grammar::logos)]
enum Piece {
    #[token("<td class=\"error-text\">")]
    Error,
    #[token("<")]
    Open,
    #[regex("[^<]+")]
    Text,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Build {
    Finished,
    Failed,
}

impl Build {
    #[must_use]
    pub fn recorded(page: &str) -> Build {
        Piece::lexer(page).find(|piece| *piece == Ok(Piece::Error)).map_or(Build::Finished, |_error| Build::Failed)
    }
}
