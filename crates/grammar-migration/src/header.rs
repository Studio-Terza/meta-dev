use grammar::logos::Logos;

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
#[logos(crate = grammar::logos)]
enum Opening {
    #[token("-- no-transaction")]
    NoTransaction,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Transaction {
    Wrapped,
    Unwrapped,
}

impl From<&str> for Transaction {
    fn from(sql: &str) -> Transaction {
        match Opening::lexer(sql).next() {
            Some(Ok(Opening::NoTransaction)) => Transaction::Unwrapped,
            Some(Err(())) | None => Transaction::Wrapped,
        }
    }
}

impl Transaction {
    pub fn chosen<Choice>(self, wrapped: Choice, unwrapped: Choice) -> Choice {
        match self {
            Transaction::Wrapped => wrapped,
            Transaction::Unwrapped => unwrapped,
        }
    }
}
