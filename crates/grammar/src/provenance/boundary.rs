use logos::{Lexer, Logos};

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
enum Spelling {
    #[regex("agent-(0|-?[1-9][0-9]*)", Spelling::counted)]
    Agent(Option<i64>),
    #[regex("import-[1-9][0-9]*", Spelling::lined)]
    Import(Option<usize>),
}

pub trait Originating {
    type Output;

    fn agent(self, run: i64) -> Self::Output;

    fn import(self, line: usize) -> Self::Output;
}

#[derive(Debug)]
pub struct Origins;

impl Origins {
    const AGENT: &'static str = "agent-";
    const IMPORT: &'static str = "import-";

    pub fn read<Origin: Originating>(text: &str, origin: Origin) -> Option<Origin::Output> {
        let spelled: Vec<Result<Spelling, ()>> = Spelling::lexer(text).collect();
        <[Result<Spelling, ()>; 1]>::try_from(spelled)
            .ok()
            .and_then(|[only]| only.ok())
            .and_then(|spelling| spelling.originated(origin))
    }

    #[must_use]
    pub fn agent(run: i64) -> String {
        format!("{}{run}", Origins::AGENT)
    }

    #[must_use]
    pub fn import(line: usize) -> String {
        format!("{}{line}", Origins::IMPORT)
    }
}

impl Spelling {
    fn counted(lexer: &Lexer<'_, Spelling>) -> Option<i64> {
        lexer.slice().get(Origins::AGENT.len()..).and_then(|digits| digits.parse::<i64>().ok())
    }

    fn lined(lexer: &Lexer<'_, Spelling>) -> Option<usize> {
        lexer.slice().get(Origins::IMPORT.len()..).and_then(|digits| digits.parse::<usize>().ok())
    }

    fn originated<Origin: Originating>(self, origin: Origin) -> Option<Origin::Output> {
        match self {
            Spelling::Agent(run) => run.map(|id| origin.agent(id)),
            Spelling::Import(line) => line.map(|number| origin.import(number)),
        }
    }
}
