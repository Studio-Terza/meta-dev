use invariants::Ignoring;
use logos::{Lexer, Logos};

use crate::{Grammar, Unlexed, Whole};

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
enum Piece<'text> {
    #[regex("<task-id>[^<]+</task-id>", Piece::inner)]
    Task(&'text str),
    #[regex("[^<]+")]
    Other,
    #[token("<")]
    Open,
}

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
enum Stem<'text> {
    #[regex("agent-a[0-9a-f]{16}", |lexer| lexer.slice().get(Stem::AGENT..))]
    Background(&'text str),
    #[regex("agent-a[a-z0-9-]+-[0-9a-f]{16}", Stem::teammate)]
    Teammate(Named<'text>),
    #[regex("[^/]+", |lexer| lexer.slice(), priority = 1)]
    Plain(&'text str),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Named<'text> {
    id: &'text str,
    name: &'text str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TaskNotice<'text>(&'text str);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TaskFile<'text>(Stem<'text>);

impl<'text> Piece<'text> {
    const OPEN: usize = "<task-id>".len();
    const CLOSE: usize = "</task-id>".len();

    fn inner(lexer: &Lexer<'text, Piece<'text>>) -> Option<&'text str> {
        let slice = lexer.slice();
        slice.get(Self::OPEN..slice.len().saturating_sub(Self::CLOSE))
    }
}

impl<'text> Stem<'text> {
    const AGENT: usize = "agent-".len();
    const NAMED: usize = "agent-a".len();
    const HASH: usize = "-0123456789abcdef".len();

    fn teammate(lexer: &Lexer<'text, Stem<'text>>) -> Option<Named<'text>> {
        let slice = lexer.slice();
        let id = slice.get(Self::AGENT..)?;
        let name = slice.get(Self::NAMED..slice.len().saturating_sub(Self::HASH))?;
        Some(Named { id, name })
    }
}

impl<'text> Grammar<'text> for TaskNotice<'text> {
    fn read(text: &'text str) -> Option<TaskNotice<'text>> {
        Piece::lexer(text).filter_map(|piece| piece.ignoring(Unlexed::NotThisGrammar)).find_map(|piece| match piece {
            Piece::Task(task) => Some(TaskNotice(task)),
            Piece::Other | Piece::Open => None,
        })
    }
}

impl<'text> TaskNotice<'text> {
    #[must_use]
    pub fn task(&self) -> &'text str {
        self.0
    }
}

impl<'text> Grammar<'text> for TaskFile<'text> {
    fn read(stem: &'text str) -> Option<TaskFile<'text>> {
        Whole::exactly(stem).map(|[only]| TaskFile(only))
    }
}

impl<'text> TaskFile<'text> {
    #[must_use]
    pub fn ids(&self) -> [Option<&'text str>; 2] {
        match self.0 {
            Stem::Background(id) | Stem::Plain(id) => [Some(id), None],
            Stem::Teammate(Named { id, name }) => [Some(id), Some(name)],
        }
    }
}
