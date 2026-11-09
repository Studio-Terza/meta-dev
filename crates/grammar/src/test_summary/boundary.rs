use logos::{Lexer, Logos};

#[derive(Logos, Clone, Debug, PartialEq)]
enum Piece {
    #[token("test result:")]
    Head,
    #[regex(r"[ \t]+(ok|FAILED)\.")]
    Outcome,
    #[regex(r"[ \t]+[0-9]+[ \t]+(passed|failed|ignored|measured|filtered out);")]
    Tally,
    #[regex(r"[ \t]+finished[ \t]+in[ \t]+")]
    Finishing,
    #[regex(r"[0-9]+(\.[0-9]+)?", Piece::number)]
    Number(f64),
    #[token("s")]
    Second,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Summary {
    seconds: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Opened<'line>(&'line str);

impl Opened<'_> {
    #[must_use]
    pub fn summary(self) -> Option<Summary> {
        Piece::lexer(self.0).collect::<Result<Vec<Piece>, ()>>().ok().and_then(|pieces| Piece::summary(&pieces))
    }
}

impl Summary {
    #[must_use]
    pub fn opened(line: &str) -> Option<Opened<'_>> {
        Piece::lexer(line).next().filter(|first| *first == Ok(Piece::Head)).map(|_head| Opened(line))
    }

    #[must_use]
    pub fn seconds(self) -> f64 {
        self.seconds
    }
}

impl Piece {
    const FRAME: [Piece; 4] = [Piece::Head, Piece::Outcome, Piece::Finishing, Piece::Second];

    fn number(lexer: &Lexer<'_, Piece>) -> Option<f64> {
        lexer.slice().parse().ok()
    }

    fn summary(pieces: &[Piece]) -> Option<Summary> {
        let ([head, outcome], rest) = pieces.split_first_chunk::<2>()?;
        let (tallies, [finishing, number, second]) = rest.split_last_chunk::<3>()?;
        Some([head, outcome, finishing, second].map(Piece::clone))
            .filter(|frame| *frame == Self::FRAME)
            .filter(|_frame| tallies.iter().all(|tally| *tally == Piece::Tally))
            .and_then(|_frame| number.seconds())
            .map(|seconds| Summary { seconds })
    }

    fn seconds(&self) -> Option<f64> {
        match self {
            Piece::Number(seconds) => Some(*seconds),
            Piece::Head | Piece::Outcome | Piece::Tally | Piece::Finishing | Piece::Second => None,
        }
    }
}
