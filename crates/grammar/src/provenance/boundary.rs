use logos::{Lexer, Logos};

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
enum Spelling {
    #[regex("agent -?[0-9]+", Spelling::counted)]
    Agent(Option<i64>),
}

#[derive(Debug)]
pub struct Agents;

impl Agents {
    const PREFIX: usize = "agent ".len();

    #[must_use]
    pub fn numbered(text: &str) -> Option<i64> {
        let spelled: Vec<Result<Spelling, ()>> = Spelling::lexer(text).collect();
        <[Result<Spelling, ()>; 1]>::try_from(spelled).ok().and_then(|[only]| only.ok()).and_then(Spelling::run)
    }
}

impl Spelling {
    fn counted(lexer: &Lexer<'_, Spelling>) -> Option<i64> {
        lexer.slice().get(Agents::PREFIX..).and_then(|digits| digits.parse::<i64>().ok())
    }

    fn run(self) -> Option<i64> {
        match self {
            Spelling::Agent(run) => run,
        }
    }
}
