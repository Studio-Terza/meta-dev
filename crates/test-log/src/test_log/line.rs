use super::binary::Binary;
use super::reading::Reading;
use super::test_result::TestResult;

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
            Line::Binary(binary) => reading.in_binary(binary),
            Line::Result(result) => reading.recording(&result),
            Line::Other => reading,
        }
    }
}

impl From<&str> for Line {
    fn from(text: &str) -> Line {
        text.parse().map(Line::Binary).or_else(|_| text.parse().map(Line::Result)).unwrap_or(Line::Other)
    }
}
