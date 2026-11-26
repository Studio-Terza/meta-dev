use std::fmt;

use grammar::logos::{Lexer, Logos};
use grammar::{Ignoring, Unlexed};
use newtype::Listed;
use strum::{Display, IntoStaticStr, VariantArray};

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
        text.parse::<Fixed>()
            .ignoring(Unlexed::NotThisGrammar)
            .map(Self::fixed)
            .or_else(|| Target::read(text).map(Self::target))
    }
}

#[derive(Display, IntoStaticStr, VariantArray, Listed, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fixed {
    #[strum(serialize = "")]
    Built,
    #[strum(serialize = " (check)")]
    Checked,
    #[strum(serialize = " (check-test)")]
    CheckedTest,
    #[strum(serialize = " build-script")]
    BuildScript,
    #[strum(serialize = " build-script (run)")]
    BuildRun,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Target {
    name: String,
    kind: Kind,
    mode: Mode,
}

#[derive(Display, IntoStaticStr, VariantArray, Listed, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[strum(serialize_all = "snake_case")]
pub enum Kind {
    Bin,
    Test,
    Bench,
    Example,
}

#[derive(Display, IntoStaticStr, VariantArray, Listed, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Mode {
    #[strum(serialize = "(check)")]
    Check,
    #[strum(serialize = "(check-test)")]
    CheckTest,
}

impl Target {
    #[must_use]
    pub fn mode(&self) -> Mode {
        self.mode
    }

    fn read(text: &str) -> Option<Target> {
        let pieces = Piece::lexer(text).collect::<Result<Vec<Piece>, ()>>().ignoring(Unlexed::NotThisGrammar)?;
        let [space, name, kind, mode] = <&[Piece; 4]>::try_from(pieces.as_slice()).ignoring(Unlexed::OtherShape)?;
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
