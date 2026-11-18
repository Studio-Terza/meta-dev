use std::ffi::OsString;
use std::iter::Peekable;
use std::path::PathBuf;

use algebra::{Monoid, Semigroup};
use design_store::Scope;
use grammar_flag::Flags;

use super::super::failure::Failure;
use super::{Flag, Token, Valued, Verb, Words};

#[derive(Debug)]
pub(super) enum Said {
    Verb(Verb),
    Word(Token),
    Flagged(Flag),
    Valued(Valued, OsString),
    Workspace(PathBuf),
}

#[derive(Clone, Copy, Debug)]
pub(super) enum Wanting {
    Path,
    Value(Valued),
}

#[derive(Debug)]
pub(super) enum Heard {
    Whole(Said),
    Wanting(Wanting),
}

#[derive(Debug)]
pub(super) struct Saying<I: Iterator<Item = OsString>> {
    arguments: Peekable<I>,
    verb: Option<Verb>,
}

impl Said {
    fn verb(&self) -> Option<Verb> {
        match self {
            Said::Verb(verb) => Some(*verb),
            Said::Word(_) | Said::Flagged(_) | Said::Valued(..) | Said::Workspace(_) => None,
        }
    }
}

impl Wanting {
    fn filled(self, value: OsString) -> Said {
        match self {
            Wanting::Path => Said::Workspace(PathBuf::from(value)),
            Wanting::Value(valued) => Said::Valued(valued, value),
        }
    }
}

impl Heard {
    fn completed(self, following: impl FnOnce() -> Option<OsString>) -> Result<Said, Failure> {
        match self {
            Heard::Whole(said) => Ok(said),
            Heard::Wanting(wanting) => following().ok_or(Failure::Usage).map(|value| wanting.filled(value)),
        }
    }
}

impl From<Said> for Words {
    fn from(said: Said) -> Words {
        match said {
            Said::Verb(verb) => Words { verb: Some(verb), ..Words::default() },
            Said::Word(word) => Words { spoken: vec![word], ..Words::default() },
            Said::Flagged(flag) => {
                Words { spoken: flag.token().into_iter().collect(), scope: flag.scope(), ..Words::default() }
            }
            Said::Valued(valued, value) => Words { spoken: vec![Token::Valued(valued, value)], ..Words::default() },
            Said::Workspace(path) => Words { workspace: Some(path), ..Words::default() },
        }
    }
}

impl Semigroup for Words {
    fn combine(self, other: Words) -> Words {
        Words {
            verb: self.verb.or(other.verb),
            spoken: self.spoken.combine(other.spoken),
            workspace: other.workspace.or(self.workspace),
            scope: [self.scope, other.scope].into_iter().find(|scope| *scope == Scope::Everyone).unwrap_or_default(),
        }
    }
}

impl Monoid for Words {}

impl<I: Iterator<Item = OsString>> Iterator for Saying<I> {
    type Item = Result<Said, Failure>;

    fn next(&mut self) -> Option<Result<Said, Failure>> {
        let heard = self.arguments.next().map(|argument| self.heard(argument))?;
        Some(
            heard
                .and_then(|listened| listened.completed(|| self.arguments.next()))
                .inspect(|said| self.verb = self.verb.or(said.verb())),
        )
    }
}

impl<I: Iterator<Item = OsString>> Saying<I> {
    pub(super) fn said(arguments: I) -> impl Iterator<Item = Result<Said, Failure>> {
        Saying { arguments: arguments.peekable(), verb: None }
    }

    fn heard(&self, argument: OsString) -> Result<Heard, Failure> {
        match argument.to_str().and_then(Flags::spelled) {
            Some(spelled) => spelled.parse().and_then(|flag: Flag| flag.heard(self.verb)),
            None => Heard::plain(self.verb, argument),
        }
    }
}

impl Heard {
    fn plain(verb: Option<Verb>, argument: OsString) -> Result<Heard, Failure> {
        match verb {
            None => argument
                .to_str()
                .ok_or(Failure::Usage)
                .and_then(str::parse)
                .map(|named| Heard::Whole(Said::Verb(named))),
            Some(_) => Ok(Heard::Whole(Said::Word(Token::Word(argument)))),
        }
    }
}
