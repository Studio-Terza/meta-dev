use logos::{Lexer, Logos};

#[derive(Logos, Clone, Debug, PartialEq)]
enum Piece {
    #[token("test result:")]
    Head,
    #[regex("[ \t]+")]
    Gap,
    #[token("ok.")]
    #[token("FAILED.")]
    Outcome,
    #[regex(r"[0-9]+(\.[0-9]+)?", Piece::number)]
    Number(f64),
    #[token("passed;")]
    #[token("failed;")]
    #[token("ignored;")]
    #[token("measured;")]
    #[token("filtered out;")]
    Tally,
    #[token("finished")]
    Finished,
    #[token("in")]
    In,
    #[token("s")]
    Second,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Summary {
    seconds: f64,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("{0:?} opens like a libtest result line but does not read as one; the test output format may have changed")]
pub struct Malformed(String);

impl Summary {
    pub fn read(line: &str) -> Result<Option<Summary>, Malformed> {
        let pieces: Vec<Result<Piece, ()>> = Piece::lexer(line).collect();
        let opened = pieces.first().is_some_and(|first| *first == Ok(Piece::Head));
        let read = pieces.into_iter().collect::<Result<Vec<Piece>, ()>>().ok();
        opened
            .then(|| read.and_then(|pieces| Piece::summary(&pieces)).ok_or_else(|| Malformed(line.to_owned())))
            .transpose()
    }

    #[must_use]
    pub fn seconds(self) -> f64 {
        self.seconds
    }
}

impl Piece {
    fn number(lexer: &Lexer<'_, Piece>) -> Option<f64> {
        lexer.slice().parse().ok()
    }

    fn summary(pieces: &[Piece]) -> Option<Summary> {
        match pieces {
            [Piece::Head, Piece::Gap, Piece::Outcome, tallies @ ..] => Piece::finished(tallies),
            _ => None,
        }
    }

    fn finished(pieces: &[Piece]) -> Option<Summary> {
        match pieces {
            [Piece::Gap, Piece::Number(_), Piece::Gap, Piece::Tally, rest @ ..] => Piece::finished(rest),
            [Piece::Gap, Piece::Finished, Piece::Gap, Piece::In, Piece::Gap, Piece::Number(seconds), Piece::Second] => {
                Some(Summary { seconds: *seconds })
            }
            _ => None,
        }
    }
}
