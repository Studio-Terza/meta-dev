use std::fmt;
use std::str::FromStr;

use logos::{Lexer, Logos};
use named_derive::Named;

#[derive(Logos, Clone, Debug, PartialEq, Eq)]
enum Piece {
    #[token(" ")]
    Space,
    #[token("build-script")]
    BuildScript,
    #[token("(run)")]
    Run,
    #[token("\"bin\"", |_| Kind::Bin)]
    #[token("\"test\"", |_| Kind::Test)]
    #[token("\"bench\"", |_| Kind::Bench)]
    #[token("\"example\"", |_| Kind::Example)]
    Kind(Kind),
    #[token("(check)", |_| Mode::Check)]
    #[token("(check-test)", |_| Mode::CheckTest)]
    Mode(Mode),
    #[regex(r#"[^ "()]+"#, Piece::word)]
    Word(String),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Label {
    Built,
    Checked,
    CheckedTest,
    BuildScript,
    BuildRun,
    Target(Target),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Target {
    name: String,
    kind: Kind,
    mode: Mode,
}

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    #[name("bin")]
    Bin,
    #[name("test")]
    Test,
    #[name("bench")]
    Bench,
    #[name("example")]
    Example,
}

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Mode {
    #[name("(check)")]
    Check,
    #[name("(check-test)")]
    CheckTest,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("{0:?} is not a label cargo's timings give a unit of a clippy build")]
pub struct Unlabeled(String);

impl Target {
    #[must_use]
    pub fn mode(&self) -> Mode {
        self.mode
    }
}

impl Piece {
    fn word(lexer: &Lexer<'_, Piece>) -> String {
        lexer.slice().to_owned()
    }

    fn labelled(pieces: &[Piece]) -> Option<Label> {
        match pieces {
            [] => Some(Label::Built),
            [Piece::Space, Piece::Mode(Mode::Check)] => Some(Label::Checked),
            [Piece::Space, Piece::Mode(Mode::CheckTest)] => Some(Label::CheckedTest),
            [Piece::Space, Piece::BuildScript] => Some(Label::BuildScript),
            [Piece::Space, Piece::BuildScript, Piece::Space, Piece::Run] => Some(Label::BuildRun),
            [Piece::Space, Piece::Word(name), Piece::Space, Piece::Kind(kind), Piece::Space, Piece::Mode(mode)] => {
                Some(Label::Target(Target { name: name.clone(), kind: *kind, mode: *mode }))
            }
            _ => None,
        }
    }
}

impl FromStr for Label {
    type Err = Unlabeled;

    fn from_str(text: &str) -> Result<Label, Unlabeled> {
        Piece::lexer(text)
            .collect::<Result<Vec<Piece>, ()>>()
            .ok()
            .and_then(|pieces| Piece::labelled(&pieces))
            .ok_or_else(|| Unlabeled(text.to_owned()))
    }
}

impl fmt::Display for Target {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, " {} \"{}\" {}", self.name, self.kind, self.mode)
    }
}
