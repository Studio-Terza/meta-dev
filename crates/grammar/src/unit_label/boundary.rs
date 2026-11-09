use std::fmt;

use logos::{Lexer, Logos};
use named_derive::Named;

#[derive(Logos, Clone, Debug, PartialEq, Eq)]
enum Piece {
    #[token(" ")]
    Space,
    #[token(" build-script")]
    BuildScript,
    #[token(" (run)")]
    Run,
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
        Piece::lexer(text).collect::<Result<Vec<Piece>, ()>>().ok().and_then(|pieces| {
            Fixed::read(&pieces).map(Self::fixed).or_else(|| Target::read(&pieces).map(Self::target))
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fixed {
    Built,
    Checked,
    CheckedTest,
    BuildScript,
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

impl Fixed {
    const BUILD_RUN: [Piece; 2] = [Piece::BuildScript, Piece::Run];

    fn read(pieces: &[Piece]) -> Option<Fixed> {
        <&[Piece; 0]>::try_from(pieces)
            .ok()
            .map(|_none| Fixed::Built)
            .or_else(|| <&[Piece; 1]>::try_from(pieces).ok().and_then(|[only]| Fixed::alone(only)))
            .or_else(|| Some(pieces).filter(|label| **label == Self::BUILD_RUN).map(|_run| Fixed::BuildRun))
    }

    fn alone(piece: &Piece) -> Option<Fixed> {
        match piece {
            Piece::Mode(mode) => Some(mode.library()),
            Piece::BuildScript => Some(Fixed::BuildScript),
            Piece::Space | Piece::Run | Piece::Kind(_) | Piece::Word(_) => None,
        }
    }
}

impl Target {
    #[must_use]
    pub fn mode(&self) -> Mode {
        self.mode
    }

    fn read(pieces: &[Piece]) -> Option<Target> {
        let [space, name, kind, mode] = <&[Piece; 4]>::try_from(pieces).ok()?;
        Some(space)
            .filter(|leading| **leading == Piece::Space)
            .and_then(|_space| Some(Target { name: name.named()?.to_owned(), kind: kind.kind()?, mode: mode.mode()? }))
    }
}

impl Mode {
    fn library(self) -> Fixed {
        match self {
            Mode::Check => Fixed::Checked,
            Mode::CheckTest => Fixed::CheckedTest,
        }
    }
}

impl Piece {
    fn word(lexer: &Lexer<'_, Piece>) -> String {
        lexer.slice().to_owned()
    }

    fn named(&self) -> Option<&str> {
        match self {
            Piece::Word(word) => Some(word),
            Piece::Space | Piece::BuildScript | Piece::Run | Piece::Kind(_) | Piece::Mode(_) => None,
        }
    }

    fn kind(&self) -> Option<Kind> {
        match self {
            Piece::Kind(kind) => Some(*kind),
            Piece::Space | Piece::BuildScript | Piece::Run | Piece::Mode(_) | Piece::Word(_) => None,
        }
    }

    fn mode(&self) -> Option<Mode> {
        match self {
            Piece::Mode(mode) => Some(*mode),
            Piece::Space | Piece::BuildScript | Piece::Run | Piece::Kind(_) | Piece::Word(_) => None,
        }
    }
}

impl fmt::Display for Target {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, " {} \"{}\" {}", self.name, self.kind, self.mode)
    }
}
