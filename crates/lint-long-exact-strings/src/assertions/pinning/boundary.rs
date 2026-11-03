#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scan {
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

    pub fn next(self, letter: char) -> Scan {
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
