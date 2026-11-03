use lint_layout::{Name, Word, words};

#[derive(Debug)]
pub struct DoubleWords;

impl DoubleWords {
    const DOUBLES: [Word<'static>; 5] = words!["fake", "stub", "mock", "spy", "dummy"];

    pub fn name(name: &Name) -> bool {
        name.has_camel_word(&Self::DOUBLES)
    }
}
