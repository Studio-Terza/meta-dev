use lint_layout::Matching;
use lint_layout::Name;
use lint_layout::Word;

#[derive(Debug)]
pub struct Suffix;

impl Matching for Suffix {
    fn holds(name: &Name, word: Word<'_>) -> bool {
        name.words().last().is_some_and(|last| last.lowered() == word)
    }
}
