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

    fn from_str(text: &str) -> Result<Pinning, Infallible> {
        const LINE_BREAK: char = '\n';
        Ok(match (text.contains(LINE_BREAK), text.chars().fold(Scan::Text, Scan::next)) {
            (true, _) => Pinning::LineBreak,
            (false, Scan::Debug) => Pinning::Debug,
            (false, _) => Pinning::Plain,
        })
    }
}
