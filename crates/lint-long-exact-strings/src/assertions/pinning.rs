mod boundary;

use std::convert::Infallible;
use std::str::FromStr;

use boundary::Scan;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pinning {
    LineBreak,
    Debug,
    Plain,
}

impl FromStr for Pinning {
    type Err = Infallible;

    fn from_str(text: &str) -> Result<Self, Infallible> {
        const LINE_BREAK: char = '\n';
        let scanned =
            Some(text.chars().fold(Scan::Text, Scan::next)).filter(|scan| *scan == Scan::Debug).map(|_| Self::Debug);
        Ok(text.contains(LINE_BREAK).then_some(Self::LineBreak).or(scanned).unwrap_or(Self::Plain))
    }
}
