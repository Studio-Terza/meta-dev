use std::fmt;

use grammar::logos::{Lexer, Logos};
use named_derive::Named;

#[derive(Logos, Clone, Debug, PartialEq, Eq)]
#[logos(crate = grammar::logos)]
enum Piece {
    #[token(" ")]
    Space,
    #[token(" \"bin\"", |_| Kind::Bin)]
    #[token(" \"test\"", |_| Kind::Test)]
    #[token(" \"bench\"", |_| Kind::Bench)]
    #[token(" \"example\"", |_| Kind::Example)]
    Kind(Kind),
    #[token(" (check)", |_| Mode::Check)]
    #[token(" (check-test)", |_| Mode::CheckTest)]
    Mode(Mode),
    #[regex(r#"[^ "()]+"#, Piece::word)]
    Word(String),
}

pub trait Unit: Sized {
    fn fixed(fixed: Fixed) -> Self;

    fn target(target: Target) -> Self;

    #[must_use]
    fn labelled(text: &str) -> Option<Self> {
        text.parse::<Fixed>().ok().map(Self::fixed).or_else(|| Target::read(text).map(Self::target))
    }
}

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fixed {
    #[name("")]
    Built,
    #[name(" (check)")]
    Checked,
    #[name(" (check-test)")]
    CheckedTest,
    #[name(" build-script")]
    BuildScript,
    #[name(" build-script (run)")]
    BuildRun,
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

impl Target {
    #[must_use]
    pub fn mode(&self) -> Mode {
        self.mode
    }

    fn read(text: &str) -> Option<Target> {
        let pieces = Piece::lexer(text).collect::<Result<Vec<Piece>, ()>>().ok()?;
        let [space, name, kind, mode] = <&[Piece; 4]>::try_from(pieces.as_slice()).ok()?;
        Some(space)
            .filter(|leading| **leading == Piece::Space)
            .and_then(|_space| Some(Target { name: name.named()?.to_owned(), kind: kind.kind()?, mode: mode.mode()? }))
    }
}

impl Piece {
    fn word(lexer: &Lexer<'_, Piece>) -> String {
        lexer.slice().to_owned()
    }

    fn named(&self) -> Option<&str> {
        match self {
            Piece::Word(word) => Some(word),
            Piece::Space | Piece::Kind(_) | Piece::Mode(_) => None,
        }
    }

    fn kind(&self) -> Option<Kind> {
        match self {
            Piece::Kind(kind) => Some(*kind),
            Piece::Space | Piece::Mode(_) | Piece::Word(_) => None,
        }
    }

    fn mode(&self) -> Option<Mode> {
        match self {
            Piece::Mode(mode) => Some(*mode),
            Piece::Space | Piece::Kind(_) | Piece::Word(_) => None,
        }
    }
}

impl fmt::Display for Target {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, " {} \"{}\" {}", self.name, self.kind, self.mode)
    }
}
