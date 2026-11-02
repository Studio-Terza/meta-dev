use std::convert::Infallible;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pinning {
    LineBreak,
    Debug,
    Plain,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Scan {
    Text,
    Opened,
    Named,
    Spec,
    Questioned,
    Debug,
}

impl Scan {
    const OPEN: char = '{';
    const CLOSE: char = '}';
    const SPEC: char = ':';
    const DEBUG: char = '?';

    fn next(self, letter: char) -> Scan {
        match (self, letter) {
            (Scan::Debug, _) | (Scan::Questioned, Self::CLOSE) => Scan::Debug,
            (Scan::Text, Self::OPEN) => Scan::Opened,
            (Scan::Text, _) | (Scan::Opened | Scan::Named | Scan::Spec, Self::CLOSE) | (Scan::Opened, Self::OPEN) => {
                Scan::Text
            }
            (Scan::Opened | Scan::Named, Self::SPEC) => Scan::Spec,
            (Scan::Opened | Scan::Named, _) => Scan::Named,
            (Scan::Spec | Scan::Questioned, Self::DEBUG) => Scan::Questioned,
            (Scan::Spec | Scan::Questioned, _) => Scan::Spec,
        }
    }
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
