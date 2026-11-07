use super::boundary::Binary;
use super::boundary::TestResult;
use super::reading::Reading;

#[derive(Debug, PartialEq, Eq)]
pub enum Line {
    Binary(Binary),
    Result(TestResult),
    Other,
}

impl Line {
    #[must_use]
    pub fn continuing(self, reading: Reading) -> Reading {
        match self {
            Self::Binary(binary) => reading.in_binary(binary),
            Self::Result(result) => reading.recording(&result),
            Self::Other => reading,
        }
    }
}

impl From<&str> for Line {
    fn from(text: &str) -> Self {
        text.parse().map(Line::Binary).or_else(|_| text.parse().map(Line::Result)).unwrap_or(Self::Other)
    }
}
